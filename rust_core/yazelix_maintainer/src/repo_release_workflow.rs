//! Maintainer release orchestration for tagged Yazelix releases.

use crate::repo_child_release::validate_child_release_transaction;
use crate::repo_contract_validation::{
    UpgradeContractOptions, validate_config_surface_contract, validate_flake_interface,
    validate_nix_customization_api, validate_readme_version, validate_upgrade_contract,
};
use crate::repo_docs_validation::validate_docs_experience;
use crate::repo_issue_sync::run_issue_sync;
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
const RELEASE_BRANCH_REF: &str = "main";
const WORKFLOW_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(60);
const WORKFLOW_DISCOVERY_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseWorkflowOptions {
    pub version: String,
    pub dry_run: bool,
    pub no_push: bool,
    pub no_watch: bool,
    pub dispatch_missing_workflows: bool,
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
    #[serde(default)]
    status: String,
    #[serde(default)]
    conclusion: Option<String>,
    #[serde(default)]
    event: String,
    #[serde(rename = "headBranch", default)]
    head_branch: Option<String>,
    #[serde(default)]
    url: Option<String>,
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
        dispatch_missing_workflows: false,
    };
    for arg in iter {
        match arg.as_str() {
            "--dry-run" => options.dry_run = true,
            "--no-push" => options.no_push = true,
            "--no-watch" => options.no_watch = true,
            "--dispatch-missing-workflows" => options.dispatch_missing_workflows = true,
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
    let mut runner = ProcessReleaseCommandRunner;
    run_repo_release_workflow_with_runner(repo_root, options, &mut runner)
}

fn run_repo_release_workflow_with_runner(
    repo_root: &Path,
    options: &ReleaseWorkflowOptions,
    runner: &mut dyn ReleaseCommandRunner,
) -> Result<ReleaseWorkflowResult, String> {
    ensure_clean_git_worktree(repo_root, runner)?;
    if options.dry_run {
        ensure_issue_contract_clean(repo_root)?;
    } else {
        runner.git(repo_root, &["pull", "--rebase"])?;
        ensure_clean_git_worktree(repo_root, runner)?;
        ensure_issue_contract_synced(repo_root, runner)?;
    }
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

    let bump = perform_version_bump(repo_root, &options.version)?;
    run_release_validators(repo_root)?;

    let mut pushed = false;
    let mut watched_workflows = Vec::new();
    if !options.no_push {
        watched_workflows = push_release_refs_after_checks(repo_root, options, runner, &bump)?;
        pushed = true;
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
    let mutation_count = summary.mutation_count();
    if mutation_count == 0 {
        Ok(())
    } else {
        Err(format!(
            "GitHub/Beads contract needs {mutation_count} repair action(s). Run `yzx dev sync_issues`, commit the Beads changes, then rerun release."
        ))
    }
}

fn ensure_issue_contract_synced(
    repo_root: &Path,
    runner: &mut dyn ReleaseCommandRunner,
) -> Result<Option<String>, String> {
    let planned_summary = run_issue_sync(repo_root, true)?;
    if planned_summary.mutation_count() == 0 {
        return Ok(None);
    }

    let synced_summary = run_issue_sync(repo_root, false)?;
    let commit_sha = commit_beads_changes_if_needed(repo_root, runner)?;
    let remaining_summary = run_issue_sync(repo_root, true)?;
    if remaining_summary.mutation_count() != 0 {
        return Err(format!(
            "GitHub/Beads contract still needs {} repair action(s) after automatic sync. Inspect `yzx dev sync_issues --dry-run` before releasing.",
            remaining_summary.mutation_count()
        ));
    }
    if synced_summary.mutation_count() > 0 {
        ensure_clean_git_worktree(repo_root, runner)?;
    }
    Ok(commit_sha)
}

fn commit_beads_changes_if_needed(
    repo_root: &Path,
    runner: &mut dyn ReleaseCommandRunner,
) -> Result<Option<String>, String> {
    runner.git(repo_root, &["add", ".beads/"])?;
    let status = runner.git(repo_root, &["status", "--porcelain", ".beads/"])?;
    if status.trim().is_empty() {
        return Ok(None);
    }

    runner.git(
        repo_root,
        &["commit", "-m", "Sync GitHub issues into Beads for release"],
    )?;
    runner.git(repo_root, &["rev-parse", "HEAD"]).map(Some)
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

fn ensure_clean_git_worktree(
    repo_root: &Path,
    runner: &mut dyn ReleaseCommandRunner,
) -> Result<(), String> {
    let status = runner.git(repo_root, &["status", "--porcelain"])?;
    if status.trim().is_empty() {
        Ok(())
    } else {
        Err("release requires a clean git worktree".to_string())
    }
}

fn push_release_refs_after_checks(
    repo_root: &Path,
    options: &ReleaseWorkflowOptions,
    runner: &mut dyn ReleaseCommandRunner,
    bump: &VersionBumpResult,
) -> Result<Vec<String>, String> {
    runner.git(repo_root, &["push"])?;
    let watched_workflows = if options.no_watch {
        Vec::new()
    } else {
        watch_release_workflows(repo_root, options, runner, &bump.commit_sha)?
    };
    runner.git(repo_root, &["push", "origin", &bump.tag])?;
    Ok(watched_workflows)
}

fn watch_release_workflows(
    repo_root: &Path,
    options: &ReleaseWorkflowOptions,
    runner: &mut dyn ReleaseCommandRunner,
    commit_sha: &str,
) -> Result<Vec<String>, String> {
    let mut watched = Vec::new();
    for workflow_name in REQUIRED_RELEASE_WORKFLOWS {
        let run = wait_for_workflow_run(
            repo_root,
            runner,
            commit_sha,
            workflow_name,
            options.dispatch_missing_workflows,
            WORKFLOW_DISCOVERY_TIMEOUT,
            WORKFLOW_DISCOVERY_INTERVAL,
        )?;
        if run.release_state() == WorkflowReleaseState::Pending {
            runner.gh(
                repo_root,
                &[
                    "run",
                    "watch",
                    &run.database_id.to_string(),
                    "--exit-status",
                ],
            )?;
        }
        watched.push((*workflow_name).to_string());
    }
    Ok(watched)
}

fn wait_for_workflow_run(
    repo_root: &Path,
    runner: &mut dyn ReleaseCommandRunner,
    commit_sha: &str,
    workflow_name: &str,
    dispatch_missing_workflows: bool,
    discovery_timeout: Duration,
    discovery_interval: Duration,
) -> Result<WorkflowRun, String> {
    let mut deadline = Instant::now() + discovery_timeout;
    let mut dispatched = false;
    loop {
        let runs = list_workflow_runs(repo_root, runner)?;
        if let Some(run) = runs
            .iter()
            .find(|run| run.matches_release_ref(commit_sha, workflow_name))
        {
            return match run.release_state() {
                WorkflowReleaseState::Pending | WorkflowReleaseState::Success => Ok(run.clone()),
                WorkflowReleaseState::Failed => Err(format_workflow_terminal_error(run)),
            };
        }
        if Instant::now() >= deadline {
            if dispatch_missing_workflows && !dispatched {
                runner.gh(
                    repo_root,
                    &[
                        "workflow",
                        "run",
                        workflow_name,
                        "--ref",
                        RELEASE_BRANCH_REF,
                    ],
                )?;
                dispatched = true;
                deadline = Instant::now() + discovery_timeout;
                continue;
            }
            return Err(format_missing_workflow_error(
                workflow_name,
                commit_sha,
                &runs,
                dispatch_missing_workflows,
            ));
        }
        runner.sleep(discovery_interval);
    }
}

fn list_workflow_runs(
    repo_root: &Path,
    runner: &mut dyn ReleaseCommandRunner,
) -> Result<Vec<WorkflowRun>, String> {
    let raw = runner.gh(
        repo_root,
        &[
            "run",
            "list",
            "--limit",
            "50",
            "--json",
            "databaseId,headSha,workflowName,status,conclusion,event,headBranch,url",
        ],
    )?;
    let runs: Vec<WorkflowRun> = serde_json::from_str(&raw)
        .map_err(|error| format!("Failed to parse GitHub workflow runs JSON: {error}"))?;
    Ok(runs)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkflowReleaseState {
    Pending,
    Success,
    Failed,
}

impl WorkflowRun {
    fn matches_release_ref(&self, commit_sha: &str, workflow_name: &str) -> bool {
        self.head_sha == commit_sha
            && self.workflow_name == workflow_name
            && self.matches_release_event_and_branch()
    }

    fn matches_release_event_and_branch(&self) -> bool {
        match self.event.as_str() {
            "push" => self.head_branch.as_deref() == Some(RELEASE_BRANCH_REF),
            "workflow_dispatch" => matches!(
                self.head_branch.as_deref(),
                Some(RELEASE_BRANCH_REF) | Some("") | None
            ),
            _ => false,
        }
    }

    fn release_state(&self) -> WorkflowReleaseState {
        if self.status == "completed" {
            return if self.conclusion.as_deref() == Some("success") {
                WorkflowReleaseState::Success
            } else {
                WorkflowReleaseState::Failed
            };
        }
        WorkflowReleaseState::Pending
    }

    fn summary(&self) -> String {
        format!(
            "{}#{} sha={} status={} conclusion={} event={} branch={}{}",
            self.workflow_name,
            self.database_id,
            short_sha(&self.head_sha),
            empty_as_unknown(&self.status),
            self.conclusion.as_deref().unwrap_or("unknown"),
            empty_as_unknown(&self.event),
            self.head_branch.as_deref().unwrap_or("unknown"),
            self.url
                .as_deref()
                .map(|url| format!(" url={url}"))
                .unwrap_or_default()
        )
    }
}

fn format_workflow_terminal_error(run: &WorkflowRun) -> String {
    format!(
        "Required workflow `{}` reached a terminal non-success state before release tag push.\nRun: {}\nRemote tag has not been pushed.",
        run.workflow_name,
        run.summary()
    )
}

fn format_missing_workflow_error(
    workflow_name: &str,
    commit_sha: &str,
    runs: &[WorkflowRun],
    dispatch_missing_workflows: bool,
) -> String {
    let nearby = runs
        .iter()
        .filter(|run| run.workflow_name == workflow_name || run.head_sha == commit_sha)
        .take(8)
        .map(|run| format!("  - {}", run.summary()))
        .collect::<Vec<_>>();
    let nearby = if nearby.is_empty() {
        "  - none".to_string()
    } else {
        nearby.join("\n")
    };
    let dispatch_hint = if dispatch_missing_workflows {
        "Automatic dispatch was attempted once.".to_string()
    } else {
        format!(
            "Safe next command before pushing the tag: gh workflow run \"{workflow_name}\" --ref main"
        )
    };
    format!(
        "Timed out waiting for required workflow `{workflow_name}` on release commit {commit_sha}.\nSearched GitHub workflow runs by workflowName=`{workflow_name}`, headSha=`{commit_sha}`, event=`push` with headBranch=`{RELEASE_BRANCH_REF}`, or event=`workflow_dispatch` with headBranch=`{RELEASE_BRANCH_REF}`/omitted; the release tag has not been pushed.\nNearby runs found:\n{nearby}\n{dispatch_hint}"
    )
}

fn short_sha(value: &str) -> &str {
    value.get(..8).unwrap_or(value)
}

fn empty_as_unknown(value: &str) -> &str {
    if value.is_empty() { "unknown" } else { value }
}

trait ReleaseCommandRunner {
    fn git(&mut self, repo_root: &Path, args: &[&str]) -> Result<String, String>;
    fn gh(&mut self, repo_root: &Path, args: &[&str]) -> Result<String, String>;

    fn sleep(&mut self, duration: Duration) {
        sleep(duration);
    }
}

struct ProcessReleaseCommandRunner;

impl ReleaseCommandRunner for ProcessReleaseCommandRunner {
    fn git(&mut self, repo_root: &Path, args: &[&str]) -> Result<String, String> {
        run_command_in_dir("git", args, repo_root)
    }

    fn gh(&mut self, repo_root: &Path, args: &[&str]) -> Result<String, String> {
        run_command_in_dir("gh", args, repo_root)
    }
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
    use std::collections::VecDeque;
    use std::path::PathBuf;

    #[derive(Default)]
    struct FakeReleaseCommandRunner {
        calls: Vec<String>,
        git_outputs: VecDeque<Result<String, String>>,
        gh_outputs: VecDeque<Result<String, String>>,
        sleeps: Vec<Duration>,
    }

    impl FakeReleaseCommandRunner {
        fn with_git_outputs(outputs: Vec<Result<String, String>>) -> Self {
            Self {
                git_outputs: outputs.into(),
                ..Self::default()
            }
        }

        fn push_gh_output(&mut self, output: Result<String, String>) {
            self.gh_outputs.push_back(output);
        }
    }

    impl ReleaseCommandRunner for FakeReleaseCommandRunner {
        fn git(&mut self, _repo_root: &Path, args: &[&str]) -> Result<String, String> {
            self.calls.push(format!("git {}", args.join(" ")));
            self.git_outputs
                .pop_front()
                .unwrap_or_else(|| Ok(String::new()))
        }

        fn gh(&mut self, _repo_root: &Path, args: &[&str]) -> Result<String, String> {
            self.calls.push(format!("gh {}", args.join(" ")));
            self.gh_outputs
                .pop_front()
                .unwrap_or_else(|| Ok(String::new()))
        }

        fn sleep(&mut self, duration: Duration) {
            self.sleeps.push(duration);
        }
    }

    fn release_options() -> ReleaseWorkflowOptions {
        ReleaseWorkflowOptions {
            version: "v17.6".to_string(),
            dry_run: false,
            no_push: false,
            no_watch: false,
            dispatch_missing_workflows: false,
        }
    }

    fn fake_bump() -> VersionBumpResult {
        VersionBumpResult {
            repo_root: PathBuf::from("/repo"),
            previous_version: "v17.5".to_string(),
            target_version: "v17.6".to_string(),
            release_date: "2026-06-08".to_string(),
            commit_message: "Bump version to v17.6".to_string(),
            commit_sha: "abc123456789".to_string(),
            tag: "v17.6".to_string(),
        }
    }

    fn workflow_runs_json(status: &str, conclusion: Option<&str>) -> String {
        serde_json::json!([
            {
                "databaseId": 11,
                "headSha": "abc123456789",
                "workflowName": "CI",
                "status": status,
                "conclusion": conclusion,
                "event": "push",
                "headBranch": "main",
                "url": "https://github.example/runs/11"
            },
            {
                "databaseId": 12,
                "headSha": "abc123456789",
                "workflowName": "Publish Nix Cache",
                "status": status,
                "conclusion": conclusion,
                "event": "push",
                "headBranch": "main",
                "url": "https://github.example/runs/12"
            }
        ])
        .to_string()
    }

    fn non_release_branch_workflow_run_json() -> String {
        serde_json::json!([
            {
                "databaseId": 13,
                "headSha": "abc123456789",
                "workflowName": "CI",
                "status": "completed",
                "conclusion": "success",
                "event": "pull_request",
                "headBranch": "feature",
                "url": "https://github.example/runs/13"
            }
        ])
        .to_string()
    }

    fn dispatched_workflow_run_without_head_branch_json() -> String {
        serde_json::json!([
            {
                "databaseId": 14,
                "headSha": "abc123456789",
                "workflowName": "CI",
                "status": "completed",
                "conclusion": "success",
                "event": "workflow_dispatch",
                "url": "https://github.example/runs/14"
            }
        ])
        .to_string()
    }

    // Defends: release dry-runs cannot accidentally push, watch workflows, or create tags.
    #[test]
    fn parse_release_workflow_args_forces_dry_run_to_no_push_and_no_watch() {
        let options =
            parse_release_workflow_args(vec!["v17.5".to_string(), "--dry-run".to_string()])
                .unwrap();

        assert!(options.dry_run);
        assert!(options.no_push);
        assert!(options.no_watch);
        assert!(!options.dispatch_missing_workflows);
    }

    // Defends: maintainer release automation still watches only the release-critical workflows after branch/tag dedupe.
    #[test]
    fn required_release_workflows_stay_minimal() {
        assert_eq!(REQUIRED_RELEASE_WORKFLOWS, ["CI", "Publish Nix Cache"]);
    }

    // Regression: v17.5 pushed the release tag before CI/cache watcher success.
    #[test]
    fn release_pushes_tag_only_after_required_workflow_watches() {
        let mut runner = FakeReleaseCommandRunner::default();
        runner.push_gh_output(Ok(workflow_runs_json("in_progress", None)));
        runner.push_gh_output(Ok(String::new()));
        runner.push_gh_output(Ok(workflow_runs_json("in_progress", None)));
        runner.push_gh_output(Ok(String::new()));

        let watched = push_release_refs_after_checks(
            Path::new("/repo"),
            &release_options(),
            &mut runner,
            &fake_bump(),
        )
        .unwrap();

        assert_eq!(watched, ["CI", "Publish Nix Cache"]);
        assert_eq!(runner.calls.first().unwrap(), "git push");
        assert_eq!(runner.calls.last().unwrap(), "git push origin v17.6");
        let tag_push_index = runner
            .calls
            .iter()
            .position(|call| call == "git push origin v17.6")
            .unwrap();
        let last_watch_index = runner
            .calls
            .iter()
            .rposition(|call| call.starts_with("gh run watch "))
            .unwrap();
        assert!(tag_push_index > last_watch_index);
    }

    // Defends: missing release CI fails with a precise diagnostic before the tag push path can run.
    #[test]
    fn missing_workflow_error_names_search_and_safe_next_command() {
        let mut runner = FakeReleaseCommandRunner::default();
        runner.push_gh_output(Ok("[]".to_string()));

        let error = wait_for_workflow_run(
            Path::new("/repo"),
            &mut runner,
            "abc123456789",
            "CI",
            false,
            Duration::ZERO,
            Duration::ZERO,
        )
        .unwrap_err();

        assert!(error.contains("workflowName=`CI`"));
        assert!(error.contains("headSha=`abc123456789`"));
        assert!(error.contains("headBranch=`main`"));
        assert!(error.contains("event=`push`"));
        assert!(error.contains("event=`workflow_dispatch`"));
        assert!(error.contains("release tag has not been pushed"));
        assert!(error.contains("gh workflow run \"CI\" --ref main"));
    }

    // Defends: release watching does not accept a same-SHA run from a non-release branch/event.
    #[test]
    fn release_workflow_discovery_ignores_non_release_ref_runs() {
        let mut runner = FakeReleaseCommandRunner::default();
        runner.push_gh_output(Ok(non_release_branch_workflow_run_json()));

        let error = wait_for_workflow_run(
            Path::new("/repo"),
            &mut runner,
            "abc123456789",
            "CI",
            false,
            Duration::ZERO,
            Duration::ZERO,
        )
        .unwrap_err();

        assert!(error.contains("headBranch=`main`"));
        assert!(error.contains("CI#13 sha=abc12345 status=completed conclusion=success event=pull_request branch=feature"));
    }

    // Defends: explicit release dispatch can recover even if GitHub omits headBranch from run-list JSON.
    #[test]
    fn release_workflow_discovery_accepts_dispatched_run_without_branch_field() {
        let mut runner = FakeReleaseCommandRunner::default();
        runner.push_gh_output(Ok(dispatched_workflow_run_without_head_branch_json()));

        let run = wait_for_workflow_run(
            Path::new("/repo"),
            &mut runner,
            "abc123456789",
            "CI",
            false,
            Duration::ZERO,
            Duration::ZERO,
        )
        .unwrap();

        assert_eq!(run.database_id, 14);
    }

    // Defends: workflow dispatch is explicit, bounded, and still waits for a run on the release commit.
    #[test]
    fn missing_workflow_can_dispatch_when_explicitly_configured() {
        let mut runner = FakeReleaseCommandRunner::default();
        runner.push_gh_output(Ok("[]".to_string()));
        runner.push_gh_output(Ok(String::new()));
        runner.push_gh_output(Ok(workflow_runs_json("completed", Some("success"))));

        let run = wait_for_workflow_run(
            Path::new("/repo"),
            &mut runner,
            "abc123456789",
            "CI",
            true,
            Duration::ZERO,
            Duration::ZERO,
        )
        .unwrap();

        assert_eq!(run.database_id, 11);
        assert!(
            runner
                .calls
                .iter()
                .any(|call| { call == "gh workflow run CI --ref main" })
        );
    }

    // Defends: automatic release issue-sync commits only Beads state before the version bump commit.
    #[test]
    fn beads_sync_commit_stages_only_beads_and_returns_commit_sha() {
        let mut runner = FakeReleaseCommandRunner::with_git_outputs(vec![
            Ok(String::new()),
            Ok("M  .beads/issues.jsonl".to_string()),
            Ok(String::new()),
            Ok("bead-sync-sha".to_string()),
        ]);

        let commit = commit_beads_changes_if_needed(Path::new("/repo"), &mut runner).unwrap();

        assert_eq!(commit.as_deref(), Some("bead-sync-sha"));
        assert_eq!(
            runner.calls,
            [
                "git add .beads/",
                "git status --porcelain .beads/",
                "git commit -m Sync GitHub issues into Beads for release",
                "git rev-parse HEAD"
            ]
        );
    }

    // Defends: comment-only GitHub sync does not create an empty Beads commit.
    #[test]
    fn beads_sync_commit_skips_commit_when_jsonl_is_clean() {
        let mut runner =
            FakeReleaseCommandRunner::with_git_outputs(vec![Ok(String::new()), Ok(String::new())]);

        let commit = commit_beads_changes_if_needed(Path::new("/repo"), &mut runner).unwrap();

        assert_eq!(commit, None);
        assert_eq!(
            runner.calls,
            ["git add .beads/", "git status --porcelain .beads/"]
        );
    }
}
