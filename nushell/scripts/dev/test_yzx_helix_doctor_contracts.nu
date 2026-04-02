#!/usr/bin/env nu
# Test lane: maintainer
# Defends: docs/specs/test_suite_governance.md

use ./yzx_test_helpers.nu [get_repo_root setup_managed_config_fixture]

def pin_fixture_to_repo [fixture: record] {
    let repo_root = (get_repo_root)
    $fixture
    | upsert repo_root $repo_root
    | upsert yzx_script ($repo_root | path join "nushell" "scripts" "core" "yazelix.nu")
}

def run_doctor_command [fixture: record] {
    with-env {
        HOME: $fixture.tmp_home
        XDG_CONFIG_HOME: ($fixture.tmp_home | path join ".config")
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
        YAZELIX_STATE_DIR: ($fixture.tmp_home | path join ".local" "share" "yazelix")
    } {
        ^nu -c $"use \"($fixture.yzx_script)\" *; yzx doctor --verbose" | complete
    }
}

def test_yzx_doctor_reports_helix_import_guidance_for_personal_config [] {
    print "🧪 Testing yzx doctor points personal Helix config users at `yzx import helix`..."

    let fixture = (pin_fixture_to_repo (setup_managed_config_fixture
        "yazelix_doctor_helix_import_guidance"
        (open --raw ((get_repo_root) | path join "yazelix_default.toml"))
    ))

    let result = (try {
        let native_helix_dir = ($fixture.tmp_home | path join ".config" "helix")
        mkdir $native_helix_dir
        '[editor]
line-number = "relative"
' | save --force --raw ($native_helix_dir | path join "config.toml")

        let output = (run_doctor_command $fixture)
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Personal Helix config has not been imported into Yazelix-managed Helix")
            and ($stdout | str contains "yzx import helix")
        ) {
            print "  ✅ yzx doctor now gives focused Helix import guidance instead of relying on personal ~/.config/helix state"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_yzx_doctor_warns_when_generated_helix_config_is_stale [] {
    print "🧪 Testing yzx doctor warns when the generated Helix config loses the Yazelix reveal binding..."

    let fixture = (pin_fixture_to_repo (setup_managed_config_fixture
        "yazelix_doctor_helix_generated_stale"
        (open --raw ((get_repo_root) | path join "yazelix_default.toml"))
    ))

    let result = (try {
        let generated_helix_dir = ($fixture.tmp_home | path join ".local" "share" "yazelix" "configs" "helix")
        mkdir $generated_helix_dir
        '[keys.normal]
A-r = ":noop"
' | save --force --raw ($generated_helix_dir | path join "config.toml")

        let output = (run_doctor_command $fixture)
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Managed Helix generated config is stale or invalid")
            and ($stdout | str contains "Launch a managed Helix session again to regenerate it")
        ) {
            print "  ✅ yzx doctor detects stale generated Helix config instead of trusting whatever is on disk"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

export def run_helix_doctor_contract_tests [] {
    [
        (test_yzx_doctor_reports_helix_import_guidance_for_personal_config)
        (test_yzx_doctor_warns_when_generated_helix_config_is_stale)
    ]
}

export def main [] {
    let results = (run_helix_doctor_contract_tests)
    let passed = ($results | where {|result| $result } | length)
    let total = ($results | length)

    if $passed == $total {
        print $"✅ All Helix doctor contract tests passed \(($passed)/($total)\)"
    } else {
        error make { msg: $"Helix doctor contract tests failed \(($passed)/($total)\)" }
    }
}
