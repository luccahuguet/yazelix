#!/usr/bin/env nu
# Interactive launch sequence for the active Yazelix runtime

use ../utils/config_parser.nu parse_yazelix_config
use ../utils/constants.nu [ZELLIJ_CONFIG_PATHS, YAZELIX_LOGS_DIR]
use ../utils/ascii_art.nu get_yazelix_colors
use ../utils/common.nu [require_yazelix_runtime_dir resolve_zellij_default_shell]
use ../utils/failure_classes.nu [format_failure_classification]
use ./materialization_orchestrator.nu [regenerate_runtime_configs record_current_materialized_state]
use ../utils/startup_profile.nu [profile_startup_step]
use ../utils/upgrade_summary.nu [maybe_show_first_run_upgrade_summary]
use ../setup/welcome.nu [show_welcome build_welcome_message]

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
        let classification = (format_failure_classification "generated-state" "Run `yzx doctor` to inspect generated-state issues, or fix the configured layout name if it points at a missing file.")
        error make {msg: $"Zellij layout not found: ($resolved)\nRun `yzx doctor` to inspect the generated-state contract, or check the configured layout name.\n($classification)"}
    }

    if (($resolved | path type) != "file") {
        error make {msg: $"Zellij layout path is not a file: ($resolved)"}
    }

    $resolved
}

def main [cwd_override?: string, layout_override?: string, --verbose] {
    let config = parse_yazelix_config
    let yazelix_dir = (require_existing_directory (require_yazelix_runtime_dir) "Yazelix runtime directory")
    let quiet_mode = ($env.YAZELIX_ENV_ONLY? == "true")
    let profile_exit_before_zellij = ($env.YAZELIX_STARTUP_PROFILE_EXIT_BEFORE_ZELLIJ? == "true")
    let skip_welcome_screen = (
        ($config.skip_welcome_screen? | default false)
        or ($env.YAZELIX_STARTUP_PROFILE_SKIP_WELCOME? == "true")
    )

    let log_dir = ($YAZELIX_LOGS_DIR | str replace "~" $env.HOME)
    mkdir $log_dir
    let colors = get_yazelix_colors
    let welcome_message = build_welcome_message $yazelix_dir $colors
    profile_startup_step "inner" "show_welcome" {
        show_welcome $skip_welcome_screen $quiet_mode $config.welcome_style $config.welcome_duration_seconds $config.show_macchina_on_welcome $welcome_message $log_dir $colors
    } {
        skipped: ($skip_welcome_screen or $quiet_mode)
    }
    let upgrade_summary = (try { profile_startup_step "inner" "show_upgrade_summary" {
        maybe_show_first_run_upgrade_summary
    } } catch {|err|
        if $verbose {
            print $"⚠️ Failed to render upgrade summary: ($err.msg)"
        }
        null
    })
    if ($upgrade_summary != null) and ($upgrade_summary.shown? | default false) {
        print ""
    }

    let applied_runtime_state = (try {
        profile_startup_step "inner" "materialize_runtime_configs" {
            if $verbose {
                print "🔧 Preparing Yazelix generated runtime state..."
            }
            regenerate_runtime_configs $yazelix_dir --quiet=(not $verbose)
        }
    } catch { |err|
            error make {msg: $"Failed to prepare Yazelix generated runtime state: ($err.msg)\nRun `yzx doctor` to inspect the runtime and generated-state contract, then restart Yazelix after fixing the reported problem."}
    })

    let merged_zellij_dir = ($ZELLIJ_CONFIG_PATHS.merged_config_dir | str replace "~" $env.HOME)
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
        let from_plan = (
            $applied_runtime_state.zellij_layout_path?
            | default ""
            | into string
            | str trim
        )
        if ($from_plan | is-empty) {
            error make {
                msg: (
                    "Yazelix materialization plan did not return a managed Zellij layout path. "
                    + "Run `yzx doctor` to inspect the runtime and generated-state contract."
                )
            }
        }
        $from_plan
    }
    let layout_path = (require_existing_layout $resolved_layout_path)

    # Record that the current config/runtime state has been successfully applied
    # once generated config has been refreshed inside the prepared runtime.
    profile_startup_step "inner" "record_runtime_state" {
        record_current_materialized_state $applied_runtime_state
    } | ignore

    cd $launch_process_cwd

    if $profile_exit_before_zellij {
        profile_startup_step "inner" "zellij_handoff_ready" {
            null
        } {
            layout_path: $layout_path
            default_shell: $zellij_default_shell
            persistent_sessions: ($config.persistent_sessions? | default "false")
        } | ignore
        return
    }

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
