use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use zellij_tile::prelude::*;

use crate::panes::pane_id_to_string;
use crate::{State, RESULT_DENIED, RESULT_INVALID_PAYLOAD, RESULT_MISSING, RESULT_OK};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct SidebarYaziState {
    pub(crate) pane_id: String,
    pub(crate) yazi_id: String,
    pub(crate) cwd: String,
}

#[derive(Deserialize)]
struct SidebarYaziStateRegistration {
    pane_id: String,
    yazi_id: String,
    cwd: String,
}

impl State {
    pub(crate) fn reconcile_sidebar_yazi_state(&mut self) {
        let sidebar_pane_id_by_tab: HashMap<usize, String> = self
            .managed_panes_by_tab
            .iter()
            .filter_map(|(tab_position, managed_tab_panes)| {
                pane_id_to_string(managed_tab_panes.sidebar.map(|pane| pane.pane_id))
                    .map(|pane_id| (*tab_position, pane_id))
            })
            .collect();

        self.sidebar_yazi_state_by_tab
            .retain(|tab_position, sidebar_state| {
                sidebar_pane_id_by_tab
                    .get(tab_position)
                    .map(|pane_id| pane_id == &sidebar_state.pane_id)
                    .unwrap_or(false)
            });
    }

    pub(crate) fn register_sidebar_yazi_state(&mut self, pipe_message: &PipeMessage) {
        if !self.permissions_granted {
            self.respond(pipe_message, RESULT_DENIED);
            return;
        }

        let Some(payload) = pipe_message.payload.as_deref() else {
            self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
            return;
        };

        let registration: SidebarYaziStateRegistration = match serde_json::from_str(payload) {
            Ok(request) => request,
            Err(_) => {
                self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
                return;
            }
        };

        let pane_id = registration.pane_id.trim().to_string();
        let yazi_id = registration.yazi_id.trim().to_string();
        let cwd = registration.cwd.trim().to_string();
        if pane_id.is_empty() || yazi_id.is_empty() || cwd.is_empty() {
            self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
            return;
        }

        let Some(tab_position) = self.find_tab_position_for_sidebar_pane_id(&pane_id) else {
            self.respond(pipe_message, RESULT_MISSING);
            return;
        };

        self.sidebar_yazi_state_by_tab.insert(
            tab_position,
            SidebarYaziState {
                pane_id,
                yazi_id,
                cwd,
            },
        );
        self.respond(pipe_message, RESULT_OK);
    }

    pub(crate) fn get_active_sidebar_yazi_state_snapshot(
        &self,
        active_tab_position: usize,
    ) -> Option<&SidebarYaziState> {
        let expected_pane_id = self
            .managed_panes_by_tab
            .get(&active_tab_position)
            .and_then(|managed_tab_panes| {
                pane_id_to_string(managed_tab_panes.sidebar.map(|pane| pane.pane_id))
            })?;

        let sidebar_state = self.sidebar_yazi_state_by_tab.get(&active_tab_position)?;
        if sidebar_state.pane_id == expected_pane_id {
            Some(sidebar_state)
        } else {
            None
        }
    }

    fn find_tab_position_for_sidebar_pane_id(&self, pane_id: &str) -> Option<usize> {
        self.managed_panes_by_tab
            .iter()
            .find_map(|(tab_position, managed_tab_panes)| {
                let candidate =
                    pane_id_to_string(managed_tab_panes.sidebar.map(|pane| pane.pane_id))?;
                if candidate == pane_id {
                    Some(*tab_position)
                } else {
                    None
                }
            })
    }
}
