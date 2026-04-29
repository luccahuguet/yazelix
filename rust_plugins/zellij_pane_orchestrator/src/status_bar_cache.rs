use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use yazelix_pane_orchestrator::status_bar_cache_contract::resolve_status_bar_cache_runtime;
use zellij_tile::prelude::*;

use crate::State;

const INITIAL_AGENT_USAGE_REFRESH_DELAY: Duration = Duration::from_secs(5);
const AGENT_USAGE_REFRESH_INTERVAL: Duration = Duration::from_secs(120);
const AGENT_USAGE_PROVIDER_TIMEOUT_MS: &str = "1500";
const AGENT_USAGE_MAX_AGE_SECONDS: &str = "120";

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

    pub(crate) fn schedule_initial_status_bar_agent_usage_refresh(&mut self) {
        self.schedule_status_bar_agent_usage_refresh_after(INITIAL_AGENT_USAGE_REFRESH_DELAY);
    }

    pub(crate) fn handle_status_bar_agent_usage_timer(&mut self) {
        let now = Instant::now();
        let Some(next_refresh) = self.status_bar_agent_usage_next_refresh else {
            self.schedule_initial_status_bar_agent_usage_refresh();
            return;
        };
        if now < next_refresh {
            self.schedule_status_bar_agent_usage_refresh_after(
                next_refresh.saturating_duration_since(now),
            );
            return;
        }

        if !self.permissions_granted {
            self.schedule_status_bar_agent_usage_refresh_after(INITIAL_AGENT_USAGE_REFRESH_DELAY);
            return;
        }

        self.refresh_status_bar_agent_usage_cache();
        self.schedule_status_bar_agent_usage_refresh_after(AGENT_USAGE_REFRESH_INTERVAL);
    }

    fn schedule_status_bar_agent_usage_refresh_after(&mut self, delay: Duration) {
        self.status_bar_agent_usage_next_refresh = Some(Instant::now() + delay);
        set_timeout(delay.as_secs_f64().max(0.5));
    }

    fn refresh_status_bar_agent_usage_cache(&mut self) {
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
            "status-cache-refresh-agent-usage",
            "--path",
            runtime.cache_path.as_str(),
            "--max-age-seconds",
            AGENT_USAGE_MAX_AGE_SECONDS,
            "--timeout-ms",
            AGENT_USAGE_PROVIDER_TIMEOUT_MS,
        ];
        run_command_with_env_variables_and_cwd(&command, runtime.env, runtime.cwd, BTreeMap::new());
    }
}
