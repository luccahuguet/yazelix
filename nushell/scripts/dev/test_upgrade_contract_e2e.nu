#!/usr/bin/env nu
# Test lane: maintainer

use ./yzx_test_helpers.nu [get_repo_root]

def log_line [log_file: string, line: string] {
    print $line
    $"($line)\n" | save --append --raw $log_file
}

def log_block [log_file: string, title: string, content: string] {
    log_line $log_file $"=== ($title) ==="
    if ($content | is-empty) {
        log_line $log_file "<empty>"
    } else {
        for line in ($content | lines) {
            log_line $log_file $line
        }
    }
    log_line $log_file ""
}

def setup_repo_fixture [] {
    let repo_root = (get_repo_root)
    let tmp_root = (^mktemp -d /tmp/yazelix_upgrade_contract_XXXXXX | str trim)
    let fixture_root = ($tmp_root | path join "repo")
    let log_file = ($tmp_root | path join "upgrade_contract_e2e.log")

    mkdir $fixture_root
    mkdir ($fixture_root | path join "docs")
    mkdir ($fixture_root | path join "nushell")
    mkdir ($fixture_root | path join "nushell" "scripts")
    ^cp ($repo_root | path join "CHANGELOG.md") ($fixture_root | path join "CHANGELOG.md")
    ^cp ($repo_root | path join "docs" "upgrade_notes.toml") ($fixture_root | path join "docs" "upgrade_notes.toml")
    mkdir ($fixture_root | path join "nushell" "scripts" "utils")
    ^cp ($repo_root | path join "nushell" "scripts" "utils" "constants.nu") ($fixture_root | path join "nushell" "scripts" "utils" "constants.nu")
    ^git -C $fixture_root init --quiet
    ^git -C $fixture_root config user.email "codex@example.com"
    ^git -C $fixture_root config user.name "Codex"
    ^git -C $fixture_root add -A
    ^git -C $fixture_root commit --quiet -m "Fixture baseline"
    "" | save --force --raw $log_file

    {
        repo_root: $repo_root
        fixture_root: $fixture_root
        log_file: $log_file
        changelog: ($fixture_root | path join "CHANGELOG.md")
        notes: ($fixture_root | path join "docs" "upgrade_notes.toml")
    }
}

def run_validator [fixture: record, ci: bool = false] {
    if $ci {
        do {
            cd $fixture.repo_root
            ^nix develop -c cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_repo_validator -- --repo-root $fixture.fixture_root validate-upgrade-contract --ci --diff-base HEAD~1
        } | complete
    } else {
        do {
            cd $fixture.repo_root
            ^nix develop -c cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_repo_validator -- --repo-root $fixture.fixture_root validate-upgrade-contract
        } | complete
    }
}

def commit_fixture_change [fixture: record, message: string] {
    ^git -C $fixture.fixture_root config user.email "codex@example.com"
    ^git -C $fixture.fixture_root config user.name "Codex"
    ^git -C $fixture.fixture_root add -A
    ^git -C $fixture.fixture_root commit --quiet -m $message
}

def run_current_repo_case [] {
    let fixture = (setup_repo_fixture)
    let log_file = $fixture.log_file

    log_line $log_file "Case: current repo validates cleanly"
    let result = (run_validator $fixture)
    log_block $log_file "Validator stdout" ($result.stdout | str trim)
    log_block $log_file "Validator stderr" ($result.stderr | str trim)

    let ok = ($result.exit_code == 0) and (($result.stdout | str contains "Upgrade contract is valid"))
    if $ok { log_line $log_file "Result: PASS" } else { log_line $log_file "Result: FAIL" }

    rm -rf ($fixture.fixture_root | path dirname)
    $ok
}

def run_unreleased_migration_available_case [] {
    let fixture = (setup_repo_fixture)
    let log_file = $fixture.log_file

    log_line $log_file "Case: unreleased migration_available is rejected"
    let broken_notes = (
        open $fixture.notes
        | upsert releases.unreleased.upgrade_impact "migration_available"
        | upsert releases.unreleased.migration_ids []
    )
    $broken_notes | to toml | save --force $fixture.notes

    log_block $log_file "Broken upgrade_notes.toml" (open --raw $fixture.notes)

    let result = (run_validator $fixture)
    log_block $log_file "Validator stdout" ($result.stdout | str trim)
    log_block $log_file "Validator stderr" ($result.stderr | str trim)

    let ok = (
        ($result.exit_code != 0)
        and (([$result.stdout $result.stderr] | str join "\n") | str contains "must not use migration_available because v15 no longer ships a live config migration engine")
    )
    if $ok { log_line $log_file "Result: PASS" } else { log_line $log_file "Result: FAIL" }

    rm -rf ($fixture.fixture_root | path dirname)
    $ok
}

def run_ci_ack_only_notes_case [] {
    let fixture = (setup_repo_fixture)
    let log_file = $fixture.log_file

    log_line $log_file "Case: CI accepts ack-only structured note updates without a changelog edit"
    let updated_notes = (
        open $fixture.notes
        | upsert releases.unreleased.acknowledged_guarded_changes ["nushell/scripts/utils/config_schema.nu"]
    )
    $updated_notes | to toml | save --force $fixture.notes
    commit_fixture_change $fixture "Ack guarded change in upgrade notes only"

    log_block $log_file "Updated upgrade_notes.toml" (open --raw $fixture.notes)

    let result = (run_validator $fixture true)
    log_block $log_file "Validator stdout" ($result.stdout | str trim)
    log_block $log_file "Validator stderr" ($result.stderr | str trim)

    let ok = ($result.exit_code == 0) and (($result.stdout | str contains "Upgrade contract is valid in CI mode"))
    if $ok { log_line $log_file "Result: PASS" } else { log_line $log_file "Result: FAIL" }

    rm -rf ($fixture.fixture_root | path dirname)
    $ok
}

def run_ci_summary_without_changelog_case [] {
    let fixture = (setup_repo_fixture)
    let log_file = $fixture.log_file

    log_line $log_file "Case: CI rejects user-facing structured note changes without a changelog edit"
    let updated_notes = (
        open $fixture.notes
        | upsert releases.unreleased.summary ["A real unreleased note without the matching changelog update."]
    )
    $updated_notes | to toml | save --force $fixture.notes
    commit_fixture_change $fixture "Change unreleased summary without changelog"

    log_block $log_file "Updated upgrade_notes.toml" (open --raw $fixture.notes)

    let result = (run_validator $fixture true)
    log_block $log_file "Validator stdout" ($result.stdout | str trim)
    log_block $log_file "Validator stderr" ($result.stderr | str trim)

    let combined_output = ([$result.stdout $result.stderr] | str join "\n")
    let ok = ($result.exit_code != 0) and ($combined_output | str contains "CHANGELOG.md and docs/upgrade_notes.toml must change together")
    if $ok { log_line $log_file "Result: PASS" } else { log_line $log_file "Result: FAIL" }

    rm -rf ($fixture.fixture_root | path dirname)
    $ok
}

def run_ci_series_only_notes_case [] {
    let fixture = (setup_repo_fixture)
    let log_file = $fixture.log_file

    log_line $log_file "Case: CI accepts README-series-only structured note updates without a changelog edit"
    let updated_notes = (
        open $fixture.notes
        | upsert series.v13.summary [
            "Plugin-managed workspaces and deterministic editor/sidebar routing define the v13 experience."
            "Workspace retargeting, upgrade UX, and popup/runtime hardening kept tightening through the series."
            "The public command surface is cleaner, with better inspection, release-note, and recovery paths."
        ]
    )
    $updated_notes | to toml | save --force $fixture.notes
    commit_fixture_change $fixture "Adjust v13 README series summary only"

    log_block $log_file "Updated upgrade_notes.toml" (open --raw $fixture.notes)

    let result = (run_validator $fixture true)
    log_block $log_file "Validator stdout" ($result.stdout | str trim)
    log_block $log_file "Validator stderr" ($result.stderr | str trim)

    let ok = ($result.exit_code == 0) and (($result.stdout | str contains "Upgrade contract is valid in CI mode"))
    if $ok { log_line $log_file "Result: PASS" } else { log_line $log_file "Result: FAIL" }

    rm -rf ($fixture.fixture_root | path dirname)
    $ok
}

export def main [] {
    let results = [
        (run_current_repo_case)
        (run_unreleased_migration_available_case)
        (run_ci_ack_only_notes_case)
        (run_ci_summary_without_changelog_case)
        (run_ci_series_only_notes_case)
    ]
    let passed = ($results | where {|result| $result } | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ Upgrade contract e2e checks passed \(($passed)/($total)\)"
    } else {
        print $"❌ Upgrade contract e2e checks failed \(($passed)/($total)\)"
        error make {msg: "upgrade contract e2e checks failed"}
    }
}
