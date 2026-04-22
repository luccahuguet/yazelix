#!/usr/bin/env nu
# yzx launch command - Launch Yazelix in a new terminal window

use ../utils/common.nu [require_yazelix_runtime_dir resolve_yazelix_nu_bin]
use ../utils/failure_classes.nu [format_failure_classification]
use ../utils/startup_profile.nu [profile_startup_step propagate_startup_profile_env]
use ../utils/yzx_core_bridge.nu [build_default_yzx_core_error_surface compute_runtime_env_via_yzx_core run_yzx_core_request_json_command]

const RUNTIME_CONTRACT_EVALUATE_COMMAND = "runtime-contract.evaluate"

def build_runtime_check_detail_lines [check: record] {
    mut detail_lines = []

    if (($check.details? | default "") | is-not-empty) {
        $detail_lines = ($detail_lines | append $check.details)
    }

    let recovery = ($check.recovery? | default "" | into string | str trim)
    let failure_class = ($check.failure_class? | default "" | into string | str trim)
    if ($recovery | is-not-empty) and ($failure_class | is-not-empty) {
        $detail_lines = ($detail_lines | append (format_failure_classification $failure_class $recovery))
    } else if ($recovery | is-not-empty) {
        $detail_lines = ($detail_lines | append $recovery)
    }

    $detail_lines
}

def runtime_check_to_error [check: record] {
    ([ $check.message ] | append (build_runtime_check_detail_lines $check) | str join "\n")
}

def require_launch_runtime_script [script_path: string] {
    let runtime_dir = (require_yazelix_runtime_dir)
    let data = (run_yzx_core_request_json_command
        $runtime_dir
        (build_default_yzx_core_error_surface)
        $RUNTIME_CONTRACT_EVALUATE_COMMAND
        {
            runtime_scripts: [
                {
                    id: "launch_runtime_script"
                    label: "launch script"
                    owner_surface: "launch"
                    path: ($script_path | path expand)
                }
            ]
        }
        "Yazelix Rust runtime-contract helper returned invalid JSON.")
    let check = (
        $data.checks?
        | default []
        | where {|candidate| ($candidate.id? | default "") == "launch_runtime_script" }
        | get -o 0
    )
    if $check == null {
        error make {msg: "Missing runtime-contract check result for `launch_runtime_script`."}
    }
    if ($check.status? | default "") != "ok" {
        error make {msg: (runtime_check_to_error $check)}
    }

    let resolved_path = ($check.path? | default "" | into string | str trim)
    if ($resolved_path | is-empty) {
        error make {msg: "Launch runtime-script preflight succeeded but omitted a resolved path."}
    }

    $resolved_path
}

def propagate_test_env [runtime_env: record] {
    mut env_block = $runtime_env
    if ($env.YAZELIX_CONFIG_OVERRIDE? | is-not-empty) {
        $env_block = ($env_block | upsert YAZELIX_CONFIG_OVERRIDE $env.YAZELIX_CONFIG_OVERRIDE)
    }
    if ($env.YAZELIX_LAYOUT_OVERRIDE? | is-not-empty) {
        $env_block = ($env_block | upsert YAZELIX_LAYOUT_OVERRIDE $env.YAZELIX_LAYOUT_OVERRIDE)
    }
    if ($env.YAZELIX_SWEEP_TEST_ID? | is-not-empty) {
        $env_block = ($env_block | upsert YAZELIX_SWEEP_TEST_ID $env.YAZELIX_SWEEP_TEST_ID)
    }

    $env_block
}

# Launch Yazelix.
export def "yzx launch" [
    --path(-p): string = "" # Start in specific directory
    --home             # Start in home directory
    --terminal(-t): string = ""  # Override terminal selection
    --verbose          # Enable verbose logging
] {
    let verbose_mode = $verbose
    if $verbose_mode {
        print "🔍 yzx launch: verbose mode enabled"
    }

    let runtime_env = (profile_startup_step "launch" "compute_runtime_env" {
        compute_runtime_env_via_yzx_core
    })
    let requested_path = $path
    let requested_terminal = $terminal
    let launch_cwd = if $home {
        $env.HOME
    } else if ($requested_path | is-not-empty) {
        $requested_path
    } else {
        pwd
    }

    let runtime_dir = (require_yazelix_runtime_dir)
    let launch_script = (require_launch_runtime_script ($runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu"))
    mut launch_args = [$launch_script]
    if ($launch_cwd | is-not-empty) {
        $launch_args = ($launch_args | append $launch_cwd)
    }
    if ($requested_terminal | is-not-empty) {
        $launch_args = ($launch_args | append "--terminal" | append $requested_terminal)
    }
    if $verbose_mode {
        $launch_args = ($launch_args | append "--verbose")
    }

    let nu_bin = (resolve_yazelix_nu_bin)
    let final_launch_args = $launch_args
    let env_block = (propagate_startup_profile_env (propagate_test_env $runtime_env))
    if $verbose_mode {
        print $"⚙️ Executing launch_yazelix.nu from runtime: ($runtime_dir)"
        print $"   cwd: ($launch_cwd)"
    }

    profile_startup_step "launch" "terminal_handoff" {
        with-env $env_block {
            ^$nu_bin ...$final_launch_args
        }
    }
}
