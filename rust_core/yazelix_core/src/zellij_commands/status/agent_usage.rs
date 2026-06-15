//! Status-cache widget routing that remains owned by Yazelix.
//!
//! Non-workspace widgets are runnable `yazelix_zellij_bar_widget` commands owned by yazelix-zellij-bar.

use super::{render_zjstatus_workspace_widget, status_bar_cache_status_bus};
use crate::bridge::CoreError;
use serde_json::Value;

const STATUS_CACHE_WIDGET_NAMES: &[&str] = &["workspace"];

pub(crate) fn render_status_cache_widget(cache: &Value, widget: &str) -> Result<String, CoreError> {
    let status_bus = status_bar_cache_status_bus(cache);
    match widget {
        "workspace" => Ok(status_bus
            .map(render_zjstatus_workspace_widget)
            .unwrap_or_default()),
        _ => Err(CoreError::usage(format!(
            "zellij status-cache-widget requires one of: {}",
            STATUS_CACHE_WIDGET_NAMES.join(", ")
        ))),
    }
}
