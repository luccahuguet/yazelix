mod editor;
mod layout;
mod panes;
mod sidebar_yazi;
mod transient;
mod workspace;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};

use panes::{FocusContext, ManagedTabPanes};
use workspace::{bootstrap_workspace_root, WorkspaceState};
use yazelix_pane_orchestrator::horizontal_focus_contract::HorizontalDirection;
use zellij_tile::prelude::*;

pub(crate) const RESULT_OK: &str = "ok";
pub(crate) const RESULT_FOCUSED_EDITOR: &str = "focused_editor";
pub(crate) const RESULT_FOCUSED_SIDEBAR: &str = "focused_sidebar";
pub(crate) const RESULT_OPENED_SIDEBAR: &str = "opened_sidebar";
pub(crate) const RESULT_MISSING: &str = "missing";
pub(crate) const RESULT_NOT_READY: &str = "not_ready";
pub(crate) const RESULT_DENIED: &str = "permissions_denied";
pub(crate) const RESULT_INVALID_PAYLOAD: &str = "invalid_payload";
pub(crate) const RESULT_UNKNOWN_LAYOUT: &str = "unknown_layout";
pub(crate) const RESULT_UNSUPPORTED_EDITOR: &str = "unsupported_editor";
pub(crate) const COMMAND_STEP_DELAY_MS: u64 = 35;
pub(crate) const SWAP_LAYOUT_STEP_DELAY_MS: u64 = 1;

#[derive(Default)]
struct State {
    active_tab_position: Option<usize>,
    active_swap_layout_name_by_tab: HashMap<usize, Option<String>>,
    last_known_layout_variant_by_tab: RefCell<HashMap<usize, layout::LayoutVariant>>,
    focus_context_by_tab: HashMap<usize, FocusContext>,
    focused_terminal_pane_by_tab: HashMap<usize, PaneId>,
    fallback_terminal_pane_by_tab: HashMap<usize, PaneId>,
    managed_panes_by_tab: HashMap<usize, ManagedTabPanes>,
    terminal_panes_by_tab: HashMap<usize, Vec<panes::TerminalPaneLayout>>,
    user_pane_count_by_tab: HashMap<usize, usize>,
    workspace_state_by_tab: HashMap<usize, WorkspaceState>,
    sidebar_yazi_state_by_tab: HashMap<usize, sidebar_yazi::SidebarYaziState>,
    seen_tab_positions: HashSet<usize>,
    initial_workspace_state: Option<WorkspaceState>,
    override_layout_config: layout::OverrideLayoutConfig,
    transient_pane_config: transient::TransientPaneConfig,
    permissions_granted: bool,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        set_selectable(false);
        let plugin_ids = get_plugin_ids();
        let bootstrap_root = bootstrap_workspace_root(&plugin_ids.initial_cwd);
        self.initial_workspace_state = Some(WorkspaceState::from_bootstrap_root(bootstrap_root));
        self.override_layout_config = layout::OverrideLayoutConfig {
            zjstatus_segments: layout::ZjstatusSegments {
                widget_tray: configuration
                    .get("widget_tray_segment")
                    .cloned()
                    .unwrap_or_default(),
                custom_text: configuration
                    .get("custom_text_segment")
                    .cloned()
                    .unwrap_or_default(),
            },
            sidebar_width_percent: configuration
                .get("sidebar_width_percent")
                .and_then(|value| value.parse::<usize>().ok())
                .filter(|value| (10..=40).contains(value))
                .unwrap_or(layout::DEFAULT_SIDEBAR_WIDTH_PERCENT),
        };
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::OpenTerminalsOrPlugins,
            PermissionType::RunCommands,
            PermissionType::WriteToStdin,
            PermissionType::ReadCliPipes,
        ]);
        self.transient_pane_config = transient::TransientPaneConfig::from_plugin_configuration(
            &configuration,
            &plugin_ids.initial_cwd,
        );
        subscribe(&[
            EventType::TabUpdate,
            EventType::PaneUpdate,
            EventType::PermissionRequestResult,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::TabUpdate(tabs) => {
                self.active_tab_position =
                    tabs.iter().find(|tab| tab.active).map(|tab| tab.position);
                self.reconcile_workspace_state(&tabs);
                {
                    let mut last_known_layout_variant_by_tab =
                        self.last_known_layout_variant_by_tab.borrow_mut();
                    for tab in &tabs {
                        if let Some(layout_variant) = tab
                            .active_swap_layout_name
                            .as_deref()
                            .and_then(layout::LayoutVariant::from_layout_name)
                        {
                            last_known_layout_variant_by_tab.insert(tab.position, layout_variant);
                        }
                    }
                }
                self.active_swap_layout_name_by_tab = tabs
                    .into_iter()
                    .map(|tab| (tab.position, tab.active_swap_layout_name))
                    .collect();
            }
            Event::PaneUpdate(pane_manifest) => {
                self.managed_panes_by_tab = panes::build_managed_panes_by_tab(&pane_manifest);
                self.focus_context_by_tab =
                    panes::build_focus_context_by_tab(&pane_manifest, &self.focus_context_by_tab);
                self.focused_terminal_pane_by_tab =
                    panes::build_focused_terminal_pane_by_tab(&pane_manifest);
                self.fallback_terminal_pane_by_tab =
                    panes::build_fallback_terminal_pane_by_tab(&pane_manifest);
                self.terminal_panes_by_tab = panes::build_terminal_panes_by_tab(&pane_manifest);
                self.user_pane_count_by_tab = panes::build_user_pane_count_by_tab(&pane_manifest);
                self.reconcile_sidebar_yazi_state();
            }
            Event::PermissionRequestResult(status) => {
                self.permissions_granted = status == PermissionStatus::Granted;
            }
            _ => {}
        }
        false
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        match pipe_message.name.as_str() {
            "focus_editor" => {
                self.focus_managed_pane(&pipe_message, panes::ManagedPaneKind::Editor);
                false
            }
            "focus_sidebar" => {
                self.focus_managed_pane(&pipe_message, panes::ManagedPaneKind::Sidebar);
                false
            }
            "toggle_editor_sidebar_focus" => {
                self.toggle_editor_sidebar_focus(&pipe_message);
                false
            }
            "move_focus_left_or_tab" => {
                self.move_horizontal_focus_or_tab(&pipe_message, HorizontalDirection::Left);
                false
            }
            "move_focus_right_or_tab" => {
                self.move_horizontal_focus_or_tab(&pipe_message, HorizontalDirection::Right);
                false
            }
            "smart_reveal" => {
                self.smart_reveal(&pipe_message);
                false
            }
            "open_file" => {
                self.open_file_in_managed_editor(&pipe_message);
                false
            }
            "set_managed_editor_cwd" => {
                self.set_managed_editor_cwd(&pipe_message);
                false
            }
            "next_family" => {
                self.switch_layout_family(&pipe_message, layout::FamilyDirection::Next);
                false
            }
            "previous_family" => {
                self.switch_layout_family(&pipe_message, layout::FamilyDirection::Previous);
                false
            }
            "toggle_sidebar" => {
                self.toggle_sidebar(&pipe_message);
                false
            }
            "register_sidebar_yazi_state" => {
                self.register_sidebar_yazi_state(&pipe_message);
                false
            }
            "get_active_sidebar_yazi_state" => {
                self.get_active_sidebar_yazi_state(&pipe_message);
                false
            }
            "retarget_workspace" => {
                self.retarget_workspace(&pipe_message);
                false
            }
            "open_terminal_in_cwd" => {
                self.open_terminal_in_cwd(&pipe_message);
                false
            }
            "open_workspace_terminal" => {
                self.open_workspace_terminal(&pipe_message);
                false
            }
            "open_transient_pane" => {
                self.open_transient_pane(&pipe_message);
                false
            }
            "toggle_transient_pane" => {
                self.toggle_transient_pane(&pipe_message);
                false
            }
            "debug_editor_state" => {
                self.debug_editor_state(&pipe_message);
                false
            }
            "debug_write_literal" => {
                self.debug_write_literal(&pipe_message);
                false
            }
            "debug_send_escape" => {
                self.debug_send_escape(&pipe_message);
                false
            }
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, _cols: usize) {}
}

impl State {
    pub(crate) fn ensure_action_ready(&self, pipe_message: &PipeMessage) -> Option<usize> {
        if !self.permissions_granted {
            self.respond(pipe_message, RESULT_DENIED);
            return None;
        }

        let Some(active_tab_position) = self.active_tab_position else {
            self.respond(pipe_message, RESULT_NOT_READY);
            return None;
        };

        Some(active_tab_position)
    }

    pub(crate) fn respond(&self, pipe_message: &PipeMessage, result: &str) {
        if let PipeSource::Cli(pipe_id) = &pipe_message.source {
            cli_pipe_output(pipe_id, result);
        }
    }
}
