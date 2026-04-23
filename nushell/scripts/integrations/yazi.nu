#!/usr/bin/env nu
# Yazi integration utilities for Yazelix

use ../utils/logging.nu log_to_file
use ../utils/runtime_paths.nu [get_yazelix_runtime_dir]
use ../utils/yzx_core_bridge.nu [build_default_yzx_core_error_surface run_yzx_core_json_command run_zellij_pipe]

def resolve_optional_command [configured: any, fallback: string] {
    let raw = ($configured | default "" | into string | str trim)
    if ($raw | is-empty) {
        $fallback
    } else {
        $raw
    }
}

def command_is_available [command: string] {
    let normalized = ($command | str trim)
    if ($normalized | is-empty) {
        false
    } else if (($normalized | str contains "/") or ($normalized | str starts-with "~")) {
        (($normalized | path expand) | path exists)
    } else {
        (which $normalized | is-not-empty)
    }
}

export def get_yazi_command [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let facts = (run_yzx_core_json_command $runtime_dir (build_default_yzx_core_error_surface) [
        "integration-facts.compute"
    ] "Yazelix Rust integration-facts helper returned invalid JSON.")
    resolve_optional_command ($facts.yazi_command? | default null) "yazi"
}

export def get_ya_command [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let facts = (run_yzx_core_json_command $runtime_dir (build_default_yzx_core_error_surface) [
        "integration-facts.compute"
    ] "Yazelix Rust integration-facts helper returned invalid JSON.")
    resolve_optional_command ($facts.ya_command? | default null) "ya"
}

def has_ya_command [] {
    command_is_available (get_ya_command)
}

export def is_sidebar_enabled [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let facts = (run_yzx_core_json_command $runtime_dir (build_default_yzx_core_error_surface) [
        "integration-facts.compute"
    ] "Yazelix Rust integration-facts helper returned invalid JSON.")
    ($facts.enable_sidebar? | default true)
}

def run_ya_emit_to [yazi_id: string, action: string, ...args: string] {
    let ya_command = (get_ya_command)
    let result = (^$ya_command "emit-to" $yazi_id $action ...$args | complete)
    if $result.exit_code != 0 {
        let stderr = ($result.stderr | default "" | str trim)
        let stdout = ($result.stdout | default "" | str trim)
        let details = if ($stderr | is-not-empty) {
            $stderr
        } else if ($stdout | is-not-empty) {
            $stdout
        } else {
            $"exit code ($result.exit_code)"
        }
        error make {msg: $"Failed to emit Yazi action `($action)` to instance `($yazi_id)`: ($details)"}
    }
}

export def consume_bootstrap_sidebar_cwd [] {
    let cwd_file = ($env.YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE? | default "" | str trim)
    if ($cwd_file | is-empty) {
        return null
    }

    let expanded_file = ($cwd_file | path expand)
    if not ($expanded_file | path exists) {
        return null
    }

    let requested_path = (open --raw $expanded_file | str trim)
    rm -f $expanded_file

    if ($requested_path | is-empty) {
        return null
    }

    let expanded_path = ($requested_path | path expand)
    if not ($expanded_path | path exists) {
        return null
    }

    if (($expanded_path | path type) == "dir") {
        $expanded_path
    } else {
        $expanded_path | path dirname
    }
}

def resolve_reveal_target_path [buffer_name: string] {
    if ($buffer_name | is-empty) {
        error make {msg: "Buffer name not provided"}
    }

    let normalized_buffer_name = if ($buffer_name | str contains "~") {
        $buffer_name | path expand
    } else {
        $buffer_name
    }

    let full_path = if ($normalized_buffer_name | path type) != "relative" {
        $normalized_buffer_name | path expand
    } else {
        ($env.PWD | path join $normalized_buffer_name | path expand)
    }

    if not ($full_path | path exists) {
        error make {msg: $"Resolved path '($full_path)' does not exist."}
    }

    $full_path
}

export def get_active_sidebar_state [] {
    if ($env.ZELLIJ? | is-empty) {
        return null
    }

    let state = (try {
        let raw = (run_zellij_pipe "get_active_tab_session_state")
        $raw | from json
    } catch {
        null
    })

    if ($state | is-empty) or ($state.raw? | is-not-empty) {
        return null
    }

    let sidebar_state = ($state.sidebar_yazi? | default null)
    if $sidebar_state == null {
        return null
    }

    let sidebar_yazi_id = ($sidebar_state.yazi_id? | default "" | into string | str trim)
    let sidebar_cwd = ($sidebar_state.cwd? | default "" | into string | str trim)
    if ($sidebar_yazi_id | is-empty) or ($sidebar_cwd | is-empty) {
        return null
    }

    {
        yazi_id: $sidebar_yazi_id
        cwd: $sidebar_cwd
    }
}

def get_active_sidebar_yazi_action_context [] {
    if not (is_sidebar_enabled) {
        return {status: "skipped", reason: "sidebar_disabled", sidebar_state: null}
    }

    if ($env.ZELLIJ? | is-empty) {
        return {status: "skipped", reason: "outside_zellij", sidebar_state: null}
    }

    if not (has_ya_command) {
        return {status: "skipped", reason: "ya_missing", sidebar_state: null}
    }

    let sidebar_state = (get_active_sidebar_state)
    if ($sidebar_state | is-empty) {
        return {status: "skipped", reason: "sidebar_yazi_missing", sidebar_state: null}
    }

    {status: "ok", sidebar_state: $sidebar_state}
}

export def sync_active_sidebar_yazi_to_directory [target_path: path, log_file: string = "yazi_sync.log"] {
    let action_context = (get_active_sidebar_yazi_action_context)
    if $action_context.status != "ok" {
        return ($action_context | reject sidebar_state)
    }

    sync_sidebar_yazi_state_to_directory $action_context.sidebar_state $target_path $log_file
}

export def sync_sidebar_yazi_state_to_directory [sidebar_state: record, target_path: path, log_file: string = "yazi_sync.log"] {
    let expanded_target_path = ($target_path | path expand)
    let target_dir = if (($expanded_target_path | path type) == "dir") {
        $expanded_target_path
    } else {
        $expanded_target_path | path dirname
    }

    try {
        run_ya_emit_to $sidebar_state.yazi_id "cd" $target_dir
        log_to_file $log_file $"Synced active sidebar Yazi to directory: ($target_dir)"
        {status: "ok", target_dir: $target_dir}
    } catch {|err|
        log_to_file $log_file $"Failed to sync active sidebar Yazi to directory '($target_dir)': ($err.msg)"
        {status: "error", reason: $err.msg, target_dir: $target_dir}
    }
}

export def refresh_active_sidebar_yazi [log_file: string = "yazi_refresh.log"] {
    let action_context = (get_active_sidebar_yazi_action_context)
    if $action_context.status != "ok" {
        return ($action_context | reject sidebar_state)
    }
    let sidebar_state = $action_context.sidebar_state

    try {
        run_ya_emit_to $sidebar_state.yazi_id "refresh"
        run_ya_emit_to $sidebar_state.yazi_id "plugin" "git" "refresh-sidebar"
        let sidebar_cwd = ($sidebar_state.cwd? | default "" | str trim)
        if ($sidebar_cwd | is-not-empty) {
            run_ya_emit_to $sidebar_state.yazi_id "plugin" "starship" $sidebar_cwd
        }
        log_to_file $log_file $"Refreshed active sidebar Yazi instance and reran sidebar integrations: ($sidebar_state.yazi_id)"
        {status: "ok", yazi_id: $sidebar_state.yazi_id}
    } catch {|err|
        log_to_file $log_file $"Failed to refresh active sidebar Yazi instance '($sidebar_state.yazi_id)': ($err.msg)"
        {status: "error", reason: $err.msg, yazi_id: $sidebar_state.yazi_id}
    }
}

# Navigate Yazi to the directory of the current Helix buffer
export def reveal_in_yazi [buffer_name: string] {
    log_to_file "reveal_in_yazi.log" $"reveal_in_yazi called with buffer_name: '($buffer_name)'"

    # Check if sidebar mode is enabled
    let sidebar_enabled = is_sidebar_enabled
    if (not $sidebar_enabled) {
        let friendly_msg = "📂 Reveal in Yazi only works in sidebar mode. You're currently using no-sidebar mode."
        let tip_msg = "💡 Tip: Use your editor-local file picker in no-sidebar mode, or enable sidebar mode in yazelix.toml"
        print $"($friendly_msg)\n($tip_msg)"
        log_to_file "reveal_in_yazi.log" "Sidebar mode disabled - reveal_in_yazi not available"
        return
    }

    if ($env.ZELLIJ? | is-empty) {
        let error_msg = "Reveal in Yazi only works inside a Yazelix/Zellij session."
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    if not (has_ya_command) {
        let error_msg = $"The configured Yazi CLI `\(get_ya_command\)` is not available in this environment."
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    let full_path = try {
        resolve_reveal_target_path $buffer_name
    } catch {|err|
        log_to_file "reveal_in_yazi.log" $"ERROR: ($err.msg)"
        print $"Error: ($err.msg)"
        return
    }

    log_to_file "reveal_in_yazi.log" $"Resolved full path: '($full_path)'"

    let sidebar_state = (get_active_sidebar_state)
    let sidebar_yazi_id = ($sidebar_state.yazi_id? | default "" | str trim)
    if ($sidebar_yazi_id | is-empty) {
        let error_msg = "Managed sidebar Yazi is not available in the current tab. Open the sidebar and try again."
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
        return
    }

    log_to_file "reveal_in_yazi.log" $"Managed sidebar Yazi ID found: '($sidebar_yazi_id)'"

    try {
        # Use 'reveal' command instead of 'cd' to both navigate to directory and select the file
        run_ya_emit_to $sidebar_yazi_id "reveal" $full_path
        log_to_file "reveal_in_yazi.log" $"Successfully sent 'reveal ($full_path)' command to managed sidebar yazi instance ($sidebar_yazi_id)"

        let focus_response = (run_zellij_pipe "focus_sidebar")
        let focus_ok = (match ($focus_response | str trim) {
            "ok" | "focused" | "focused_sidebar" | "opened_sidebar" => true
            _ => false
        })
        if $focus_ok {
            log_to_file "reveal_in_yazi.log" "Successfully focused managed sidebar pane"
        } else {
            let error_msg = $"Managed sidebar pane focus failed \(status=($focus_response)\). Ensure the Yazelix pane orchestrator plugin is loaded and the sidebar pane title is 'sidebar'."
            log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
            print $"Error: ($error_msg)"
        }
    } catch {|err|
        let error_msg = $"Failed to execute yazi/zellij commands: ($err.msg)"
        log_to_file "reveal_in_yazi.log" $"ERROR: ($error_msg)"
        print $"Error: ($error_msg)"
    }
}
