//! Config-surface doctor findings (presence, legacy surfaces, stale schema diagnostics).
//! Bead: yazelix-ulb2.4.4

use crate::active_config_surface::{primary_config_paths, validate_primary_config_surface};
use crate::bridge::{CoreError, ErrorClass};
use crate::config_normalize::{
    ConfigDiagnostic, ConfigDiagnosticReport, NormalizeConfigRequest, normalize_config,
};
use crate::native_config_status::path_present;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
    pub fix_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_diagnostic_report: Option<ConfigDiagnosticReport>,
}

impl DoctorConfigFinding {
    fn new(status: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: status.into(),
            message: message.into(),
            details: None,
            fix_available: false,
            fix_action: None,
            config_diagnostic_report: None,
        }
    }

    fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    fn with_diagnostic_report(mut self, report: ConfigDiagnosticReport) -> Self {
        self.config_diagnostic_report = Some(report);
        self
    }
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
            findings: vec![
                DoctorConfigFinding::new("error", "Could not reconcile Yazelix config surfaces")
                    .with_details(format_surface_reconcile_error(&error)),
            ],
        };
    }

    if path_present(&paths.user_config) {
        let mut findings = vec![
            DoctorConfigFinding::new("ok", "Using custom config.toml configuration")
                .with_details(path_to_string(&paths.user_config)),
        ];

        let diagnostic_request = NormalizeConfigRequest {
            config_path: paths.user_config.clone(),
            default_config_path: paths.default_config_path.clone(),
            contract_path: paths.contract_path.clone(),
        };

        match collect_doctor_diagnostic_report(&diagnostic_request) {
            Ok(report) if report.issue_count > 0 => {
                let details = render_doctor_config_details(&report);
                findings.push(
                    DoctorConfigFinding::new(
                        "warning",
                        format!(
                            "Stale or unsupported config.toml entries detected ({} issues)",
                            report.issue_count
                        ),
                    )
                    .with_details(details)
                    .with_diagnostic_report(report),
                );
            }
            Ok(_) => {}
            Err(error) => {
                findings.push(
                    DoctorConfigFinding::new(
                        "error",
                        "Could not validate config.toml against the current schema",
                    )
                    .with_details(format_validation_error(&error)),
                );
            }
        }

        return DoctorConfigEvaluateData { findings };
    }

    if legacy_nix_config.exists() {
        return DoctorConfigEvaluateData {
            findings: vec![
                DoctorConfigFinding::new("warning", "Legacy yazelix.nix configuration detected")
                    .with_details(path_to_string(&legacy_nix_config)),
            ],
        };
    }

    let inherited = NormalizeConfigRequest {
        config_path: paths.user_config,
        default_config_path: paths.default_config_path,
        contract_path: paths.contract_path,
    };
    match normalize_config(&inherited) {
        Ok(_) => DoctorConfigEvaluateData {
            findings: vec![
                DoctorConfigFinding::new(
                    "info",
                    "Using default configuration (config_default.toml)",
                )
                .with_details("No explicit config.toml overrides are present"),
            ],
        },
        Err(error) => DoctorConfigEvaluateData {
            findings: vec![
                DoctorConfigFinding::new(
                    "error",
                    "Could not validate the inherited Yazelix configuration",
                )
                .with_details(format_validation_error(&error)),
            ],
        },
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

            if matches!(
                error.code(),
                "invalid_toml" | "invalid_main_config_toml" | "settings_jsonc_not_object"
            ) {
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
        "stale_old_settings_input" => {
            if let Some(user_config) = details.get("user_config").and_then(Value::as_str) {
                lines.push(format!("canonical settings: {user_config}"));
            }
            if let Some(old_flat) = details.get("old_flat_user_config").and_then(Value::as_str) {
                lines.push(format!("old flat main: {old_flat}"));
            }
            if let Some(legacy_main) = details.get("legacy_user_config").and_then(Value::as_str) {
                lines.push(format!("old nested main: {legacy_main}"));
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
        ErrorClass::Runtime | ErrorClass::Internal => "runtime",
        ErrorClass::Usage | ErrorClass::Io => "host-dependency",
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
        "runtime" => "runtime problem",
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
    use tempfile::TempDir;

    fn write_runtime_config_contract(runtime_dir: &Path) {
        std::fs::create_dir_all(runtime_dir.join("config_metadata")).unwrap();
        std::fs::write(
            runtime_dir.join("config_default.toml"),
            include_str!("../../../config_default.toml"),
        )
        .unwrap();
        std::fs::write(
            runtime_dir.join("config_metadata/main_config_contract.toml"),
            include_str!("../../../config_metadata/main_config_contract.toml"),
        )
        .unwrap();
    }

    // Defends: an absent user config is healthy inherited state, not a repair finding.
    #[test]
    fn missing_user_settings_report_packaged_inheritance() {
        let tmp = TempDir::new().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        let config_dir = tmp.path().join("config");
        write_runtime_config_contract(&runtime_dir);

        let report = evaluate_doctor_config_report(&DoctorConfigEvaluateRequest {
            config_dir,
            runtime_dir,
        });

        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].status, "info");
        assert!(!report.findings[0].fix_available);
        assert!(report.findings[0].fix_action.is_none());
    }

    // Regression: doctor validates inherited packaged values instead of treating file presence as health.
    #[test]
    fn damaged_packaged_defaults_are_reported() {
        let tmp = TempDir::new().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        let config_dir = tmp.path().join("config");
        write_runtime_config_contract(&runtime_dir);
        let damaged = std::fs::read_to_string(runtime_dir.join("config_default.toml"))
            .unwrap()
            .replacen("mode = \"dark\"\n", "", 1);
        std::fs::write(runtime_dir.join("config_default.toml"), damaged).unwrap();

        let report = evaluate_doctor_config_report(&DoctorConfigEvaluateRequest {
            config_dir,
            runtime_dir,
        });

        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].status, "error");
        assert!(
            report.findings[0]
                .details
                .as_deref()
                .is_some_and(|details| {
                    details.contains("missing the required default")
                        && details.contains("Failure class: runtime problem")
                })
        );
    }

    // Regression: a dangling canonical owner is a visible config error, not inherited state.
    #[cfg(unix)]
    #[test]
    fn dangling_user_settings_owner_is_not_reported_as_inherited() {
        use std::os::unix::fs::symlink;

        let tmp = TempDir::new().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        let config_dir = tmp.path().join("config");
        std::fs::create_dir_all(&runtime_dir).unwrap();
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(runtime_dir.join("config_default.toml"), "[core]\n").unwrap();
        symlink(
            config_dir.join("missing.toml"),
            config_dir.join("config.toml"),
        )
        .unwrap();

        let report = evaluate_doctor_config_report(&DoctorConfigEvaluateRequest {
            config_dir,
            runtime_dir,
        });

        assert_eq!(report.findings.len(), 2);
        assert_eq!(report.findings[1].status, "error");
        assert!(
            report.findings[1]
                .message
                .contains("Could not validate config.toml")
        );
    }
}
