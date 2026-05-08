//! Zellij integration commands for `yzx_control`.

use crate::bridge::CoreError;
use crate::pane_orchestrator_client::run_pane_orchestrator_command;

mod pipe;
mod status;
mod workspace;

pub use pipe::{run_zellij_get_workspace_root, run_zellij_pipe};
pub use status::{
    probe_active_tab_session_state, run_zellij_inspect_session, run_zellij_status_bus,
    run_zellij_status_cache_heartbeat, run_zellij_status_cache_refresh_claude_usage,
    run_zellij_status_cache_refresh_codex_usage, run_zellij_status_cache_refresh_opencode_go_usage,
    run_zellij_status_cache_widget, run_zellij_status_cache_write,
};
pub use workspace::{
    run_zellij_open_editor, run_zellij_open_editor_cwd, run_zellij_open_terminal,
    run_zellij_retarget,
};

pub const INTERNAL_ZELLIJ_CONTROL_SUBCOMMANDS: &[&str] = &[
    "pipe",
    "get-workspace-root",
    "inspect-session",
    "status-bus",
    "status-cache-write",
    "status-cache-heartbeat",
    "status-cache-widget",
    "status-cache-refresh-claude-usage",
    "status-cache-refresh-codex-usage",
    "status-cache-refresh-opencode-go-usage",
    "retarget",
    "open-editor",
    "open-editor-cwd",
    "open-terminal",
];

pub(crate) fn run_pane_orchestrator_runtime_config_reload(
    payload: &str,
) -> Result<String, CoreError> {
    run_pane_orchestrator_command("reload_runtime_config", payload)
}

pub fn internal_zellij_control_subcommands_usage() -> String {
    INTERNAL_ZELLIJ_CONTROL_SUBCOMMANDS.join("|")
}
