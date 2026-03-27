mod editor;
mod layout;
mod panes;
mod workspace;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use yazelix_pane_orchestrator::horizontal_focus_contract::HorizontalDirection;
use panes::{FocusContext, ManagedTabPanes};
use workspace::WorkspaceState;
use zellij_tile::prelude::*;

pub(crate) const RESULT_OK: &str = "ok";
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
    seen_tab_positions: HashSet<usize>,
    initial_workspace_state: Option<WorkspaceState>,
    permissions_granted: bool,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        set_selectable(false);
        let plugin_ids = get_plugin_ids();
        let bootstrap_root = env::var("HOME")
            .ok()
            .filter(|home| !home.trim().is_empty())
            .unwrap_or_else(|| plugin_ids.initial_cwd.display().to_string());
        self.initial_workspace_state = Some(WorkspaceState::from_bootstrap_root(
            bootstrap_root,
        ));
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::OpenTerminalsOrPlugins,
            PermissionType::WriteToStdin,
            PermissionType::ReadCliPipes,
        ]);
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
                self.terminal_panes_by_tab =
                    panes::build_terminal_panes_by_tab(&pane_manifest);
                self.user_pane_count_by_tab = panes::build_user_pane_count_by_tab(&pane_manifest);
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
            "debug_layout_state" => {
                self.debug_layout_state(&pipe_message);
                false
            }
            "debug_override_build_state" => {
                self.debug_override_build_state(&pipe_message);
                false
            }
            "set_workspace_root" => {
                self.set_workspace_root(&pipe_message);
                false
            }
            "set_workspace_root_and_cd_focused_pane" => {
                self.set_workspace_root_and_cd_focused_pane(&pipe_message);
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

    pub(crate) fn append_layout_debug_log(&self, message: &str) {
        let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("/"));
        let log_dir = format!("{home_dir}/.local/share/yazelix/logs");
        let _ = fs::create_dir_all(&log_dir);
        let log_path = format!("{log_dir}/pane_orchestrator_layout.log");
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs().to_string())
            .unwrap_or_else(|_| String::from("0"));

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
            let _ = writeln!(file, "[{timestamp}] {message}");
        }
    }
}
