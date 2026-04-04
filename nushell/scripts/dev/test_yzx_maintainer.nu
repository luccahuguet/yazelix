#!/usr/bin/env nu
# Test runner for maintainer-only yzx checks
# Test lane: maintainer

use ../utils/common.nu [get_yazelix_state_dir]
use ../utils/devenv_cli.nu resolve_preferred_devenv_path

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Defends: issue/bead reconciliation planning catches create, reopen, close, and duplicate cases.
def test_issue_bead_reconciliation_plan [] {
    print "🧪 Testing issue/bead reconciliation plans create, reopen, close, and reject duplicates..."

    try {
        let command = '
            source nushell/scripts/utils/issue_bead_contract.nu
            let github_issues = [
                {number: 500, state: "OPEN", title: "Missing bead", url: "https://github.com/luccahuguet/yazelix/issues/500", createdAt: "2026-03-22T12:30:00Z", body: ""}
                {number: 501, state: "OPEN", title: "Closed bead should reopen", url: "https://github.com/luccahuguet/yazelix/issues/501", createdAt: "2026-03-22T12:31:00Z", body: ""}
                {number: 502, state: "CLOSED", title: "Open bead should close", url: "https://github.com/luccahuguet/yazelix/issues/502", createdAt: "2026-03-22T12:32:00Z", body: ""}
                {number: 503, state: "OPEN", title: "Already aligned", url: "https://github.com/luccahuguet/yazelix/issues/503", createdAt: "2026-03-22T12:33:00Z", body: ""}
                {number: 504, state: "OPEN", title: "Duplicate bead", url: "https://github.com/luccahuguet/yazelix/issues/504", createdAt: "2026-03-22T12:34:00Z", body: ""}
                {number: 399, state: "OPEN", title: "Grandfathered backlog issue", url: "https://github.com/luccahuguet/yazelix/issues/399", createdAt: "2026-03-21T23:59:59Z", body: ""}
            ]
            let beads = [
                {id: "yazelix-reopen", status: "closed", external_ref: "https://github.com/luccahuguet/yazelix/issues/501"}
                {id: "yazelix-close", status: "open", external_ref: "https://github.com/luccahuguet/yazelix/issues/502"}
                {id: "yazelix-noop", status: "open", external_ref: "https://github.com/luccahuguet/yazelix/issues/503"}
                {id: "yazelix-dup-a", status: "open", external_ref: "https://github.com/luccahuguet/yazelix/issues/504"}
                {id: "yazelix-dup-b", status: "closed", external_ref: "https://github.com/luccahuguet/yazelix/issues/504"}
                {id: "yazelix-old", status: "open", external_ref: "https://github.com/luccahuguet/yazelix/issues/399"}
            ]
            let plan = (plan_issue_bead_reconciliation $github_issues $beads)
            {
                action_kinds: ($plan.actions | each { |action| $action.kind } | sort),
                errors: ($plan.errors | sort)
            } | to json -r
        '
        let output = (^nu -c $command | complete)
        let stdout = ($output.stdout | str trim)
        let resolved = ($stdout | lines | last | from json)
        let expected_actions = ["close" "create" "noop" "reopen"]
        let expected_errors = ["Duplicate beads for GitHub issue #504: yazelix-dup-a, yazelix-dup-b"]

        if ($output.exit_code == 0) and ($resolved.action_kinds == $expected_actions) and ($resolved.errors == $expected_errors) {
            print "  ✅ Reconciliation planning matches the contract surface"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) resolved=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Defends: issue/bead comment planning keeps one canonical Beads mapping comment.
def test_issue_bead_comment_plan [] {
    print "🧪 Testing issue/bead comment planning creates, repairs legacy placeholders, updates stale comments, and accepts canonical comments..."

    try {
        let command = '
            source nushell/scripts/utils/issue_bead_contract.nu
            let issue = {number: 600, state: "OPEN", title: "Comment contract", url: "https://github.com/luccahuguet/yazelix/issues/600", createdAt: "2026-03-22T12:40:00Z", body: ""}
            let bead = {id: "yazelix-comment", status: "open", external_ref: $issue.url}
            let missing = (plan_issue_bead_comment_sync $issue $bead [])
            let placeholder = (plan_issue_bead_comment_sync $issue $bead [{id: "IC_placeholder", body: "$action.body"}])
            let stale = (plan_issue_bead_comment_sync $issue $bead [{id: "IC_stale", body: "Tracked in Beads as `yazelix-old`."}])
            let current = (plan_issue_bead_comment_sync $issue $bead [{id: "IC_current", body: "Automated: Tracked in Beads as `yazelix-comment`."}])
            {
                missing: $missing.kind
                placeholder: $placeholder.kind
                stale: $stale.kind
                current: $current.kind
                expected_body: (canonical_issue_bead_comment_body $bead.id)
            } | to json -r
        '
        let output = (^nu -c $command | complete)
        let stdout = ($output.stdout | str trim)
        let resolved = ($stdout | lines | last | from json)

        if (
            ($output.exit_code == 0)
            and ($resolved.missing == "create")
            and ($resolved.placeholder == "update")
            and ($resolved.stale == "update")
            and ($resolved.current == "noop")
            and ($resolved.expected_body == "Automated: Tracked in Beads as `yazelix-comment`.")
        ) {
            print "  ✅ Comment planning enforces one canonical Beads comment body"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) resolved=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
# Defends: runtime-owned devenv resolution remains the maintainer source of truth.
def test_preferred_devenv_resolution_uses_runtime_owned_cli [] {
    print "🧪 Testing preferred devenv resolution prefers the runtime-owned CLI over older profile entries..."

    let expected = (get_yazelix_state_dir | path join "runtime" "current" "bin" "devenv" | path expand)
    if not ($expected | path exists) {
        print "  ❌ installed runtime-owned devenv is required for this maintainer test"
        return false
    }

    try {
        let resolved = (resolve_preferred_devenv_path | path expand)
        if $resolved == $expected {
            print "  ✅ Preferred devenv resolution now matches the runtime-owned source of truth"
            true
        } else {
            print $"  ❌ Unexpected result: resolved=($resolved) expected=($expected)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Regression: repo-local devenv shells clear inherited installed-runtime aliases so source entrypoints use the checkout.
def test_source_devenv_shell_clears_inherited_runtime_aliases [] {
    print "🧪 Testing repo-local devenv shells clear inherited runtime aliases..."

    let repo_root = ($env.PWD | path expand)
    let fake_runtime = (get_yazelix_state_dir | path join "runtime" "current" | path expand)
    let devenv_bin = (resolve_preferred_devenv_path)

    try {
        let output = (with-env {
            YAZELIX_RUNTIME_DIR: $fake_runtime
            YAZELIX_DIR: "/nix/store/fake-yazelix-runtime"
            YAZELIX_SHELLHOOK_SKIP_WELCOME: "true"
            YAZELIX_ENV_ONLY: "true"
        } {
            ^$devenv_bin --quiet shell -- bash -lc 'printf "%s|%s|%s|%s\n" "$(printenv YAZELIX_RUNTIME_DIR 2>/dev/null || printf unset)" "$(printenv YAZELIX_DIR 2>/dev/null || printf unset)" "$DEVENV_ROOT" "$EDITOR"' | complete
        })
        let summary = ($output.stdout | lines | last | default "")
        let expected_editor = ($repo_root | path join "shells" "posix" "yazelix_hx.sh")

        if ($output.exit_code == 0) and ($summary == $"unset|unset|($repo_root)|($expected_editor)") {
            print "  ✅ Repo-local devenv shell now clears inherited runtime aliases and exports an absolute managed Helix wrapper from DEVENV_ROOT"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) summary=($summary) stderr=(($output.stderr | str trim))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Strength: defect=1 behavior=2 resilience=1 cost=1 uniqueness=1 total=6/10
# Invariant: generated Nushell initializers restore current PATH entries ahead of the saved PATH.
def test_nushell_initializer_restores_current_path_first [] {
    print "🧪 Testing the generated Nushell initializer preserves current PATH precedence..."

    try {
        let temp_home = (^mktemp -d | str trim)
        let yazelix_dir = $env.PWD
        let output = (with-env { HOME: $temp_home YAZELIX_QUIET_MODE: "true" } {
            ^nu nushell/scripts/setup/initializers.nu $yazelix_dir "nu" | complete
        })
        let aggregate = ($temp_home | path join ".local" "share" "yazelix" "initializers" "nushell" "yazelix_init.nu")
        let content = if ($aggregate | path exists) { open --raw $aggregate } else { "" }
        rm -rf $temp_home

        if ($output.exit_code == 0) and ($content | str contains '$env.PATH = ($current_path | append $initial_path | uniq)') {
            print "  ✅ Generated initializer keeps current PATH entries ahead of the saved PATH"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) aggregate=($aggregate)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def main [] {
    print "=== Testing yzx Maintainer Commands ==="
    print ""

    let results = [
        (test_issue_bead_reconciliation_plan)
        (test_issue_bead_comment_plan)
        (test_preferred_devenv_resolution_uses_runtime_owned_cli)
        (test_source_devenv_shell_clears_inherited_runtime_aliases)
        (test_nushell_initializer_restores_current_path_first)
    ]

    let passed = ($results | where {|result| $result } | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ All yzx maintainer tests passed \(($passed)/($total)\)"
    } else {
        print $"❌ Some maintainer tests failed \(($passed)/($total)\)"
        error make { msg: "yzx maintainer tests failed" }
    }
}
