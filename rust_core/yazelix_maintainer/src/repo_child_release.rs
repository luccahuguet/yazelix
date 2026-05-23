//! Maintainer validation for coupled child-repo release transactions.

use crate::repo_validation::ValidationReport;
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChildInputLock {
    node: String,
    owner: String,
    repo: String,
    rev: String,
}

pub fn validate_child_release_transaction(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let lock_path = repo_root.join("flake.lock");
    let raw = fs::read_to_string(&lock_path)
        .map_err(|error| format!("Failed to read {}: {error}", lock_path.display()))?;
    let inputs = locked_child_inputs(&raw)?;
    if inputs.is_empty() {
        report
            .errors
            .push("flake.lock contains no first-party Yazelix child inputs".to_string());
        return Ok(report);
    }

    for input in inputs {
        if let Some(warning) = local_child_checkout_warning(repo_root, &input)? {
            report.warnings.push(warning);
        }
        if !remote_rev_is_fetchable(&input)? {
            report.errors.push(format!(
                "Child input `{}` pins unpublished or unreachable commit {} for {}/{}. Push the child repo first, update flake.lock to the published revision, then run no-overrides validation.",
                input.node, input.rev, input.owner, input.repo
            ));
        }
    }

    Ok(report)
}

fn locked_child_inputs(raw_lock: &str) -> Result<Vec<ChildInputLock>, String> {
    let parsed: JsonValue = serde_json::from_str(raw_lock)
        .map_err(|error| format!("Invalid flake.lock JSON: {error}"))?;
    let nodes = parsed
        .get("nodes")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| "flake.lock is missing object `nodes`".to_string())?;
    let mut inputs = Vec::new();
    for (node, data) in nodes {
        let Some(locked) = data.get("locked").and_then(JsonValue::as_object) else {
            continue;
        };
        if locked.get("type").and_then(JsonValue::as_str) != Some("github") {
            continue;
        }
        let owner = locked
            .get("owner")
            .and_then(JsonValue::as_str)
            .unwrap_or("");
        let repo = locked.get("repo").and_then(JsonValue::as_str).unwrap_or("");
        let rev = locked.get("rev").and_then(JsonValue::as_str).unwrap_or("");
        if owner != "luccahuguet" || !repo.starts_with("yazelix-") || rev.is_empty() {
            continue;
        }
        inputs.push(ChildInputLock {
            node: node.to_string(),
            owner: owner.to_string(),
            repo: repo.to_string(),
            rev: rev.to_string(),
        });
    }
    inputs.sort_by(|left, right| left.node.cmp(&right.node));
    Ok(inputs)
}

fn local_child_checkout_warning(
    repo_root: &Path,
    input: &ChildInputLock,
) -> Result<Option<String>, String> {
    let Some(parent) = repo_root.parent() else {
        return Ok(None);
    };
    let checkout = parent.join(&input.repo);
    if !checkout.join(".git").exists() {
        return Ok(None);
    }

    let status = Command::new("git")
        .args([
            "-C",
            checkout.to_string_lossy().as_ref(),
            "status",
            "--short",
        ])
        .output()
        .map_err(|error| {
            format!(
                "Failed to inspect local child checkout {}: {error}",
                checkout.display()
            )
        })?;
    if !status.status.success() {
        return Err(format!(
            "Failed to inspect local child checkout {}\n{}",
            checkout.display(),
            String::from_utf8_lossy(&status.stderr).trim()
        ));
    }
    let dirty = String::from_utf8_lossy(&status.stdout).trim().to_string();
    if dirty.is_empty() {
        return Ok(None);
    }
    Ok(Some(format!(
        "Local child checkout {} has uncommitted changes; finish or stash them before running a coupled release transaction.",
        checkout.display()
    )))
}

fn remote_rev_is_fetchable(input: &ChildInputLock) -> Result<bool, String> {
    let url = format!("https://github.com/{}/{}.git", input.owner, input.repo);
    let probe_dir = remote_rev_probe_dir(input)?;
    let init = Command::new("git")
        .args(["init", "--bare"])
        .arg(&probe_dir)
        .output()
        .map_err(|error| format!("Failed to initialize {}: {error}", probe_dir.display()))?;
    if !init.status.success() {
        return Err(format!(
            "Failed to initialize {}\n{}",
            probe_dir.display(),
            String::from_utf8_lossy(&init.stderr).trim()
        ));
    }

    let fetch = Command::new("git")
        .arg("-C")
        .arg(&probe_dir)
        .args(["fetch", "--depth=1", &url, &input.rev])
        .output()
        .map_err(|error| {
            format!(
                "Failed to run `git fetch --depth=1 {url} {}`: {error}",
                input.rev
            )
        });
    let cleanup = fs::remove_dir_all(&probe_dir)
        .map_err(|error| format!("Failed to remove {}: {error}", probe_dir.display()));

    let fetch = fetch?;
    cleanup?;
    if fetch.status.success() {
        return Ok(true);
    }

    let stderr = String::from_utf8_lossy(&fetch.stderr);
    if fetch_failure_means_missing_revision(&stderr) {
        return Ok(false);
    }

    Err(format!(
        "Failed to fetch locked revision {} from {url}\n{}",
        input.rev,
        stderr.trim()
    ))
}

fn remote_rev_probe_dir(input: &ChildInputLock) -> Result<PathBuf, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("System clock is before UNIX_EPOCH: {error}"))?
        .as_nanos();
    Ok(std::env::temp_dir().join(format!(
        "yazelix_child_release_probe_{}_{}_{}",
        std::process::id(),
        stamp,
        input.repo
    )))
}

fn fetch_failure_means_missing_revision(stderr: &str) -> bool {
    stderr.contains("not our ref")
        || stderr.contains("couldn't find remote ref")
        || stderr.contains("Server does not allow request for unadvertised object")
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: the child-release validator scopes itself to first-party Yazelix GitHub inputs instead of every flake dependency.
    #[test]
    fn locked_child_inputs_collects_first_party_yazelix_nodes_only() {
        let inputs = locked_child_inputs(
            r#"{
              "nodes": {
                "nixpkgs": {
                  "locked": {
                    "type": "github",
                    "owner": "NixOS",
                    "repo": "nixpkgs",
                    "rev": "1111111111111111111111111111111111111111"
                  }
                },
                "yazelixScreen": {
                  "locked": {
                    "type": "github",
                    "owner": "luccahuguet",
                    "repo": "yazelix-screen",
                    "rev": "2222222222222222222222222222222222222222"
                  }
                }
              }
            }"#,
        )
        .unwrap();

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].node, "yazelixScreen");
        assert_eq!(inputs[0].repo, "yazelix-screen");
    }

    // Regression: unpublished-child detection must treat a missing fetched object as a validation error without conflating transport failures with unpublished commits.
    #[test]
    fn fetch_failure_classifier_identifies_unreachable_revision() {
        assert!(fetch_failure_means_missing_revision(
            "fatal: remote error: upload-pack: not our ref cccccccccccccccccccccccccccccccccccccccc"
        ));
        assert!(fetch_failure_means_missing_revision(
            "fatal: couldn't find remote ref cccccccccccccccccccccccccccccccccccccccc"
        ));
        assert!(!fetch_failure_means_missing_revision(
            "fatal: unable to access 'https://github.com/example/repo.git/': Could not resolve host: github.com"
        ));
    }
}
