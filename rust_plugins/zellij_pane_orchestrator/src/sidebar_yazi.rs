use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use yazelix_pane_orchestrator::sidebar_state_contract::{
    find_tab_for_sidebar_pane_id, retain_tab_local_sidebar_pane_state,
};
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
        let sidebar_pane_id_by_tab = self.sidebar_pane_id_by_tab();
        retain_tab_local_sidebar_pane_state(
            &mut self.sidebar_yazi_state_by_tab,
            &sidebar_pane_id_by_tab,
            |sidebar_state| sidebar_state.pane_id.as_str(),
        );
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
        find_tab_for_sidebar_pane_id(&self.sidebar_pane_id_by_tab(), pane_id)
    }

    fn sidebar_pane_id_by_tab(&self) -> HashMap<usize, String> {
        self.managed_panes_by_tab
            .iter()
            .filter_map(|(tab_position, managed_tab_panes)| {
                pane_id_to_string(managed_tab_panes.sidebar.map(|pane| pane.pane_id))
                    .map(|pane_id| (*tab_position, pane_id))
            })
            .collect()
    }
}
