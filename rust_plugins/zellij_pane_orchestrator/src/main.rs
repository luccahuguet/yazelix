mod editor;
mod layout;
mod panes;
mod workspace;

use std::collections::{BTreeMap, HashMap, HashSet};

use panes::{FocusContext, ManagedTabPanes};
use workspace::WorkspaceState;
use zellij_tile::prelude::*;

pub(crate) const RESULT_OK: &str = "ok";
pub(crate) const RESULT_MISSING: &str = "missing";
pub(crate) const RESULT_NOT_READY: &str = "not_ready";
pub(crate) const RESULT_DENIED: &str = "permissions_denied";
pub(crate) const RESULT_INVALID_PAYLOAD: &str = "invalid_payload";
pub(crate) const RESULT_MISSING_WORKSPACE: &str = "missing_workspace";
pub(crate) const RESULT_UNKNOWN_LAYOUT: &str = "unknown_layout";
pub(crate) const RESULT_UNSUPPORTED_EDITOR: &str = "unsupported_editor";
pub(crate) const COMMAND_STEP_DELAY_MS: u64 = 35;
pub(crate) const SWAP_LAYOUT_STEP_DELAY_MS: u64 = 1;

#[derive(Default)]
struct State {
    active_tab_position: Option<usize>,
    active_swap_layout_name_by_tab: HashMap<usize, Option<String>>,
    focus_context_by_tab: HashMap<usize, FocusContext>,
    managed_panes_by_tab: HashMap<usize, ManagedTabPanes>,
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
        self.initial_workspace_state = Some(WorkspaceState::from_root(
            plugin_ids.initial_cwd.display().to_string(),
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
                let previous_active_tab_position = self.active_tab_position;
                self.active_tab_position =
                    tabs.iter().find(|tab| tab.active).map(|tab| tab.position);
                self.reconcile_workspace_state(previous_active_tab_position, &tabs);
                self.active_swap_layout_name_by_tab = tabs
                    .into_iter()
                    .map(|tab| (tab.position, tab.active_swap_layout_name))
                    .collect();
            }
            Event::PaneUpdate(pane_manifest) => {
                self.managed_panes_by_tab = panes::build_managed_panes_by_tab(&pane_manifest);
                self.focus_context_by_tab =
                    panes::build_focus_context_by_tab(&pane_manifest, &self.focus_context_by_tab);
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
            "open_file" => {
                self.open_file_in_managed_editor(&pipe_message);
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
            "set_workspace_root" => {
                self.set_workspace_root(&pipe_message);
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
}
