#!/usr/bin/env nu

use ./test_yzx_helpers.nu [get_repo_root]

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

    ^cp -R $repo_root $fixture_root
    "" | save --force --raw $log_file

    {
        fixture_root: $fixture_root
        log_file: $log_file
        validator: ($fixture_root | path join "nushell" "scripts" "dev" "validate_upgrade_contract.nu")
        changelog: ($fixture_root | path join "CHANGELOG.md")
        notes: ($fixture_root | path join "docs" "upgrade_notes.toml")
    }
}

def run_validator [fixture: record, ci: bool = false] {
    if $ci {
        ^nu $fixture.validator --ci | complete
    } else {
        ^nu $fixture.validator | complete
    }
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

def run_broken_notes_case [] {
    let fixture = (setup_repo_fixture)
    let log_file = $fixture.log_file

    log_line $log_file "Case: broken upgrade notes are rejected"
    let broken_notes = (
        open $fixture.notes
        | upsert releases.unreleased.migration_ids []
    )
    $broken_notes | to toml | save --force $fixture.notes

    log_block $log_file "Broken upgrade_notes.toml" (open --raw $fixture.notes)

    let result = (run_validator $fixture)
    log_block $log_file "Validator stdout" ($result.stdout | str trim)
    log_block $log_file "Validator stderr" ($result.stderr | str trim)

    let ok = ($result.exit_code != 0) and (([$result.stdout $result.stderr] | str join "\n") | str contains "must list migration_ids")
    if $ok { log_line $log_file "Result: PASS" } else { log_line $log_file "Result: FAIL" }

    rm -rf ($fixture.fixture_root | path dirname)
    $ok
}

export def main [] {
    let results = [
        (run_current_repo_case)
        (run_broken_notes_case)
    ]
    let passed = ($results | where $it == true | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ Upgrade contract e2e checks passed \(($passed)/($total)\)"
    } else {
        print $"❌ Upgrade contract e2e checks failed \(($passed)/($total)\)"
        error make {msg: "upgrade contract e2e checks failed"}
    }
}
