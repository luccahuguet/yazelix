//! Zellij integration commands for `yzx_control`.

mod pipe;
mod status;
mod workspace;

pub use pipe::{
    run_zellij_get_workspace_root, run_zellij_pipe, run_zellij_refresh_terminal_title_activity,
};
pub use status::{
    probe_active_tab_session_state, run_zellij_inspect_session, run_zellij_status_bus,
    run_zellij_status_cache_heartbeat, run_zellij_status_cache_widget,
    run_zellij_status_cache_write,
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
    "refresh-terminal-title-activity",
    "retarget",
    "open-editor",
    "open-editor-cwd",
    "open-terminal",
];

pub fn internal_zellij_control_subcommands_usage() -> String {
    INTERNAL_ZELLIJ_CONTROL_SUBCOMMANDS.join("|")
}
