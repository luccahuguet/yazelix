//! Rust-owned GitHub/Beads issue-sync policy for maintainer workflows.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::process::Command;

const CONTRACT_START: &str = "2026-03-22T00:00:00Z";

#[derive(Debug, Clone, Deserialize)]
struct GithubIssue {
    number: i64,
    state: String,
    title: String,
    url: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(default)]
    body: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GithubCommentsEnvelope {
    #[serde(default)]
    comments: Vec<GithubComment>,
}

#[derive(Debug, Clone, Deserialize)]
struct GithubComment {
    id: String,
    #[serde(default)]
    body: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BeadIssue {
    id: String,
    status: String,
    #[serde(default)]
    external_ref: String,
}

#[derive(Debug, Clone)]
struct IssueAction {
    kind: String,
    issue: GithubIssue,
    bead: Option<BeadIssue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IssueSyncSummary {
    pub created: usize,
    pub reopened: usize,
    pub closed: usize,
    pub unchanged: usize,
    pub comments_created: usize,
    pub comments_updated: usize,
    pub comments_unchanged: usize,
}

#[derive(Debug, Clone)]
struct IssueCommentAction {
    kind: String,
    issue: GithubIssue,
    bead: BeadIssue,
    body: String,
    comment: Option<GithubComment>,
}

pub fn run_issue_sync(repo_root: &Path, dry_run: bool) -> Result<IssueSyncSummary, String> {
    let github_issues = load_contract_github_issues()?;
    let beads = load_contract_beads()?;
    let (actions, errors) = plan_issue_bead_reconciliation(&github_issues, &beads);
    if !errors.is_empty() {
        return Err(format!(
            "GitHub/Beads reconciliation is blocked:\n{}",
            errors
                .into_iter()
                .map(|error| format!("  - {error}"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }

    let mutating_actions = actions
        .iter()
        .filter(|action| matches!(action.kind.as_str(), "create" | "reopen" | "close"))
        .cloned()
        .collect::<Vec<_>>();
    let initial_comment_actions = collect_issue_comment_actions(&github_issues, &beads)?;
    let mutating_comment_actions = initial_comment_actions
        .iter()
        .filter(|action| matches!(action.kind.as_str(), "create" | "update"))
        .cloned()
        .collect::<Vec<_>>();

    if dry_run {
        println!("GitHub→Beads local sync plan:");
        if mutating_actions.is_empty() {
            println!("  No changes needed.");
        } else {
            for action in &mutating_actions {
                println!("  - {}", format_issue_action(action));
            }
        }
        println!();
        println!("GitHub issue comment plan:");
        if mutating_comment_actions.is_empty() {
            println!("  No changes needed.");
        } else {
            for action in &mutating_comment_actions {
                println!("  - {}", format_comment_action(action));
            }
        }
        return Ok(build_summary(&actions, &initial_comment_actions));
    }

    if mutating_actions.is_empty() && mutating_comment_actions.is_empty() {
        println!("✅ GitHub issues and local Beads are already aligned.");
        return Ok(build_summary(&actions, &initial_comment_actions));
    }

    if !mutating_actions.is_empty() {
        println!("🔄 Syncing GitHub issue lifecycle into local Beads...");
    }
    for action in &mutating_actions {
        match action.kind.as_str() {
            "create" => {
                let created = create_bead_from_github_issue(&action.issue)?;
                println!(
                    "  ✅ Created {} for GitHub issue #{}",
                    created.id, action.issue.number
                );
                if action.issue.state != "OPEN" {
                    close_bead(&created.id, "Closed on GitHub")?;
                    println!(
                        "  ✅ Closed {} to match GitHub issue #{}",
                        created.id, action.issue.number
                    );
                }
            }
            "reopen" => {
                reopen_bead(action.bead.as_ref().unwrap())?;
                println!(
                    "  ✅ Reopened {} for GitHub issue #{}",
                    action.bead.as_ref().unwrap().id,
                    action.issue.number
                );
            }
            "close" => {
                close_bead(&action.bead.as_ref().unwrap().id, "Closed on GitHub")?;
                println!(
                    "  ✅ Closed {} for GitHub issue #{}",
                    action.bead.as_ref().unwrap().id,
                    action.issue.number
                );
            }
            _ => {}
        }
    }

    export_beads_jsonl(repo_root)?;

    let refreshed_issues = load_contract_github_issues()?;
    let refreshed_beads = load_contract_beads()?;
    let comment_actions = collect_issue_comment_actions(&refreshed_issues, &refreshed_beads)?;
    let mutating_comment_actions = comment_actions
        .iter()
        .filter(|action| matches!(action.kind.as_str(), "create" | "update"))
        .cloned()
        .collect::<Vec<_>>();
    if !mutating_comment_actions.is_empty() {
        println!("🔄 Syncing canonical Beads comments onto GitHub issues...");
        for action in &mutating_comment_actions {
            match action.kind.as_str() {
                "create" => {
                    create_issue_comment(action)?;
                    println!("  ✅ Added Beads comment to GitHub issue #{}", action.issue.number);
                }
                "update" => {
                    update_issue_comment(action)?;
                    println!(
                        "  ✅ Updated Beads comment on GitHub issue #{}",
                        action.issue.number
                    );
                }
                _ => {}
            }
        }
    }

    validate_issue_bead_contract(repo_root)?;
    println!("✅ GitHub issue lifecycle is now synced into local Beads.");
    Ok(build_summary(&actions, &comment_actions))
}

fn run_command(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("Failed to run {} {}: {}", program, args.join(" "), error))?;
    if !output.status.success() {
        return Err(format!(
            "{} {} failed\n{}",
            program,
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn load_contract_github_issues() -> Result<Vec<GithubIssue>, String> {
    let raw = run_command(
        "gh",
        &[
            "issue",
            "list",
            "--state",
            "all",
            "--limit",
            "1000",
            "--json",
            "number,state,title,url,createdAt,body",
        ],
    )?;
    serde_json::from_str(&raw).map_err(|error| format!("Failed to parse GitHub issues JSON: {error}"))
}

fn load_contract_beads() -> Result<Vec<BeadIssue>, String> {
    let raw = run_command("bd", &["list", "--all", "--limit", "0", "--json"])?;
    let value: Value =
        serde_json::from_str(&raw).map_err(|error| format!("Failed to parse Beads JSON: {error}"))?;
    let issues = value.get("issues").cloned().unwrap_or(value);
    serde_json::from_value(issues).map_err(|error| format!("Failed to decode Beads issues: {error}"))
}

fn load_issue_comments(issue_number: i64) -> Result<Vec<GithubComment>, String> {
    let raw = run_command("gh", &["issue", "view", &issue_number.to_string(), "--json", "comments"])?;
    let parsed: GithubCommentsEnvelope =
        serde_json::from_str(&raw).map_err(|error| format!("Failed to parse issue comments JSON: {error}"))?;
    Ok(parsed.comments)
}

fn issue_is_in_contract(issue: &GithubIssue) -> bool {
    issue.created_at.as_str() >= CONTRACT_START
}

fn canonical_issue_bead_comment_body(bead_id: &str) -> String {
    format!("Automated: Tracked in Beads as `{bead_id}`.")
}

fn infer_issue_type_from_body(body: &str) -> String {
    let marker = "### Issue Type";
    let Some(index) = body.find(marker) else {
        return "task".to_string();
    };
    let tail = &body[index + marker.len()..];
    let issue_type = tail
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("### "))
        .unwrap_or("")
        .to_ascii_lowercase();
    match issue_type.as_str() {
        "task" | "bug" | "feature" | "epic" | "chore" | "decision" => issue_type,
        "docs" => "chore".to_string(),
        "question" => "decision".to_string(),
        _ => "task".to_string(),
    }
}

fn build_imported_issue_description(issue: &GithubIssue) -> String {
    if issue.body.trim().is_empty() {
        format!("Imported GitHub issue #{}.", issue.number)
    } else {
        format!("Imported GitHub issue #{}.\n\n{}", issue.number, issue.body.trim())
    }
}

fn plan_issue_bead_reconciliation(
    github_issues: &[GithubIssue],
    beads: &[BeadIssue],
) -> (Vec<IssueAction>, Vec<String>) {
    let mut actions = Vec::new();
    let mut errors = Vec::new();

    for issue in github_issues.iter().filter(|issue| issue_is_in_contract(issue)) {
        let matches = beads
            .iter()
            .filter(|bead| bead.external_ref == issue.url)
            .cloned()
            .collect::<Vec<_>>();
        if matches.is_empty() {
            actions.push(IssueAction {
                kind: "create".to_string(),
                issue: issue.clone(),
                bead: None,
            });
            continue;
        }
        if matches.len() > 1 {
            errors.push(format!(
                "Duplicate beads for GitHub issue #{}: {}",
                issue.number,
                matches
                    .iter()
                    .map(|bead| bead.id.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            continue;
        }
        let bead = matches[0].clone();
        let is_github_open = issue.state == "OPEN";
        let is_bead_closed = bead.status == "closed";
        let kind = if is_github_open && is_bead_closed {
            "reopen"
        } else if !is_github_open && !is_bead_closed {
            "close"
        } else {
            "noop"
        };
        actions.push(IssueAction {
            kind: kind.to_string(),
            issue: issue.clone(),
            bead: Some(bead),
        });
    }

    (actions, errors)
}

fn find_issue_bead_comment(comments: &[GithubComment]) -> Option<GithubComment> {
    comments.iter().find(|comment| {
        let body = comment.body.trim();
        body.starts_with("Tracked in Beads as `")
            || body.starts_with("Automated: Tracked in Beads as `")
    })
    .cloned()
}

fn plan_issue_bead_comment_sync(
    issue: &GithubIssue,
    bead: &BeadIssue,
    comments: &[GithubComment],
) -> IssueCommentAction {
    let expected_body = canonical_issue_bead_comment_body(&bead.id);
    let existing = find_issue_bead_comment(comments);
    if let Some(comment) = existing.clone() {
        if comment.body.trim() == expected_body {
            return IssueCommentAction {
                kind: "noop".to_string(),
                issue: issue.clone(),
                bead: bead.clone(),
                body: expected_body,
                comment: Some(comment),
            };
        }
        return IssueCommentAction {
            kind: "update".to_string(),
            issue: issue.clone(),
            bead: bead.clone(),
            body: expected_body,
            comment: existing,
        };
    }

    IssueCommentAction {
        kind: "create".to_string(),
        issue: issue.clone(),
        bead: bead.clone(),
        body: expected_body,
        comment: None,
    }
}

fn collect_issue_comment_actions(
    github_issues: &[GithubIssue],
    beads: &[BeadIssue],
) -> Result<Vec<IssueCommentAction>, String> {
    let mut actions = Vec::new();
    for issue in github_issues {
        let matches = beads
            .iter()
            .filter(|bead| bead.external_ref == issue.url)
            .cloned()
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            continue;
        }
        let bead = matches[0].clone();
        let comments = load_issue_comments(issue.number)?;
        actions.push(plan_issue_bead_comment_sync(issue, &bead, &comments));
    }
    Ok(actions)
}

fn create_bead_from_github_issue(issue: &GithubIssue) -> Result<BeadIssue, String> {
    let issue_type = infer_issue_type_from_body(&issue.body);
    let description = build_imported_issue_description(issue);
    let raw = run_command(
        "bd",
        &[
            "create",
            &issue.title,
            "--type",
            &issue_type,
            "--priority",
            "2",
            "--description",
            &description,
            "--external-ref",
            &issue.url,
            "--json",
        ],
    )?;
    serde_json::from_str(&raw).map_err(|error| format!("Failed to parse created bead JSON: {error}"))
}

fn reopen_bead(bead: &BeadIssue) -> Result<(), String> {
    run_command("bd", &["update", &bead.id, "--status", "open", "--json"]).map(|_| ())
}

fn close_bead(bead_id: &str, reason: &str) -> Result<(), String> {
    run_command("bd", &["close", bead_id, "--reason", reason, "--json"]).map(|_| ())
}

fn create_issue_comment(action: &IssueCommentAction) -> Result<(), String> {
    run_command(
        "gh",
        &[
            "issue",
            "comment",
            &action.issue.number.to_string(),
            "--body",
            &action.body,
        ],
    )
    .map(|_| ())
}

fn update_issue_comment(action: &IssueCommentAction) -> Result<(), String> {
    let comment = action.comment.as_ref().expect("comment for update");
    let mutation = "mutation($id: ID!, $body: String!) { updateIssueComment(input: { id: $id, body: $body }) { issueComment { id } } }";
    run_command(
        "gh",
        &[
            "api",
            "graphql",
            "-f",
            &format!("query={mutation}"),
            "-F",
            &format!("id={}", comment.id),
            "-F",
            &format!("body={}", action.body),
        ],
    )
    .map(|_| ())
}

fn export_beads_jsonl(repo_root: &Path) -> Result<(), String> {
    let export_path = repo_root.join(".beads/issues.jsonl");
    run_command("bd", &["export", "-o", export_path.to_str().unwrap()]).map(|_| ())
}

fn validate_issue_bead_contract(repo_root: &Path) -> Result<(), String> {
    let script = repo_root.join(".github/scripts/validate_issue_bead_contract.nu");
    run_command("nu", &[script.to_str().unwrap()]).map(|_| ())
}

fn format_issue_action(action: &IssueAction) -> String {
    match action.kind.as_str() {
        "create" => format!("create bead for #{} ({})", action.issue.number, action.issue.title),
        "reopen" => format!(
            "reopen {} for #{} ({})",
            action.bead.as_ref().unwrap().id,
            action.issue.number,
            action.issue.title
        ),
        "close" => format!(
            "close {} for #{} ({})",
            action.bead.as_ref().unwrap().id,
            action.issue.number,
            action.issue.title
        ),
        _ => format!("noop #{} ({})", action.issue.number, action.issue.title),
    }
}

fn format_comment_action(action: &IssueCommentAction) -> String {
    match action.kind.as_str() {
        "create" => format!("create comment for #{} -> {}", action.issue.number, action.bead.id),
        "update" => format!("update comment for #{} -> {}", action.issue.number, action.bead.id),
        _ => format!("noop comment #{} -> {}", action.issue.number, action.bead.id),
    }
}

fn build_summary(actions: &[IssueAction], comment_actions: &[IssueCommentAction]) -> IssueSyncSummary {
    IssueSyncSummary {
        created: actions.iter().filter(|action| action.kind == "create").count(),
        reopened: actions.iter().filter(|action| action.kind == "reopen").count(),
        closed: actions.iter().filter(|action| action.kind == "close").count(),
        unchanged: actions.iter().filter(|action| action.kind == "noop").count(),
        comments_created: comment_actions
            .iter()
            .filter(|action| action.kind == "create")
            .count(),
        comments_updated: comment_actions
            .iter()
            .filter(|action| action.kind == "update")
            .count(),
        comments_unchanged: comment_actions
            .iter()
            .filter(|action| action.kind == "noop")
            .count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn github_issue(number: i64, state: &str, url: &str, created_at: &str, body: &str) -> GithubIssue {
        GithubIssue {
            number,
            state: state.to_string(),
            title: format!("Issue {number}"),
            url: url.to_string(),
            created_at: created_at.to_string(),
            body: body.to_string(),
        }
    }

    fn bead(id: &str, status: &str, external_ref: &str) -> BeadIssue {
        BeadIssue {
            id: id.to_string(),
            status: status.to_string(),
            external_ref: external_ref.to_string(),
        }
    }

    // Test lane: default
    // Defends: GitHub/Beads lifecycle reconciliation still creates, reopens, and closes the expected bead actions after the Nu owner is removed.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn plan_issue_bead_reconciliation_preserves_lifecycle_contract() {
        let issue_create = github_issue(1, "OPEN", "https://example.com/1", CONTRACT_START, "");
        let issue_reopen = github_issue(2, "OPEN", "https://example.com/2", CONTRACT_START, "");
        let issue_close = github_issue(3, "CLOSED", "https://example.com/3", CONTRACT_START, "");
        let beads = vec![
            bead("yazelix-a", "closed", &issue_reopen.url),
            bead("yazelix-b", "open", &issue_close.url),
        ];

        let (actions, errors) =
            plan_issue_bead_reconciliation(&[issue_create, issue_reopen, issue_close], &beads);
        assert!(errors.is_empty());
        assert_eq!(actions.iter().filter(|action| action.kind == "create").count(), 1);
        assert_eq!(actions.iter().filter(|action| action.kind == "reopen").count(), 1);
        assert_eq!(actions.iter().filter(|action| action.kind == "close").count(), 1);
    }

    // Defends: imported GitHub issue bodies still preserve the shared type inference mapping instead of drifting through maintainer-only aliases.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn infer_issue_type_maps_supported_and_legacy_values() {
        assert_eq!(infer_issue_type_from_body("### Issue Type\nbug\n"), "bug");
        assert_eq!(infer_issue_type_from_body("### Issue Type\nquestion\n"), "decision");
        assert_eq!(infer_issue_type_from_body("### Issue Type\ndocs\n"), "chore");
        assert_eq!(infer_issue_type_from_body(""), "task");
    }

    // Defends: canonical Beads comments still converge to one stable body instead of accumulating stale variants.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn plan_issue_comment_sync_updates_stale_comment_body() {
        let issue = github_issue(4, "OPEN", "https://example.com/4", CONTRACT_START, "");
        let bead = bead("yazelix-4", "open", &issue.url);
        let comments = vec![GithubComment {
            id: "comment-id".to_string(),
            body: "Tracked in Beads as `old`".to_string(),
        }];

        let action = plan_issue_bead_comment_sync(&issue, &bead, &comments);
        assert_eq!(action.kind, "update");
        assert_eq!(action.body, "Automated: Tracked in Beads as `yazelix-4`.");
    }
}
