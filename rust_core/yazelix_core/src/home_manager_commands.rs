//! `yzx home_manager` family implemented in Rust for `yzx_control`.

use crate::bridge::{CoreError, ErrorClass};
use crate::install_ownership_env::install_ownership_request_from_env;
use crate::install_ownership_report::{
    HOME_MANAGER_PREPARE_ACTION_ARCHIVE_PATH, HOME_MANAGER_PREPARE_ACTION_REMOVE_PROFILE_ENTRY,
    HomeManagerPrepareArtifact, evaluate_install_ownership_report,
};
use serde_json::json;
use std::fs;
use std::io::{self, BufRead, Write};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const HOME_MANAGER_PREPARE_BACKUP_LABEL: &str = "home-manager-prepare";

struct PrepareArgs {
    apply: bool,
    yes: bool,
    help: bool,
}

struct ArchivedArtifact {
    label: String,
    path: String,
    backup_path: String,
}

struct RemovedProfileArtifact {
    label: String,
    description: String,
    remove_target: String,
}

pub fn run_yzx_home_manager(args: &[String]) -> Result<i32, CoreError> {
    if args.is_empty() || matches!(args[0].as_str(), "help" | "-h" | "--help") {
        print_home_manager_root();
        return Ok(0);
    }

    match args[0].as_str() {
        "prepare" => run_home_manager_prepare(&args[1..]),
        other => Err(CoreError::usage(format!(
            "Unknown yzx home_manager argument: {other}. Try `yzx home_manager` or `yzx home_manager prepare --help`."
        ))),
    }
}

fn print_home_manager_root() {
    println!("Yazelix Home Manager helpers");
    println!(
        "  yzx home_manager prepare   Preview or remove takeover blockers before Home Manager takeover"
    );
    println!(
        "  yzx update home_manager    Refresh the current flake input, then print `home-manager switch`"
    );
}

fn print_home_manager_prepare_help() {
    println!("Preview or remove manual-install takeover blockers before Home Manager takeover");
    println!();
    println!("Usage:");
    println!("  yzx home_manager prepare [--apply] [--yes]");
    println!();
    println!("Flags:");
    println!(
        "      --apply  Archive file-based takeover artifacts and remove standalone default-profile Yazelix entries"
    );
    println!("      --yes    Skip confirmation prompt when using --apply");
}

fn parse_prepare_args(args: &[String]) -> Result<PrepareArgs, CoreError> {
    let mut parsed = PrepareArgs {
        apply: false,
        yes: false,
        help: false,
    };

    for arg in args {
        match arg.as_str() {
            "--apply" => parsed.apply = true,
            "--yes" => parsed.yes = true,
            "-h" | "--help" | "help" => parsed.help = true,
            other => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx home_manager prepare: {other}"
                )));
            }
        }
    }

    Ok(parsed)
}

fn format_prepare_artifact(artifact: &HomeManagerPrepareArtifact) -> String {
    let tag = if artifact.class == "blocker" {
        "[BLOCKER]"
    } else {
        "[CLEANUP]"
    };

    format!("  - {tag} {}: {}", artifact.label, artifact.path)
}

fn render_prepare_preview(artifacts: &[HomeManagerPrepareArtifact]) -> String {
    let blockers: Vec<&HomeManagerPrepareArtifact> = artifacts
        .iter()
        .filter(|artifact| artifact.class == "blocker")
        .collect();
    let cleanup: Vec<&HomeManagerPrepareArtifact> = artifacts
        .iter()
        .filter(|artifact| artifact.class == "cleanup")
        .collect();

    let mut lines = vec!["Yazelix Home Manager takeover preview".to_string()];

    if !blockers.is_empty() {
        lines.push(String::new());
        lines.push("Blocking manual-install artifacts:".to_string());
        lines.extend(blockers.into_iter().map(format_prepare_artifact));
    }

    if !cleanup.is_empty() {
        lines.push(String::new());
        lines.push("Cleanup-only manual-install artifacts:".to_string());
        lines.extend(cleanup.into_iter().map(format_prepare_artifact));
    }

    lines.push(String::new());
    lines.push(
        "Run `yzx home_manager prepare --apply` to archive file artifacts and remove standalone default-profile Yazelix entries before `home-manager switch`."
            .to_string(),
    );
    lines.join("\n")
}

fn print_no_artifacts_message() {
    println!("No manual-install Yazelix artifacts need Home Manager takeover prep.");
    println!("Next step:");
    println!("  home-manager switch");
}

fn backup_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn archive_artifacts(
    artifacts: &[HomeManagerPrepareArtifact],
    backup_label: &str,
) -> Result<Vec<ArchivedArtifact>, CoreError> {
    let timestamp = backup_timestamp();
    let mut archived = Vec::with_capacity(artifacts.len());

    for artifact in artifacts {
        let backup_path = format!("{}.{}-backup-{}", artifact.path, backup_label, timestamp);
        fs::rename(&artifact.path, &backup_path).map_err(|source| {
            CoreError::io(
                "home_manager_prepare_archive",
                format!(
                    "Could not archive the manual-install artifact at {}.",
                    artifact.path
                ),
                "Fix permissions or move the path manually, then rerun `yzx home_manager prepare --apply`.",
                artifact.path.clone(),
                source,
            )
        })?;
        archived.push(ArchivedArtifact {
            label: artifact.label.clone(),
            path: artifact.path.clone(),
            backup_path,
        });
    }

    Ok(archived)
}

fn prepare_artifact_action(artifact: &HomeManagerPrepareArtifact) -> &str {
    artifact
        .action
        .as_deref()
        .unwrap_or(HOME_MANAGER_PREPARE_ACTION_ARCHIVE_PATH)
}

fn remove_profile_artifacts(
    artifacts: &[HomeManagerPrepareArtifact],
) -> Result<Vec<RemovedProfileArtifact>, CoreError> {
    let remove_targets = artifacts
        .iter()
        .filter_map(|artifact| artifact.remove_target.clone())
        .collect::<Vec<_>>();
    if remove_targets.is_empty() {
        return Ok(Vec::new());
    }

    let output = Command::new("nix")
        .arg("profile")
        .arg("remove")
        .args(&remove_targets)
        .output()
        .map_err(|error| {
            CoreError::classified(
                ErrorClass::Runtime,
                "home_manager_prepare_remove_profile_entries",
                "Could not remove the standalone default-profile Yazelix package entries.",
                "Run `nix profile remove <entry>` yourself, then rerun `yzx home_manager prepare --apply`.",
                json!({
                    "command": format!("nix profile remove {}", remove_targets.join(" ")),
                    "error": error.to_string(),
                }),
            )
        })?;
    if !output.status.success() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "home_manager_prepare_remove_profile_entries",
            "Could not remove the standalone default-profile Yazelix package entries.",
            "Run `nix profile remove <entry>` yourself, then rerun `yzx home_manager prepare --apply`.",
            json!({
                "command": format!("nix profile remove {}", remove_targets.join(" ")),
                "stdout": String::from_utf8_lossy(&output.stdout).trim(),
                "stderr": String::from_utf8_lossy(&output.stderr).trim(),
            }),
        ));
    }

    Ok(artifacts
        .iter()
        .filter_map(|artifact| {
            artifact
                .remove_target
                .as_ref()
                .map(|remove_target| RemovedProfileArtifact {
                    label: artifact.label.clone(),
                    description: artifact.path.clone(),
                    remove_target: remove_target.clone(),
                })
        })
        .collect())
}

fn read_confirmation() -> String {
    let _ = io::stdout().flush();
    let mut line = String::new();
    let _ = io::stdin().lock().read_line(&mut line);
    line.trim().to_lowercase()
}

fn evaluate_prepare_artifacts() -> Result<Vec<HomeManagerPrepareArtifact>, CoreError> {
    let request = install_ownership_request_from_env()?;
    Ok(evaluate_install_ownership_report(&request).prepare_artifacts)
}

fn run_home_manager_prepare(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_prepare_args(args)?;
    if parsed.help {
        print_home_manager_prepare_help();
        return Ok(0);
    }

    let artifacts = evaluate_prepare_artifacts()?;

    if !parsed.apply {
        if artifacts.is_empty() {
            print_no_artifacts_message();
        } else {
            println!("{}", render_prepare_preview(&artifacts));
        }
        return Ok(0);
    }

    if artifacts.is_empty() {
        print_no_artifacts_message();
        return Ok(0);
    }

    if !parsed.yes {
        println!(
            "⚠️  This archives file-based Yazelix takeover artifacts and removes standalone default-profile Yazelix entries so Home Manager can take over cleanly."
        );
        println!(
            "   Archived files stay next to the original path with a timestamped `.home-manager-prepare-backup-*` suffix."
        );
        print!("Continue? [y/N]: ");
        let confirm = read_confirmation();
        if confirm != "y" && confirm != "yes" {
            println!("Aborted.");
            return Ok(0);
        }
    }

    let archive_targets = artifacts
        .iter()
        .filter(|artifact| {
            prepare_artifact_action(artifact) == HOME_MANAGER_PREPARE_ACTION_ARCHIVE_PATH
        })
        .cloned()
        .collect::<Vec<_>>();
    let profile_remove_targets = artifacts
        .iter()
        .filter(|artifact| {
            prepare_artifact_action(artifact) == HOME_MANAGER_PREPARE_ACTION_REMOVE_PROFILE_ENTRY
        })
        .cloned()
        .collect::<Vec<_>>();

    let archived = archive_artifacts(&archive_targets, HOME_MANAGER_PREPARE_BACKUP_LABEL)?;
    let removed = remove_profile_artifacts(&profile_remove_targets)?;

    if !archived.is_empty() {
        println!("Archived manual-install artifacts for Home Manager takeover:");
        for artifact in archived {
            println!(
                "  - {}: {} -> {}",
                artifact.label, artifact.path, artifact.backup_path
            );
        }
    }
    if !removed.is_empty() {
        if !archive_targets.is_empty() {
            println!();
        }
        println!("Removed standalone default-profile Yazelix entries:");
        for artifact in removed {
            println!(
                "  - {}: {} (removed with `nix profile remove {}`)",
                artifact.label, artifact.description, artifact.remove_target
            );
        }
    }
    println!("Next step:");
    println!("  home-manager switch");
    Ok(0)
}
