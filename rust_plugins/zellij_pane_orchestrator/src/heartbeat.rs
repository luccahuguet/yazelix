use std::collections::BTreeMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use yazelix_pane_orchestrator::orchestrator_heartbeat_contract::{
    build_orchestrator_heartbeat_payload, OrchestratorHeartbeatPayload,
};
use yazelix_pane_orchestrator::status_bar_cache_contract::resolve_status_bar_cache_runtime;
use zellij_tile::prelude::*;

use crate::State;

const HEARTBEAT_INITIAL_FLUSH_DELAY: Duration = Duration::from_secs(5);
const HEARTBEAT_FLUSH_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Default)]
pub(crate) struct OrchestratorHeartbeat {
    pub(crate) started_at_unix_seconds: u64,
    pub(crate) next_flush: Option<Instant>,
    last_event_kind: Option<String>,
    last_event_at_unix_seconds: Option<u64>,
    last_timer_at_unix_seconds: Option<u64>,
    last_pipe_name: Option<String>,
    last_pipe_at_unix_seconds: Option<u64>,
    last_status_cache_write_at_unix_seconds: Option<u64>,
    status_refresh_started_at_by_name: BTreeMap<String, u64>,
}

impl OrchestratorHeartbeat {
    fn payload(&self, now: u64) -> serde_json::Value {
        build_orchestrator_heartbeat_payload(OrchestratorHeartbeatPayload {
            heartbeat_at_unix_seconds: now,
            started_at_unix_seconds: self.started_at_unix_seconds,
            last_event_kind: self.last_event_kind.clone(),
            last_event_at_unix_seconds: self.last_event_at_unix_seconds,
            last_timer_at_unix_seconds: self.last_timer_at_unix_seconds,
            last_pipe_name: self.last_pipe_name.clone(),
            last_pipe_at_unix_seconds: self.last_pipe_at_unix_seconds,
            last_status_cache_write_at_unix_seconds: self.last_status_cache_write_at_unix_seconds,
            status_refresh_started_at_by_name: self.status_refresh_started_at_by_name.clone(),
        })
    }
}

pub(crate) fn unix_time_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(crate) fn event_kind(event: &Event) -> &'static str {
    match event {
        Event::TabUpdate(_) => "tab_update",
        Event::PaneUpdate(_) => "pane_update",
        Event::PermissionRequestResult(_) => "permission_request_result",
        Event::InputReceived => "input_received",
        Event::Timer(_) => "timer",
        Event::PaneClosed(_) => "pane_closed",
        Event::CommandPaneExited(_, _, _) => "command_pane_exited",
        _ => "other",
    }
}

impl State {
    pub(crate) fn initialize_orchestrator_heartbeat(&mut self) {
        let now = unix_time_seconds();
        self.orchestrator_heartbeat.started_at_unix_seconds = now;
        self.schedule_orchestrator_heartbeat_after(HEARTBEAT_INITIAL_FLUSH_DELAY);
    }

    pub(crate) fn record_orchestrator_event(&mut self, kind: &str) {
        let now = unix_time_seconds();
        self.orchestrator_heartbeat.last_event_kind = Some(kind.to_string());
        self.orchestrator_heartbeat.last_event_at_unix_seconds = Some(now);
    }

    pub(crate) fn record_orchestrator_timer(&mut self) {
        self.orchestrator_heartbeat.last_timer_at_unix_seconds = Some(unix_time_seconds());
    }

    pub(crate) fn record_orchestrator_pipe(&mut self, name: &str) {
        let now = unix_time_seconds();
        self.orchestrator_heartbeat.last_pipe_name = Some(name.to_string());
        self.orchestrator_heartbeat.last_pipe_at_unix_seconds = Some(now);
    }

    pub(crate) fn record_status_cache_write(&mut self) {
        self.orchestrator_heartbeat
            .last_status_cache_write_at_unix_seconds = Some(unix_time_seconds());
    }

    pub(crate) fn record_status_refresh_start(&mut self, name: &str) {
        self.orchestrator_heartbeat
            .status_refresh_started_at_by_name
            .insert(name.to_string(), unix_time_seconds());
    }

    pub(crate) fn handle_orchestrator_heartbeat_timer(&mut self) {
        let Some(next_flush) = self.orchestrator_heartbeat.next_flush else {
            self.schedule_orchestrator_heartbeat_after(HEARTBEAT_INITIAL_FLUSH_DELAY);
            return;
        };
        if Instant::now() < next_flush {
            return;
        }

        self.flush_orchestrator_heartbeat();
        self.schedule_orchestrator_heartbeat_after(HEARTBEAT_FLUSH_INTERVAL);
    }

    fn schedule_orchestrator_heartbeat_after(&mut self, delay: Duration) {
        self.orchestrator_heartbeat.next_flush = Some(Instant::now() + delay);
    }

    fn flush_orchestrator_heartbeat(&mut self) {
        if !self.permissions_granted {
            return;
        }

        let Some(runtime) = self.status_bar_cache_runtime.clone().or_else(|| {
            let session_env = get_session_environment_variables();
            resolve_status_bar_cache_runtime(&session_env)
        }) else {
            return;
        };
        self.status_bar_cache_runtime = Some(runtime.clone());

        let payload = self
            .orchestrator_heartbeat
            .payload(unix_time_seconds())
            .to_string();
        let command = [
            runtime.yzx_control_path.as_str(),
            "zellij",
            "status-cache-heartbeat",
            "--path",
            runtime.cache_path.as_str(),
            "--payload",
            payload.as_str(),
        ];
        run_command_with_env_variables_and_cwd(&command, runtime.env, runtime.cwd, BTreeMap::new());
    }
}
