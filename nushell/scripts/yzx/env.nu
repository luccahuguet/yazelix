#!/usr/bin/env nu
# yzx env command - Load Yazelix environment without UI

use ../utils/environment_bootstrap.nu *
use ../utils/config_state.nu [mark_config_state_applied]

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

# Load yazelix environment without UI
export def "yzx env" [
    --no-shell(-n)  # Keep current shell instead of launching configured shell
    --skip-refresh(-s)  # Skip explicit refresh trigger and allow potentially stale environment
] {
    use ../utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    # Prepare environment (shared with start_yazelix.nu)
    let env_prep = prepare_environment
    let config = $env_prep.config
    let needs_refresh = $env_prep.needs_refresh
    let should_refresh = ($needs_refresh and (not $skip_refresh))
    let refresh_reason = ($env_prep.config_state.refresh_reason? | default "config or devenv inputs changed since last launch")

    let original_dir = (pwd)

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

    if $skip_refresh and $needs_refresh {
        print "⚠️  Skipping explicit refresh trigger; environment may be stale."
        print "   If tools/env vars look outdated, rerun without --skip-refresh or run 'yzx refresh'."
    } else if $needs_refresh {
        print $"♻️  ($refresh_reason) – rebuilding environment"
    }

    if $no_shell {
        # For --no-shell, preserve the invoking shell when possible.
        let shell_command = (resolve_shell_command $invoking_shell_name)
        if $has_setpriv {
            run_in_devenv_shell_command "setpriv" "--pdeathsig" "TERM" "--" ...$shell_command --cwd $original_dir --env-only --quiet --force-refresh=$should_refresh
        } else {
            # macOS and other systems without setpriv use POSIX trap fallback.
            run_in_devenv_shell_command "sh" "-c" $trap_supervisor "_" ...$shell_command --cwd $original_dir --env-only --quiet --force-refresh=$should_refresh
        }

        if $should_refresh {
            mark_config_state_applied $env_prep.config_state
        }
    } else {
        # Launch configured shell
        let shell_command = (resolve_shell_command $configured_shell_name --login)
        let shell_exec = ($shell_command | first)
        # Prefer Linux parent-death signaling for force-close paths.
        # Fall back to POSIX trap on systems without setpriv (for example macOS).

        try {
            with-env {SHELL: $shell_exec} {
                if $has_setpriv {
                    run_in_devenv_shell_command "setpriv" "--pdeathsig" "TERM" "--" ...$shell_command --cwd $original_dir --env-only --quiet --force-refresh=$should_refresh
                } else {
                    run_in_devenv_shell_command "sh" "-c" $trap_supervisor "_" ...$shell_command --cwd $original_dir --env-only --quiet --force-refresh=$should_refresh
                }
            }

            if $should_refresh {
                mark_config_state_applied $env_prep.config_state
            }
        } catch {|err|
            print $"❌ Failed to launch configured shell: ($err.msg)"
            print "   Tip: rerun with 'yzx env --no-shell' to stay in your current shell."
            exit 1
        }
    }
}
