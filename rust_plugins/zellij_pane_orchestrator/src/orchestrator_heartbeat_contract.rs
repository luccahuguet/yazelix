use std::collections::BTreeMap;

use serde_json::{json, Value};

pub const ORCHESTRATOR_HEARTBEAT_SCHEMA_VERSION: i32 = 1;
pub const ORCHESTRATOR_HEARTBEAT_STALE_AFTER_SECONDS: u64 = 90;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrchestratorHeartbeatPayload {
    pub heartbeat_at_unix_seconds: u64,
    pub started_at_unix_seconds: u64,
    pub last_event_kind: Option<String>,
    pub last_event_at_unix_seconds: Option<u64>,
    pub last_timer_at_unix_seconds: Option<u64>,
    pub last_pipe_name: Option<String>,
    pub last_pipe_at_unix_seconds: Option<u64>,
    pub last_status_cache_write_at_unix_seconds: Option<u64>,
    pub status_refresh_started_at_by_name: BTreeMap<String, u64>,
}

pub fn build_orchestrator_heartbeat_payload(input: OrchestratorHeartbeatPayload) -> Value {
    let mut status_refreshes = serde_json::Map::new();
    for (name, started_at) in input.status_refresh_started_at_by_name {
        status_refreshes.insert(
            name,
            json!({
                "started_at_unix_seconds": started_at,
            }),
        );
    }

    json!({
        "schema_version": ORCHESTRATOR_HEARTBEAT_SCHEMA_VERSION,
        "heartbeat_at_unix_seconds": input.heartbeat_at_unix_seconds,
        "stale_after_seconds": ORCHESTRATOR_HEARTBEAT_STALE_AFTER_SECONDS,
        "started_at_unix_seconds": input.started_at_unix_seconds,
        "last_event": input.last_event_kind.zip(input.last_event_at_unix_seconds).map(|(kind, at)| {
            json!({
                "kind": kind,
                "at_unix_seconds": at,
            })
        }),
        "last_timer_at_unix_seconds": input.last_timer_at_unix_seconds,
        "last_pipe": input.last_pipe_name.zip(input.last_pipe_at_unix_seconds).map(|(name, at)| {
            json!({
                "name": name,
                "at_unix_seconds": at,
            })
        }),
        "last_status_cache_write_at_unix_seconds": input.last_status_cache_write_at_unix_seconds,
        "status_refreshes": status_refreshes,
    })
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::{
        build_orchestrator_heartbeat_payload, OrchestratorHeartbeatPayload,
        ORCHESTRATOR_HEARTBEAT_STALE_AFTER_SECONDS,
    };
    use serde_json::json;

    // Defends: pane-orchestrator heartbeat facts carry liveness timestamps without status-bar presentation markup.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn heartbeat_payload_exposes_liveness_facts_without_bar_formatting() {
        let payload = build_orchestrator_heartbeat_payload(OrchestratorHeartbeatPayload {
            heartbeat_at_unix_seconds: 30,
            started_at_unix_seconds: 10,
            last_event_kind: Some("timer".to_string()),
            last_event_at_unix_seconds: Some(20),
            last_timer_at_unix_seconds: Some(20),
            last_pipe_name: Some("toggle_transient_pane".to_string()),
            last_pipe_at_unix_seconds: Some(19),
            last_status_cache_write_at_unix_seconds: Some(18),
            status_refresh_started_at_by_name: [("codex_usage".to_string(), 17)]
                .into_iter()
                .collect(),
        });

        assert_eq!(
            payload,
            json!({
                "schema_version": 1,
                "heartbeat_at_unix_seconds": 30,
                "stale_after_seconds": ORCHESTRATOR_HEARTBEAT_STALE_AFTER_SECONDS,
                "started_at_unix_seconds": 10,
                "last_event": {
                    "kind": "timer",
                    "at_unix_seconds": 20
                },
                "last_timer_at_unix_seconds": 20,
                "last_pipe": {
                    "name": "toggle_transient_pane",
                    "at_unix_seconds": 19
                },
                "last_status_cache_write_at_unix_seconds": 18,
                "status_refreshes": {
                    "codex_usage": {
                        "started_at_unix_seconds": 17
                    }
                }
            })
        );
        assert!(!payload.to_string().contains("#["));
    }
}
