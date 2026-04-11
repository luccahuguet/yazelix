#!/usr/bin/env nu
# yzx env command - Load Yazelix runtime environment without UI

use ../utils/doctor.nu print_runtime_version_drift_warning
use ../utils/entrypoint_config_migrations.nu [run_entrypoint_config_migration_preflight]
use ../utils/environment_bootstrap.nu [prepare_environment]
use ../utils/runtime_env.nu get_runtime_env

# Build shell command from shell name.
# --login keeps existing behavior for default yzx env mode.
def resolve_shell_command [shell_name: string, --login] {
    let normalized = ($shell_name | str downcase)

    if $login {
        match $normalized {
            "nu" => ["nu" "--login"]
            "bash" => ["bash" "--login"]
            "fish" => ["fish" "-l"]
            "zsh" => ["zsh" "-l"]
            _ => [$normalized]
        }
    } else {
        match $normalized {
            "nu" => ["nu"]
            "bash" => ["bash"]
            "fish" => ["fish"]
            "zsh" => ["zsh"]
            _ => [$normalized]
        }
    }
}

def run_runtime_shell [runtime_env: record, shell_command: list<string>, --cwd: string] {
    let has_setpriv = (which setpriv | is-not-empty)
    let trap_supervisor = "trap 'kill 0' HUP TERM; exec \"$@\""
    let exec_cmd = if $has_setpriv {
        ["setpriv", "--pdeathsig", "TERM", "--"] | append $shell_command
    } else {
        ["sh", "-c", $trap_supervisor, "_"] | append $shell_command
    }
    let exec_bin = ($exec_cmd | first)
    let exec_args = ($exec_cmd | skip 1)

    with-env $runtime_env {
        if ($cwd | is-not-empty) {
            cd ($cwd | path expand)
        }
        ^$exec_bin ...$exec_args
    }
}

# Load yazelix environment without UI.
export def "yzx env" [
    --no-shell(-n)  # Keep current shell instead of launching configured shell
] {
    print_runtime_version_drift_warning
    run_entrypoint_config_migration_preflight "yzx env" | ignore

    let env_prep = prepare_environment
    let config = $env_prep.config
    let original_dir = (pwd)
    let configured_shell_name = ($config.default_shell? | default "nu" | str downcase)
    let invoking_shell_name = (
        if ($env.SHELL? | is-not-empty) {
            $env.SHELL | path basename | str downcase
        } else {
            $configured_shell_name
        }
    )
    let shell_command = if $no_shell {
        resolve_shell_command $invoking_shell_name
    } else {
        resolve_shell_command $configured_shell_name --login
    }
    let shell_exec = ($shell_command | first)
    let runtime_env = ((get_runtime_env $config) | upsert SHELL $shell_exec)

    try {
        run_runtime_shell $runtime_env $shell_command --cwd $original_dir
    } catch {|err|
        print $"❌ Failed to launch Yazelix runtime shell: ($err.msg)"
        print "   Tip: rerun with 'yzx env --no-shell' to stay in your current shell."
        exit 1
    }
}
