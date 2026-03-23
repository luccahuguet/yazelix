use std::collections::{BTreeMap, HashMap};

use yazelix_popup_runner::popup_contract::{PopupTogglePlan, resolve_popup_toggle};
use yazelix_popup_runner::popup_pane_contract::{PopupPaneState, select_popup_pane};
use zellij_tile::prelude::*;

const RESULT_OK: &str = "ok";
const RESULT_MISSING: &str = "missing";
const RESULT_NOT_READY: &str = "not_ready";
const RESULT_DENIED: &str = "permissions_denied";

#[derive(Default)]
struct State {
    active_tab_position: Option<usize>,
    popup_panes_by_tab: HashMap<usize, PopupPaneState>,
    permissions_granted: bool,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        set_selectable(false);
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
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
                self.active_tab_position = tabs.iter().find(|tab| tab.active).map(|tab| tab.position);
            }
            Event::PaneUpdate(pane_manifest) => {
                self.popup_panes_by_tab = pane_manifest
                    .panes
                    .into_iter()
                    .filter_map(|(tab_position, panes)| select_popup_pane(&panes).map(|pane| (tab_position, pane)))
                    .collect();
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
            "toggle_popup" => self.toggle_popup(&pipe_message),
            "has_popup" => self.has_popup(&pipe_message),
            _ => {}
        }
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {}
}

impl State {
    fn toggle_popup(&self, pipe_message: &PipeMessage) {
        let Some(active_tab_position) = self.ensure_action_ready(pipe_message) else {
            return;
        };

        let popup_pane = self.popup_panes_by_tab.get(&active_tab_position).copied();
        match resolve_popup_toggle(popup_pane.is_some(), popup_pane.map(|pane| pane.is_focused).unwrap_or(false)) {
            PopupTogglePlan::OpenPopup => {
                self.respond(pipe_message, RESULT_MISSING);
            }
            PopupTogglePlan::FocusPopup => {
                if let Some(popup_pane) = popup_pane {
                    focus_pane_with_id(popup_pane.pane_id, true);
                    self.respond(pipe_message, RESULT_OK);
                } else {
                    self.respond(pipe_message, RESULT_MISSING);
                }
            }
            PopupTogglePlan::ClosePopup => {
                if let Some(popup_pane) = popup_pane {
                    close_pane_with_id(popup_pane.pane_id);
                    self.respond(pipe_message, RESULT_OK);
                } else {
                    self.respond(pipe_message, RESULT_MISSING);
                }
            }
        }
    }

    fn ensure_action_ready(&self, pipe_message: &PipeMessage) -> Option<usize> {
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

    fn respond(&self, pipe_message: &PipeMessage, result: &str) {
        if let PipeSource::Cli(pipe_id) = &pipe_message.source {
            cli_pipe_output(pipe_id, result);
        }
    }

    fn has_popup(&self, pipe_message: &PipeMessage) {
        let Some(active_tab_position) = self.active_tab_position else {
            self.respond(pipe_message, RESULT_NOT_READY);
            return;
        };

        if self.popup_panes_by_tab.contains_key(&active_tab_position) {
            self.respond(pipe_message, "true");
        } else {
            self.respond(pipe_message, "false");
        }
    }
}

#[cfg(test)]
mod tests {
    // Covered by lib-level popup_contract and popup_pane_contract tests.
}
