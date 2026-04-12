#!/usr/bin/env nu

use repo_checkout.nu require_yazelix_repo_root
use issue_bead_contract.nu [
    build_imported_issue_description
    find_issue_bead_comment
    infer_issue_type_from_body
    load_contract_beads
    load_contract_github_issues
    load_issue_comments
    plan_issue_bead_reconciliation
    plan_issue_bead_comment_sync
]

def print_issue_sync_summary [actions: list] {
    let created = ($actions | where kind == "create" | length)
    let reopened = ($actions | where kind == "reopen" | length)
    let closed = ($actions | where kind == "close" | length)
    let unchanged = ($actions | where kind == "noop" | length)

    print ""
    print "Issue sync summary:"
    print $"  Created: ($created)"
    print $"  Reopened: ($reopened)"
    print $"  Closed: ($closed)"
    print $"  Already aligned: ($unchanged)"
}

def print_issue_comment_sync_summary [actions: list] {
    let created = ($actions | where kind == "create" | length)
    let updated = ($actions | where kind == "update" | length)
    let unchanged = ($actions | where kind == "noop" | length)

    print ""
    print "Issue comment sync summary:"
    print $"  Created: ($created)"
    print $"  Updated: ($updated)"
    print $"  Already aligned: ($unchanged)"
}

def fail_issue_sync_plan [errors: list] {
    print "❌ GitHub/Beads reconciliation is blocked:"
    $errors | each { |err| print $"   - ($err)" }
    error make { msg: "issue sync plan is invalid" }
}

def create_bead_from_github_issue [issue: record] {
    let issue_type = (infer_issue_type_from_body ($issue.body? | default ""))
    let description = (build_imported_issue_description $issue)
    let output = (
        ^br create $issue.title
            --type $issue_type
            --priority 2
            --description $description
            --external-ref $issue.url
            --json
        | complete
    )

    if $output.exit_code != 0 {
        error make {
            msg: $"Failed to create bead for GitHub issue #($issue.number): ($output.stderr | str trim)"
        }
    }

    $output.stdout | from json
}

def reopen_bead_from_github_issue [action: record] {
    let issue = $action.issue
    let bead = $action.bead
    let output = (^br update $bead.id --status open --json | complete)

    if $output.exit_code != 0 {
        error make {
            msg: $"Failed to reopen bead ($bead.id) for GitHub issue #($issue.number): ($output.stderr | str trim)"
        }
    }
}

def close_bead_from_github_issue [action: record] {
    let issue = $action.issue
    let bead = $action.bead
    let output = (^br close $bead.id --reason "Closed on GitHub" --json | complete)

    if $output.exit_code != 0 {
        error make {
            msg: $"Failed to close bead ($bead.id) for GitHub issue #($issue.number): ($output.stderr | str trim)"
        }
    }
}

def format_issue_sync_action [action: record] {
    let issue = $action.issue
    match $action.kind {
        "create" => $"create bead for #($issue.number) (($issue.title))"
        "reopen" => $"reopen ($action.bead.id) for #($issue.number) (($issue.title))"
        "close" => $"close ($action.bead.id) for #($issue.number) (($issue.title))"
        _ => $"noop #($issue.number) (($issue.title))"
    }
}

def collect_issue_bead_comment_actions [github_issues: list, beads: list] {
    let issue_records = (
        $github_issues
        | where { |issue|
            let matches = (
                $beads
                | where { |bead| (($bead.external_ref? | default "") == $issue.url) }
            )
            ($matches | length) == 1
        }
    )

    $issue_records | each { |issue|
        let bead = (
            $beads
            | where { |candidate| (($candidate.external_ref? | default "") == $issue.url) }
            | first
        )
        let comments = (load_issue_comments $issue.number)
        plan_issue_bead_comment_sync $issue $bead $comments
    }
}

def create_issue_bead_comment [action: record] {
    let output = (^gh issue comment $action.issue.number --body $action.body | complete)
    if $output.exit_code != 0 {
        error make {
            msg: $"Failed to create Beads comment for GitHub issue #($action.issue.number): ($output.stderr | str trim)"
        }
    }
}

def update_issue_bead_comment [action: record] {
    let mutation = 'mutation($id: ID!, $body: String!) { updateIssueComment(input: { id: $id, body: $body }) { issueComment { id } } }'
    let output = (
        ^gh api graphql
            -f $"query=($mutation)"
            -F $"id=($action.comment.id)"
            -F $"body=($action.body)"
        | complete
    )

    if $output.exit_code != 0 {
        error make {
            msg: $"Failed to update Beads comment for GitHub issue #($action.issue.number): ($output.stderr | str trim)"
        }
    }
}

def format_issue_comment_action [action: record] {
    match $action.kind {
        "create" => $"create comment for #($action.issue.number) -> ($action.bead.id)"
        "update" => $"update comment for #($action.issue.number) -> ($action.bead.id)"
        _ => $"noop comment #($action.issue.number) -> ($action.bead.id)"
    }
}

export def run_dev_issue_sync [dry_run: bool = false] {
    let github_issues = load_contract_github_issues
    let beads = load_contract_beads
    let plan = (plan_issue_bead_reconciliation $github_issues $beads)
    let actions = $plan.actions
    let errors = $plan.errors

    if not ($errors | is-empty) {
        fail_issue_sync_plan $errors
    }

    let mutating_actions = ($actions | where kind in ["create" "reopen" "close"])

    let initial_comment_actions = (collect_issue_bead_comment_actions $github_issues $beads)
    let mutating_comment_actions = ($initial_comment_actions | where kind in ["create" "update"])

    if $dry_run {
        print "GitHub→Beads local sync plan:"
        if ($mutating_actions | is-empty) {
            print "  No changes needed."
        } else {
            $mutating_actions | each { |action| print $"  - (format_issue_sync_action $action)" }
        }
        print ""
        print "GitHub issue comment plan:"
        if ($mutating_comment_actions | is-empty) {
            print "  No changes needed."
        } else {
            $mutating_comment_actions | each { |action| print $"  - (format_issue_comment_action $action)" }
        }
        print_issue_sync_summary $actions
        print_issue_comment_sync_summary $initial_comment_actions
        return
    }

    if ($mutating_actions | is-empty) and ($mutating_comment_actions | is-empty) {
        print "✅ GitHub issues and local Beads are already aligned."
        print_issue_sync_summary $actions
        print_issue_comment_sync_summary $initial_comment_actions
        return
    }

    if not ($mutating_actions | is-empty) {
        print "🔄 Syncing GitHub issue lifecycle into local Beads..."
    }
    for action in $mutating_actions {
        match $action.kind {
            "create" => {
                let created_bead = (create_bead_from_github_issue $action.issue)
                print $"  ✅ Created ($created_bead.id) for GitHub issue #($action.issue.number)"
                if $action.issue.state != "OPEN" {
                    close_bead_from_github_issue {
                        issue: $action.issue
                        bead: $created_bead
                    }
                    print $"  ✅ Closed ($created_bead.id) to match GitHub issue #($action.issue.number)"
                }
            }
            "reopen" => {
                reopen_bead_from_github_issue $action
                print $"  ✅ Reopened ($action.bead.id) for GitHub issue #($action.issue.number)"
            }
            "close" => {
                close_bead_from_github_issue $action
                print $"  ✅ Closed ($action.bead.id) for GitHub issue #($action.issue.number)"
            }
        }
    }

    ^br sync --flush-only

    let refreshed_github_issues = load_contract_github_issues
    let refreshed_beads = load_contract_beads
    let comment_actions = (collect_issue_bead_comment_actions $refreshed_github_issues $refreshed_beads)
    let mutating_comment_actions = ($comment_actions | where kind in ["create" "update"])

    if not ($mutating_comment_actions | is-empty) {
        print "🔄 Syncing canonical Beads comments onto GitHub issues..."
        for action in $mutating_comment_actions {
            match $action.kind {
                "create" => {
                    create_issue_bead_comment $action
                    print $"  ✅ Added Beads comment to GitHub issue #($action.issue.number)"
                }
                "update" => {
                    update_issue_bead_comment $action
                    print $"  ✅ Updated Beads comment on GitHub issue #($action.issue.number)"
                }
            }
        }
    }

    let validator = (^nu ((require_yazelix_repo_root) | path join ".github" "scripts" "validate_issue_bead_contract.nu") | complete)
    if $validator.exit_code != 0 {
        print ($validator.stdout | str trim)
        let stderr = ($validator.stderr | str trim)
        if ($stderr | is-not-empty) {
            print $stderr
        }
        error make { msg: "Issue sync completed but contract validation failed" }
    }

    print "✅ GitHub issue lifecycle is now synced into local Beads."
    print_issue_sync_summary $actions
    print_issue_comment_sync_summary $comment_actions
}
