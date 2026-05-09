// Test lane: default
use super::*;
use serde_json::{Value, json};
use std::ffi::OsStr;
use std::path::PathBuf;

const STATUS_CACHE_TEST_PAYLOAD: &str = r#"{"schema_version":1,"active_tab_position":0,"workspace":{"root":"/tmp/yazelix-demo","source":"explicit"},"managed_panes":{"editor_pane_id":"terminal:1","sidebar_pane_id":"terminal:2"},"focus_context":"sidebar","layout":{"active_swap_layout_name":"single_open","sidebar_collapsed":false},"sidebar_yazi":null,"transient_panes":{"popup":null,"menu":null},"extensions":{"ai_pane_activity":[]}}"#;

fn status_cache_test_status_bus() -> Value {
    serde_json::from_str(STATUS_CACHE_TEST_PAYLOAD).unwrap()
}

mod cache;
mod widgets;
