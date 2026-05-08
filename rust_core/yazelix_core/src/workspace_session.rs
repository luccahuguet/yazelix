//! Active-tab workspace/session request and response helpers shared by Yazelix command adapters.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SidebarState {
    pub(crate) yazi_id: String,
    pub(crate) cwd: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SidebarYaziRegistration {
    pub(crate) pane_id: String,
    pub(crate) yazi_id: String,
    pub(crate) cwd: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkspaceRetargetResult {
    pub(crate) status: String,
    pub(crate) editor_status: String,
    pub(crate) sidebar_state: Option<SidebarState>,
    pub(crate) reason: Option<String>,
}

impl WorkspaceRetargetResult {
    pub(crate) fn status(&self) -> &str {
        self.status.as_str()
    }
}

pub(crate) fn workspace_tab_name(workspace_root: &Path) -> String {
    workspace_root
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or("unnamed")
        .to_string()
}

pub(crate) fn workspace_retarget_payload(
    workspace_root: &Path,
    cd_focused_pane: bool,
    editor_kind: Option<&str>,
    sidebar_yazi: Option<&SidebarYaziRegistration>,
) -> String {
    json!({
        "workspace_root": workspace_root.display().to_string(),
        "cd_focused_pane": cd_focused_pane,
        "editor": editor_kind
            .map(str::trim)
            .filter(|editor| !editor.is_empty()),
        "sidebar_yazi": sidebar_yazi,
    })
    .to_string()
}

pub(crate) fn managed_editor_open_payload(
    editor_kind: &str,
    file_paths: &[PathBuf],
    working_dir: &Path,
) -> String {
    let file_path_strings = file_paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    let first_file_path = file_path_strings.first().cloned().unwrap_or_default();

    json!({
        "editor": editor_kind,
        "file_path": first_file_path,
        "file_paths": file_path_strings,
        "working_dir": working_dir.display().to_string(),
    })
    .to_string()
}

pub(crate) fn open_terminal_in_cwd_payload(cwd: &Path) -> String {
    json!({
        "cwd": cwd.display().to_string(),
    })
    .to_string()
}

#[derive(Debug, Deserialize)]
struct WorkspaceRetargetResponse {
    status: String,
    #[serde(default)]
    editor_status: String,
    #[serde(default)]
    sidebar_yazi_id: Option<String>,
    #[serde(default)]
    sidebar_yazi_cwd: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ActiveTabSessionStateV1 {
    #[serde(default)]
    sidebar_yazi: Option<SessionSidebarYazi>,
}

#[derive(Debug, Deserialize)]
struct SessionSidebarYazi {
    yazi_id: String,
    cwd: String,
}

pub(crate) fn parse_workspace_retarget_response(raw: &str) -> WorkspaceRetargetResult {
    match raw.trim() {
        "missing" | "not_ready" | "permissions_denied" | "invalid_payload" => {
            WorkspaceRetargetResult {
                status: raw.trim().to_string(),
                editor_status: String::new(),
                sidebar_state: None,
                reason: None,
            }
        }
        other => match serde_json::from_str::<WorkspaceRetargetResponse>(other) {
            Ok(parsed) => {
                let sidebar_state = parsed
                    .sidebar_yazi_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|yazi_id| SidebarState {
                        yazi_id: yazi_id.to_string(),
                        cwd: parsed
                            .sidebar_yazi_cwd
                            .as_deref()
                            .map(str::trim)
                            .unwrap_or("")
                            .to_string(),
                    });
                WorkspaceRetargetResult {
                    status: parsed.status,
                    editor_status: parsed.editor_status,
                    sidebar_state,
                    reason: None,
                }
            }
            Err(_) => WorkspaceRetargetResult {
                status: "error".to_string(),
                editor_status: String::new(),
                sidebar_state: None,
                reason: Some(other.to_string()),
            },
        },
    }
}

pub(crate) fn current_tab_workspace_root_from_json(
    raw: &str,
    include_bootstrap: bool,
) -> Option<String> {
    let parsed = serde_json::from_str::<serde_json::Value>(raw).ok()?;
    let workspace = parsed.get("workspace")?;
    let root = workspace.get("root")?.as_str()?.trim();
    if root.is_empty() {
        return None;
    }
    let source = workspace
        .get("source")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    if !include_bootstrap && source == "bootstrap" {
        return None;
    }
    Some(root.to_string())
}

pub(crate) fn parse_active_sidebar_state(raw: &str) -> Option<SidebarState> {
    let parsed = serde_json::from_str::<ActiveTabSessionStateV1>(raw).ok()?;
    let sidebar = parsed.sidebar_yazi?;
    let yazi_id = sidebar.yazi_id.trim();
    let cwd = sidebar.cwd.trim();
    if yazi_id.is_empty() || cwd.is_empty() {
        return None;
    }

    Some(SidebarState {
        yazi_id: yazi_id.to_string(),
        cwd: cwd.to_string(),
    })
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: active-tab workspace parsing keeps bootstrap roots optional for callers that need only user-set/plugin roots.
    #[test]
    fn workspace_root_parser_respects_bootstrap_boundary() {
        let raw = r#"{"workspace":{"root":"/tmp/demo","source":"bootstrap"}}"#;

        assert_eq!(current_tab_workspace_root_from_json(raw, false), None);
        assert_eq!(
            current_tab_workspace_root_from_json(raw, true),
            Some("/tmp/demo".to_string())
        );
    }

    // Defends: retarget responses expose a typed sidebar state instead of leaking plugin wire fields into command adapters.
    #[test]
    fn retarget_response_parser_extracts_sidebar_state() {
        let parsed = parse_workspace_retarget_response(
            r#"{"status":"ok","editor_status":"ok","sidebar_yazi_id":"yazi-123","sidebar_yazi_cwd":"/home/sidebar"}"#,
        );

        assert_eq!(parsed.status(), "ok");
        assert_eq!(parsed.editor_status, "ok");
        assert_eq!(
            parsed.sidebar_state,
            Some(SidebarState {
                yazi_id: "yazi-123".into(),
                cwd: "/home/sidebar".into(),
            })
        );
    }

    // Defends: simple pane-orchestrator retarget status strings stay typed and do not require JSON payloads.
    #[test]
    fn retarget_response_parser_keeps_simple_status_strings() {
        assert_eq!(
            parse_workspace_retarget_response("missing").status(),
            "missing"
        );
        assert_eq!(
            parse_workspace_retarget_response("permissions_denied").status(),
            "permissions_denied"
        );
    }

    // Regression: reveal and editor flows use the pane-orchestrator session snapshot as the only live sidebar identity source.
    #[test]
    fn active_sidebar_state_parser_reads_session_snapshot() {
        let state = parse_active_sidebar_state(
            r#"{"schema_version":1,"active_tab_position":0,"focus_context":"sidebar","managed_panes":{"editor_pane_id":null,"sidebar_pane_id":"terminal:0"},"layout":{"active_swap_layout_name":null,"sidebar_collapsed":false},"sidebar_yazi":{"yazi_id":"plugin-yazi-id","cwd":"/home/plugin"}}"#,
        );

        assert_eq!(
            state,
            Some(SidebarState {
                yazi_id: "plugin-yazi-id".into(),
                cwd: "/home/plugin".into(),
            })
        );
    }
}
