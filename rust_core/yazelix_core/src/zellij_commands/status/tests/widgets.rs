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

// Regression: bootstrap workspace roots are startup fallbacks, not active-tab labels, so legacy cache widgets must not display them as authoritative.
#[test]
fn zjstatus_workspace_widget_hides_bootstrap_workspace_roots() {
    let value = decode_status_bus_snapshot(
        r#"{"schema_version":1,"active_tab_position":0,"workspace":{"root":"/tmp/evo-import","source":"bootstrap"},"managed_panes":{"editor_pane_id":null,"sidebar_pane_id":null},"focus_context":"other","layout":{"active_swap_layout_name":null,"sidebar_collapsed":null},"sidebar_yazi":null,"transient_panes":{"popup":null,"menu":null},"extensions":{"ai_pane_activity":[]}}"#,
    )
    .unwrap();

    assert_eq!(render_zjstatus_workspace_widget(&value), "");
}

// Defends: non-workspace widgets are owned by yazelix_zellij_bar_widget, not by Yazelix status-cache renderers.
#[test]
fn status_cache_widget_rejects_non_workspace_widgets() {
    let err = render_status_cache_widget(&json!({}), "cursor").unwrap_err();

    assert!(err.message().contains("requires one of: workspace"));
}
