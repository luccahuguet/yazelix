use std::collections::HashSet;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use serde::Deserialize;
use zellij_tile::prelude::*;

use crate::{
    State, COMMAND_STEP_DELAY_MS, RESULT_INVALID_PAYLOAD, RESULT_MISSING_WORKSPACE, RESULT_OK,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkspaceState {
    pub(crate) root: String,
}

#[derive(Deserialize)]
struct WorkspaceRootRequest {
    workspace_root: String,
}

#[derive(Deserialize)]
struct OpenTerminalRequest {
    cwd: String,
}

impl State {
    pub(crate) fn reconcile_workspace_state(
        &mut self,
        previous_active_tab_position: Option<usize>,
        tabs: &[TabInfo],
    ) {
        let current_tab_positions: HashSet<usize> = tabs.iter().map(|tab| tab.position).collect();
        self.workspace_state_by_tab
            .retain(|tab_position, _| current_tab_positions.contains(tab_position));
        self.seen_tab_positions
            .retain(|tab_position| current_tab_positions.contains(tab_position));

        let active_tab_position = tabs.iter().find(|tab| tab.active).map(|tab| tab.position);

        if let Some(active_tab_position) = active_tab_position {
            let is_new_tab = !self.seen_tab_positions.contains(&active_tab_position);
            if !self
                .workspace_state_by_tab
                .contains_key(&active_tab_position)
            {
                let inherited_workspace_state = if is_new_tab {
                    previous_active_tab_position
                        .and_then(|previous_active| {
                            self.workspace_state_by_tab.get(&previous_active).cloned()
                        })
                        .or_else(|| self.initial_workspace_state.clone())
                } else if self.workspace_state_by_tab.is_empty() {
                    self.initial_workspace_state.clone()
                } else {
                    None
                };

                if let Some(workspace_state) = inherited_workspace_state {
                    self.workspace_state_by_tab
                        .insert(active_tab_position, workspace_state);
                }
            }
        }

        self.seen_tab_positions = current_tab_positions;
    }

    pub(crate) fn set_workspace_root(&mut self, pipe_message: &PipeMessage) {
        let Some(active_tab_position) = self.ensure_action_ready(pipe_message) else {
            return;
        };

        let Some(payload) = pipe_message.payload.as_deref() else {
            self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
            return;
        };

        let workspace_root_request: WorkspaceRootRequest = match serde_json::from_str(payload) {
            Ok(request) => request,
            Err(_) => {
                self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
                return;
            }
        };

        let workspace_state = WorkspaceState::from_root(workspace_root_request.workspace_root);
        rename_tab(
            tab_index_from_position(active_tab_position),
            &tab_name_from_workspace_root(&workspace_state.root),
        );
        self.workspace_state_by_tab
            .insert(active_tab_position, workspace_state);
        self.respond(pipe_message, RESULT_OK);
    }

    pub(crate) fn set_workspace_root_and_cd_focused_pane(&mut self, pipe_message: &PipeMessage) {
        let Some(active_tab_position) = self.ensure_action_ready(pipe_message) else {
            return;
        };

        let Some(payload) = pipe_message.payload.as_deref() else {
            self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
            return;
        };

        let workspace_root_request: WorkspaceRootRequest = match serde_json::from_str(payload) {
            Ok(request) => request,
            Err(_) => {
                self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
                return;
            }
        };

        let Some(focused_pane_id) = self.get_focused_terminal_pane(pipe_message) else {
            return;
        };

        let workspace_state = WorkspaceState::from_root(workspace_root_request.workspace_root);
        rename_tab(
            tab_index_from_position(active_tab_position),
            &tab_name_from_workspace_root(&workspace_state.root),
        );
        self.workspace_state_by_tab
            .insert(active_tab_position, workspace_state.clone());

        write_chars_to_pane_id(
            &change_directory_command(&workspace_state.root),
            focused_pane_id,
        );
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_to_pane_id(vec![13], focused_pane_id);

        self.respond(pipe_message, RESULT_OK);
    }

    pub(crate) fn open_workspace_terminal(&self, pipe_message: &PipeMessage) {
        let Some(active_tab_position) = self.ensure_action_ready(pipe_message) else {
            return;
        };

        let Some(workspace_state) = self
            .workspace_state_by_tab
            .get(&active_tab_position)
            .cloned()
            .or_else(|| {
                if self.seen_tab_positions.len() <= 1 {
                    self.initial_workspace_state.clone()
                } else {
                    None
                }
            })
        else {
            self.respond(pipe_message, RESULT_MISSING_WORKSPACE);
            return;
        };

        open_terminal(&workspace_state.root);
        rename_tab(
            tab_index_from_position(active_tab_position),
            &tab_name_from_workspace_root(&workspace_state.root),
        );
        self.respond(pipe_message, RESULT_OK);
    }

    pub(crate) fn open_terminal_in_cwd(&self, pipe_message: &PipeMessage) {
        let Some(payload) = pipe_message.payload.as_deref() else {
            self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
            return;
        };

        let open_terminal_request: OpenTerminalRequest = match serde_json::from_str(payload) {
            Ok(request) => request,
            Err(_) => {
                self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
                return;
            }
        };

        open_terminal(&open_terminal_request.cwd);
        self.respond(pipe_message, RESULT_OK);
    }
}

impl WorkspaceState {
    pub(crate) fn from_root(root: String) -> Self {
        Self { root }
    }
}

pub(crate) fn tab_name_from_workspace_root(workspace_root: &str) -> String {
    let trimmed = workspace_root.trim_end_matches(std::path::MAIN_SEPARATOR);
    let candidate = if trimmed.is_empty() {
        workspace_root
    } else {
        trimmed
    };

    Path::new(candidate)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("unnamed")
        .to_string()
}

fn tab_index_from_position(tab_position: usize) -> u32 {
    // Zellij reports tabs to plugins by 0-based position, but rename_tab targets the 1-based tab index.
    u32::try_from(tab_position + 1).expect("tab position should fit in u32")
}

fn change_directory_command(path: &str) -> String {
    format!("cd \"{}\"", escape_double_quoted_path(path))
}

fn escape_double_quoted_path(path: &str) -> String {
    path.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "\\$")
        .replace('`', "\\`")
}
