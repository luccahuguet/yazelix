//! Helix-focused doctor findings (runtime conflicts, runtime health, managed integration).
//! Bead: yazelix-ulb2.4.2

use crate::helix_materialization::{MANAGED_REVEAL_COMMAND, build_managed_helix_contract_json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn default_reveal_binding_expected() -> String {
    MANAGED_REVEAL_COMMAND.into()
}

#[derive(Debug, Deserialize)]
pub struct HelixDoctorEvaluateRequest {
    pub home_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub config_dir: PathBuf,
    pub user_config_helix_runtime_dir: PathBuf,
    #[serde(default)]
    pub hx_exe_path: Option<PathBuf>,
    pub include_runtime_health: bool,
    /// When `None`, managed Helix integration checks are skipped (e.g. yazelix.toml could not be parsed).
    #[serde(default)]
    pub editor_command: Option<String>,
    pub managed_helix_user_config_path: PathBuf,
    pub native_helix_config_path: PathBuf,
    pub generated_helix_config_path: PathBuf,
    #[serde(default)]
    pub expected_managed_config: Option<Value>,
    #[serde(default)]
    pub build_managed_config_error: Option<String>,
    #[serde(default = "default_reveal_binding_expected")]
    pub reveal_binding_expected: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HelixRuntimeConflictEntry {
    pub path: String,
    pub priority: i32,
    pub name: String,
    pub severity: String,
}

#[derive(Debug, Serialize)]
pub struct HelixDoctorFinding {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub fix_available: bool,
    pub fix_commands: Vec<String>,
    pub conflicts: Vec<HelixRuntimeConflictEntry>,
}

#[derive(Debug, Serialize)]
pub struct HelixDoctorEvaluateData {
    pub runtime_conflicts: HelixDoctorFinding,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_health: Option<HelixDoctorFinding>,
    pub managed_integration: Vec<HelixDoctorFinding>,
}

pub fn evaluate_helix_doctor_report(
    request: &HelixDoctorEvaluateRequest,
) -> HelixDoctorEvaluateData {
    let runtime_conflicts = evaluate_runtime_conflicts(request);
    let runtime_health = if request.include_runtime_health {
        Some(evaluate_runtime_health(request))
    } else {
        None
    };
    let managed_integration = evaluate_managed_integration(request);

    HelixDoctorEvaluateData {
        runtime_conflicts,
        runtime_health,
        managed_integration,
    }
}

fn is_helix_editor_command(editor: &str) -> bool {
    let t = editor.trim();
    t.is_empty() || t.ends_with("/hx") || t == "hx" || t.ends_with("/helix") || t == "helix"
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn helix_health_runtime_directories(request: &HelixDoctorEvaluateRequest) -> Vec<PathBuf> {
    let output = match &request.hx_exe_path {
        Some(p) => Command::new(p).arg("--health").output(),
        None => Command::new("hx").arg("--health").output(),
    };
    let Ok(out) = output else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("Runtime directories:") else {
            continue;
        };
        let rest = rest.trim();
        return rest
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter(|p| Path::new(p).exists())
            .map(PathBuf::from)
            .collect();
    }
    Vec::new()
}

fn evaluate_runtime_conflicts(request: &HelixDoctorEvaluateRequest) -> HelixDoctorFinding {
    let user_runtime = &request.user_config_helix_runtime_dir;
    let mut conflicts: Vec<HelixRuntimeConflictEntry> = Vec::new();
    let mut has_high_priority_conflict = false;

    if user_runtime.exists() {
        conflicts.push(HelixRuntimeConflictEntry {
            path: path_to_string(user_runtime),
            priority: 2,
            name: "User config runtime".into(),
            severity: "error".into(),
        });
        has_high_priority_conflict = true;
    }

    let all_runtimes = helix_health_runtime_directories(request);
    let effective_runtime = all_runtimes.first().cloned();

    if let Some(ref hx_path) = request.hx_exe_path {
        if hx_path.exists() {
            let exe_runtime = hx_path
                .parent()
                .map(|p| p.join("runtime"))
                .unwrap_or_else(|| PathBuf::from("runtime"));
            if exe_runtime.exists() {
                if effective_runtime.as_ref() != Some(&exe_runtime) {
                    conflicts.push(HelixRuntimeConflictEntry {
                        path: path_to_string(&exe_runtime),
                        priority: 5,
                        name: "Executable sibling runtime".into(),
                        severity: "warning".into(),
                    });
                }
            }
        }
    }

    if conflicts.is_empty() {
        return HelixDoctorFinding {
            status: "ok".into(),
            message: "No conflicting Helix runtime directories found".into(),
            details: Some("Helix runtime search order will behave as intended".into()),
            fix_available: false,
            fix_commands: vec![],
            conflicts: vec![],
        };
    }

    let status = if has_high_priority_conflict {
        "error"
    } else {
        "warning"
    };
    let conflict_details = conflicts
        .iter()
        .map(|c| format!("{}: {} (priority {})", c.name, c.path, c.priority))
        .collect::<Vec<_>>()
        .join(", ");

    let message = if has_high_priority_conflict {
        "HIGH PRIORITY: ~/.config/helix/runtime will override the intended Helix runtime".into()
    } else {
        "Lower priority runtime directories found".into()
    };

    let fix_commands = if has_high_priority_conflict {
        let ur = path_to_string(user_runtime);
        vec![
            "# Backup and remove conflicting runtime:".into(),
            format!("mv {ur} {ur}.backup"),
            "# Or if you want to delete it:".into(),
            format!("rm -rf {ur}"),
        ]
    } else {
        vec![]
    };

    HelixDoctorFinding {
        status: status.into(),
        message,
        details: Some(format!(
            "Conflicting runtimes: {conflict_details}. Helix searches in priority order and will use files from higher priority directories, potentially breaking syntax highlighting."
        )),
        fix_available: has_high_priority_conflict,
        fix_commands,
        conflicts,
    }
}

fn evaluate_runtime_health(request: &HelixDoctorEvaluateRequest) -> HelixDoctorFinding {
    let all_runtimes = helix_health_runtime_directories(request);
    let primary_runtime = all_runtimes.first().cloned();

    let Some(primary) = primary_runtime else {
        return HelixDoctorFinding {
            status: "error".into(),
            message: "Helix runtime could not be resolved".into(),
            details: Some(
                "Helix did not report any valid runtime directory in `hx --health`".into(),
            ),
            fix_available: false,
            fix_commands: vec![],
            conflicts: vec![],
        };
    };

    let required_dirs = ["grammars", "queries", "themes"];
    let missing_dirs: Vec<&str> = required_dirs
        .iter()
        .copied()
        .filter(|&required_dir| {
            !all_runtimes
                .iter()
                .any(|runtime_path| runtime_path.join(required_dir).exists())
        })
        .collect();

    if !missing_dirs.is_empty() {
        return HelixDoctorFinding {
            status: "error".into(),
            message: format!("Missing required directories: {}", missing_dirs.join(", ")),
            details: Some(format!(
                "The effective Helix runtime at {} is incomplete (note: Nix may split runtime across multiple paths)",
                primary.display()
            )),
            fix_available: false,
            fix_commands: vec![],
            conflicts: vec![],
        };
    }

    let grammar_count: usize = all_runtimes
        .iter()
        .map(|runtime_path| {
            let g = runtime_path.join("grammars");
            fs::read_dir(&g).map(|d| d.count()).unwrap_or(0)
        })
        .sum();

    if grammar_count < 200 {
        return HelixDoctorFinding {
            status: "warning".into(),
            message: format!("Only {grammar_count} grammar files found (expected 200+)"),
            details: Some("Some languages may not have syntax highlighting".into()),
            fix_available: false,
            fix_commands: vec![],
            conflicts: vec![],
        };
    }

    let tutor_exists = all_runtimes
        .iter()
        .any(|runtime_path| runtime_path.join("tutor").exists());

    if !tutor_exists {
        return HelixDoctorFinding {
            status: "warning".into(),
            message: "Helix tutor file missing".into(),
            details: Some("Tutorial will not be available".into()),
            fix_available: false,
            fix_commands: vec![],
            conflicts: vec![],
        };
    }

    HelixDoctorFinding {
        status: "ok".into(),
        message: format!("Helix runtime healthy with {grammar_count} grammars"),
        details: Some(format!("Primary runtime directory: {}", primary.display())),
        fix_available: false,
        fix_commands: vec![],
        conflicts: vec![],
    }
}

fn a_r_binding_from_json(config: &Value) -> Option<String> {
    config
        .get("keys")?
        .get("normal")?
        .get("A-r")?
        .as_str()
        .map(str::to_owned)
}

fn read_a_r_binding_from_toml_file(path: &Path) -> Result<Option<String>, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("failed to read generated config: {error}"))?;
    let v: toml::Value =
        toml::from_str(&raw).map_err(|error| format!("failed to parse TOML: {error}"))?;

    Ok(v.get("keys")
        .and_then(|keys| keys.get("normal"))
        .and_then(|normal| normal.get("A-r"))
        .and_then(|binding| binding.as_str())
        .map(str::to_owned))
}

fn stale_generated_config_finding(path: &Path) -> HelixDoctorFinding {
    HelixDoctorFinding {
        status: "warning".into(),
        message: "Managed Helix generated config is stale or invalid".into(),
        details: Some(format!(
            "Generated config: {}\nExpected `A-r` to run `yzx reveal`.\nLaunch a managed Helix session again to regenerate it.",
            path.display()
        )),
        fix_available: false,
        fix_commands: vec![],
        conflicts: vec![],
    }
}

fn unreadable_generated_config_finding(path: &Path, error: &str) -> HelixDoctorFinding {
    HelixDoctorFinding {
        status: "warning".into(),
        message: "Managed Helix generated config could not be read".into(),
        details: Some(format!(
            "Generated config: {}\nUnderlying error: {}",
            path.display(),
            error
        )),
        fix_available: false,
        fix_commands: vec![],
        conflicts: vec![],
    }
}

fn evaluate_managed_integration(request: &HelixDoctorEvaluateRequest) -> Vec<HelixDoctorFinding> {
    let Some(editor) = request.editor_command.as_deref() else {
        return Vec::new();
    };
    if !is_helix_editor_command(editor) {
        return Vec::new();
    }

    let mut out: Vec<HelixDoctorFinding> = Vec::new();

    let managed = &request.managed_helix_user_config_path;
    let native = &request.native_helix_config_path;
    let expected_reveal_binding = request.reveal_binding_expected.as_str();

    if !managed.exists() && native.exists() {
        out.push(HelixDoctorFinding {
            status: "info".into(),
            message: "Personal Helix config has not been imported into Yazelix-managed Helix".into(),
            details: Some(format!(
                "Native config: {}\nManaged config: {}\nRun `yzx import helix` if you want Yazelix-managed Helix sessions to reuse that personal config.",
                native.display(),
                managed.display()
            )),
            fix_available: false,
            fix_commands: vec![],
            conflicts: vec![],
        });
    }

    if let Some(ref err) = request.build_managed_config_error {
        if !err.trim().is_empty() {
            out.push(HelixDoctorFinding {
                status: "error".into(),
                message: "Managed Helix config contract could not be built".into(),
                details: Some(err.clone()),
                fix_available: false,
                fix_commands: vec![],
                conflicts: vec![],
            });
            return out;
        }
    }

    let expected = if let Some(ref expected) = request.expected_managed_config {
        expected.clone()
    } else {
        match build_managed_helix_contract_json(&request.runtime_dir, &request.config_dir) {
            Ok(expected) => expected,
            Err(error) => {
                out.push(HelixDoctorFinding {
                    status: "error".into(),
                    message: "Managed Helix config contract could not be built".into(),
                    details: Some(format!(
                        "{}\nNext: {}",
                        error.message(),
                        error.remediation()
                    )),
                    fix_available: false,
                    fix_commands: vec![],
                    conflicts: vec![],
                });
                return out;
            }
        }
    };

    if a_r_binding_from_json(&expected).as_deref() != Some(expected_reveal_binding.trim()) {
        out.push(HelixDoctorFinding {
            status: "error".into(),
            message: "Managed Helix config contract lost the Yazelix reveal binding".into(),
            details: Some(
                "The expected managed Helix config no longer enforces `A-r = :sh yzx reveal \"%{buffer_name}\"`."
                    .into(),
            ),
            fix_available: false,
            fix_commands: vec![],
            conflicts: vec![],
        });
        return out;
    }

    let generated = &request.generated_helix_config_path;
    if !generated.exists() {
        out.push(HelixDoctorFinding {
            status: "info".into(),
            message: "Managed Helix config has not been materialized yet".into(),
            details: Some(format!(
                "Expected generated config: {}\nThis is normal before the first managed Helix launch. Yazelix will generate it on demand.",
                generated.display()
            )),
            fix_available: false,
            fix_commands: vec![],
            conflicts: vec![],
        });
        return out;
    }

    let gen_binding = match read_a_r_binding_from_toml_file(generated) {
        Ok(Some(binding)) => binding,
        Ok(None) => {
            out.push(stale_generated_config_finding(generated));
            return out;
        }
        Err(error) => {
            out.push(unreadable_generated_config_finding(generated, &error));
            return out;
        }
    };

    if gen_binding.trim() != expected_reveal_binding.trim() {
        out.push(stale_generated_config_finding(generated));
        return out;
    }

    out.push(HelixDoctorFinding {
        status: "ok".into(),
        message: "Managed Helix reveal integration is healthy".into(),
        details: Some(generated.display().to_string()),
        fix_available: false,
        fix_commands: vec![],
        conflicts: vec![],
    });

    out
}

// Test lane: default

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    fn write_executable(path: &Path, body: &str) {
        fs::write(path, body).unwrap();
        let mut perms = fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).unwrap();
    }

    // Defends: Helix doctor flags runtime conflicts when a user config runtime shadows the managed runtime.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn runtime_conflicts_flag_user_config_runtime() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let ur = home.join(".config/helix/runtime");
        fs::create_dir_all(&ur).unwrap();

        let req = HelixDoctorEvaluateRequest {
            home_dir: home.clone(),
            runtime_dir: tmp.path().join("runtime"),
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: ur.clone(),
            hx_exe_path: None,
            include_runtime_health: false,
            editor_command: Some("nvim".into()),
            managed_helix_user_config_path: home.join("m.toml"),
            native_helix_config_path: home.join("n.toml"),
            generated_helix_config_path: home.join("g.toml"),
            expected_managed_config: None,
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };
        let f = evaluate_runtime_conflicts(&req);
        assert_eq!(f.status, "error");
        assert!(!f.conflicts.is_empty());
    }

    // Defends: Helix runtime health reports `ok` when `hx --health` exposes a complete runtime tree.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn runtime_health_ok_from_fake_hx_health_output() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let rt = tmp.path().join("helix-runtime");
        fs::create_dir_all(rt.join("grammars")).unwrap();
        fs::create_dir_all(rt.join("queries")).unwrap();
        fs::create_dir_all(rt.join("themes")).unwrap();
        fs::create_dir_all(rt.join("tutor")).unwrap();
        for i in 0..250 {
            let _ = fs::File::create(rt.join("grammars").join(format!("g{i}.so")));
        }

        let fake_hx = tmp.path().join("hx");
        let health_line = format!("Runtime directories: {}", rt.to_string_lossy());
        write_executable(
            &fake_hx,
            &format!("#!/bin/sh\nprintf '%s\\n' '{health_line}'\n"),
        );

        let req = HelixDoctorEvaluateRequest {
            home_dir: home,
            runtime_dir: tmp.path().join("runtime"),
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("norun"),
            hx_exe_path: Some(fake_hx),
            include_runtime_health: true,
            editor_command: Some("hx".into()),
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: tmp.path().join("g.toml"),
            expected_managed_config: None,
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };

        let h = evaluate_runtime_health(&req);
        assert_eq!(h.status, "ok");
    }

    // Defends: managed Helix integration skips non-Helix editor commands instead of fabricating findings.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn managed_integration_skips_non_helix_editor() {
        let tmp = TempDir::new().unwrap();
        let req = HelixDoctorEvaluateRequest {
            home_dir: tmp.path().join("home"),
            runtime_dir: tmp.path().join("runtime"),
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("ur"),
            hx_exe_path: None,
            include_runtime_health: false,
            editor_command: Some("nvim".into()),
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: tmp.path().join("g.toml"),
            expected_managed_config: None,
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };
        assert!(evaluate_managed_integration(&req).is_empty());
    }

    // Defends: managed Helix integration skips checks when no editor command is configured.
    // Strength: defect=1 behavior=2 resilience=2 cost=2 uniqueness=1 total=8/10
    #[test]
    fn managed_integration_skips_when_editor_command_absent() {
        let tmp = TempDir::new().unwrap();
        let req = HelixDoctorEvaluateRequest {
            home_dir: tmp.path().join("home"),
            runtime_dir: tmp.path().join("runtime"),
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("ur"),
            hx_exe_path: None,
            include_runtime_health: false,
            editor_command: None,
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: tmp.path().join("g.toml"),
            expected_managed_config: None,
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };
        assert!(evaluate_managed_integration(&req).is_empty());
    }

    // Defends: runtime conflict detection still warns on an executable sibling runtime even without `hx --health` output.
    // Strength: defect=2 behavior=2 resilience=2 cost=2 uniqueness=1 total=9/10
    #[test]
    fn runtime_conflicts_warn_on_sibling_runtime_even_without_health_output() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let hx_dir = tmp.path().join("bin");
        let sibling_runtime = hx_dir.join("runtime");
        fs::create_dir_all(&sibling_runtime).unwrap();

        let fake_hx = hx_dir.join("hx");
        write_executable(&fake_hx, "#!/bin/sh\nexit 1\n");

        let req = HelixDoctorEvaluateRequest {
            home_dir: home,
            runtime_dir: tmp.path().join("runtime"),
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("ur"),
            hx_exe_path: Some(fake_hx),
            include_runtime_health: false,
            editor_command: None,
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: tmp.path().join("g.toml"),
            expected_managed_config: None,
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };

        let finding = evaluate_runtime_conflicts(&req);
        assert_eq!(finding.status, "warning");
        assert_eq!(finding.conflicts.len(), 1);
        assert_eq!(finding.conflicts[0].name, "Executable sibling runtime");
    }

    // Defends: missing managed Helix reveal bindings are classified as stale generated config.
    // Strength: defect=2 behavior=2 resilience=2 cost=2 uniqueness=1 total=9/10
    #[test]
    fn managed_integration_treats_missing_generated_binding_as_stale() {
        let tmp = TempDir::new().unwrap();
        let generated = tmp.path().join("generated.toml");
        fs::write(&generated, "[keys.normal]\nB-r = \":noop\"\n").unwrap();

        let req = HelixDoctorEvaluateRequest {
            home_dir: tmp.path().join("home"),
            runtime_dir: tmp.path().join("runtime"),
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("ur"),
            hx_exe_path: None,
            include_runtime_health: false,
            editor_command: Some("hx".into()),
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: generated,
            expected_managed_config: Some(serde_json::json!({
                "keys": {
                    "normal": {
                        "A-r": ":sh yzx reveal \"%{buffer_name}\""
                    }
                }
            })),
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };

        let findings = evaluate_managed_integration(&req);
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].message,
            "Managed Helix generated config is stale or invalid"
        );
    }
}
