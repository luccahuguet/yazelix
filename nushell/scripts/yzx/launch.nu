#!/usr/bin/env nu
# yzx launch command - Launch Yazelix in a new terminal window

use ../utils/entrypoint_config_migrations.nu [run_entrypoint_config_migration_preflight]
use ../utils/environment_bootstrap.nu [prepare_environment]
use ../utils/common.nu [require_yazelix_runtime_dir resolve_yazelix_nu_bin]
use ../utils/runtime_contract_checker.nu [check_runtime_script require_runtime_check]
use ../utils/runtime_env.nu get_runtime_env

def require_launch_runtime_script [script_path: string] {
    let check = (check_runtime_script $script_path "launch_runtime_script" "launch script" "launch")
    require_runtime_check $check | ignore
    $check.path
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
    --path(-p): string # Start in specific directory
    --home             # Start in home directory
    --terminal(-t): string  # Override terminal selection
    --verbose          # Enable verbose logging
] {
    run_entrypoint_config_migration_preflight "yzx launch" | ignore

    let verbose_mode = $verbose
    if $verbose_mode {
        print "🔍 yzx launch: verbose mode enabled"
    }

    let env_prep = prepare_environment --verbose=$verbose_mode
    let config = $env_prep.config
    let requested_path = ($path | default "")
    let requested_terminal = ($terminal | default "")
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
    let env_block = (propagate_test_env (get_runtime_env $config))
    if $verbose_mode {
        print $"⚙️ Executing launch_yazelix.nu from runtime: ($runtime_dir)"
        print $"   cwd: ($launch_cwd)"
    }

    with-env $env_block {
        ^$nu_bin ...$final_launch_args
    }
}
