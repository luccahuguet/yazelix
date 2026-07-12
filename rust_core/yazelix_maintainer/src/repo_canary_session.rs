//! Maintainer-only disposable Yazelix session canary.

use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default)]
pub struct CanarySessionOptions {
    pub dry_run: bool,
    pub keep_session: bool,
    pub session_name: Option<String>,
}

#[derive(Debug, Clone)]
struct CanarySessionPlan {
    session_name: String,
    temp_root: PathBuf,
    temp_home: PathBuf,
    workspace_dir: PathBuf,
    evidence_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CanaryLaunchOutcome {
    SpawnFailed(String),
    Exited { success: bool, code: Option<i32> },
}

pub fn run_disposable_canary_session(
    repo_root: &Path,
    options: &CanarySessionOptions,
) -> Result<(), String> {
    let plan = build_canary_session_plan(options)?;
    if options.dry_run {
        println!(
            "{}",
            json!({
                "session_name": plan.session_name,
                "temp_home": plan.temp_home,
                "workspace_dir": plan.workspace_dir,
                "evidence_dir": plan.evidence_dir,
                "command": "nix build --no-link --print-out-paths .#yazelix; <out>/bin/yzx enter --path <workspace> --with welcome.enabled=false",
            })
        );
        return Ok(());
    }

    let package_root = build_packaged_yazelix(repo_root)?;
    kill_zellij_session(&package_root, &plan.session_name)?;
    prepare_clean_canary_temp_root(&plan)?;

    println!("Canary session: {}", plan.session_name);
    println!("Temporary HOME: {}", plan.temp_home.display());
    println!("Evidence directory: {}", plan.evidence_dir.display());
    println!(
        "Quit Yazelix normally when the session has opened and the sidebar/editor are visible."
    );

    let workspace_arg = plan.workspace_dir.to_string_lossy().into_owned();
    let launch = match Command::new(package_root.join("bin").join("yzx"))
        .args([
            "enter",
            "--path",
            workspace_arg.as_str(),
            "--with",
            "welcome.enabled=false",
        ])
        .env("HOME", &plan.temp_home)
        .env("XDG_CONFIG_HOME", plan.temp_home.join(".config"))
        .env("XDG_DATA_HOME", plan.temp_home.join(".local").join("share"))
        .env("YAZELIX_ZELLIJ_SESSION_NAME", &plan.session_name)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
    {
        Ok(status) => CanaryLaunchOutcome::Exited {
            success: status.success(),
            code: status.code(),
        },
        Err(error) => CanaryLaunchOutcome::SpawnFailed(format!(
            "Failed to launch packaged Yazelix canary session: {error}"
        )),
    };
    let post_session_result = if matches!(launch, CanaryLaunchOutcome::Exited { .. }) {
        run_canary_post_session_checks(&package_root, &plan)
    } else {
        Ok(())
    };
    let cleanup_result = if options.keep_session {
        Ok(())
    } else {
        kill_zellij_session(&package_root, &plan.session_name)
    };
    finish_canary_session(&plan, launch, post_session_result, cleanup_result)
}

fn build_canary_session_plan(options: &CanarySessionOptions) -> Result<CanarySessionPlan, String> {
    let session_name = options
        .session_name
        .clone()
        .unwrap_or_else(default_canary_session_name);
    if !session_name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
    {
        return Err(format!(
            "Invalid canary session name `{session_name}`. Use only ASCII letters, digits, `_`, or `-`."
        ));
    }
    let temp_root = std::env::temp_dir()
        .join("yazelix_canary_sessions")
        .join(&session_name);
    Ok(CanarySessionPlan {
        session_name,
        temp_root: temp_root.clone(),
        temp_home: temp_root.join("home"),
        workspace_dir: temp_root.join("workspace"),
        evidence_dir: temp_root.join("evidence"),
    })
}

fn prepare_clean_canary_temp_root(plan: &CanarySessionPlan) -> Result<(), String> {
    match fs::symlink_metadata(&plan.temp_root) {
        Ok(metadata) if metadata.is_dir() => {
            fs::remove_dir_all(&plan.temp_root).map_err(|error| {
                format!(
                    "Failed to remove stale canary temp root {}: {error}",
                    plan.temp_root.display()
                )
            })?
        }
        Ok(_) => fs::remove_file(&plan.temp_root).map_err(|error| {
            format!(
                "Failed to remove stale canary temp root file {}: {error}",
                plan.temp_root.display()
            )
        })?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "Failed to inspect canary temp root {}: {error}",
                plan.temp_root.display()
            ));
        }
    }

    fs::create_dir_all(&plan.temp_home).map_err(|error| {
        format!(
            "Failed to create canary HOME {}: {error}",
            plan.temp_home.display()
        )
    })?;
    fs::create_dir_all(&plan.workspace_dir).map_err(|error| {
        format!(
            "Failed to create canary workspace {}: {error}",
            plan.workspace_dir.display()
        )
    })?;
    fs::create_dir_all(&plan.evidence_dir).map_err(|error| {
        format!(
            "Failed to create canary evidence dir {}: {error}",
            plan.evidence_dir.display()
        )
    })
}

fn default_canary_session_name() -> String {
    let pid = std::process::id();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("yazelix-canary-{pid}-{now}")
}

fn build_packaged_yazelix(repo_root: &Path) -> Result<PathBuf, String> {
    let output = Command::new("nix")
        .args(["build", "--no-link", "--print-out-paths", ".#yazelix"])
        .current_dir(repo_root)
        .output()
        .map_err(|error| format!("Failed to build .#yazelix for canary session: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Failed to build .#yazelix for canary session\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let package_root = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "nix build .#yazelix returned no output path".to_string())?;
    Ok(package_root)
}

fn capture_canary_doctor(package_root: &Path, plan: &CanarySessionPlan) -> Result<(), String> {
    let output = Command::new(package_root.join("bin").join("yzx"))
        .args(["doctor", "--json"])
        .env("HOME", &plan.temp_home)
        .env("XDG_CONFIG_HOME", plan.temp_home.join(".config"))
        .env("XDG_DATA_HOME", plan.temp_home.join(".local").join("share"))
        .output()
        .map_err(|error| format!("Failed to run canary doctor capture: {error}"))?;
    fs::write(plan.evidence_dir.join("doctor.json"), &output.stdout)
        .map_err(|error| format!("Failed to write canary doctor evidence: {error}"))?;
    fs::write(plan.evidence_dir.join("doctor.stderr"), &output.stderr)
        .map_err(|error| format!("Failed to write canary doctor stderr: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Canary doctor capture failed with status {}",
            output.status.code().unwrap_or(1)
        ));
    }
    Ok(())
}

fn run_canary_post_session_checks(
    package_root: &Path,
    plan: &CanarySessionPlan,
) -> Result<(), String> {
    let mut errors = Vec::new();
    if let Err(error) = capture_canary_doctor(package_root, plan) {
        errors.push(error);
    }
    if let Err(error) = validate_canary_generated_evidence(plan) {
        errors.push(error);
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn finish_canary_session(
    plan: &CanarySessionPlan,
    launch: CanaryLaunchOutcome,
    post_session_result: Result<(), String>,
    cleanup_result: Result<(), String>,
) -> Result<(), String> {
    let mut details = Vec::new();
    if let Err(error) = post_session_result {
        details.push(format!("Post-session validation failed: {error}"));
    }
    if let Err(error) = cleanup_result {
        details.push(format!("Canary Zellij cleanup failed: {error}"));
    }

    let mut message = match launch {
        CanaryLaunchOutcome::SpawnFailed(error) => {
            format!("{error}. Evidence kept at {}", plan.evidence_dir.display())
        }
        CanaryLaunchOutcome::Exited { success: true, .. } if details.is_empty() => {
            println!(
                "Canary completed. Evidence kept at {}",
                plan.evidence_dir.display()
            );
            return Ok(());
        }
        CanaryLaunchOutcome::Exited { success: true, .. } => format!(
            "Canary session exited normally, but validation or cleanup failed. Evidence kept at {}",
            plan.evidence_dir.display()
        ),
        CanaryLaunchOutcome::Exited {
            success: false,
            code,
        } => format!(
            "Canary session exited with status {}. Evidence kept at {}",
            code.unwrap_or(1),
            plan.evidence_dir.display()
        ),
    };
    if !details.is_empty() {
        message.push('\n');
        message.push_str(&details.join("\n"));
    }
    Err(message)
}

fn validate_canary_generated_evidence(plan: &CanarySessionPlan) -> Result<(), String> {
    let state = plan.temp_home.join(".local").join("share").join("yazelix");
    let zellij_config = state.join("configs").join("zellij").join("config.kdl");
    let zellij_side_layout = state
        .join("configs")
        .join("zellij")
        .join("layouts")
        .join("yzx_side.kdl");
    let yazi_config = state.join("configs").join("yazi").join("yazi.toml");
    let expected = [
        zellij_config.as_path(),
        zellij_side_layout.as_path(),
        yazi_config.as_path(),
    ];
    let missing = expected
        .iter()
        .filter(|path| !path.is_file())
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    if missing.is_empty() {
        fs::write(
            plan.evidence_dir.join("generated_assets.txt"),
            expected
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .map_err(|error| format!("Failed to write generated-asset evidence: {error}"))?;
    } else {
        return Err(format!(
            "Canary session did not generate required workspace assets:\n{}",
            missing.join("\n")
        ));
    }

    let zellij_config_raw = read_canary_evidence_file(&zellij_config)?;
    let zellij_side_layout_raw = read_canary_evidence_file(&zellij_side_layout)?;
    let yazi_config_raw = read_canary_evidence_file(&yazi_config)?;
    let checks = canary_contract_checks(
        &zellij_config_raw,
        &zellij_side_layout_raw,
        &yazi_config_raw,
    );
    fs::write(
        plan.evidence_dir.join("generated_contracts.json"),
        serde_json::to_string_pretty(&checks)
            .map_err(|error| format!("Failed to encode canary contract evidence: {error}"))?,
    )
    .map_err(|error| format!("Failed to write canary contract evidence: {error}"))?;
    let failed = checks
        .iter()
        .filter_map(|check| (!check.ok).then_some(check.name.as_str()))
        .collect::<Vec<_>>();
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Canary generated assets are missing required workspace contract markers:\n{}",
            failed.join("\n")
        ))
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct CanaryContractCheck {
    name: String,
    ok: bool,
}

fn canary_contract_checks(
    zellij_config: &str,
    zellij_side_layout: &str,
    yazi_config: &str,
) -> Vec<CanaryContractCheck> {
    [
        (
            "pane_orchestrator_plugin",
            zellij_config.contains("yazelix_pane_orchestrator.wasm"),
        ),
        ("popup_plugin", zellij_config.contains("yzpp.wasm")),
        (
            "right_sidebar_keybinding",
            zellij_config.contains("toggle_editor_right_sidebar_focus"),
        ),
        (
            "bottom_popup_toggle",
            zellij_config.contains("yzx_bottom_popup"),
        ),
        ("top_popup_toggle", zellij_config.contains("yzx_top_popup")),
        (
            "left_yazi_sidebar_layout",
            zellij_side_layout.contains("pane name=\"sidebar\"")
                && zellij_side_layout.contains("args \"sidebar\" \"yazi\""),
        ),
        (
            "managed_editor_yazi_entrypoint",
            yazi_config.contains("yzx_control zellij open-editor"),
        ),
    ]
    .into_iter()
    .map(|(name, ok)| CanaryContractCheck {
        name: name.to_string(),
        ok,
    })
    .collect()
}

fn read_canary_evidence_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("Failed to read canary evidence {}: {error}", path.display()))
}

fn kill_zellij_session(package_root: &Path, session_name: &str) -> Result<(), String> {
    let zellij = package_root.join("libexec").join("zellij");
    if !zellij.is_file() {
        return Err(format!(
            "Packaged Zellij is missing at {}; cannot isolate canary session cleanup",
            zellij.display()
        ));
    }
    let _ = Command::new(zellij)
        .args(["kill-session", session_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("Failed to run packaged Zellij cleanup: {error}"))?;
    Ok(())
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_canary_plan(temp_root: &Path) -> CanarySessionPlan {
        CanarySessionPlan {
            session_name: "yazelix-canary-test".to_string(),
            temp_root: temp_root.to_path_buf(),
            temp_home: temp_root.join("home"),
            workspace_dir: temp_root.join("workspace"),
            evidence_dir: temp_root.join("evidence"),
        }
    }

    // Defends: the disposable session canary uses a named Zellij session so cleanup cannot target the user's active session.
    #[test]
    fn canary_session_plan_uses_safe_named_session() {
        let plan = build_canary_session_plan(&CanarySessionOptions {
            dry_run: true,
            keep_session: false,
            session_name: Some("yazelix-canary-test".to_string()),
        })
        .unwrap();

        assert_eq!(plan.session_name, "yazelix-canary-test");
        assert!(
            plan.temp_root
                .ends_with("yazelix_canary_sessions/yazelix-canary-test")
        );
        assert!(plan.temp_home.ends_with("yazelix-canary-test/home"));
    }

    // Regression: reject shell-like session names before they can reach the Zellij cleanup command.
    #[test]
    fn canary_session_plan_rejects_unsafe_session_name() {
        let err = build_canary_session_plan(&CanarySessionOptions {
            dry_run: true,
            keep_session: false,
            session_name: Some("bad/name".to_string()),
        })
        .unwrap_err();

        assert!(err.contains("Invalid canary session name"));
    }

    // Regression: reusing an explicit canary session name must not let stale generated evidence satisfy a later run.
    #[test]
    fn prepare_clean_canary_temp_root_removes_stale_evidence() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("canary");
        let plan = test_canary_plan(&root);
        fs::create_dir_all(&plan.evidence_dir).unwrap();
        let stale_evidence = plan.evidence_dir.join("generated_assets.txt");
        fs::write(&stale_evidence, "stale").unwrap();

        prepare_clean_canary_temp_root(&plan).unwrap();

        assert!(!stale_evidence.exists());
        assert!(plan.temp_home.is_dir());
        assert!(plan.workspace_dir.is_dir());
        assert!(plan.evidence_dir.is_dir());
    }

    // Regression: a failed launch must remain the primary error even when later evidence checks also fail.
    #[test]
    fn canary_finish_reports_launch_status_before_post_session_errors() {
        let temp = tempdir().unwrap();
        let plan = test_canary_plan(temp.path());

        let err = finish_canary_session(
            &plan,
            CanaryLaunchOutcome::Exited {
                success: false,
                code: Some(23),
            },
            Err("missing generated assets".to_string()),
            Ok(()),
        )
        .unwrap_err();

        assert!(err.starts_with("Canary session exited with status 23."));
        assert!(err.contains("Post-session validation failed: missing generated assets"));
    }

    // Regression: post-session validation failures must not hide cleanup failures.
    #[test]
    fn canary_finish_reports_cleanup_failure_after_validation_failure() {
        let temp = tempdir().unwrap();
        let plan = test_canary_plan(temp.path());

        let err = finish_canary_session(
            &plan,
            CanaryLaunchOutcome::Exited {
                success: true,
                code: Some(0),
            },
            Err("doctor failed".to_string()),
            Err("cleanup failed".to_string()),
        )
        .unwrap_err();

        assert!(err.starts_with("Canary session exited normally"));
        assert!(err.contains("Post-session validation failed: doctor failed"));
        assert!(err.contains("Canary Zellij cleanup failed: cleanup failed"));
    }

    // Defends: canary evidence checks cover the generated plugin, popup, sidebar, and Yazi editor contracts instead of only checking files exist.
    #[test]
    fn canary_contract_checks_report_required_workspace_markers() {
        let checks = canary_contract_checks(
            r#"pane_orchestrator location="file:/x/yazelix_pane_orchestrator.wasm"
yzpp location="file:/x/yzpp.wasm"
name "toggle_editor_right_sidebar_focus"
pane_title "yzx_bottom_popup"
pane_title "yzx_top_popup"
"#,
            r#"pane name="sidebar" {
  args "sidebar" "yazi"
}"#,
            "yzx_control zellij open-editor %s",
        );

        assert!(checks.iter().all(|check| check.ok), "{checks:?}");
        let missing = canary_contract_checks("", "", "");
        assert!(
            missing
                .iter()
                .any(|check| !check.ok && check.name == "pane_orchestrator_plugin")
        );
    }
}
