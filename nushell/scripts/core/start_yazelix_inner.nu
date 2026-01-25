#!/usr/bin/env nu
# Interactive launch sequence (runs inside devenv shell)

use ../utils/config_parser.nu parse_yazelix_config
use ../utils/constants.nu [ZELLIJ_CONFIG_PATHS, YAZELIX_ENV_VARS, YAZELIX_LOGS_DIR]
use ../utils/ascii_art.nu get_yazelix_colors
use ../setup/welcome.nu [show_welcome build_welcome_message]
use ../setup/yazi_config_merger.nu generate_merged_yazi_config
use ../setup/zellij_config_merger.nu generate_merged_zellij_config

def main [cwd_override?: string, layout_override?: string] {
    let config = parse_yazelix_config
    let yazelix_dir = ($env.HOME | path join ".config" "yazelix")
    let quiet_mode = ($env.YAZELIX_ENV_ONLY? == "true")

    let log_dir = ($YAZELIX_LOGS_DIR | str replace "~" $env.HOME)
    mkdir $log_dir
    let colors = get_yazelix_colors
    let welcome_message = build_welcome_message $yazelix_dir $config.helix_mode $colors
    show_welcome $config.skip_welcome_screen $quiet_mode $config.ascii_art_mode $config.show_macchina_on_welcome $welcome_message $log_dir $colors

    print "🔧 Preparing Yazi configuration..."
    if ($env.YAZELIX_VERBOSE? == "true") {
        generate_merged_yazi_config $yazelix_dir | ignore
    } else {
        generate_merged_yazi_config $yazelix_dir --quiet | ignore
    }

    let merged_zellij_dir = ($ZELLIJ_CONFIG_PATHS.merged_config_dir | str replace "~" $env.HOME)
    generate_merged_zellij_config $yazelix_dir | ignore

    let working_dir = if ($cwd_override | is-not-empty) {
        $cwd_override
    } else if ($env.YAZELIX_LAUNCH_CWD? | is-not-empty) {
        $env.YAZELIX_LAUNCH_CWD
    } else {
        $env.HOME
    }

    let layout_path = if ($layout_override | is-not-empty) {
        $layout_override
    } else {
        let layout = if ($env.ZELLIJ_DEFAULT_LAYOUT? | is-not-empty) {
            $env.ZELLIJ_DEFAULT_LAYOUT
        } else {
            $YAZELIX_ENV_VARS.ZELLIJ_DEFAULT_LAYOUT
        }
        if ($layout | str contains "/") or ($layout | str ends-with ".kdl") {
            $layout
        } else {
            $"($merged_zellij_dir)/layouts/($layout).kdl"
        }
    }

    if ($config.persistent_sessions == "true") {
        # Check if session already exists
        let existing_sessions = (do { ^zellij list-sessions } | complete)
        let session_exists = if $existing_sessions.exit_code == 0 {
            let sessions = ($existing_sessions.stdout | lines | where {|s| $s | str contains $config.session_name})
            ($sessions | is-not-empty)
        } else {
            false
        }

        if $session_exists {
            # Warn if --path is used with an existing persistent session
            if ($cwd_override | is-not-empty) {
                print $"⚠️  Session '($config.session_name)' already exists - --path ignored."
                print $"   To start in a new directory, first run: zellij kill-session ($config.session_name)"
            }
            # Attach to existing session without options to avoid inconsistent state
            ^zellij --config-dir $merged_zellij_dir attach $config.session_name
        } else {
            # Create new session with all options
            ^zellij --config-dir $merged_zellij_dir attach -c $config.session_name options --default-cwd $working_dir --default-layout $layout_path --pane-frames false --default-shell $config.default_shell
        }
    } else {
        ^zellij --config-dir $merged_zellij_dir options --default-cwd $working_dir --default-layout $layout_path --pane-frames false --default-shell $config.default_shell
    }
}
