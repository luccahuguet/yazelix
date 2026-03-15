#!/usr/bin/env nu
# Interactive launch sequence (runs inside devenv shell)

use ../utils/config_parser.nu parse_yazelix_config
use ../utils/config_state.nu [compute_config_state mark_config_state_applied]
use ../utils/constants.nu [ZELLIJ_CONFIG_PATHS, YAZELIX_LOGS_DIR]
use ../utils/ascii_art.nu get_yazelix_colors
use ../setup/welcome.nu [show_welcome build_welcome_message]
use ../setup/yazi_config_merger.nu generate_merged_yazi_config
use ../setup/zellij_config_merger.nu generate_merged_zellij_config

def require_existing_directory [path_value: string, label: string] {
    let resolved = ($path_value | path expand)

    if not ($resolved | path exists) {
        error make {msg: $"Missing ($label): ($resolved)"}
    }

    if (($resolved | path type) != "dir") {
        error make {msg: $"($label) is not a directory: ($resolved)"}
    }

    $resolved
}

def require_existing_layout [layout_path: string] {
    let resolved = ($layout_path | path expand)

    if not ($resolved | path exists) {
        error make {msg: $"Zellij layout not found: ($resolved)\nRun `yzx refresh` to regenerate layouts, or check the configured layout name."}
    }

    if (($resolved | path type) != "file") {
        error make {msg: $"Zellij layout path is not a file: ($resolved)"}
    }

    $resolved
}

def resolve_session_default_cwd [working_dir: string] {
    if (($env.YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE? | default "") | is-not-empty) {
        $env.HOME
    } else {
        $working_dir
    }
}

def resolve_launch_process_cwd [working_dir: string] {
    if (($env.YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE? | default "") | is-not-empty) {
        $env.HOME
    } else {
        $working_dir
    }
}

def main [cwd_override?: string, layout_override?: string, --verbose] {
    let config = parse_yazelix_config
    let sidebar_enabled = ($config.enable_sidebar? | default true)
    let configured_layout = if $sidebar_enabled { "yzx_side" } else { "yzx_no_side" }
    let yazelix_dir = (require_existing_directory ($env.HOME | path join ".config" "yazelix") "Yazelix runtime directory")
    let quiet_mode = ($env.YAZELIX_ENV_ONLY? == "true")

    let log_dir = ($YAZELIX_LOGS_DIR | str replace "~" $env.HOME)
    mkdir $log_dir
    let colors = get_yazelix_colors
    let welcome_message = build_welcome_message $yazelix_dir $config.helix_mode $colors
    show_welcome $config.skip_welcome_screen $quiet_mode $config.ascii_art_mode $config.show_macchina_on_welcome $welcome_message $log_dir $colors

    print "🔧 Preparing Yazi configuration..."
    try {
        if $verbose {
            generate_merged_yazi_config $yazelix_dir | ignore
        } else {
            generate_merged_yazi_config $yazelix_dir --quiet | ignore
        }
    } catch { |err|
        error make {msg: $"Failed to generate Yazi configuration: ($err.msg)\nRun `yzx doctor` to inspect the runtime, then rerun `yzx refresh` if needed."}
    }

    let merged_zellij_dir = ($ZELLIJ_CONFIG_PATHS.merged_config_dir | str replace "~" $env.HOME)
    try {
        generate_merged_zellij_config $yazelix_dir | ignore
    } catch { |err|
        error make {msg: $"Failed to generate Zellij configuration: ($err.msg)\nRun `yzx doctor` to inspect the runtime, then rerun `yzx refresh` if needed."}
    }

    let working_dir = if ($cwd_override | is-not-empty) {
        $cwd_override
    } else {
        $env.HOME
    }
    let session_default_cwd = (resolve_session_default_cwd $working_dir)
    let launch_process_cwd = (resolve_launch_process_cwd $working_dir)

    let resolved_layout_path = if ($layout_override | is-not-empty) {
        $layout_override
    } else {
        let layout = if ($env.YAZELIX_LAYOUT_OVERRIDE? | is-not-empty) {
            $env.YAZELIX_LAYOUT_OVERRIDE
        } else if ($env.YAZELIX_SWEEP_TEST_ID? | is-not-empty) and ($env.ZELLIJ_DEFAULT_LAYOUT? | is-not-empty) {
            $env.ZELLIJ_DEFAULT_LAYOUT
        } else {
            $configured_layout
        }
        if ($layout | str contains "/") or ($layout | str ends-with ".kdl") {
            $layout
        } else {
            $"($merged_zellij_dir)/layouts/($layout).kdl"
        }
    }
    let layout_path = (require_existing_layout $resolved_layout_path)

    # Record that the current config/input state has been successfully applied
    # once we are inside the prepared Yazelix runtime.
    mark_config_state_applied (compute_config_state)

    # Keep restart-only Yazi bootstrap separate from the pane/session cwd defaults.
    cd $launch_process_cwd

    if ($config.persistent_sessions == "true") {
        # Check if session already exists
        let existing_sessions = (do { ^zellij list-sessions } | complete)
        let session_exists = if $existing_sessions.exit_code == 0 {
            let sessions = (
                $existing_sessions.stdout
                | lines
                | each {|line|
                    let clean_line = (
                        $line
                        | str replace -ra '\u001b\[[0-9;]*[A-Za-z]' ''
                        | str replace -r '^>\s*' ''
                        | str trim
                    )
                    if ($clean_line | is-empty) {
                        null
                    } else {
                        $clean_line
                        | split row " "
                        | where {|token| $token != ""}
                        | first
                    }
                }
                | where ($it | is-not-empty)
            )
            ($sessions | any {|name| $name == $config.session_name})
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
            ^zellij --config-dir $merged_zellij_dir attach -c $config.session_name options --default-cwd $session_default_cwd --default-layout $layout_path --pane-frames false --default-shell $config.default_shell
        }
    } else {
        ^zellij --config-dir $merged_zellij_dir options --default-cwd $session_default_cwd --default-layout $layout_path --pane-frames false --default-shell $config.default_shell
    }
}
