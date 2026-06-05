//! Maintainer release orchestration for tagged Yazelix releases.

use crate::repo_child_release::validate_child_release_transaction;
use crate::repo_contract_validation::{
    UpgradeContractOptions, validate_config_surface_contract, validate_flake_interface,
    validate_nix_customization_api, validate_readme_version, validate_upgrade_contract,
};
use crate::repo_docs_validation::validate_docs_experience;
use crate::repo_issue_sync::{IssueSyncSummary, run_issue_sync};
use crate::repo_validation::{
    ValidationReport, validate_contracts, validate_rust_test_traceability,
};
use crate::repo_version_bump::{VersionBumpResult, perform_version_bump};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

const REQUIRED_RELEASE_WORKFLOWS: &[&str] = &["CI", "Publish Nix Cache"];
const WORKFLOW_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(60);
const WORKFLOW_DISCOVERY_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseWorkflowOptions {
    pub version: String,
    pub dry_run: bool,
    pub no_push: bool,
    pub no_watch: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReleaseWorkflowResult {
    pub version: String,
    pub dry_run: bool,
    pub commit_sha: Option<String>,
    pub tag: Option<String>,
    pub pushed: bool,
    pub watched_workflows: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct WorkflowRun {
    #[serde(rename = "databaseId")]
    database_id: i64,
    #[serde(rename = "headSha")]
    head_sha: String,
    #[serde(rename = "workflowName")]
    workflow_name: String,
}

pub fn parse_release_workflow_args(args: Vec<String>) -> Result<ReleaseWorkflowOptions, String> {
    let mut iter = args.into_iter();
    let Some(version) = iter.next() else {
        return Err("Missing VERSION for release".to_string());
    };
    let mut options = ReleaseWorkflowOptions {
        version,
        dry_run: false,
        no_push: false,
        no_watch: false,
    };
    for arg in iter {
        match arg.as_str() {
            "--dry-run" => options.dry_run = true,
            "--no-push" => options.no_push = true,
            "--no-watch" => options.no_watch = true,
            _ => return Err(format!("Unknown release option `{arg}`")),
        }
    }
    if options.dry_run {
        options.no_push = true;
        options.no_watch = true;
    }
    if options.no_push {
        options.no_watch = true;
    }
    Ok(options)
}

pub fn run_repo_release_workflow(
    repo_root: &Path,
    options: &ReleaseWorkflowOptions,
) -> Result<ReleaseWorkflowResult, String> {
    ensure_clean_git_worktree(repo_root)?;
    ensure_issue_contract_clean(repo_root)?;
    run_release_validators(repo_root)?;

    if options.dry_run {
        return Ok(ReleaseWorkflowResult {
            version: options.version.clone(),
            dry_run: true,
            commit_sha: None,
            tag: None,
            pushed: false,
            watched_workflows: Vec::new(),
        });
    }

    run_git(repo_root, &["pull", "--rebase"])?;
    ensure_clean_git_worktree(repo_root)?;

    let bump = perform_version_bump(repo_root, &options.version)?;
    run_release_validators(repo_root)?;

    let mut pushed = false;
    let mut watched_workflows = Vec::new();
    if !options.no_push {
        run_git(repo_root, &["push"])?;
        run_git(repo_root, &["push", "origin", &bump.tag])?;
        pushed = true;
        if !options.no_watch {
            watched_workflows = watch_release_workflows(repo_root, &bump.commit_sha)?;
        }
    }

    Ok(result_from_bump(options, &bump, pushed, watched_workflows))
}

fn result_from_bump(
    options: &ReleaseWorkflowOptions,
    bump: &VersionBumpResult,
    pushed: bool,
    watched_workflows: Vec<String>,
) -> ReleaseWorkflowResult {
    ReleaseWorkflowResult {
        version: options.version.clone(),
        dry_run: false,
        commit_sha: Some(bump.commit_sha.clone()),
        tag: Some(bump.tag.clone()),
        pushed,
        watched_workflows,
    }
}

fn ensure_issue_contract_clean(repo_root: &Path) -> Result<(), String> {
    let summary = run_issue_sync(repo_root, true)?;
    let mutation_count = issue_sync_mutation_count(&summary);
    if mutation_count == 0 {
        Ok(())
    } else {
        Err(format!(
            "GitHub/Beads contract needs {mutation_count} repair action(s). Run `yzx dev sync_issues`, commit the Beads changes, then rerun release."
        ))
    }
}

fn issue_sync_mutation_count(summary: &IssueSyncSummary) -> usize {
    summary.created
        + summary.reopened
        + summary.closed
        + summary.comments_created
        + summary.comments_updated
}

fn run_release_validators(repo_root: &Path) -> Result<(), String> {
    run_validator("validate-upgrade-contract", || {
        validate_upgrade_contract(
            repo_root,
            &UpgradeContractOptions {
                ci: false,
                diff_base: None,
            },
        )
    })?;
    run_validator("validate-readme-version", || {
        validate_readme_version(repo_root)
    })?;
    run_validator("validate-docs-experience", || {
        validate_docs_experience(repo_root)
    })?;
    run_validator("validate-config-surface-contract", || {
        validate_config_surface_contract(repo_root)
    })?;
    run_validator("validate-contracts", || validate_contracts(repo_root))?;
    run_validator("validate-rust-test-traceability", || {
        validate_rust_test_traceability(repo_root)
    })?;
    run_validator("validate-flake-interface", || {
        validate_flake_interface(repo_root)
    })?;
    run_validator("validate-nix-customization-api", || {
        validate_nix_customization_api(repo_root)
    })?;
    run_validator("validate-child-release-transaction", || {
        validate_child_release_transaction(repo_root)
    })
}

fn run_validator<F>(name: &str, run: F) -> Result<(), String>
where
    F: FnOnce() -> Result<ValidationReport, String>,
{
    let report = run()?;
    if report.errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{name} failed:\n{}",
            report
                .errors
                .iter()
                .map(|error| format!("  - {error}"))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

fn ensure_clean_git_worktree(repo_root: &Path) -> Result<(), String> {
    let status = run_git(repo_root, &["status", "--porcelain"])?;
    if status.trim().is_empty() {
        Ok(())
    } else {
        Err("release requires a clean git worktree".to_string())
    }
}

fn watch_release_workflows(repo_root: &Path, commit_sha: &str) -> Result<Vec<String>, String> {
    let mut watched = Vec::new();
    for workflow_name in REQUIRED_RELEASE_WORKFLOWS {
        let run_id = wait_for_workflow_run(repo_root, commit_sha, workflow_name)?;
        run_gh(
            repo_root,
            &["run", "watch", &run_id.to_string(), "--exit-status"],
        )?;
        watched.push((*workflow_name).to_string());
    }
    Ok(watched)
}

fn wait_for_workflow_run(
    repo_root: &Path,
    commit_sha: &str,
    workflow_name: &str,
) -> Result<i64, String> {
    let deadline = Instant::now() + WORKFLOW_DISCOVERY_TIMEOUT;
    loop {
        if let Some(run_id) = find_workflow_run(repo_root, commit_sha, workflow_name)? {
            return Ok(run_id);
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "Timed out waiting for workflow `{workflow_name}` on commit {commit_sha}"
            ));
        }
        sleep(WORKFLOW_DISCOVERY_INTERVAL);
    }
}

fn find_workflow_run(
    repo_root: &Path,
    commit_sha: &str,
    workflow_name: &str,
) -> Result<Option<i64>, String> {
    let raw = run_gh(
        repo_root,
        &[
            "run",
            "list",
            "--limit",
            "30",
            "--json",
            "databaseId,headSha,workflowName",
        ],
    )?;
    let runs: Vec<WorkflowRun> = serde_json::from_str(&raw)
        .map_err(|error| format!("Failed to parse GitHub workflow runs JSON: {error}"))?;
    Ok(runs
        .into_iter()
        .find(|run| run.head_sha == commit_sha && run.workflow_name == workflow_name)
        .map(|run| run.database_id))
}

fn run_git(repo_root: &Path, args: &[&str]) -> Result<String, String> {
    run_command_in_dir("git", args, repo_root)
}

fn run_gh(repo_root: &Path, args: &[&str]) -> Result<String, String> {
    run_command_in_dir("gh", args, repo_root)
}

fn run_command_in_dir(program: &str, args: &[&str], repo_root: &Path) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|error| format!("Failed to run {program}: {error}"))?;
    command_stdout(program, args, output)
}

fn command_stdout(
    program: &str,
    args: &[&str],
    output: std::process::Output,
) -> Result<String, String> {
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(format!(
            "{} {} failed\n{}",
            program,
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: release dry-runs cannot accidentally push, watch workflows, or create tags.
    #[test]
    fn parse_release_workflow_args_forces_dry_run_to_no_push_and_no_watch() {
        let options =
            parse_release_workflow_args(vec!["v17.5".to_string(), "--dry-run".to_string()])
                .unwrap();

        assert!(options.dry_run);
        assert!(options.no_push);
        assert!(options.no_watch);
    }

    // Defends: maintainer release automation still watches only the release-critical workflows after branch/tag dedupe.
    #[test]
    fn required_release_workflows_stay_minimal() {
        assert_eq!(REQUIRED_RELEASE_WORKFLOWS, ["CI", "Publish Nix Cache"]);
    }
}
