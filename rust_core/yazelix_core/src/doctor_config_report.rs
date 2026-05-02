//! Config-surface doctor findings (presence, legacy surfaces, stale schema diagnostics).
//! Bead: yazelix-ulb2.4.4

use crate::active_config_surface::{
    ensure_managed_toml_tooling_config, primary_config_paths, validate_primary_config_surface,
};
use crate::bridge::{CoreError, ErrorClass};
use crate::config_normalize::{
    ConfigDiagnostic, ConfigDiagnosticReport, NormalizeConfigRequest, normalize_config,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct DoctorConfigEvaluateRequest {
    pub config_dir: PathBuf,
    pub runtime_dir: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct DoctorConfigFinding {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub fix_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_diagnostic_report: Option<ConfigDiagnosticReport>,
}

#[derive(Debug, Serialize)]
pub struct DoctorConfigEvaluateData {
    pub findings: Vec<DoctorConfigFinding>,
}

pub fn evaluate_doctor_config_report(
    request: &DoctorConfigEvaluateRequest,
) -> DoctorConfigEvaluateData {
    let paths = primary_config_paths(&request.runtime_dir, &request.config_dir);
    let legacy_nix_config = request.config_dir.join("yazelix.nix");

    if let Err(error) = validate_primary_config_surface(&paths) {
        return DoctorConfigEvaluateData {
            findings: vec![DoctorConfigFinding {
                status: "error".into(),
                message: "Could not reconcile Yazelix config surfaces".into(),
                details: Some(format_surface_reconcile_error(&error)),
                fix_available: false,
                config_diagnostic_report: None,
            }],
        };
    }

    if let Err(error) = ensure_managed_toml_tooling_config(
        &paths.runtime_toml_tooling_config,
        &paths.managed_toml_tooling_config,
    ) {
        return DoctorConfigEvaluateData {
            findings: vec![DoctorConfigFinding {
                status: "error".into(),
                message: "Could not reconcile Yazelix config surfaces".into(),
                details: Some(format_surface_reconcile_error(&error)),
                fix_available: false,
                config_diagnostic_report: None,
            }],
        };
    }

    if paths.user_config.exists() {
        let mut findings = vec![DoctorConfigFinding {
            status: "ok".into(),
            message: "Using custom yazelix.toml configuration".into(),
            details: Some(path_to_string(&paths.user_config)),
            fix_available: false,
            config_diagnostic_report: None,
        }];

        let diagnostic_request = NormalizeConfigRequest {
            config_path: paths.user_config.clone(),
            default_config_path: paths.default_config_path.clone(),
            contract_path: paths.contract_path.clone(),
            include_missing: false,
        };

        match collect_doctor_diagnostic_report(&diagnostic_request) {
            Ok(report) if report.issue_count > 0 => {
                let details = render_doctor_config_details(&report);
                findings.push(DoctorConfigFinding {
                    status: "warning".into(),
                    message: format!(
                        "Stale or unsupported yazelix.toml entries detected ({} issues)",
                        report.issue_count
                    ),
                    details: Some(details),
                    fix_available: false,
                    config_diagnostic_report: Some(report),
                });
            }
            Ok(_) => {}
            Err(error) => {
                findings.push(DoctorConfigFinding {
                    status: "error".into(),
                    message: "Could not validate yazelix.toml against the current schema".into(),
                    details: Some(format_validation_error(&error)),
                    fix_available: false,
                    config_diagnostic_report: None,
                });
            }
        }

        return DoctorConfigEvaluateData { findings };
    }

    if legacy_nix_config.exists() {
        return DoctorConfigEvaluateData {
            findings: vec![DoctorConfigFinding {
                status: "warning".into(),
                message: "Legacy yazelix.nix configuration detected".into(),
                details: Some(path_to_string(&legacy_nix_config)),
                fix_available: false,
                config_diagnostic_report: None,
            }],
        };
    }

    if paths.default_config_path.exists() {
        return DoctorConfigEvaluateData {
            findings: vec![DoctorConfigFinding {
                status: "info".into(),
                message: "Using default configuration (yazelix_default.toml)".into(),
                details: Some("Consider copying to yazelix.toml for customization".into()),
                fix_available: true,
                config_diagnostic_report: None,
            }],
        };
    }

    DoctorConfigEvaluateData {
        findings: vec![DoctorConfigFinding {
            status: "error".into(),
            message: "No configuration file found".into(),
            details: Some("Neither yazelix.toml nor yazelix_default.toml exists".into()),
            fix_available: false,
            config_diagnostic_report: None,
        }],
    }
}

fn collect_doctor_diagnostic_report(
    request: &NormalizeConfigRequest,
) -> Result<ConfigDiagnosticReport, CoreError> {
    match normalize_config(request) {
        Ok(data) => Ok(data.diagnostic_report),
        Err(error) if matches!(error.class(), ErrorClass::Config) => {
            if error.code() == "unsupported_config" {
                return deserialize_config_diagnostic_report(error.details());
            }

            if error.code() == "invalid_toml" {
                return Err(error);
            }

            Ok(build_single_error_config_diagnostic_report(
                &request.config_path,
                &error,
            ))
        }
        Err(error) => Err(error),
    }
}

/// Matches `nushell/scripts/utils/config_report_rendering.nu` `render_doctor_config_details`.
fn render_doctor_config_details(report: &ConfigDiagnosticReport) -> String {
    if report.issue_count == 0 {
        return "No stale or unsupported config issues detected.".to_string();
    }

    let mut lines = vec![
        format!("Config report for: {}", report.config_path),
        format!("Issues: {}", report.issue_count),
    ];

    for diagnostic in &report.doctor_diagnostics {
        lines.push(String::new());
        lines.push(diagnostic.headline.clone());
        for detail in &diagnostic.detail_lines {
            lines.push(format!("  {detail}"));
        }
    }

    lines.push(String::new());
    lines.push("Review the listed fields manually.".to_string());
    lines.push("Blunt fallback: `yzx reset config`".to_string());
    lines.join("\n")
}

fn deserialize_config_diagnostic_report(value: Value) -> Result<ConfigDiagnosticReport, CoreError> {
    serde_json::from_value(value).map_err(|error| {
        CoreError::classified(
            ErrorClass::Internal,
            "invalid_config_diagnostic_report",
            format!("Rust config helper emitted an invalid config diagnostic report: {error}"),
            "Rebuild or reinstall Yazelix so the Rust helper and Nushell bridge agree on the config report schema.",
            serde_json::json!({}),
        )
    })
}

fn build_single_error_config_diagnostic_report(
    config_path: &Path,
    error: &CoreError,
) -> ConfigDiagnosticReport {
    let details = error.details();
    let path = details
        .get("field")
        .and_then(Value::as_str)
        .unwrap_or("<root>")
        .to_string();
    let message = error.message();
    let remediation = error.remediation();
    let status = error.code().to_string();

    let diagnostic = ConfigDiagnostic {
        category: "config".into(),
        path: path.clone(),
        status,
        blocking: true,
        fix_available: false,
        headline: format!("Invalid config value at {path}"),
        detail_lines: vec![
            message,
            format!("Next: {remediation}"),
            "Next: Run `yzx doctor --verbose` to review the full config report.".into(),
        ],
    };

    ConfigDiagnosticReport {
        config_path: path_to_string(config_path),
        schema_diagnostics: vec![diagnostic.clone()],
        doctor_diagnostics: vec![diagnostic.clone()],
        blocking_diagnostics: vec![diagnostic],
        issue_count: 1,
        blocking_count: 1,
        fixable_count: 0,
        has_blocking: true,
        has_fixable_config_issues: false,
    }
}

fn format_surface_reconcile_error(error: &CoreError) -> String {
    let details = error.details();
    let mut lines = vec![error.message()];

    match error.code() {
        "duplicate_config_surfaces" => {
            if let Some(user_config) = details.get("user_config").and_then(Value::as_str) {
                lines.push(format!("user_configs main: {user_config}"));
            }
            if let Some(legacy_main) = details.get("legacy_user_config").and_then(Value::as_str) {
                lines.push(format!("legacy main: {legacy_main}"));
            }
        }
        "legacy_root_config_surface" => {
            if let Some(legacy_main) = details.get("legacy_main").and_then(Value::as_str) {
                lines.push(format!("legacy main: {legacy_main}"));
            }
            if let Some(current_main) = details.get("current_main").and_then(Value::as_str) {
                lines.push(format!("current main: {current_main}"));
            }
        }
        "missing_runtime_toml_tooling_config" => {
            if let Some(path) = details.get("path").and_then(Value::as_str) {
                lines.push(format!("runtime support file: {path}"));
            }
        }
        _ => {
            if let Some(path) = details.get("path").and_then(Value::as_str) {
                lines.push(path.to_string());
            }
        }
    }

    lines.push(String::new());
    lines.push(format_failure_classification(
        "config",
        &error.remediation(),
    ));
    lines.join("\n")
}

fn format_validation_error(error: &CoreError) -> String {
    let failure_class = match error.class() {
        ErrorClass::Config => "config",
        _ => "host-dependency",
    };

    [
        error.message(),
        String::new(),
        format_failure_classification(failure_class, &error.remediation()),
    ]
    .join("\n")
}

fn format_failure_classification(failure_class: &str, recovery_hint: &str) -> String {
    let label = match failure_class.trim().to_lowercase().as_str() {
        "config" => "config problem",
        "generated-state" => "generated-state problem",
        "host-dependency" => "host-dependency problem",
        _ => "problem",
    };
    format!("Failure class: {label}.\nRecovery: {recovery_hint}")
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_normalize::ConfigDiagnostic;

    // Defends: doctor config details prose matches the historical Nushell renderer shape for verbose output.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn render_doctor_config_details_matches_expected_shape() {
        let report = ConfigDiagnosticReport {
            config_path: "/tmp/yazelix.toml".into(),
            schema_diagnostics: vec![],
            doctor_diagnostics: vec![ConfigDiagnostic {
                category: "schema".into(),
                path: "core.stale_field".into(),
                status: "unknown_field".into(),
                blocking: false,
                fix_available: false,
                headline: "Unknown config field: core.stale_field".into(),
                detail_lines: vec!["line one".into()],
            }],
            blocking_diagnostics: vec![],
            issue_count: 1,
            blocking_count: 0,
            fixable_count: 0,
            has_blocking: false,
            has_fixable_config_issues: false,
        };

        let out = render_doctor_config_details(&report);
        assert!(out.contains("Config report for: /tmp/yazelix.toml"));
        assert!(out.contains("Issues: 1"));
        assert!(out.contains("Unknown config field: core.stale_field"));
        assert!(out.contains("  line one"));
        assert!(out.contains("Review the listed fields manually."));
        assert!(out.contains("yzx reset config"));
    }

    // Defends: zero-issue report must not emit a misleading multi-line “stale config” block.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn render_doctor_config_details_empty_issues_message() {
        let report = ConfigDiagnosticReport {
            config_path: "x".into(),
            schema_diagnostics: vec![],
            doctor_diagnostics: vec![],
            blocking_diagnostics: vec![],
            issue_count: 0,
            blocking_count: 0,
            fixable_count: 0,
            has_blocking: false,
            has_fixable_config_issues: false,
        };
        assert_eq!(
            render_doctor_config_details(&report),
            "No stale or unsupported config issues detected."
        );
    }
}
