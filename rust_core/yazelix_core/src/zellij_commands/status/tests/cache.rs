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

// Regression: usage widgets should first-paint from recent sibling/shared caches before the new window writes its status-bus cache.
#[test]
fn usage_widgets_render_from_existing_caches_before_status_bus_write() {
    let temp = tempfile::tempdir().unwrap();
    let sessions_dir = temp.path().join("state").join("sessions");
    let new_cache_path = sessions_dir.join("window_b").join("status_bar_cache.json");
    let now = unix_time_seconds();

    let claude_shared_path =
        claude_usage_shared_cache_path_from_status_cache_path(&new_cache_path).unwrap();
    write_json_value_atomic(
        &claude_shared_path,
        &json!({
            "schema_version": CLAUDE_USAGE_CACHE_SCHEMA_VERSION,
            "claude": {
                "updated_at_unix_seconds": now,
                "five_hour_tokens": 42_000_000u64,
                "weekly_tokens": 420_000_000u64,
                "five_hour_remaining_percent": 73u64,
                "weekly_remaining_percent": 81u64,
                "status": "ok"
            }
        }),
        "claude_usage_cache_test",
    )
    .unwrap();
    let codex_shared_path =
        codex_usage_shared_cache_path_from_status_cache_path(&new_cache_path).unwrap();
    write_json_value_atomic(
        &codex_shared_path,
        &json!({
            "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
            "codex": {
                "updated_at_unix_seconds": now,
                "five_hour_tokens": 138424632u64,
                "weekly_tokens": 1335519960u64,
                "five_hour_remaining_percent": 49u64,
                "weekly_remaining_percent": 80u64,
                "five_hour_reset_at_unix_seconds": now + 2 * HOUR_SECONDS,
                "weekly_reset_at_unix_seconds": now + 3 * DAY_SECONDS,
                "five_hour_window_seconds": 5 * HOUR_SECONDS,
                "weekly_window_seconds": 7 * DAY_SECONDS,
                "status": "ok"
            }
        }),
        "codex_usage_cache_test",
    )
    .unwrap();
    let opencode_go_shared_path =
        opencode_go_usage_shared_cache_path_from_status_cache_path(&new_cache_path).unwrap();
    write_json_value_atomic(
        &opencode_go_shared_path,
        &json!({
            "schema_version": OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
            "opencode_go": {
                "updated_at_unix_seconds": now,
                "five_hour_tokens": 0u64,
                "five_hour_remaining_percent": 100u64,
                "weekly_tokens": 85_000_000u64,
                "weekly_remaining_percent": 60u64,
                "monthly_tokens": 85_000_000u64,
                "monthly_remaining_percent": 80u64,
                "status": "ok"
            }
        }),
        "opencode_go_usage_cache_test",
    )
    .unwrap();

    let mut claude_cache =
        status_cache_value_for_widget_path(&new_cache_path, "claude_usage", now).unwrap();
    hydrate_status_cache_claude_usage(&mut claude_cache, &new_cache_path);
    assert_eq!(
        render_status_cache_widget(&claude_cache, "claude_usage").unwrap(),
        " [claude 5h|42M|73% wk|420M|81%]"
    );

    let mut codex_cache =
        status_cache_value_for_widget_path(&new_cache_path, "codex_usage", now).unwrap();
    hydrate_status_cache_codex_usage(&mut codex_cache, &new_cache_path);
    assert_eq!(
        render_status_cache_widget(&codex_cache, "codex_usage").unwrap(),
        " [codex 3h/5h 49% · 4d/7d 80%]"
    );

    let mut opencode_go_cache =
        status_cache_value_for_widget_path(&new_cache_path, "opencode_go_usage", now).unwrap();
    hydrate_status_cache_opencode_go_usage(&mut opencode_go_cache, &new_cache_path);
    assert_eq!(
        render_status_cache_widget(&opencode_go_cache, "opencode_go_usage").unwrap(),
        " [go 5h|0|100% wk|85M|60% mo|85M|80%]"
    );

    assert!(status_cache_value_for_widget_path(&new_cache_path, "workspace", now).is_none());
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

// Regression: zjstatus command execution can preserve only the cache path, so usage refresh still needs the sibling config snapshot.
#[test]
fn session_config_path_can_be_recovered_from_cache_path() {
    assert_eq!(
        session_config_path_from_values(
            None,
            Some(PathBuf::from("/tmp/session/status_bar_cache.json")),
        ),
        Some(PathBuf::from("/tmp/session/config_snapshot.json"))
    );
    assert_eq!(
        session_config_path_from_environ_bytes(
            b"PATH=/bin\0YAZELIX_SESSION_CONFIG_PATH=/tmp/session/config_snapshot.json\0",
        ),
        Some(PathBuf::from("/tmp/session/config_snapshot.json"))
    );
}

// Regression: refresh commands receive an explicit cache path from the plugin, so they must recover the sibling config snapshot without relying on ambient env.
#[test]
fn usage_widget_settings_can_be_recovered_from_cache_path() {
    let temp = tempfile::tempdir().unwrap();
    let cache_path = temp.path().join("window").join("status_bar_cache.json");
    let config_path = temp.path().join("window").join("config_snapshot.json");
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    fs::write(
        &config_path,
        json!({
            "normalized_config": {
                "zellij_widget_tray": ["claude_usage", "opencode_go_usage"],
                "zellij_claude_usage_display": "quota",
                "zellij_claude_usage_periods": ["week"],
                "zellij_opencode_go_usage_display": "quota",
                "zellij_opencode_go_usage_periods": ["5h", "month"]
            }
        })
        .to_string(),
    )
    .unwrap();

    assert!(usage_widget_enabled_from_status_cache_path(
        &cache_path,
        "opencode_go_usage"
    ));
    assert!(usage_widget_enabled_from_status_cache_path(
        &cache_path,
        "claude_usage"
    ));
    let settings = agent_usage_widget_settings_from_status_cache_path(&cache_path);
    assert_eq!(settings.claude_display, WindowedUsageDisplay::Quota);
    assert_eq!(settings.claude_periods, vec![WindowedUsagePeriod::Weekly]);
    assert_eq!(settings.codex_display, WindowedUsageDisplay::Quota);
    assert_eq!(settings.opencode_go_display, WindowedUsageDisplay::Quota);
    assert_eq!(
        settings.opencode_go_periods,
        vec![WindowedUsagePeriod::FiveHour, WindowedUsagePeriod::Monthly]
    );
}
