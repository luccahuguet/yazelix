//! Status-cache widget routing that remains owned by Yazelix.
//!
//! Provider usage widgets are runnable `yazelix_bar_widget` commands owned by yazelix-bar.

use super::{
    render_zjstatus_cursor_widget, render_zjstatus_workspace_widget, status_bar_cache_status_bus,
};
use crate::bridge::CoreError;
use serde_json::Value;

#[cfg(test)]
pub(crate) fn render_status_cache_widget(cache: &Value, widget: &str) -> Result<String, CoreError> {
    render_status_cache_widget_for_yazelix_owned_widgets(cache, widget)
}

pub(crate) fn render_status_cache_widget_for_yazelix_owned_widgets(
    cache: &Value,
    widget: &str,
) -> Result<String, CoreError> {
    let status_bus = status_bar_cache_status_bus(cache);
    match widget {
        "workspace" => Ok(status_bus
            .map(render_zjstatus_workspace_widget)
            .unwrap_or_default()),
        "cursor" => Ok(render_zjstatus_cursor_widget(cache)),
        _ => Err(CoreError::usage(format!(
            "zellij status-cache-widget requires one of: {}",
            status_cache_widget_names().join(", ")
        ))),
    }
}

pub(crate) fn status_cache_widget_names() -> Vec<&'static str> {
    vec!["workspace", "cursor"]
}
