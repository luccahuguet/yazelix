#!/usr/bin/env nu
# yzx env command - Load Yazelix environment without UI

use ../utils/environment_bootstrap.nu *
use ../utils/config_state.nu [mark_config_state_applied]

# Load yazelix environment without UI
export def "yzx env" [
    --no-shell(-n)  # Keep current shell instead of launching configured shell
] {
    use ../utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    # Prepare environment (shared with start_yazelix.nu)
    let env_prep = prepare_environment
    let config = $env_prep.config
    let needs_refresh = $env_prep.needs_refresh

    let original_dir = (pwd)

    let shell_supervisor = "if command -v setpriv >/dev/null 2>&1; then exec setpriv --pdeathsig TERM -- \"$@\"; else trap 'kill 0' HUP TERM; exec \"$@\"; fi"

    if $no_shell {
        # For --no-shell, preserve current behavior and launch bash in devenv.
        run_in_devenv_shell_command "sh" "-c" $shell_supervisor "_" "bash" --cwd $original_dir --env-only --quiet --force-refresh=$needs_refresh

        if $needs_refresh {
            mark_config_state_applied $env_prep.config_state
        }
    } else {
        # Launch configured shell
        let shell_name = ($config.default_shell? | default "nu" | str downcase)
        let shell_command = match $shell_name {
            "nu" => ["nu" "--login"]
            "bash" => ["bash" "--login"]
            "fish" => ["fish" "-l"]
            "zsh" => ["zsh" "-l"]
            _ => [$shell_name]
        }
        let shell_exec = ($shell_command | first)
        # Launch configured shell through a small sh supervisor.
        # Prefer Linux parent-death signaling (setpriv --pdeathsig TERM) for
        # force-close paths; fall back to HUP/TERM trap when unavailable.

        try {
            with-env {SHELL: $shell_exec} {
                run_in_devenv_shell_command "sh" "-c" $shell_supervisor "_" ...$shell_command --cwd $original_dir --env-only --quiet --force-refresh=$needs_refresh
            }

            if $needs_refresh {
                mark_config_state_applied $env_prep.config_state
            }
        } catch {|err|
            print $"❌ Failed to launch configured shell: ($err.msg)"
            print "   Tip: rerun with 'yzx env --no-shell' to stay in your current shell."
            exit 1
        }
    }
}
