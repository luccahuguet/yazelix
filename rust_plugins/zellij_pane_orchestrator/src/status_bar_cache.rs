use std::collections::BTreeMap;

use yazelix_pane_orchestrator::status_bar_cache_contract::resolve_status_bar_cache_runtime;
use zellij_tile::prelude::*;

use crate::State;

impl State {
    pub(crate) fn refresh_status_bar_cache(&mut self) {
        if !self.permissions_granted {
            return;
        }
        let Some(active_tab_position) = self.active_tab_position else {
            return;
        };
        let Ok(payload) =
            serde_json::to_string(&self.active_tab_session_state_snapshot(active_tab_position))
        else {
            return;
        };
        if self.status_bar_cache_last_payload.as_deref() == Some(payload.as_str()) {
            return;
        }

        let Some(runtime) = self.status_bar_cache_runtime.clone().or_else(|| {
            let session_env = get_session_environment_variables();
            resolve_status_bar_cache_runtime(&session_env)
        }) else {
            return;
        };
        self.status_bar_cache_runtime = Some(runtime.clone());

        let command = [
            runtime.yzx_control_path.as_str(),
            "zellij",
            "status-cache-write",
            "--path",
            runtime.cache_path.as_str(),
            "--payload",
            payload.as_str(),
        ];
        run_command_with_env_variables_and_cwd(&command, runtime.env, runtime.cwd, BTreeMap::new());
        self.status_bar_cache_last_payload = Some(payload);
    }
}
