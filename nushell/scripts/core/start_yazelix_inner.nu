#!/usr/bin/env nu
# Interactive launch sequence for the active Yazelix runtime

use ../utils/runtime_paths.nu [get_yazelix_state_dir require_yazelix_runtime_dir]
use ../utils/runtime_commands.nu [resolve_zellij_default_shell]
use ../utils/yzx_core_bridge.nu [profile_startup_step]
use ../utils/yzx_core_bridge.nu [build_default_yzx_core_error_surface run_yzx_core_json_command]
use ../setup/welcome.nu [show_welcome build_welcome_message get_yazelix_colors]

const CONSTANTS_DATA_PATH = ((path self | path dirname) | path join ".." "utils" "constants_data.json")

def get_zellij_config_paths [] {
    (open $CONSTANTS_DATA_PATH).zellij_config_paths
}

const RUNTIME_MATERIALIZATION_MATERIALIZE_COMMAND = "runtime-materialization.materialize"
const STARTUP_HANDOFF_CAPTURE_COMMAND = "startup-handoff.capture"
const SESSION_CONFIG_SNAPSHOT_WRITE_COMMAND = "session-config-snapshot.write"

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
        error make {msg: $"Zellij layout not found: ($resolved)\nRun `yzx doctor` to inspect the generated-state contract, or check the configured layout name."}
    }

    if (($resolved | path type) != "file") {
        error make {msg: $"Zellij layout path is not a file: ($resolved)"}
    }

    $resolved
}

def regenerate_runtime_configs [runtime_dir: string, --quiet] {
    let result = (profile_startup_step "materialization_orchestrator" "materialize_runtime_state" {
        (run_yzx_core_json_command
            $runtime_dir
            (build_default_yzx_core_error_surface)
            [$RUNTIME_MATERIALIZATION_MATERIALIZE_COMMAND "--from-env"]
            "Yazelix Rust runtime-materialization materialize helper returned invalid JSON.")
    })

    if (not $quiet) and (($result.plan.status? | default "") != "noop") {
        print "✅ Generated runtime state materialized."
    }

    $result.plan
}

def --env prepare_session_config_snapshot [runtime_dir: string, applied_runtime_state: record] {
    let launch_id = (date now | into int | into string)
    let request = {
        state_dir: (get_yazelix_state_dir)
        snapshot_id: $launch_id
        source_config_file: ($applied_runtime_state.config_file? | default "" | into string)
        source_config_hash: ($applied_runtime_state.config_hash? | default "" | into string)
        runtime_dir: $runtime_dir
        runtime_hash: ($applied_runtime_state.runtime_hash? | default "" | into string)
        normalized_config: ($applied_runtime_state.config? | default {})
    }
    let snapshot = (run_yzx_core_json_command
        $runtime_dir
        (build_default_yzx_core_error_surface)
        [$SESSION_CONFIG_SNAPSHOT_WRITE_COMMAND "--request-json" ($request | to json -r)]
        "Yazelix Rust session config snapshot helper returned invalid JSON.")
    let snapshot_path = ($snapshot.snapshot_path? | default "" | into string | str trim)

    if ($snapshot_path | is-empty) {
        error make {
            msg: (
                "Yazelix session config snapshot helper did not return a snapshot path. "
                + "Run `yzx doctor` to inspect the runtime and generated-state contract."
            )
        }
    }

    $env.YAZELIX_SESSION_CONFIG_PATH = $snapshot_path
    $env.YAZELIX_STATUS_BAR_CACHE_PATH = ($snapshot_path | path dirname | path join "status_bar_cache.json")
    $snapshot_path
}

def capture_startup_handoff_context [
    runtime_dir: string
    applied_runtime_state: record
    working_dir: string
    session_default_cwd: string
    launch_process_cwd: string
    merged_zellij_dir: string
    layout_path: string
    zellij_default_shell: string
    startup_facts: record
    --verbose
] {
    let materialization_status = (
        $applied_runtime_state.status?
        | default "noop"
        | into string
        | str trim
    )
    if $materialization_status == "noop" {
        return
    }

    let request = {
        state_dir: (get_yazelix_state_dir)
        working_dir: $working_dir
        session_default_cwd: $session_default_cwd
        launch_process_cwd: $launch_process_cwd
        zellij_config_dir: $merged_zellij_dir
        layout_path: $layout_path
        default_shell: $zellij_default_shell
        materialization_status: $materialization_status
        materialization_reason: ($applied_runtime_state.reason? | default "" | into string)
        materialization_should_regenerate: ($applied_runtime_state.should_regenerate? | default false)
        materialization_should_sync_static_assets: ($applied_runtime_state.should_sync_static_assets? | default false)
        missing_artifacts: ($applied_runtime_state.missing_artifacts? | default [])
    }

    try {
        let capture = (run_yzx_core_json_command
            $runtime_dir
            (build_default_yzx_core_error_surface)
            [$STARTUP_HANDOFF_CAPTURE_COMMAND "--request-json" ($request | to json -r)]
            "Yazelix Rust startup-handoff capture helper returned invalid JSON.")
        if $verbose and ($capture.recorded? | default false) {
            let capture_path = ($capture.context_path? | default ($capture.latest_path? | default "unknown"))
            print $"📝 Startup handoff context: ($capture_path)"
        }
    } catch {|err|
        print $"⚠️ Failed to write startup handoff context: ($err.msg)"
    }
}

def main [cwd_override?: string, layout_override?: string, --verbose] {
    let yazelix_dir = (require_existing_directory (require_yazelix_runtime_dir) "Yazelix runtime directory")
    let startup_facts = (run_yzx_core_json_command
        $yazelix_dir
        (build_default_yzx_core_error_surface)
        ["startup-facts.compute"]
        "Yazelix Rust startup-facts helper returned invalid JSON.")
    let quiet_mode = ($env.YAZELIX_ENV_ONLY? == "true")
    let profile_exit_before_zellij = ($env.YAZELIX_STARTUP_PROFILE_EXIT_BEFORE_ZELLIJ? == "true")
    let skip_welcome_screen = (
        ($startup_facts.skip_welcome_screen? | default false)
        or ($env.YAZELIX_STARTUP_PROFILE_SKIP_WELCOME? == "true")
    )

    let log_dir = (get_yazelix_state_dir | path join "logs")
    mkdir $log_dir
    let colors = get_yazelix_colors
    let welcome_facts = {
        terminals: ($startup_facts.terminals? | default [])
    }
    let welcome_message = build_welcome_message $yazelix_dir $colors $welcome_facts
    profile_startup_step "inner" "show_welcome" {
        show_welcome $skip_welcome_screen $quiet_mode $startup_facts.welcome_style $startup_facts.welcome_duration_seconds $startup_facts.show_macchina_on_welcome $welcome_message $log_dir $colors
    } {
        skipped: ($skip_welcome_screen or $quiet_mode)
    }
    let upgrade_summary = (try { profile_startup_step "inner" "show_upgrade_summary" {
        (run_yzx_core_json_command
            $yazelix_dir
            (build_default_yzx_core_error_surface)
            ["upgrade-summary.first-run"]
            "Yazelix Rust first-run upgrade-summary helper returned invalid JSON.")
    } } catch {|err|
        if $verbose {
            print $"⚠️ Failed to render upgrade summary: ($err.msg)"
        }
        null
    })
    if ($upgrade_summary != null) and ($upgrade_summary.shown? | default false) {
        let output = ($upgrade_summary.output? | default "" | into string)
        if ($output | is-not-empty) {
            print $output
        }
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
    let session_config_snapshot_path = (prepare_session_config_snapshot $yazelix_dir $applied_runtime_state)

    let merged_zellij_dir = ((get_zellij_config_paths).merged_config_dir | str replace "~" $env.HOME)
    let working_dir = if ($cwd_override | is-not-empty) {
        $cwd_override
    } else {
        $env.HOME
    }
    let session_default_cwd = $working_dir
    let launch_process_cwd = $working_dir
    let zellij_default_shell = (resolve_zellij_default_shell $yazelix_dir $startup_facts.default_shell)

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

    capture_startup_handoff_context $yazelix_dir $applied_runtime_state $working_dir $session_default_cwd $launch_process_cwd $merged_zellij_dir $layout_path $zellij_default_shell $startup_facts --verbose=$verbose

    cd $launch_process_cwd

    if $profile_exit_before_zellij {
        profile_startup_step "inner" "zellij_handoff_ready" {
            null
        } {
            layout_path: $layout_path
            default_shell: $zellij_default_shell
            session_config_snapshot_path: $session_config_snapshot_path
        } | ignore
        return
    }

    ^zellij --config-dir $merged_zellij_dir options --default-cwd $session_default_cwd --default-layout $layout_path --default-shell $zellij_default_shell
}
