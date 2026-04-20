//! Doctor findings for runtime distribution capability and shared runtime preflight.
//! Bead: yazelix-ulb2.4.3

use crate::bridge::CoreError;
use crate::runtime_contract::{
    evaluate_runtime_contract, GeneratedLayoutCheckRequest, LinuxGhosttyDesktopGraphicsRequest,
    RuntimeCheckData, RuntimeContractEvaluateRequest, RuntimeScriptCheckRequest,
    TerminalSupportCheckRequest,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct SharedRuntimePreflightInput {
    pub zellij_layout_path: PathBuf,
    #[serde(default)]
    pub terminals: Vec<String>,
    pub startup_script_path: PathBuf,
    pub launch_script_path: PathBuf,
    #[serde(default)]
    pub command_search_paths: Vec<PathBuf>,
    pub platform_name: String,
}

#[derive(Debug, Deserialize)]
pub struct DoctorRuntimeEvaluateRequest {
    pub runtime_dir: PathBuf,
    pub yazelix_state_dir: PathBuf,
    pub has_home_manager_managed_install: bool,
    pub is_manual_runtime_reference_path: bool,
    #[serde(default)]
    pub shared_runtime: Option<SharedRuntimePreflightInput>,
}

#[derive(Debug, Serialize)]
pub struct DoctorRuntimeDoctorFinding {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub fix_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_contract_check: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_surface: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DoctorRuntimeEvaluateData {
    pub distribution: DoctorRuntimeDoctorFinding,
    pub shared_runtime_preflight: Vec<DoctorRuntimeDoctorFinding>,
}

pub fn evaluate_doctor_runtime_report(
    request: &DoctorRuntimeEvaluateRequest,
) -> DoctorRuntimeEvaluateData {
    let distribution = distribution_finding(
        &request.runtime_dir,
        request.has_home_manager_managed_install,
        request.is_manual_runtime_reference_path,
    );
    let managed_layouts_root = request
        .yazelix_state_dir
        .join("configs")
        .join("zellij")
        .join("layouts");

    let shared_runtime_preflight = match &request.shared_runtime {
        None => Vec::new(),
        Some(shared) => match build_shared_preflight_findings(
            shared,
            &managed_layouts_root,
            &request.runtime_dir,
        ) {
            Ok(v) => v,
            Err(e) => vec![DoctorRuntimeDoctorFinding {
                status: "error".into(),
                message: "Shared runtime preflight evaluation failed".into(),
                details: Some(e.to_string()),
                fix_available: false,
                fix_action: None,
                capability_tier: None,
                capability_mode: None,
                runtime_contract_check: None,
                owner_surface: None,
            }],
        },
    };

    DoctorRuntimeEvaluateData {
        distribution,
        shared_runtime_preflight,
    }
}

fn is_package_runtime_root(runtime_dir: &Path) -> bool {
    runtime_dir.join("yazelix_default.toml").exists()
        && runtime_dir.join("bin").join("yzx").exists()
        && runtime_dir.join("libexec").join("nu").exists()
}

fn distribution_finding(
    runtime_dir: &Path,
    has_home_manager_managed_install: bool,
    is_manual_runtime_reference_path: bool,
) -> DoctorRuntimeDoctorFinding {
    let (mode, tier, message, details) = if has_home_manager_managed_install {
        (
            "home_manager_managed",
            "full",
            "Runtime/distribution capability: Home Manager-managed full runtime",
            "Home Manager owns the packaged Yazelix runtime path and update transition in this mode.",
        )
    } else if is_manual_runtime_reference_path {
        (
            "installer_managed",
            "full",
            "Runtime/distribution capability: compatibility installer runtime",
            "This runtime still has legacy installer-owned artifacts from older releases. Current Yazelix no longer ships `#install`; reinstall into a Nix profile or move to Home Manager.",
        )
    } else if is_package_runtime_root(runtime_dir) {
        (
            "package_runtime",
            "narrowed",
            "Runtime/distribution capability: store/package runtime",
            "This Yazelix runtime runs directly from a packaged runtime root.",
        )
    } else {
        (
            "runtime_root_only",
            "narrowed",
            "Runtime/distribution capability: runtime-root-only mode",
            "This Yazelix session has a runtime root but no package-manager-owned distribution surface.",
        )
    };

    DoctorRuntimeDoctorFinding {
        status: "info".into(),
        message: message.into(),
        details: Some(details.into()),
        fix_available: false,
        fix_action: None,
        capability_tier: Some(tier.into()),
        capability_mode: Some(mode.into()),
        runtime_contract_check: None,
        owner_surface: None,
    }
}

fn normalize_failure_class(class: &str) -> String {
    match class.trim().to_lowercase().as_str() {
        "config" => "config problem".into(),
        "generated-state" => "generated-state problem".into(),
        "host-dependency" => "host-dependency problem".into(),
        _ => "problem".into(),
    }
}

fn format_failure_classification(failure_class: &str, recovery_hint: &str) -> String {
    let label = normalize_failure_class(failure_class);
    format!("Failure class: {label}.\nRecovery: {recovery_hint}")
}

fn build_runtime_check_detail_lines(check: &RuntimeCheckData) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(d) = &check.details {
        let t = d.trim();
        if !t.is_empty() {
            lines.push(d.clone());
        }
    }
    let recovery = check.recovery.as_deref().unwrap_or("").trim();
    let fc = check.failure_class.as_deref().unwrap_or("").trim();
    if !recovery.is_empty() && !fc.is_empty() {
        lines.push(format_failure_classification(fc, recovery));
    } else if !recovery.is_empty() {
        lines.push(recovery.to_string());
    }
    lines
}

fn runtime_check_to_doctor_finding(
    check: &RuntimeCheckData,
    managed_layouts_root: &Path,
) -> DoctorRuntimeDoctorFinding {
    let detail_lines = build_runtime_check_detail_lines(check);
    let details = if detail_lines.is_empty() {
        None
    } else {
        Some(detail_lines.join("\n"))
    };

    let status = if check.status == "ok" {
        "ok".to_string()
    } else {
        check.severity.clone()
    };

    let mut finding = DoctorRuntimeDoctorFinding {
        status,
        message: check.message.clone(),
        details,
        fix_available: false,
        fix_action: None,
        capability_tier: None,
        capability_mode: None,
        runtime_contract_check: Some(check.id.clone()),
        owner_surface: Some(check.owner_surface.clone()),
    };

    if check.id == "generated_layout"
        && check.status != "ok"
        && check.failure_class.as_deref() == Some("generated-state")
    {
        if let Some(ref p) = check.path {
            if is_managed_generated_layout_path(p, managed_layouts_root) {
                finding.fix_available = true;
                finding.fix_action = Some("repair_generated_runtime_state".into());
            }
        }
    }

    finding
}

fn is_managed_generated_layout_path(layout_path: &str, managed_dir: &Path) -> bool {
    let layout = Path::new(layout_path);
    if let (Ok(can_layout), Ok(can_root)) = (layout.canonicalize(), managed_dir.canonicalize()) {
        return can_layout.starts_with(&can_root);
    }
    let root_s = managed_dir.to_string_lossy();
    let root_norm = root_s.trim_end_matches('/');
    layout_path == root_norm
        || layout_path.starts_with(&format!("{root_norm}/"))
}

fn build_contract_request(
    shared: &SharedRuntimePreflightInput,
    runtime_dir: &Path,
) -> RuntimeContractEvaluateRequest {
    let runtime_script_requests: Vec<RuntimeScriptCheckRequest> = vec![
        RuntimeScriptCheckRequest {
            id: "startup_runtime_script".into(),
            label: "startup script".into(),
            owner_surface: "doctor".into(),
            path: shared.startup_script_path.clone(),
        },
        RuntimeScriptCheckRequest {
            id: "launch_runtime_script".into(),
            label: "launch script".into(),
            owner_surface: "doctor".into(),
            path: shared.launch_script_path.clone(),
        },
    ];

    RuntimeContractEvaluateRequest {
        working_dir: None,
        runtime_scripts: runtime_script_requests,
        generated_layout: Some(GeneratedLayoutCheckRequest {
            owner_surface: "doctor".into(),
            path: shared.zellij_layout_path.clone(),
        }),
        terminal_support: Some(TerminalSupportCheckRequest {
            owner_surface: "launch".into(),
            requested_terminal: String::new(),
            terminals: shared.terminals.clone(),
            command_search_paths: shared.command_search_paths.clone(),
        }),
        linux_ghostty_desktop_graphics_support: Some(LinuxGhosttyDesktopGraphicsRequest {
            owner_surface: "doctor".into(),
            terminals: shared.terminals.clone(),
            runtime_dir: Some(runtime_dir.to_path_buf()),
            command_search_paths: shared.command_search_paths.clone(),
            platform_name: Some(shared.platform_name.clone()),
        }),
    }
}

fn build_shared_preflight_findings(
    shared: &SharedRuntimePreflightInput,
    managed_layouts_root: &Path,
    runtime_dir: &Path,
) -> Result<Vec<DoctorRuntimeDoctorFinding>, CoreError> {
    let contract_req = build_contract_request(shared, runtime_dir);
    let data = evaluate_runtime_contract(&contract_req)?;
    Ok(data
        .checks
        .iter()
        .map(|c| runtime_check_to_doctor_finding(c, managed_layouts_root))
        .collect())
}

// Test lane: default

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn distribution_prefers_home_manager_over_package_shape() {
        let tmp = TempDir::new().unwrap();
        let rt = tmp.path().join("rt");
        std::fs::create_dir_all(rt.join("bin")).unwrap();
        std::fs::write(rt.join("yazelix_default.toml"), "").unwrap();
        std::fs::write(rt.join("bin").join("yzx"), "").unwrap();
        std::fs::create_dir_all(rt.join("libexec").join("nu")).unwrap();

        let f = distribution_finding(&rt, true, false);
        assert_eq!(f.capability_mode.as_deref(), Some("home_manager_managed"));
    }

    #[test]
    fn managed_layout_sets_repair_fix_action() {
        let tmp = TempDir::new().unwrap();
        let layouts = tmp.path().join("layouts");
        std::fs::create_dir_all(&layouts).unwrap();
        let layout_file = layouts.join("x.kdl");
        std::fs::write(&layout_file, "").unwrap();

        let check = RuntimeCheckData {
            id: "generated_layout".into(),
            status: "missing".into(),
            severity: "error".into(),
            owner_surface: "doctor".into(),
            message: "test".into(),
            details: None,
            recovery: None,
            failure_class: Some("generated-state".into()),
            blocking: true,
            path: Some(layout_file.to_string_lossy().into_owned()),
            candidates: None,
        };

        let f = runtime_check_to_doctor_finding(&check, &layouts);
        assert!(f.fix_available);
        assert_eq!(
            f.fix_action.as_deref(),
            Some("repair_generated_runtime_state")
        );
    }
}
