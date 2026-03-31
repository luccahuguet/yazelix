#!/usr/bin/env nu
# Test runner for maintainer-only yzx checks

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

def test_runtime_pin_versions_use_repo_shell [] {
    print "🧪 Testing runtime pin versions use repo-shell nix and preferred devenv CLI..."

    if (which nix | is-empty) {
        print "  ❌ nix is required for maintainer tests"
        return false
    }

    if (which devenv | is-empty) {
        print "  ❌ devenv is required for maintainer tests"
        return false
    }

    try {
        let command = 'source nushell/scripts/yzx/dev.nu; let versions = (get_runtime_pin_versions); print ({ nix_version: $versions.nix_version, devenv_version: $versions.devenv_version, nix_raw: (get_tool_version_from_repo_shell "nix"), devenv_raw: (get_tool_version_from_repo_shell "devenv") } | to json -r)'
        let output = if (which timeout | is-not-empty) {
            ^timeout 30 nu -c $command | complete
        } else {
            ^nu -c $command | complete
        }
        let stdout = ($output.stdout | str trim)
        let resolved = ($stdout | lines | last | from json)

        if ($output.exit_code == 0) and ($resolved.nix_raw | str contains $resolved.nix_version) and ($resolved.devenv_raw | str contains $resolved.devenv_version) {
            print "  ✅ Runtime pins are derived from the repo shell versions"
            true
        } else if $output.exit_code == 124 {
            print "  ❌ Timed out while resolving runtime pins from the repo shell"
            false
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) resolved=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_preferred_devenv_resolution_uses_profile_entry [] {
    print "🧪 Testing preferred devenv resolution uses the active Nix profile entry when available..."

    if (which nix | is-empty) {
        print "  ❌ nix is required for maintainer tests"
        return false
    }

    try {
        let command = '
            source nushell/scripts/utils/devenv_cli.nu
            let profile = (try { ^nix profile list --json | from json } catch { null })
            let profile_store = if $profile == null {
                ""
            } else {
                $profile | get -o elements.devenv.storePaths.0 | default ""
            }
            let expected = if ($profile_store | is-not-empty) {
                $profile_store | path join "bin" "devenv"
            } else {
                which devenv | where type == "external" | get path | first
            }
            {
                resolved: (resolve_preferred_devenv_path)
                expected: $expected
            } | to json -r
        '
        let output = (^nu -c $command | complete)
        let stdout = ($output.stdout | str trim)
        let resolved = ($stdout | lines | last | from json)

        if ($output.exit_code == 0) and ($resolved.resolved == $resolved.expected) {
            print "  ✅ Preferred devenv resolution matches the intended source of truth"
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
        (test_runtime_pin_versions_use_repo_shell)
        (test_preferred_devenv_resolution_uses_profile_entry)
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
