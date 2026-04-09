#!/usr/bin/env nu
# Shared devenv-backed adapter helpers for Yazelix runtime commands.

use config_parser.nu parse_yazelix_config
use build_policy.nu [get_max_cores get_max_jobs get_yazelix_nix_config]
use devenv_cli.nu [is_preferred_devenv_available resolve_preferred_devenv_path]
use common.nu [require_yazelix_runtime_dir]
use config_state.nu [compute_config_state record_materialized_state]
use launch_state.nu [record_launch_profile_state resolve_profile_from_build_shell_output]
use runtime_project.nu [materialize_yazelix_runtime_project_dir]
use startup_profile.nu [profile_startup_step]

def format_command_for_display [command_parts: list<string>] {
    $command_parts
    | each { |part|
        let value = ($part | into string)
        if ($value | str contains " ") {
            $"\"($value)\""
        } else {
            $value
        }
    }
    | str join " "
}

def get_stderr_tail [stderr: string, max_lines: int = 5] {
    $stderr
    | default ""
    | lines
    | last $max_lines
    | str join "\n"
    | str trim
}

def print_completed_output [result: record] {
    let stdout_text = ($result.stdout | default "")
    let stderr_text = ($result.stderr | default "")

    if ($stdout_text | is-not-empty) {
        print --raw $stdout_text
    }
    if ($stderr_text | is-not-empty) {
        print --stderr --raw $stderr_text
    }
}

export def format_command_failure_summary [
    label: string
    command_parts: list<string>
    exit_code: int
    stderr: string
    recovery_hint: string
    --stderr-streamed
] {
    let command_text = (format_command_for_display $command_parts)
    mut lines = [
        $"❌ ($label) \(exit code: ($exit_code)\)"
        $"   Command: ($command_text)"
    ]

    if $stderr_streamed {
        $lines = ($lines | append "   stderr tail: output was streamed directly above.")
    } else {
        let stderr_tail = (get_stderr_tail $stderr)
        if ($stderr_tail | is-empty) {
            $lines = ($lines | append "   stderr tail: no stderr output was captured.")
        } else {
            $lines = ($lines | append "   stderr tail:")
            let indented_tail = (
                $stderr_tail
                | lines
                | each { |line| $"     ($line)" }
            )
            $lines = ($lines | append $indented_tail)
        }
    }

    if (($recovery_hint | str trim) | is-not-empty) {
        $lines = ($lines | append $"   Recovery: ($recovery_hint)")
    }

    $lines | str join "\n"
}

# Check if unfree pack is enabled in yazelix.toml
export def is_unfree_enabled [] {
    let config = parse_yazelix_config
    let pack_names = ($config.pack_names? | default [])
    $pack_names | any { |name| $name == "unfree" }
}

# Resolve absolute Yazelix directory from HOME
def resolve_yazelix_dir [] {
    try {
        require_yazelix_runtime_dir
    } catch {|err|
        print $"Error: ($err.msg)"
        exit 1
    }
}

def resolve_refresh_output_mode [mode: string] {
    let refresh_output = ($mode | into string | str downcase)
    let allowed = ["quiet", "normal", "full"]

    if not ($refresh_output in $allowed) {
        let allowed_text = ($allowed | str join ", ")
        error make {msg: $"Invalid refresh output mode '($refresh_output)'. Expected one of: ($allowed_text)"}
    }

    $refresh_output
}

def print_runtime_devenv_repair_hint [] {
    print "     yzx update runtime"
}

export def get_refresh_output_mode [config] {
    resolve_refresh_output_mode ($config.refresh_output? | default "normal")
}

# Build a base devenv command from the canonical Yazelix directory
export def get_devenv_base_command [
    --max-jobs: string = ""  # Concurrent build job strategy or explicit count from yazelix config
    --build-cores: string = ""  # Build core strategy or explicit count from yazelix config
    --quiet             # Include --quiet in devenv arguments
    --devenv-verbose    # Include --verbose in devenv arguments
    --refresh-eval-cache  # Include --refresh-eval-cache in devenv arguments
    --skip-shellhook-welcome  # Keep noninteractive shellHook entry quiet
    --startup-profile-phase: string = ""  # Optional shellHook phase tag for startup profiling
] {
    let yazelix_dir = resolve_yazelix_dir
    let devenv_project_dir = (materialize_yazelix_runtime_project_dir)
    let devenv_path = (resolve_preferred_devenv_path)
    let nix_config = get_yazelix_nix_config
    let requested_max_jobs = $max_jobs
    let requested_build_cores = $build_cores
    let resolved_max_jobs = if ($requested_max_jobs | is-not-empty) {
        get_max_jobs $requested_max_jobs
    } else {
        get_max_jobs
    }
    let max_cores = if ($requested_build_cores | is-not-empty) {
        get_max_cores $requested_build_cores
    } else {
        get_max_cores
    }

    mut cmd = [
        "env"
        "-C"
        $devenv_project_dir
        $"NIX_CONFIG=($nix_config)"
        $"YAZELIX_RUNTIME_DIR=($yazelix_dir)"
    ]

    if $skip_shellhook_welcome {
        $cmd = ($cmd | append "YAZELIX_SHELLHOOK_SKIP_WELCOME=true")
    }
    if ($startup_profile_phase | is-not-empty) {
        $cmd = ($cmd | append $"YAZELIX_STARTUP_PROFILE_PHASE=($startup_profile_phase)")
    }

    $cmd = (
        $cmd
        | append [
        $devenv_path
        "--max-jobs"
        ($resolved_max_jobs | into string)
        "--cores"
        ($max_cores | into string)
    ])

    if $quiet {
        $cmd = ($cmd | append "--quiet")
    }
    if $devenv_verbose {
        $cmd = ($cmd | append "--verbose")
    }
    if $refresh_eval_cache {
        $cmd = ($cmd | append "--refresh-eval-cache")
    }

    $cmd
}

export def rebuild_yazelix_environment [
    --max-jobs: string = ""  # Concurrent build job strategy or explicit count from yazelix config
    --build-cores: string = ""  # Build core strategy or explicit count from yazelix config
    --refresh-eval-cache  # Refresh devenv eval cache before rebuilding
    --output-mode: string = "normal"  # quiet | normal | full
] {
    let refresh_output = resolve_refresh_output_mode $output_mode
    let requested_max_jobs = $max_jobs
    let requested_build_cores = $build_cores
    let devenv_base = get_devenv_base_command --max-jobs $requested_max_jobs --build-cores $requested_build_cores --refresh-eval-cache=$refresh_eval_cache --quiet=($refresh_output == "quiet") --devenv-verbose=($refresh_output == "full") --skip-shellhook-welcome --startup-profile-phase "build_shell"
    let devenv_cmd = ($devenv_base | append ["build", "shell"])
    let cmd_bin = ($devenv_cmd | first)
    let cmd_args = ($devenv_cmd | skip 1)

    let rebuild_result = (profile_startup_step "bootstrap" "devenv.build_shell" {
        if $refresh_output == "quiet" {
            if (is_unfree_enabled) {
                with-env {NIXPKGS_ALLOW_UNFREE: "1"} {
                    let result = (^$cmd_bin ...$cmd_args | complete)
                    {
                        exit_code: $result.exit_code
                        stdout: ($result.stdout | default "")
                        stderr: ($result.stderr | default "")
                        stderr_streamed: false
                    }
                }
            } else {
                let result = (^$cmd_bin ...$cmd_args | complete)
                {
                    exit_code: $result.exit_code
                    stdout: ($result.stdout | default "")
                    stderr: ($result.stderr | default "")
                    stderr_streamed: false
                }
            }
        } else if (is_unfree_enabled) {
            with-env {NIXPKGS_ALLOW_UNFREE: "1"} {
                let result = (do { ^$cmd_bin ...$cmd_args } | complete)
                print_completed_output $result
                {
                    exit_code: $result.exit_code
                    stdout: ($result.stdout | default "")
                    stderr: ($result.stderr | default "")
                    stderr_streamed: true
                }
            }
        } else {
            let result = (do { ^$cmd_bin ...$cmd_args } | complete)
            print_completed_output $result
            {
                exit_code: $result.exit_code
                stdout: ($result.stdout | default "")
                stderr: ($result.stderr | default "")
                stderr_streamed: true
            }
        }
    } {
        command: ($devenv_cmd | str join " ")
        refresh_output: $refresh_output
    })

    if $rebuild_result.exit_code != 0 {
        print (format_command_failure_summary
            "Environment rebuild failed"
            $devenv_cmd
            $rebuild_result.exit_code
            $rebuild_result.stderr
            "Run `yzx doctor` to inspect the runtime, then rerun `yzx refresh` or `yzx restart` once the underlying build failure is fixed."
            --stderr-streamed=$rebuild_result.stderr_streamed
        )
        exit $rebuild_result.exit_code
    }

    let applied_state = (compute_config_state)
    let built_profile = (resolve_profile_from_build_shell_output ($rebuild_result.stdout | default ""))
    if ($built_profile | is-empty) {
        print "❌ Environment rebuild completed but Yazelix could not resolve the resulting DEVENV_PROFILE from the build output."
        print "   Recovery: rerun `yzx refresh --verbose` or `yzx restart --verbose`, and inspect the final `devenv build shell` result."
        exit 1
    }

    record_materialized_state $applied_state
    record_launch_profile_state $applied_state $built_profile
}

# Check if already in Yazelix or Nix environment
export def check_environment_status [] {
    let already_in_env = (
        ($env.IN_YAZELIX_SHELL? == "true")
        or ($env.IN_NIX_SHELL? | is-not-empty)
    )

    {
        already_in_env: $already_in_env
        in_nix_shell: ($env.IN_NIX_SHELL? | is-not-empty)
        in_yazelix_shell: ($env.IN_YAZELIX_SHELL? == "true")
    }
}

# Run a command with args inside devenv shell (no string interpolation)
export def run_in_devenv_shell_command [
    command: string
    ...args: string
    --max-jobs: string = ""  # Concurrent build job strategy or explicit count from yazelix config
    --build-cores: string = ""  # Build core strategy or explicit count from yazelix config
    --cwd: string = ""      # Run command in this directory
    --runtime-dir: string = ""  # Explicit Yazelix runtime root to expose inside devenv
    --env-only         # Set YAZELIX_ENV_ONLY=true
    --force-shell      # Enter devenv shell even when already in an activated Yazelix environment
    --verbose          # Enable verbose output
    --quiet            # Run devenv with --quiet flag
    --skip-welcome     # Set shellhook-only welcome suppression for bootstrap entry
    --force-refresh    # Force environment refresh
    --refresh-output-mode: string = "normal"  # quiet | normal | full when forcing refresh
] {
    let env_status = check_environment_status
    let verbose_mode = $verbose
    let refresh_output = resolve_refresh_output_mode $refresh_output_mode
    let requested_max_jobs = $max_jobs
    let requested_build_cores = $build_cores
    let requested_runtime_dir = $runtime_dir

    if ($command | is-empty) {
        print "Error: No command provided"
        exit 1
    }

    if (which env | is-empty) {
        print "Error: env command not found - cannot run command in devenv shell"
        exit 1
    }

    let requested_cwd = $cwd
    let resolved_cwd = if ($requested_cwd | is-not-empty) { $requested_cwd | path expand } else { "" }
    let resolved_runtime_dir = if ($requested_runtime_dir | is-not-empty) { $requested_runtime_dir | path expand } else { "" }
    let exec_cmd = if ($resolved_cwd | is-not-empty) {
        ["env", "-C", $resolved_cwd] | append $command | append $args
    } else {
        [$command] | append $args
    }
    let exec_bin = ($exec_cmd | first)
    let exec_args = ($exec_cmd | skip 1)

    if $env_status.already_in_env and (not $force_refresh) and (not $force_shell) {
        if $verbose_mode {
            print "⚙️ Executing command directly in existing environment"
        }
        ^$exec_bin ...$exec_args
        return
    }

    if not (is_preferred_devenv_available) {
        print ""
        print "❌ devenv command not found in the installed Yazelix runtime."
        print "   Repair the runtime with:"
        print_runtime_devenv_repair_hint
        print "   Then rerun the command after the runtime refresh finishes."
        print ""
        exit 1
    }

    if $verbose_mode {
        print "⚙️ Entering devenv shell before running command"
    }

    let quiet_devenv = if $force_refresh {
        $quiet or ($refresh_output == "quiet")
    } else {
        $quiet
    }
    let devenv_verbose = $force_refresh and ($refresh_output == "full") and (not $quiet_devenv)
    let devenv_base = get_devenv_base_command --max-jobs $requested_max_jobs --build-cores $requested_build_cores --quiet=$quiet_devenv --devenv-verbose=$devenv_verbose --refresh-eval-cache=$force_refresh --startup-profile-phase "shell_entry"
    let devenv_cmd = ($devenv_base | append ["shell", "--no-tui", "--no-reload", "--"] | append $exec_cmd)
    let devenv_bin = ($devenv_cmd | first)
    let devenv_args = ($devenv_cmd | skip 1)

    mut env_vars = {}
    if $env_only {
        $env_vars = ($env_vars | insert YAZELIX_ENV_ONLY "true")
    }
    if $skip_welcome {
        $env_vars = ($env_vars | insert YAZELIX_SHELLHOOK_SKIP_WELCOME "true")
    }
    if (is_unfree_enabled) {
        $env_vars = ($env_vars | insert NIXPKGS_ALLOW_UNFREE "1")
    }
    if ($resolved_runtime_dir | is-not-empty) {
        $env_vars = (
            $env_vars
            | insert YAZELIX_RUNTIME_DIR $resolved_runtime_dir
        )
    }

    # Keep the interactive shell handoff direct. Startup profiling should cover
    # owned preparation steps, not wrap the external TTY transfer itself.
    if ($env_vars | is-empty) {
        ^$devenv_bin ...$devenv_args
    } else {
        with-env $env_vars {
            ^$devenv_bin ...$devenv_args
        }
    }
}
