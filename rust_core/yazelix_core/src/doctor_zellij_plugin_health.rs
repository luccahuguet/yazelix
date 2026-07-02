//! Zellij pane-orchestrator health provider for `yzx doctor`.

use crate::bridge::CoreError;
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, runtime_dir_from_env, state_dir_from_env,
};
use crate::pane_orchestrator_client::{
    PANE_ORCHESTRATOR_PLUGIN_ALIAS, configure_zellij_control_session_env,
};
use crate::zellij_materialization::{
    ZellijMaterializationRequest, generate_zellij_materialization, zellij_permissions_cache_path,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::process::Command;

pub(crate) const SEED_ZELLIJ_PLUGIN_PERMISSIONS_FIX_ACTION: &str = "seed_zellij_plugin_permissions";

const ACTIVE_TAB_SESSION_STATE_PIPE: &str = "get_active_tab_session_state";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ZellijPluginHealthEvaluateRequest {
    pub inside_zellij: bool,
    pub sidebar_enabled: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ZellijPluginHealthFinding {
    status: String,
    message: String,
    details: Value,
    fix_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    fix_action: Option<&'static str>,
}

impl ZellijPluginHealthFinding {
    fn new(
        status: impl Into<String>,
        message: impl Into<String>,
        details: impl Into<Value>,
    ) -> Self {
        Self {
            status: status.into(),
            message: message.into(),
            details: details.into(),
            fix_available: false,
            fix_action: None,
        }
    }

    fn fixable(
        status: impl Into<String>,
        message: impl Into<String>,
        details: impl Into<Value>,
        fix_action: &'static str,
    ) -> Self {
        Self {
            status: status.into(),
            message: message.into(),
            details: details.into(),
            fix_available: true,
            fix_action: Some(fix_action),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SessionManagedPanes {
    #[serde(default)]
    editor_pane_id: Option<String>,
    #[serde(default)]
    sidebar_pane_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SessionLayout {
    #[serde(default)]
    active_swap_layout_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ActiveTabSessionStateV1 {
    active_tab_position: usize,
    managed_panes: SessionManagedPanes,
    layout: SessionLayout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ZellijPluginState {
    permissions_granted: bool,
    active_tab_position: Option<usize>,
    sidebar_pane_id: String,
    editor_pane_id: String,
    active_swap_layout_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ZellijPluginHealthProbeResult {
    Stdout(String),
    CommandError(String),
    StatusError(String),
}

pub(crate) fn zellij_plugin_health_request_from_env() -> ZellijPluginHealthEvaluateRequest {
    ZellijPluginHealthEvaluateRequest {
        inside_zellij: env::var_os("ZELLIJ").is_some(),
        sidebar_enabled: true,
    }
}

pub(crate) fn evaluate_zellij_plugin_health(
    request: &ZellijPluginHealthEvaluateRequest,
) -> Vec<ZellijPluginHealthFinding> {
    if !request.inside_zellij {
        return vec![ZellijPluginHealthFinding::new(
            "info",
            "Zellij plugin health check skipped (not inside Zellij)",
            "Run `yzx doctor` from inside the affected Yazelix session to verify Yazelix orchestrator permissions and managed pane detection.",
        )];
    }

    evaluate_zellij_plugin_health_probe_result(request, run_zellij_plugin_health_probe())
}

pub(crate) fn seed_zellij_plugin_permissions() -> Result<bool, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let state_dir = state_dir_from_env()?;
    let paths = crate::active_config_surface::resolve_active_config_paths(
        &runtime_dir,
        &config_dir,
        config_override_from_env().as_deref(),
    )?;
    let zellij_config_dir = state_dir.join("configs").join("zellij");
    let req = ZellijMaterializationRequest {
        config_path: paths.config_file,
        default_config_path: paths.default_config_path,
        contract_path: paths.contract_path,
        runtime_dir,
        zellij_config_dir,
        seed_plugin_permissions: true,
        session_terminal_label: None,
        layout_override: None,
    };
    match generate_zellij_materialization(&req) {
        Ok(_) => {
            let cache_path = zellij_permissions_cache_path()?;
            println!(
                "✅ Seeded Yazelix plugin permissions in: {}",
                cache_path.display()
            );
        }
        Err(err) => {
            println!(
                "❌ Failed to seed Yazelix plugin permissions: {}",
                err.message()
            );
            return Ok(true);
        }
    }
    Ok(false)
}

fn run_zellij_plugin_health_probe() -> ZellijPluginHealthProbeResult {
    let mut command = Command::new("zellij");
    configure_zellij_control_session_env(&mut command);
    let output = command
        .args([
            "action",
            "pipe",
            "--plugin",
            PANE_ORCHESTRATOR_PLUGIN_ALIAS,
            "--name",
            ACTIVE_TAB_SESSION_STATE_PIPE,
            "--",
            "",
        ])
        .output();

    let output = match output {
        Ok(output) => output,
        Err(error) => return ZellijPluginHealthProbeResult::CommandError(error.to_string()),
    };

    if !output.status.success() {
        return ZellijPluginHealthProbeResult::StatusError(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        );
    }

    ZellijPluginHealthProbeResult::Stdout(
        String::from_utf8_lossy(&output.stdout).trim().to_string(),
    )
}

fn evaluate_zellij_plugin_health_probe_result(
    request: &ZellijPluginHealthEvaluateRequest,
    probe_result: ZellijPluginHealthProbeResult,
) -> Vec<ZellijPluginHealthFinding> {
    let raw = match probe_result {
        ZellijPluginHealthProbeResult::Stdout(raw) => raw,
        ZellijPluginHealthProbeResult::CommandError(error)
        | ZellijPluginHealthProbeResult::StatusError(error) => {
            return vec![ZellijPluginHealthFinding::new(
                "warning",
                "Could not contact the Yazelix pane-orchestrator plugin",
                format!(
                    "Run this from inside the affected Yazelix session after fully restarting it. Underlying error: {error}"
                ),
            )];
        }
    };

    match raw.as_str() {
        "permissions_denied" => build_zellij_plugin_health_findings(
            &ZellijPluginState {
                permissions_granted: false,
                active_tab_position: None,
                sidebar_pane_id: String::new(),
                editor_pane_id: String::new(),
                active_swap_layout_name: None,
            },
            request.sidebar_enabled,
        ),
        "not_ready" | "missing" => vec![ZellijPluginHealthFinding::new(
            "warning",
            "Yazelix pane-orchestrator session state is not ready yet",
            "The plugin responded before tab/workspace state was available. Wait a moment and rerun `yzx doctor` inside this Yazelix session.",
        )],
        _ => match serde_json::from_str::<ActiveTabSessionStateV1>(&raw) {
            Ok(session) => build_zellij_plugin_health_findings(
                &ZellijPluginState {
                    permissions_granted: true,
                    active_tab_position: Some(session.active_tab_position),
                    sidebar_pane_id: session.managed_panes.sidebar_pane_id.unwrap_or_default(),
                    editor_pane_id: session.managed_panes.editor_pane_id.unwrap_or_default(),
                    active_swap_layout_name: session.layout.active_swap_layout_name,
                },
                request.sidebar_enabled,
            ),
            Err(_) => vec![ZellijPluginHealthFinding::new(
                "warning",
                "Yazelix pane-orchestrator returned an unexpected response",
                format!("Unexpected payload: {raw}"),
            )],
        },
    }
}

fn build_zellij_plugin_health_findings(
    plugin_state: &ZellijPluginState,
    sidebar_enabled: bool,
) -> Vec<ZellijPluginHealthFinding> {
    let mut results = Vec::new();

    if !plugin_state.permissions_granted {
        results.push(ZellijPluginHealthFinding::fixable(
            "error",
            "Yazelix pane-orchestrator plugin permissions not granted",
            "Yazelix normally pre-seeds bundled Zellij plugin permissions before launch. If the cache was deleted or Zellij is already prompting, run `yzx doctor --fix` and restart Yazelix; if a live prompt remains, focus the top zjstatus bar and press `y`, and answer yes to the Yazelix orchestrator popup. Yazelix workspace bindings like `Alt+m`, `Alt+Shift+H/J/K/L`, `Ctrl+y`, `Ctrl+Shift+Y`, `Alt+r`, `Alt+[`, and `Alt+]` depend on the orchestrator.",
            SEED_ZELLIJ_PLUGIN_PERMISSIONS_FIX_ACTION,
        ));
    } else {
        results.push(ZellijPluginHealthFinding::new(
            "ok",
            "Yazelix pane-orchestrator permissions granted",
            "The orchestrator plugin can handle Yazelix tab and pane actions in this Zellij session.",
        ));
    }

    if plugin_state.active_tab_position.is_none() {
        results.push(ZellijPluginHealthFinding::new(
            "warning",
            "Yazelix pane-orchestrator does not see an active tab yet",
            "The plugin may still be initializing. Wait a moment and rerun `yzx doctor` inside this Yazelix session.",
        ));
        return results;
    }

    if plugin_state.sidebar_pane_id.trim().is_empty() && sidebar_enabled {
        results.push(ZellijPluginHealthFinding::new(
            "warning",
            "Managed sidebar pane not detected in the current tab",
            "`Alt+Shift+H`, `Ctrl+y`, `Ctrl+Shift+Y`, and reveal flows may not work until the current tab uses a Yazelix managed-sidebar layout.",
        ));
    } else if sidebar_enabled {
        results.push(ZellijPluginHealthFinding::new(
            "ok",
            format!(
                "Managed sidebar pane detected: {}",
                plugin_state.sidebar_pane_id
            ),
            format!(
                "Layout state: {}",
                plugin_state
                    .active_swap_layout_name
                    .as_deref()
                    .unwrap_or("unknown")
            ),
        ));
    }

    if plugin_state.editor_pane_id.trim().is_empty() {
        results.push(ZellijPluginHealthFinding::new(
            "info",
            "Managed editor pane not detected in the current tab",
            "This is normal until you open a managed Helix or Neovim editor pane in the current tab. An editor started manually from an ordinary shell pane does not count as the managed editor pane.",
        ));
    } else {
        results.push(ZellijPluginHealthFinding::new(
            "ok",
            format!(
                "Managed editor pane detected: {}",
                plugin_state.editor_pane_id
            ),
            Value::Null,
        ));
    }

    results
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    fn request(inside_zellij: bool) -> ZellijPluginHealthEvaluateRequest {
        ZellijPluginHealthEvaluateRequest {
            inside_zellij,
            sidebar_enabled: true,
        }
    }

    // Defends: doctor maps outside-Zellij, sentinel, transport, and unexpected plugin responses without live session dependencies.
    #[test]
    fn reports_probe_protocol_states() {
        let findings = evaluate_zellij_plugin_health(&request(false));

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].status, "info");
        assert_eq!(
            findings[0].message,
            "Zellij plugin health check skipped (not inside Zellij)"
        );

        let findings = evaluate_zellij_plugin_health_probe_result(
            &request(true),
            ZellijPluginHealthProbeResult::Stdout("permissions_denied".into()),
        );
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].status, "error");
        assert_eq!(
            findings[0].fix_action,
            Some(SEED_ZELLIJ_PLUGIN_PERMISSIONS_FIX_ACTION)
        );
        assert_eq!(findings[1].status, "warning");

        let findings = evaluate_zellij_plugin_health_probe_result(
            &request(true),
            ZellijPluginHealthProbeResult::Stdout("not_ready".into()),
        );
        assert_eq!(
            findings[0].message,
            "Yazelix pane-orchestrator session state is not ready yet"
        );

        let findings = evaluate_zellij_plugin_health_probe_result(
            &request(true),
            ZellijPluginHealthProbeResult::CommandError("zellij missing".into()),
        );
        assert_eq!(
            findings[0].message,
            "Could not contact the Yazelix pane-orchestrator plugin"
        );
        assert!(
            findings[0]
                .details
                .as_str()
                .unwrap()
                .contains("Underlying error: zellij missing")
        );

        let findings = evaluate_zellij_plugin_health_probe_result(
            &request(true),
            ZellijPluginHealthProbeResult::StatusError("plugin not found".into()),
        );
        assert!(
            findings[0]
                .details
                .as_str()
                .unwrap()
                .contains("Underlying error: plugin not found")
        );

        let findings = evaluate_zellij_plugin_health_probe_result(
            &request(true),
            ZellijPluginHealthProbeResult::Stdout("not-json".into()),
        );
        assert_eq!(
            findings[0].message,
            "Yazelix pane-orchestrator returned an unexpected response"
        );
    }

    // Defends: valid plugin JSON becomes the same permission/sidebar/editor health rows.
    #[test]
    fn valid_session_json_reports_managed_panes() {
        let raw = r#"{
            "active_tab_position": 2,
            "managed_panes": {
                "sidebar_pane_id": "pane-1",
                "editor_pane_id": "pane-2"
            },
            "layout": {
                "active_swap_layout_name": "two_column"
            }
        }"#;
        let findings = evaluate_zellij_plugin_health_probe_result(
            &request(true),
            ZellijPluginHealthProbeResult::Stdout(raw.into()),
        );

        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.message.as_str())
                .collect::<Vec<_>>(),
            vec![
                "Yazelix pane-orchestrator permissions granted",
                "Managed sidebar pane detected: pane-1",
                "Managed editor pane detected: pane-2",
            ]
        );
    }
}
