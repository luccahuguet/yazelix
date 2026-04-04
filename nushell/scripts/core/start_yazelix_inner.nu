#!/usr/bin/env nu
# Interactive launch sequence (runs inside devenv shell)

use ../utils/config_parser.nu parse_yazelix_config
use ../utils/config_state.nu [compute_config_state record_materialized_state]
use ../utils/launch_state.nu [record_launch_profile_state resolve_current_session_profile]
use ../utils/constants.nu [ZELLIJ_CONFIG_PATHS, YAZELIX_LOGS_DIR]
use ../utils/ascii_art.nu get_yazelix_colors
use ../utils/common.nu [require_yazelix_runtime_dir resolve_zellij_default_shell]
use ../utils/failure_classes.nu [format_failure_classification]
use ../utils/upgrade_summary.nu [maybe_show_first_run_upgrade_summary]
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
        let classification = (format_failure_classification "generated-state" "Regenerate layouts with `yzx refresh`, or fix the configured layout name if it points at a missing file.")
        error make {msg: $"Zellij layout not found: ($resolved)\nRun `yzx refresh` to regenerate layouts, or check the configured layout name.\n($classification)"}
    }

    if (($resolved | path type) != "file") {
        error make {msg: $"Zellij layout path is not a file: ($resolved)"}
    }

    $resolved
}

def main [cwd_override?: string, layout_override?: string, --verbose] {
    let config = parse_yazelix_config
    let sidebar_enabled = ($config.enable_sidebar? | default true)
    let configured_layout = if $sidebar_enabled { "yzx_side" } else { "yzx_no_side" }
    let yazelix_dir = (require_existing_directory (require_yazelix_runtime_dir) "Yazelix runtime directory")
    let quiet_mode = ($env.YAZELIX_ENV_ONLY? == "true")

    let log_dir = ($YAZELIX_LOGS_DIR | str replace "~" $env.HOME)
    mkdir $log_dir
    let colors = get_yazelix_colors
    let welcome_message = build_welcome_message $yazelix_dir $config.helix_mode $colors
    show_welcome $config.skip_welcome_screen $quiet_mode $config.welcome_style $config.welcome_duration_seconds $config.show_macchina_on_welcome $welcome_message $log_dir $colors
    let upgrade_summary = (try { maybe_show_first_run_upgrade_summary } catch {|err|
        if $verbose {
            print $"⚠️ Failed to render upgrade summary: ($err.msg)"
        }
        null
    })
    if ($upgrade_summary != null) and ($upgrade_summary.shown? | default false) {
        print ""
    }

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
    let session_default_cwd = $working_dir
    let launch_process_cwd = $working_dir
    let zellij_default_shell = (resolve_zellij_default_shell $yazelix_dir $config.default_shell)

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
    # once we are inside the prepared Yazelix runtime, and remember the live
    # built profile for later reuse/startup checks.
    let applied_state = (compute_config_state)
    record_materialized_state $applied_state
    let built_profile = (resolve_current_session_profile)
    if ($built_profile | is-not-empty) {
        record_launch_profile_state $applied_state $built_profile
    }

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
            ^zellij --config-dir $merged_zellij_dir attach -c $config.session_name options --default-cwd $session_default_cwd --default-layout $layout_path --default-shell $zellij_default_shell
        }
    } else {
        ^zellij --config-dir $merged_zellij_dir options --default-cwd $session_default_cwd --default-layout $layout_path --default-shell $zellij_default_shell
    }
}
