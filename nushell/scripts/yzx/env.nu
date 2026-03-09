#!/usr/bin/env nu
# yzx env command - Load Yazelix environment without UI

use ../utils/environment_bootstrap.nu *
use ../utils/doctor.nu print_runtime_version_drift_warning
use ../utils/common.nu [describe_build_parallelism]
use ../utils/launch_state.nu [get_launch_env require_reused_launch_profile]

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

def run_with_launch_profile [
    config: record
    profile_path: string
    command: string
    ...args: string
    --cwd: string
] {
    let resolved_cwd = if ($cwd | is-not-empty) { $cwd | path expand } else { "" }
    let exec_cmd = if ($resolved_cwd | is-not-empty) {
        ["env", "-C", $resolved_cwd] | append $command | append $args
    } else {
        [$command] | append $args
    }
    let exec_bin = ($exec_cmd | first)
    let exec_args = ($exec_cmd | skip 1)

    with-env (get_launch_env $config $profile_path) {
        ^$exec_bin ...$exec_args
    }
}

# Load yazelix environment without UI
export def "yzx env" [
    --no-shell(-n)  # Keep current shell instead of launching configured shell
    --reuse         # Reuse the last built profile without rebuilding
    --skip-refresh(-s)  # Skip explicit refresh trigger and allow potentially stale environment
] {
    use ../utils/nix_detector.nu ensure_nix_available
    ensure_nix_available
    print_runtime_version_drift_warning

    # Prepare environment (shared with start_yazelix.nu)
    let env_prep = prepare_environment
    let config = $env_prep.config
    let needs_refresh = $env_prep.needs_refresh
    let reuse_mode = $reuse
    let should_refresh = ($needs_refresh and (not $skip_refresh) and (not $reuse_mode))
    let max_jobs = ($config.max_jobs? | default "half" | into string)
    let build_cores = ($config.build_cores? | default "2" | into string)
    let build_parallelism_description = (describe_build_parallelism $build_cores $max_jobs)

    let original_dir = (pwd)
    let env_status = check_environment_status
    let reused_launch_profile = if $reuse_mode and (not $env_status.already_in_env) {
        require_reused_launch_profile $env_prep.config_state "yzx env --reuse"
    } else {
        null
    }

    let has_setpriv = (which setpriv | is-not-empty)
    let trap_supervisor = "trap 'kill 0' HUP TERM; exec \"$@\""
    let configured_shell_name = ($config.default_shell? | default "nu" | str downcase)
    let invoking_shell_name = (
        if ($env.SHELL? | is-not-empty) {
            $env.SHELL | path basename | str downcase
        } else {
            $configured_shell_name
        }
    )

    if $reuse_mode and $needs_refresh {
        print "⚡ Reuse mode enabled - using the last built Yazelix profile without rebuild."
        print "   Local config/input changes since the last refresh are not applied."
    } else if $skip_refresh and $needs_refresh {
        print "⚠️  Skipping explicit refresh trigger; environment may be stale."
        print "   If tools/env vars look outdated, rerun without --skip-refresh or run 'yzx refresh'."
    } else if $needs_refresh {
        print $"🔄 Configuration changed - rebuilding environment using ($build_parallelism_description)..."
        rebuild_yazelix_environment --max-jobs $max_jobs --build-cores $build_cores --refresh-eval-cache
    }

    if $no_shell {
        # For --no-shell, preserve the invoking shell when possible.
        let shell_command = (resolve_shell_command $invoking_shell_name)
        if $reused_launch_profile != null {
            if $has_setpriv {
                run_with_launch_profile $config $reused_launch_profile "setpriv" "--pdeathsig" "TERM" "--" ...$shell_command --cwd $original_dir
            } else {
                run_with_launch_profile $config $reused_launch_profile "sh" "-c" $trap_supervisor "_" ...$shell_command --cwd $original_dir
            }
        } else if $has_setpriv {
            run_in_devenv_shell_command "setpriv" "--pdeathsig" "TERM" "--" ...$shell_command --max-jobs $max_jobs --build-cores $build_cores --cwd $original_dir --env-only --quiet --force-refresh=$should_refresh
        } else {
            # macOS and other systems without setpriv use POSIX trap fallback.
            run_in_devenv_shell_command "sh" "-c" $trap_supervisor "_" ...$shell_command --max-jobs $max_jobs --build-cores $build_cores --cwd $original_dir --env-only --quiet --force-refresh=$should_refresh
        }

    } else {
        # Launch configured shell
        let shell_command = (resolve_shell_command $configured_shell_name --login)
        let shell_exec = ($shell_command | first)
        # Prefer Linux parent-death signaling for force-close paths.
        # Fall back to POSIX trap on systems without setpriv (for example macOS).

        try {
            with-env {SHELL: $shell_exec} {
                if $reused_launch_profile != null {
                    if $has_setpriv {
                        run_with_launch_profile $config $reused_launch_profile "setpriv" "--pdeathsig" "TERM" "--" ...$shell_command --cwd $original_dir
                    } else {
                        run_with_launch_profile $config $reused_launch_profile "sh" "-c" $trap_supervisor "_" ...$shell_command --cwd $original_dir
                    }
                } else if $has_setpriv {
                    run_in_devenv_shell_command "setpriv" "--pdeathsig" "TERM" "--" ...$shell_command --max-jobs $max_jobs --build-cores $build_cores --cwd $original_dir --env-only --quiet --force-refresh=$should_refresh
                } else {
                    run_in_devenv_shell_command "sh" "-c" $trap_supervisor "_" ...$shell_command --max-jobs $max_jobs --build-cores $build_cores --cwd $original_dir --env-only --quiet --force-refresh=$should_refresh
                }
            }
        } catch {|err|
            print $"❌ Failed to launch configured shell: ($err.msg)"
            print "   Tip: rerun with 'yzx env --no-shell' to stay in your current shell."
            exit 1
        }
    }
}
