// Test lane: default

use super::*;

// Regression: zjstatus reads dynamic widgets from a local cache instead of invoking Zellij pipes from every bar command.
#[test]
fn status_cache_round_trip_renders_cached_workspace_fact() {
    let temp = tempfile::tempdir().unwrap();
    let cache_path = temp.path().join("window_a").join("status_bar_cache.json");

    run_zellij_status_cache_write(&[
        "--path".to_string(),
        cache_path.display().to_string(),
        "--payload".to_string(),
        STATUS_CACHE_TEST_PAYLOAD.to_string(),
    ])
    .unwrap();
    let cache = read_status_bar_cache_value(&cache_path).unwrap();

    assert_eq!(
        render_status_cache_widget(&cache, "workspace").unwrap(),
        " [yazelix-demo]"
    );
    assert!(
        !render_status_cache_widget(&cache, "workspace")
            .unwrap()
            .contains("#[")
    );
}

// Defends: heartbeat updates merge into the window-local cache without replacing status-bus or usage facts.
// Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
#[test]
fn status_cache_heartbeat_merge_preserves_cached_session_facts() {
    let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
    let status_bus_before = cache.get("status_bus").cloned();
    let agent_usage_before = cache.get("agent_usage").cloned();

    merge_orchestrator_heartbeat_into_cache(
        &mut cache,
        json!({
            "schema_version": 1,
            "heartbeat_at_unix_seconds": 2_000,
            "last_pipe": {
                "name": "focus_sidebar",
                "at_unix_seconds": 1_990
            },
            "status_refreshes": {
                "codex_usage": {
                    "started_at_unix_seconds": 1_980
                }
            }
        }),
    );
    merge_orchestrator_heartbeat_into_cache(
        &mut cache,
        json!({
            "schema_version": 1,
            "status_refreshes": {
                "codex_usage": {
                    "finished_at_unix_seconds": 2_010
                }
            }
        }),
    );

    assert_eq!(cache.get("status_bus").cloned(), status_bus_before);
    assert_eq!(cache.get("agent_usage").cloned(), agent_usage_before);
    assert_eq!(
        cache
            .pointer("/orchestrator_heartbeat/last_pipe/name")
            .and_then(Value::as_str),
        Some("focus_sidebar")
    );
    assert_eq!(
        cache
            .pointer("/orchestrator_heartbeat/status_refreshes/codex_usage/started_at_unix_seconds")
            .and_then(Value::as_u64),
        Some(1_980)
    );
    assert_eq!(
        cache
            .pointer(
                "/orchestrator_heartbeat/status_refreshes/codex_usage/finished_at_unix_seconds"
            )
            .and_then(Value::as_u64),
        Some(2_010)
    );
}

// Regression: status-bus cache rewrites must not erase heartbeat facts used to debug orchestrator stalls.
#[test]
fn status_cache_write_preserves_existing_heartbeat() {
    let temp = tempfile::tempdir().unwrap();
    let cache_path = temp.path().join("window_a").join("status_bar_cache.json");
    let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
    merge_orchestrator_heartbeat_into_cache(
        &mut cache,
        json!({
            "schema_version": 1,
            "heartbeat_at_unix_seconds": 2_000,
            "last_timer_at_unix_seconds": 1_990
        }),
    );
    write_status_bar_cache_value(&cache_path, &cache).unwrap();

    run_zellij_status_cache_write(&[
        "--path".to_string(),
        cache_path.display().to_string(),
        "--payload".to_string(),
        STATUS_CACHE_TEST_PAYLOAD.to_string(),
    ])
    .unwrap();

    let updated_cache = read_status_bar_cache_value(&cache_path).unwrap();
    assert_eq!(
        updated_cache
            .pointer("/orchestrator_heartbeat/last_timer_at_unix_seconds")
            .and_then(Value::as_u64),
        Some(1_990)
    );
}

// Defends: the status-cache write path carries all-tab activity on the same cache bus as existing widgets.
#[test]
fn status_cache_write_records_tab_activity_payload() {
    let temp = tempfile::tempdir().unwrap();
    let cache_path = temp.path().join("window_a").join("status_bar_cache.json");

    run_zellij_status_cache_write(&[
        "--path".to_string(),
        cache_path.display().to_string(),
        "--payload".to_string(),
        STATUS_CACHE_TEST_PAYLOAD.to_string(),
        "--tab-activity-payload".to_string(),
        r#"{"schema_version":1,"tabs":[{"tab_id":20,"tab_position":1,"base_name":"agent","active":false,"activity_state":"busy"}]}"#
            .to_string(),
    ])
    .unwrap();

    let cache = read_status_bar_cache_value(&cache_path).unwrap();
    assert_eq!(
        cache
            .pointer("/tab_activity/tabs/0/activity_state")
            .and_then(Value::as_str),
        Some("busy")
    );
}

// Regression: older status-cache writers must not erase tab activity used by the integrated bar-owned tab strip.
#[test]
fn status_cache_write_without_tab_activity_preserves_existing_tab_activity() {
    let temp = tempfile::tempdir().unwrap();
    let cache_path = temp.path().join("window_a").join("status_bar_cache.json");
    let mut cache = build_status_bar_cache_with_tab_activity_at(
        status_cache_test_status_bus(),
        Some(json!({
            "schema_version": 1,
            "tabs": [
                {
                    "tab_id": 20,
                    "tab_position": 1,
                    "base_name": "agent",
                    "active": false,
                    "activity_state": "alert"
                }
            ]
        })),
        1_000,
    );
    merge_orchestrator_heartbeat_into_cache(
        &mut cache,
        json!({
            "schema_version": 1,
            "heartbeat_at_unix_seconds": 2_000
        }),
    );
    write_status_bar_cache_value(&cache_path, &cache).unwrap();

    run_zellij_status_cache_write(&[
        "--path".to_string(),
        cache_path.display().to_string(),
        "--payload".to_string(),
        STATUS_CACHE_TEST_PAYLOAD.to_string(),
    ])
    .unwrap();

    let updated_cache = read_status_bar_cache_value(&cache_path).unwrap();
    assert_eq!(
        updated_cache
            .pointer("/tab_activity/tabs/0/activity_state")
            .and_then(Value::as_str),
        Some("alert")
    );
    assert_eq!(
        updated_cache
            .pointer("/orchestrator_heartbeat/heartbeat_at_unix_seconds")
            .and_then(Value::as_u64),
        Some(2_000)
    );
}

// Regression: zjstatus command execution can strip direct Yazelix cache env even though its Zellij parent still carries the launch env.
#[test]
fn status_cache_path_can_be_recovered_from_process_environ_bytes() {
    let explicit = status_bar_cache_path_from_environ_bytes(
        b"PATH=/bin\0YAZELIX_STATUS_BAR_CACHE_PATH=/tmp/window/status_bar_cache.json\0YAZELIX_SESSION_CONFIG_PATH=/tmp/other/config_snapshot.json\0",
    );
    assert_eq!(
        explicit,
        Some(PathBuf::from("/tmp/window/status_bar_cache.json"))
    );

    let derived = status_bar_cache_path_from_environ_bytes(
        b"PATH=/bin\0YAZELIX_SESSION_CONFIG_PATH=/tmp/session/config_snapshot.json\0",
    );
    assert_eq!(
        derived,
        Some(PathBuf::from("/tmp/session/status_bar_cache.json"))
    );
}
