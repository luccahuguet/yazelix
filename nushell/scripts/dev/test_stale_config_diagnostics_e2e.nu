#!/usr/bin/env nu
# Test lane: maintainer

use ./yzx_test_helpers.nu [add_fixture_log log_block log_line setup_managed_config_fixture]

def setup_fixture [label: string, raw_toml: string] {
    let fixture = (add_fixture_log (setup_managed_config_fixture $label $raw_toml) "stale_config_diagnostics_e2e.log")
    $fixture | merge {
        inner_script: ($fixture.repo_root | path join "nushell" "scripts" "core" "start_yazelix_inner.nu")
    }
}

def run_startup_probe [fixture: record] {
    with-env {
        HOME: $fixture.tmp_home
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
    } {
        ^nu -c $"source \"($fixture.inner_script)\"; try { main \"($fixture.tmp_home)\" \"($fixture.tmp_home | path join "unused.kdl")\" } catch {|err| print $err.msg }" | complete
    }
}

def run_doctor_probe [fixture: record] {
    with-env {
        HOME: $fixture.tmp_home
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
    } {
        ^nu -c $"use \"($fixture.yzx_script)\" *; yzx doctor --verbose" | complete
    }
}

def run_unknown_invalid_case [] {
    let fixture = (setup_fixture
        "yazelix_stale_config_unknown"
        '[core]
refresh_output = "loud"
'
    )
    let log_file = $fixture.log_file

    log_line $log_file "Case: unknown config field without migration guidance"
    log_line $log_file $"Temp HOME: ($fixture.tmp_home)"
    log_line $log_file $"Config path: ($fixture.config_path)"
    log_line $log_file ""
    log_block $log_file "Input TOML" (open --raw $fixture.config_path)

    let startup = (run_startup_probe $fixture)
    log_block $log_file "Startup stdout" ($startup.stdout | str trim)
    log_block $log_file "Startup stderr" ($startup.stderr | str trim)

    let doctor = (run_doctor_probe $fixture)
    log_block $log_file "Doctor stdout" ($doctor.stdout | str trim)
    log_block $log_file "Doctor stderr" ($doctor.stderr | str trim)

    let ok = (
        (($startup.stdout | str contains "Unknown config field at core.refresh_output"))
        and not (($startup.stdout | str contains "Known migration"))
        and (($doctor.stdout | str contains "Unknown config field at core.refresh_output"))
        and not (($doctor.stdout | str contains "Safe apply: `yzx doctor --fix`"))
    )

    if $ok {
        log_line $log_file "Result: PASS"
    } else {
        log_line $log_file "Result: FAIL"
    }

    rm -rf $fixture.tmp_home
    $ok
}

export def main [] {
    let results = [
        (run_unknown_invalid_case)
    ]
    let passed = ($results | where {|result| $result } | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ Stale config diagnostics e2e checks passed \(($passed)/($total)\)"
    } else {
        print $"❌ Stale config diagnostics e2e checks failed \(($passed)/($total)\)"
        error make {msg: "stale config diagnostics e2e checks failed"}
    }
}
