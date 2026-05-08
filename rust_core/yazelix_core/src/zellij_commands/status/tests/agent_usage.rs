use super::*;

fn status_cache_path_for_temp(temp_path: &Path) -> PathBuf {
    temp_path
        .join("state")
        .join("sessions")
        .join("window_a")
        .join("status_bar_cache.json")
}

#[cfg(unix)]
fn write_tokenusage_provider_script(bin_dir: &Path, script: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    fs::create_dir_all(bin_dir).unwrap();
    let provider = bin_dir.join("tu");
    fs::write(&provider, script).unwrap();
    let mut permissions = fs::metadata(&provider).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&provider, permissions).unwrap();
    provider
}

// Defends: Claude usage mirrors the compact 5h/week token/quota contract selected by claude_usage_display.
#[test]
fn status_cache_claude_usage_renders_5h_week_display_modes() {
    let cache = json!({
        "claude_usage": {
            "five_hour_tokens": 15456373u64,
            "weekly_tokens": 66610005u64,
            "five_hour_remaining_percent": 49u64,
            "weekly_remaining_percent": 80u64
        }
    });

    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "claude_usage",
            &AgentUsageWidgetSettings {
                claude_display: WindowedUsageDisplay::Both,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [claude 5h|15.5M|49% wk|66.6M|80%]"
    );
    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "claude_usage",
            &AgentUsageWidgetSettings {
                claude_display: WindowedUsageDisplay::Token,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [claude 5h|15.5M wk|66.6M]"
    );
    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "claude_usage",
            &AgentUsageWidgetSettings {
                claude_display: WindowedUsageDisplay::Quota,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [claude 5h|49% wk|80%]"
    );
}

// Defends: Codex usage renders only the compact 5h/week token/quota contract selected by codex_usage_display.
#[test]
fn status_cache_codex_usage_renders_5h_week_display_modes() {
    let cache = json!({
        "codex_usage": {
            "updated_at_unix_seconds": 10u64,
            "five_hour_tokens": 138424632u64,
            "weekly_tokens": 1335519960u64,
            "five_hour_remaining_percent": 49u64,
            "weekly_remaining_percent": 80u64,
            "five_hour_reset_at_unix_seconds": 9610u64,
            "weekly_reset_at_unix_seconds": 241210u64,
            "five_hour_window_seconds": 18000u64,
            "weekly_window_seconds": 604800u64
        }
    });

    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "codex_usage",
            &AgentUsageWidgetSettings {
                codex_display: WindowedUsageDisplay::Both,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [codex 2h20m/5h 138M 49% · 4d5h/7d 1.34B 80%]"
    );
    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "codex_usage",
            &AgentUsageWidgetSettings {
                codex_display: WindowedUsageDisplay::Token,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [codex 2h20m/5h 138M · 4d5h/7d 1.34B]"
    );
    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "codex_usage",
            &AgentUsageWidgetSettings {
                codex_display: WindowedUsageDisplay::Quota,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [codex 2h20m/5h 49% · 4d5h/7d 80%]"
    );
}

// Regression: Codex window labels show current window position instead of time remaining until reset.
#[test]
fn codex_window_label_reports_elapsed_position() {
    assert_eq!(
        format_reset_window_label(2 * DAY_SECONDS, 7 * DAY_SECONDS, 7 * HOUR_SECONDS),
        Some("5d7h/7d".to_string())
    );
    assert_eq!(
        format_reset_window_label(5 * HOUR_SECONDS, 5 * HOUR_SECONDS, 10 * MINUTE_SECONDS),
        Some("10m/5h".to_string())
    );
}

// Regression: quota-only Codex widgets must remain visible while official quota data is temporarily unavailable.
#[test]
fn status_cache_codex_usage_quota_mode_renders_partial_token_cache() {
    let cache = json!({
        "codex_usage": {
            "updated_at_unix_seconds": 10u64,
            "five_hour_tokens": 4015883u64,
            "weekly_tokens": 106335620u64,
            "status": "partial",
            "quota_backoff_until_unix_seconds": 1810u64
        }
    });

    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "codex_usage",
            &AgentUsageWidgetSettings {
                codex_display: WindowedUsageDisplay::Quota,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [codex 5h n/a · wk n/a]"
    );
}

// Defends: OpenCode Go usage renders configurable 5h/week/month token/quota windows with the short `go` label.
#[test]
fn status_cache_opencode_go_usage_renders_configured_window_display_modes() {
    let cache = json!({
        "opencode_go_usage": {
            "five_hour_tokens": 138424632u64,
            "weekly_tokens": 1335519960u64,
            "monthly_tokens": 2220000000u64,
            "five_hour_remaining_percent": 49u64,
            "weekly_remaining_percent": 80u64,
            "monthly_remaining_percent": 70u64
        }
    });

    let monthly_periods = vec![
        WindowedUsagePeriod::FiveHour,
        WindowedUsagePeriod::Weekly,
        WindowedUsagePeriod::Monthly,
    ];

    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "opencode_go_usage",
            &AgentUsageWidgetSettings {
                opencode_go_display: WindowedUsageDisplay::Both,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [go 5h|138M|49% wk|1.34B|80% mo|2.22B|70%]"
    );
    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "opencode_go_usage",
            &AgentUsageWidgetSettings {
                opencode_go_periods: vec![WindowedUsagePeriod::Weekly],
                opencode_go_display: WindowedUsageDisplay::Both,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [go wk|1.34B|80%]"
    );
    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "opencode_go_usage",
            &AgentUsageWidgetSettings {
                opencode_go_periods: monthly_periods.clone(),
                opencode_go_display: WindowedUsageDisplay::Token,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [go 5h|138M wk|1.34B mo|2.22B]"
    );
    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "opencode_go_usage",
            &AgentUsageWidgetSettings {
                opencode_go_periods: monthly_periods,
                opencode_go_display: WindowedUsageDisplay::Quota,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [go 5h|49% wk|80% mo|70%]"
    );
}

// Defends: tokenusage JSON shape for active-block, weekly, and official quota facts maps to the compact widget contract.
#[test]
fn tokenusage_json_parsers_read_windows_and_official_quota() {
    let active = json!({
        "blocks": [
            {"isActive": false, "totals": {"total_tokens": 10u64}},
            {"isActive": true, "totals": {"total_tokens": 138424632u64}}
        ]
    });
    let weekly = json!({
        "weekly": [
            {"totals": {"total_tokens": 1335519960u64}},
            {"totals": {"total_tokens": 1u64}}
        ]
    });
    let official = json!({
        "official_codex": {
            "primary_used_percent": 51.0,
            "secondary_used_percent": 20.0,
            "primary_resets_at": 8_200u64,
            "primary_window_mins": 300u64,
            "secondary_resets_at": 260_200u64,
            "secondary_window_mins": 10_080u64
        },
        "official_claude": {
            "primary_used_percent": 25.0,
            "secondary_used_percent": 35.0
        }
    });

    let codex_quota =
        tokenusage_quota_from_official_json(&official, TokenusageWindowedProvider::Codex);
    let claude_quota =
        tokenusage_quota_from_official_json(&official, TokenusageWindowedProvider::Claude);

    assert_eq!(
        tokenusage_active_block_tokens_from_json(&active),
        Some(138424632)
    );
    assert_eq!(
        tokenusage_weekly_tokens_from_json(&weekly),
        Some(1335519960)
    );
    assert_eq!(codex_quota.five_hour_remaining_percent, Some(49));
    assert_eq!(codex_quota.weekly_remaining_percent, Some(80));
    assert_eq!(codex_quota.five_hour_reset_at_unix_seconds, Some(8_200));
    assert_eq!(codex_quota.weekly_reset_at_unix_seconds, Some(260_200));
    assert_eq!(codex_quota.five_hour_window_seconds, Some(18_000));
    assert_eq!(codex_quota.weekly_window_seconds, Some(604_800));
    assert_eq!(claude_quota.five_hour_remaining_percent, Some(75));
    assert_eq!(claude_quota.weekly_remaining_percent, Some(65));
}

// Regression: the dedicated Codex refresh writes a shared cache that new windows hydrate before rendering.
#[cfg(unix)]
#[test]
fn status_cache_codex_usage_refresh_writes_shared_combined_cache() {
    let temp = tempfile::tempdir().unwrap();
    let bin_dir = temp.path().join("bin");
    write_tokenusage_provider_script(
        &bin_dir,
        r#"#!/usr/bin/env sh
if [ "$1" = "blocks" ] && [ "$2" = "--active" ]; then
  case " $* " in
*" --official-limits "*)
  printf '%s\n' '{"official_codex":{"primary_used_percent":51.0,"secondary_used_percent":20.0,"primary_resets_at":8200,"primary_window_mins":300,"secondary_resets_at":260200,"secondary_window_mins":10080}}'
  ;;
*)
  printf '%s\n' '{"blocks":[{"isActive":true,"totals":{"total_tokens":138424632}}]}'
  ;;
  esac
  exit 0
fi
if [ "$1" = "weekly" ]; then
  printf '%s\n' '{"weekly":[{"totals":{"total_tokens":1335519960}}]}'
  exit 0
fi
exit 64
"#,
    );
    let status_cache_path = status_cache_path_for_temp(temp.path());
    let shared_path =
        codex_usage_shared_cache_path_from_status_cache_path(&status_cache_path).unwrap();

    let refreshed = refresh_codex_usage_shared_cache(
        &shared_path,
        Some(bin_dir.as_os_str()),
        1_000,
        600,
        1_800,
        Duration::from_secs(1),
    )
    .unwrap();
    let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
    hydrate_status_cache_codex_usage(&mut cache, &status_cache_path);

    assert!(refreshed);
    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "codex_usage",
            &AgentUsageWidgetSettings {
                codex_display: WindowedUsageDisplay::Both,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [codex 3h/5h 138M 49% · 4d/7d 1.34B 80%]"
    );
}

// Regression: a partial Codex refresh must not erase a known 5h token count while the official reset window is unchanged.
#[cfg(unix)]
#[test]
fn codex_usage_refresh_preserves_missing_tokens_for_same_reset_window() {
    let temp = tempfile::tempdir().unwrap();
    let bin_dir = temp.path().join("bin");
    write_tokenusage_provider_script(
        &bin_dir,
        r#"#!/usr/bin/env sh
if [ "$1" = "blocks" ] && [ "$2" = "--active" ]; then
  case " $* " in
*" --official-limits "*)
  printf '%s\n' '{"official_codex":{"primary_used_percent":51.0,"secondary_used_percent":20.0,"primary_resets_at":8200,"primary_window_mins":300,"secondary_resets_at":260200,"secondary_window_mins":10080}}'
  exit 0
  ;;
*)
  exit 65
  ;;
  esac
fi
if [ "$1" = "weekly" ]; then
  printf '%s\n' '{"weekly":[{"totals":{"total_tokens":1335519960}}]}'
  exit 0
fi
exit 64
"#,
    );

    let status_cache_path = status_cache_path_for_temp(temp.path());
    let shared_path =
        codex_usage_shared_cache_path_from_status_cache_path(&status_cache_path).unwrap();
    write_json_value_atomic(
        &shared_path,
        &json!({
            "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
            "codex": {
                "updated_at_unix_seconds": 0u64,
                "five_hour_tokens": 999000u64,
                "weekly_tokens": 1000000000u64,
                "five_hour_remaining_percent": 60u64,
                "weekly_remaining_percent": 50u64,
                "five_hour_reset_at_unix_seconds": 8200u64,
                "weekly_reset_at_unix_seconds": 260200u64,
                "five_hour_window_seconds": 18000u64,
                "weekly_window_seconds": 604800u64
            }
        }),
        "codex_usage_cache_test",
    )
    .unwrap();

    let refreshed = refresh_codex_usage_shared_cache(
        &shared_path,
        Some(bin_dir.as_os_str()),
        1_000,
        600,
        1_800,
        Duration::from_secs(1),
    )
    .unwrap();
    let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
    hydrate_status_cache_codex_usage(&mut cache, &status_cache_path);

    assert!(refreshed);
    assert_eq!(
        cache
            .get("codex_usage")
            .and_then(|entry| entry.get("five_hour_tokens"))
            .and_then(Value::as_u64),
        Some(999000)
    );
    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "codex_usage",
            &AgentUsageWidgetSettings {
                codex_display: WindowedUsageDisplay::Both,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [codex 3h/5h 999k 49% · 4d/7d 1.34B 80%]"
    );
}

// Regression: transient official quota failures must not replace a previously good Codex widget with n/a labels.
#[cfg(unix)]
#[test]
fn codex_usage_refresh_preserves_previous_quota_during_probe_backoff() {
    let temp = tempfile::tempdir().unwrap();
    let bin_dir = temp.path().join("bin");
    write_tokenusage_provider_script(
        &bin_dir,
        r#"#!/usr/bin/env sh
if [ "$1" = "blocks" ] && [ "$2" = "--active" ]; then
  case " $* " in
*" --official-limits "*)
  exit 65
  ;;
*)
  printf '%s\n' '{"blocks":[{"is_active":true,"totals":{"total_tokens":999000}}]}'
  exit 0
  ;;
  esac
fi
if [ "$1" = "weekly" ]; then
  printf '%s\n' '{"weekly":[{"totals":{"total_tokens":1000000}}]}'
  exit 0
fi
exit 64
"#,
    );

    let status_cache_path = status_cache_path_for_temp(temp.path());
    let shared_path =
        codex_usage_shared_cache_path_from_status_cache_path(&status_cache_path).unwrap();
    write_json_value_atomic(
        &shared_path,
        &json!({
            "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
            "codex": {
                "updated_at_unix_seconds": 0u64,
                "five_hour_tokens": 138424632u64,
                "weekly_tokens": 1335519960u64,
                "five_hour_remaining_percent": 49u64,
                "weekly_remaining_percent": 80u64,
                "five_hour_reset_at_unix_seconds": 10000u64,
                "weekly_reset_at_unix_seconds": 260200u64,
                "five_hour_window_seconds": 18000u64,
                "weekly_window_seconds": 604800u64,
                "status": "ok"
            }
        }),
        "codex_usage_cache_test",
    )
    .unwrap();

    let refreshed = refresh_codex_usage_shared_cache(
        &shared_path,
        Some(bin_dir.as_os_str()),
        1_000,
        600,
        1_800,
        Duration::from_secs(1),
    )
    .unwrap();
    let shared_cache = read_codex_usage_shared_cache_value(&shared_path).unwrap();
    let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
    hydrate_status_cache_codex_usage(&mut cache, &status_cache_path);

    assert!(refreshed);
    assert_eq!(
        shared_cache
            .get("codex")
            .and_then(|entry| entry.get("quota_backoff_until_unix_seconds"))
            .and_then(Value::as_u64),
        Some(2_800)
    );
    assert_eq!(
        shared_cache
            .get("codex")
            .and_then(|entry| entry.get("status"))
            .and_then(Value::as_str),
        Some("partial")
    );
    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "codex_usage",
            &AgentUsageWidgetSettings {
                codex_display: WindowedUsageDisplay::Both,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [codex 2h30m/5h 999k 49% · 4d/7d 1M 80%]"
    );
}

// Regression: runtime skew must not let old Codex cache writers overwrite the cache file read by a newer schema.
#[test]
fn status_cache_codex_usage_uses_schema_scoped_shared_cache_path() {
    let temp = tempfile::tempdir().unwrap();
    let status_cache_path = status_cache_path_for_temp(temp.path());
    let shared_path =
        codex_usage_shared_cache_path_from_status_cache_path(&status_cache_path).unwrap();
    assert_eq!(
        shared_path.file_name().and_then(|name| name.to_str()),
        Some("codex_usage_cache_v2.json")
    );

    write_json_value_atomic(
        &shared_path.with_file_name("codex_usage_cache.json"),
        &json!({
            "schema_version": 1,
            "codex": {
                "updated_at_unix_seconds": 1_000u64,
                "five_hour_tokens": 138424632u64,
                "weekly_tokens": 1335519960u64,
                "five_hour_remaining_percent": 49u64,
                "weekly_remaining_percent": 80u64,
                "status": "ok"
            }
        }),
        "codex_usage_cache_test",
    )
    .unwrap();

    let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
    hydrate_status_cache_codex_usage(&mut cache, &status_cache_path);
    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "codex_usage",
            &AgentUsageWidgetSettings {
                codex_display: WindowedUsageDisplay::Both,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        ""
    );

    write_json_value_atomic(
        &shared_path,
        &json!({
            "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
            "codex": {
                "updated_at_unix_seconds": 1_000u64,
                "five_hour_tokens": 138424632u64,
                "weekly_tokens": 1335519960u64,
                "five_hour_remaining_percent": 49u64,
                "weekly_remaining_percent": 80u64,
                "five_hour_reset_at_unix_seconds": 8_200u64,
                "weekly_reset_at_unix_seconds": 260_200u64,
                "five_hour_window_seconds": 18_000u64,
                "weekly_window_seconds": 604_800u64,
                "status": "ok"
            }
        }),
        "codex_usage_cache_test",
    )
    .unwrap();

    let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
    hydrate_status_cache_codex_usage(&mut cache, &status_cache_path);
    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "codex_usage",
            &AgentUsageWidgetSettings {
                codex_display: WindowedUsageDisplay::Both,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [codex 3h/5h 138M 49% · 4d/7d 1.34B 80%]"
    );
}

fn write_opencode_go_usage_test_db(path: &Path, now: u64) {
    let connection = Connection::open(path).unwrap();
    connection
        .execute_batch(
            r#"
            CREATE TABLE message (
                id text PRIMARY KEY,
                session_id text NOT NULL,
                time_created integer NOT NULL,
                time_updated integer NOT NULL,
                data text NOT NULL
            );
            "#,
        )
        .unwrap();
    let rows = [
        (
            "within_five_hour",
            now.saturating_sub(60),
            r#"{"role":"assistant","providerID":"opencode-go","cost":3.0,"tokens":{"input":1000000,"output":2000000,"reasoning":3000000,"cache":{"read":4000000,"write":5000000}}}"#,
        ),
        (
            "within_week",
            now.saturating_sub(6 * 60 * 60),
            r#"{"role":"assistant","providerID":"opencode-go","cost":12.0,"tokens":{"total":85000000}}"#,
        ),
        (
            "within_month",
            now.saturating_sub(8 * 24 * 60 * 60),
            r#"{"role":"assistant","providerID":"opencode-go","cost":15.0,"tokens":{"total":200000000}}"#,
        ),
        (
            "wrong_provider",
            now.saturating_sub(60),
            r#"{"role":"assistant","providerID":"opencode","cost":99.0,"tokens":{"total":900000000}}"#,
        ),
        (
            "wrong_role",
            now.saturating_sub(60),
            r#"{"role":"user","providerID":"opencode-go","cost":99.0,"tokens":{"total":900000000}}"#,
        ),
    ];
    for (id, created_at, data) in rows {
        let created_at = unix_millis_from_seconds_saturating(created_at);
        connection
            .execute(
                "INSERT INTO message (id, session_id, time_created, time_updated, data) VALUES (?1, 'session', ?2, ?2, ?3)",
                rusqlite::params![id, created_at, data],
            )
            .unwrap();
    }
}

// Defends: OpenCode Go usage reads only assistant rows from OpenCode's SQLite store and converts official dollar limits to remaining percentages.
#[test]
fn opencode_go_sqlite_reader_filters_provider_and_computes_quota_windows() {
    let temp = tempfile::tempdir().unwrap();
    let db_path = temp.path().join("opencode.db");
    let now = 2_000_000u64;
    write_opencode_go_usage_test_db(&db_path, now);

    let facts = collect_opencode_go_usage_facts_from_dbs(&[db_path], now);

    assert_eq!(facts.five_hour_tokens, Some(15_000_000));
    assert_eq!(facts.weekly_tokens, Some(100_000_000));
    assert_eq!(facts.monthly_tokens, Some(300_000_000));
    assert_eq!(facts.five_hour_remaining_percent, Some(75));
    assert_eq!(facts.weekly_remaining_percent, Some(50));
    assert_eq!(facts.monthly_remaining_percent, Some(50));
}

// Regression: a quiet 5h OpenCode Go window should still render quota instead of disappearing from the combined widget.
#[test]
fn opencode_go_sqlite_reader_keeps_empty_window_quota_facts() {
    let temp = tempfile::tempdir().unwrap();
    let db_path = temp.path().join("opencode.db");
    let now = 2_000_000u64;
    let connection = Connection::open(&db_path).unwrap();
    connection
        .execute_batch(
            r#"
            CREATE TABLE message (
                id text PRIMARY KEY,
                session_id text NOT NULL,
                time_created integer NOT NULL,
                time_updated integer NOT NULL,
                data text NOT NULL
            );
            "#,
        )
        .unwrap();
    let created_at = unix_millis_from_seconds_saturating(now.saturating_sub(6 * 60 * 60));
    connection
        .execute(
            "INSERT INTO message (id, session_id, time_created, time_updated, data) VALUES ('within_week', 'session', ?1, ?1, ?2)",
            rusqlite::params![
                created_at,
                r#"{"role":"assistant","providerID":"opencode-go","cost":12.0,"tokens":{"total":85000000}}"#
            ],
        )
        .unwrap();

    let facts = collect_opencode_go_usage_facts_from_dbs(&[db_path], now);

    assert_eq!(facts.five_hour_tokens, Some(0));
    assert_eq!(facts.five_hour_remaining_percent, Some(100));
    assert_eq!(facts.weekly_tokens, Some(85_000_000));
    assert_eq!(facts.weekly_remaining_percent, Some(60));
    assert_eq!(facts.monthly_tokens, Some(85_000_000));
    assert_eq!(facts.monthly_remaining_percent, Some(80));

    let cache = json!({
        "schema_version": 1,
        "updated_at_unix_seconds": now,
        "opencode_go_usage": {
            "five_hour_tokens": facts.five_hour_tokens,
            "five_hour_remaining_percent": facts.five_hour_remaining_percent,
            "weekly_tokens": facts.weekly_tokens,
            "weekly_remaining_percent": facts.weekly_remaining_percent,
            "monthly_tokens": facts.monthly_tokens,
            "monthly_remaining_percent": facts.monthly_remaining_percent
        }
    });
    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "opencode_go_usage",
            &AgentUsageWidgetSettings::default(),
        )
        .unwrap(),
        " [go 5h|0|100% wk|85M|60% mo|85M|80%]"
    );
}

// Regression: the dedicated OpenCode Go refresh writes a shared cache that new windows hydrate before rendering.
#[test]
fn status_cache_opencode_go_usage_refresh_writes_shared_combined_cache() {
    let temp = tempfile::tempdir().unwrap();
    let db_path = temp.path().join("opencode.db");
    let now = 2_000_000;
    write_opencode_go_usage_test_db(&db_path, now);
    let status_cache_path = status_cache_path_for_temp(temp.path());
    let shared_path =
        opencode_go_usage_shared_cache_path_from_status_cache_path(&status_cache_path).unwrap();

    let refreshed =
        refresh_opencode_go_usage_shared_cache_from_dbs(&shared_path, &[db_path], now, 1_800)
            .unwrap();
    let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), now);
    hydrate_status_cache_opencode_go_usage(&mut cache, &status_cache_path);

    assert!(refreshed);
    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "opencode_go_usage",
            &AgentUsageWidgetSettings {
                opencode_go_periods: vec![
                    WindowedUsagePeriod::FiveHour,
                    WindowedUsagePeriod::Weekly,
                    WindowedUsagePeriod::Monthly,
                ],
                opencode_go_display: WindowedUsageDisplay::Both,
                ..AgentUsageWidgetSettings::default()
            },
        )
        .unwrap(),
        " [go 5h|15M|75% wk|100M|50% mo|300M|50%]"
    );
}

// Regression: old OpenCode Go shared caches without complete 5h/week/month fields must refresh instead of hiding the 5h window.
#[test]
fn opencode_go_usage_shared_cache_rejects_partial_fresh_shape() {
    let temp = tempfile::tempdir().unwrap();
    let shared_path = temp.path().join("opencode_go_usage_cache.json");

    write_json_value_atomic(
        &shared_path,
        &json!({
            "schema_version": OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
            "opencode_go": {
                "updated_at_unix_seconds": 1_000u64,
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
    assert!(!opencode_go_usage_shared_cache_is_fresh(
        &shared_path,
        1_001,
        600
    ));

    write_json_value_atomic(
        &shared_path,
        &json!({
            "schema_version": OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
            "opencode_go": {
                "updated_at_unix_seconds": 1_001u64,
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
    assert!(opencode_go_usage_shared_cache_is_fresh(
        &shared_path,
        1_002,
        600
    ));
}

// Defends: shared Codex usage caches have explicit freshness and error backoff so multiple Yazelix windows do not stampede provider calls.
#[test]
fn codex_usage_shared_cache_respects_freshness_and_backoff() {
    let temp = tempfile::tempdir().unwrap();
    let shared_path = temp.path().join("codex_usage_cache.json");

    write_json_value_atomic(
        &shared_path,
        &json!({
            "schema_version": 1,
            "codex": {
                "updated_at_unix_seconds": 1_000u64,
                "five_hour_tokens": 138424632u64,
                "weekly_tokens": 1335519960u64,
                "five_hour_remaining_percent": 49u64,
                "weekly_remaining_percent": 80u64
            }
        }),
        "codex_usage_cache_test",
    )
    .unwrap();
    assert!(!codex_usage_shared_cache_is_fresh(&shared_path, 1_100, 600));

    write_json_value_atomic(
        &shared_path,
        &json!({
            "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
            "codex": {
                "updated_at_unix_seconds": 1_000u64,
                "five_hour_tokens": 138424632u64,
                "weekly_tokens": 1335519960u64,
                "five_hour_remaining_percent": 49u64,
                "weekly_remaining_percent": 80u64
            }
        }),
        "codex_usage_cache_test",
    )
    .unwrap();
    assert!(!codex_usage_shared_cache_is_fresh(&shared_path, 1_100, 600));

    write_json_value_atomic(
        &shared_path,
        &json!({
            "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
            "codex": {
                "updated_at_unix_seconds": 1_000u64,
                "five_hour_tokens": 138424632u64,
                "weekly_tokens": 1335519960u64,
                "five_hour_remaining_percent": 49u64,
                "weekly_remaining_percent": 80u64,
                "five_hour_reset_at_unix_seconds": 8_200u64,
                "weekly_reset_at_unix_seconds": 260_200u64,
                "five_hour_window_seconds": 18_000u64,
                "weekly_window_seconds": 604_800u64
            }
        }),
        "codex_usage_cache_test",
    )
    .unwrap();
    assert!(codex_usage_shared_cache_is_fresh(&shared_path, 1_100, 600));
    assert!(!codex_usage_shared_cache_is_fresh(&shared_path, 1_700, 600));

    write_json_value_atomic(
        &shared_path,
        &json!({
            "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
            "codex": {
                "updated_at_unix_seconds": 1_700u64,
                "five_hour_tokens": 138424632u64,
                "weekly_tokens": 1335519960u64,
                "error": "quota unavailable",
                "backoff_until_unix_seconds": 2_000u64
            }
        }),
        "codex_usage_cache_test",
    )
    .unwrap();
    assert!(!codex_usage_shared_cache_is_fresh(&shared_path, 1_701, 600));
    assert!(!codex_usage_shared_cache_is_backing_off(
        &shared_path,
        1_999
    ));

    write_json_value_atomic(
        &shared_path,
        &json!({
            "schema_version": CODEX_USAGE_CACHE_SCHEMA_VERSION,
            "codex": {
                "updated_at_unix_seconds": 1_700u64,
                "error": "quota unavailable",
                "backoff_until_unix_seconds": 2_000u64
            }
        }),
        "codex_usage_cache_test",
    )
    .unwrap();
    assert!(codex_usage_shared_cache_is_backing_off(&shared_path, 1_999));
    assert!(!codex_usage_shared_cache_is_backing_off(
        &shared_path,
        2_000
    ));
}

// Regression: the dedicated Claude refresh writes a shared 5h/week token/quota cache that new windows hydrate before rendering.
#[cfg(unix)]
#[test]
fn status_cache_claude_usage_refresh_writes_shared_combined_cache() {
    let temp = tempfile::tempdir().unwrap();
    let bin_dir = temp.path().join("bin");
    write_tokenusage_provider_script(
        &bin_dir,
        r#"#!/usr/bin/env sh
if [ "$1" = "blocks" ] && [ "$2" = "--active" ]; then
  case " $* " in
*" --official-limits "*)
  printf '%s\n' '{"official_claude":{"primary_used_percent":25.0,"secondary_used_percent":35.0}}'
  ;;
*)
  printf '%s\n' '{"blocks":[{"is_active":true,"totals":{"total_tokens":15456373}}]}'
  ;;
  esac
  exit 0
fi
if [ "$1" = "weekly" ]; then
  printf '%s\n' '{"weekly":[{"totals":{"total_tokens":66610005}}]}'
  exit 0
fi
exit 64
"#,
    );
    let status_cache_path = status_cache_path_for_temp(temp.path());
    let shared_path =
        claude_usage_shared_cache_path_from_status_cache_path(&status_cache_path).unwrap();

    let refreshed = refresh_tokenusage_windowed_usage_shared_cache(
        &shared_path,
        TokenusageWindowedProvider::Claude,
        Some(bin_dir.as_os_str()),
        1_000,
        600,
        1_800,
        Duration::from_secs(1),
    );
    let mut cache = build_status_bar_cache_at(status_cache_test_status_bus(), 1_000);
    hydrate_status_cache_claude_usage(&mut cache, &status_cache_path);

    assert!(refreshed.unwrap());
    assert_eq!(
        render_status_cache_widget_with_agent_usage_settings(
            &cache,
            "claude_usage",
            &AgentUsageWidgetSettings::default(),
        )
        .unwrap(),
        " [claude 5h|15.5M|75% wk|66.6M|65%]"
    );
}

// Regression: logged-out Claude quota probes must back off without stopping cheap local token refreshes.
#[cfg(unix)]
#[test]
fn tokenusage_windowed_refresh_backs_off_missing_quota_only() {
    let temp = tempfile::tempdir().unwrap();
    let bin_dir = temp.path().join("bin");
    let calls_path = temp.path().join("tu_calls.log");
    write_tokenusage_provider_script(
        &bin_dir,
        &format!(
            r#"#!/usr/bin/env sh
printf '%s\n' "$*" >> '{}'
if [ "$1" = "blocks" ] && [ "$2" = "--active" ]; then
  case " $* " in
*" --official-limits "*)
  printf '%s\n' '{{"official_claude":null}}'
  ;;
*)
  printf '%s\n' '{{"blocks":[{{"is_active":true,"totals":{{"total_tokens":15456373}}}}]}}'
  ;;
  esac
  exit 0
fi
if [ "$1" = "weekly" ]; then
  printf '%s\n' '{{"weekly":[{{"totals":{{"total_tokens":66610005}}}}]}}'
  exit 0
fi
exit 64
"#,
            calls_path.display()
        ),
    );
    let shared_path = temp.path().join("claude_usage_cache.json");

    assert!(
        refresh_tokenusage_windowed_usage_shared_cache(
            &shared_path,
            TokenusageWindowedProvider::Claude,
            Some(bin_dir.as_os_str()),
            1_000,
            10,
            1_800,
            Duration::from_secs(1),
        )
        .unwrap()
    );
    assert!(tokenusage_windowed_usage_quota_is_backing_off(
        &shared_path,
        TokenusageWindowedProvider::Claude,
        1_001,
    ));
    assert!(
        refresh_tokenusage_windowed_usage_shared_cache(
            &shared_path,
            TokenusageWindowedProvider::Claude,
            Some(bin_dir.as_os_str()),
            1_010,
            10,
            1_800,
            Duration::from_secs(1),
        )
        .unwrap()
    );

    let calls = fs::read_to_string(calls_path).unwrap();
    assert_eq!(
        calls
            .lines()
            .filter(|line| line.contains("--official-limits"))
            .count(),
        1
    );
    assert_eq!(
        calls
            .lines()
            .filter(|line| line.starts_with("blocks --active --json --offline"))
            .count(),
        2
    );
    assert_eq!(
        calls
            .lines()
            .filter(|line| line.starts_with("weekly --json --offline"))
            .count(),
        2
    );
}

// Regression: hung tokenusage providers are killed quickly so the cache producer cannot recreate the CPU-spike failure mode.
#[cfg(unix)]
#[test]
fn tokenusage_windowed_refresh_times_out_hung_provider() {
    let temp = tempfile::tempdir().unwrap();
    let bin_dir = temp.path().join("bin");
    write_tokenusage_provider_script(&bin_dir, "#!/usr/bin/env sh\nsleep 5\n");
    let started = Instant::now();
    let shared_path = temp.path().join("claude_usage_cache.json");

    let refreshed = refresh_tokenusage_windowed_usage_shared_cache(
        &shared_path,
        TokenusageWindowedProvider::Claude,
        Some(bin_dir.as_os_str()),
        1_000,
        10,
        1_800,
        Duration::from_millis(50),
    )
    .unwrap();

    assert!(refreshed);
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(
        read_claude_usage_shared_cache_value(&shared_path)
            .and_then(|cache| cache.pointer("/claude/status").cloned())
            .and_then(|status| status.as_str().map(str::to_string)),
        Some("error".to_string())
    );
}
