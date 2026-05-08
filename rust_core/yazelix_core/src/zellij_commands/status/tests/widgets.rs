// Test lane: default

use super::*;

// Defends: status-bus consumers reject unsupported producer schema versions instead of parsing stale payloads optimistically.
#[test]
fn status_bus_decode_rejects_unsupported_schema_version() {
    let err = decode_status_bus_snapshot(
        r#"{"schema_version":99,"active_tab_position":0,"managed_panes":{"editor_pane_id":null,"sidebar_pane_id":null},"focus_context":"unknown","layout":{"active_swap_layout_name":null,"sidebar_collapsed":null}}"#,
    )
    .unwrap_err();

    assert!(
        err.message()
            .contains("Unsupported pane-orchestrator status-bus schema_version")
    );
    assert!(
        err.remediation()
            .contains("supports status-bus schema_version 1")
    );
}

// Regression: zjstatus command widgets return plain text while the template owns style markup, so command stdout cannot print literal `#[fg=...]` tags.
#[test]
fn zjstatus_status_bus_workspace_widget_renders_plain_segment_and_hides_missing_facts() {
    let value = decode_status_bus_snapshot(
        r#"{"schema_version":1,"active_tab_position":0,"workspace":{"root":"/tmp/yazelix-demo","source":"explicit"},"managed_panes":{"editor_pane_id":"terminal:1","sidebar_pane_id":"terminal:2"},"focus_context":"sidebar","layout":{"active_swap_layout_name":"single_open","sidebar_collapsed":false},"sidebar_yazi":null,"transient_panes":{"popup":null,"menu":null},"extensions":{"ai_pane_activity":[{"tab_position":0,"provider":"claude","pane_id":"terminal:2","activity":"thinking","state":"thinking"}]}}"#,
    )
    .unwrap();
    let empty = decode_status_bus_snapshot(
        r#"{"schema_version":1,"active_tab_position":0,"workspace":null,"managed_panes":{"editor_pane_id":null,"sidebar_pane_id":null},"focus_context":"other","layout":{"active_swap_layout_name":null,"sidebar_collapsed":null},"sidebar_yazi":null,"transient_panes":{"popup":null,"menu":null},"extensions":{"ai_pane_activity":[]}}"#,
    )
    .unwrap();

    assert_eq!(render_zjstatus_workspace_widget(&value), " [yazelix-demo]");
    assert!(!render_zjstatus_workspace_widget(&value).contains("#["));
    assert_eq!(render_zjstatus_workspace_widget(&empty), "");
}

// Defends: the cursor widget renders mono and split cursor previews from cached launch facts without widening the status segment.
#[test]
fn status_cache_cursor_widget_renders_cached_launch_fact() {
    let mono = json!({
        "schema_version": STATUS_BAR_CACHE_SCHEMA_VERSION,
        "updated_at_unix_seconds": 1_000,
        "status_bus": status_cache_test_status_bus(),
        "agent_usage": {},
        "cursor": {
            "terminal": "ghostty",
            "name": "reef",
            "color": "#14D9A0",
            "family": "mono"
        }
    });
    let vertical_split = json!({
        "cursor": {
            "terminal": "ghostty",
            "name": "reef",
            "color": "#00e6ff",
            "family": "split",
            "divider": "vertical",
            "primary_color": "#00e6ff",
            "secondary_color": "#00ff66"
        }
    });
    let horizontal_split = json!({
        "cursor": {
            "terminal": "ghostty",
            "name": "magma",
            "color": "#ff1600",
            "family": "split",
            "divider": "horizontal",
            "primary_color": "#ff1600",
            "secondary_color": "#2a3340"
        }
    });
    let display_color_differs_from_split_primary = json!({
        "cursor": {
            "terminal": "ghostty",
            "name": "eclipse",
            "color": "#ffd400",
            "family": "split",
            "divider": "vertical",
            "primary_color": "#2e294e",
            "secondary_color": "#ffd400"
        }
    });
    let invalid_split = json!({
        "cursor": {
            "name": "magma",
            "color": "#ff1600",
            "family": "split",
            "divider": "horizontal",
            "primary_color": "#ff1600",
            "secondary_color": "hot"
        }
    });

    assert_eq!(
        render_status_cache_widget(&mono, "cursor").unwrap(),
        " #[fg=#14d9a0,bg=default,bold][#[fg=#14d9a0,bold]█#[fg=#14d9a0,bg=default,bold] reef]"
    );
    assert_eq!(
        render_status_cache_widget(&vertical_split, "cursor").unwrap(),
        " #[fg=#00e6ff,bg=default,bold][#[fg=#00e6ff,bg=#00ff66,bold]▌#[fg=#00e6ff,bg=default,bold] reef]"
    );
    assert_eq!(
        render_status_cache_widget(&horizontal_split, "cursor").unwrap(),
        " #[fg=#ff1600,bg=default,bold][#[fg=#ff1600,bg=#2a3340,bold]▀#[fg=#ff1600,bg=default,bold] magma]"
    );
    assert_eq!(
        render_status_cache_widget(&display_color_differs_from_split_primary, "cursor").unwrap(),
        " #[fg=#ffd400,bg=default,bold][#[fg=#2e294e,bg=#ffd400,bold]▌#[fg=#ffd400,bg=default,bold] eclipse]"
    );
    assert_eq!(
        render_status_cache_widget(&invalid_split, "cursor").unwrap(),
        " #[fg=#ff1600,bg=default,bold][#[fg=#ff1600,bold]█#[fg=#ff1600,bg=default,bold] magma]"
    );
    assert_eq!(
        render_status_cache_widget(&json!({"cursor": {"name": "n/a"}}), "cursor").unwrap(),
        " #[fg=#00ff88,bg=default,bold][#[fg=#00ff88,bold]█#[fg=#00ff88,bg=default,bold] n/a]"
    );
    assert_eq!(
        render_status_cache_widget(&json!({"cursor": {"name": ""}}), "cursor").unwrap(),
        ""
    );
}

// Defends: cursor status facts are copied from launch env as small terminal-scoped data, not by parsing config on every bar refresh.
#[test]
fn cursor_status_value_uses_non_empty_launch_env_values() {
    assert_eq!(
        cursor_status_value(
            Some(OsStr::new("ghostty")),
            Some(OsStr::new("magma")),
            Some(OsStr::new("#FF1600")),
            Some(OsStr::new("split")),
            Some(OsStr::new("horizontal")),
            Some(OsStr::new("#FF1600")),
            Some(OsStr::new("#2A3340")),
        ),
        Some(json!({
            "terminal": "ghostty",
            "name": "magma",
            "color": "#ff1600",
            "family": "split",
            "divider": "horizontal",
            "primary_color": "#ff1600",
            "secondary_color": "#2a3340"
        }))
    );
    assert_eq!(
        cursor_status_value(
            Some(OsStr::new("ghostty")),
            Some(OsStr::new("  ")),
            Some(OsStr::new("#ff1600")),
            Some(OsStr::new("split")),
            Some(OsStr::new("horizontal")),
            Some(OsStr::new("#ff1600")),
            Some(OsStr::new("#2a3340")),
        ),
        None
    );
}
