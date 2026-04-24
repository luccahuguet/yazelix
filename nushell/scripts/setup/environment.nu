#!/usr/bin/env nu
# Main Yazelix environment setup script
# Shared by startup, installer, and maintainer-shell entrypoints

use ../utils/runtime_paths.nu [get_yazelix_runtime_dir get_yazelix_state_dir]
use ../utils/runtime_defaults.nu DEFAULT_SHELL
use ../utils/yzx_core_bridge.nu [profile_startup_step resolve_yzx_control_path]
use ../utils/yzx_core_bridge.nu [build_default_yzx_core_error_surface run_yzx_core_json_command]

const YZX_COMMAND_METADATA_SYNC_EXTERNS_COMMAND = "yzx-command-metadata.sync-externs"

def ensure_runtime_scripts_executable [yazelix_dir: string] {
    let runtime_root = ($yazelix_dir | path expand)
    if ($runtime_root | str starts-with "/nix/store/") {
        return
    }

    chmod +x $"($runtime_root)/shells/bash/start_yazelix.sh"
    chmod +x $"($runtime_root)/shells/posix/detached_launch_probe.sh"
    chmod +x $"($runtime_root)/shells/posix/start_yazelix.sh"
    chmod +x $"($runtime_root)/shells/posix/yazelix_hx.sh"
    chmod +x $"($runtime_root)/shells/posix/yzx_cli.sh"
}

def sync_generated_yzx_extern_bridge [runtime_root: string] {
    try {
        run_yzx_core_json_command $runtime_root (build_default_yzx_core_error_surface) [
            $YZX_COMMAND_METADATA_SYNC_EXTERNS_COMMAND
            "--runtime-dir"
            ($runtime_root | path expand)
            "--state-dir"
            (get_yazelix_state_dir)
        ] "Yazelix Rust yzx command metadata extern sync returned invalid JSON." | ignore
    } catch {|err|
        print $"⚠️  Failed to generate Nushell yzx extern bridge: ($err.msg)"
    }
}

def main [--skip-welcome] {
    let yazelix_dir = (get_yazelix_runtime_dir)
    let startup_facts = (run_yzx_core_json_command
        $yazelix_dir
        (build_default_yzx_core_error_surface)
        ["startup-facts.compute"]
        "Yazelix Rust startup-facts helper returned invalid JSON.")
    let default_shell = ($startup_facts.default_shell? | default $DEFAULT_SHELL)

    # Noninteractive shellHook entry should stay quiet even when only the
    # welcome UI is skipped, so launch/refresh rebuilds don't replay routine
    # setup chatter in the caller terminal.
    let quiet_mode = (
        ($env.YAZELIX_ENV_ONLY? == "true")
        or $skip_welcome
        or ($env.YAZELIX_SHELLHOOK_SKIP_WELCOME? == "true")
    )
    let shellhook_phase = (
        $env.YAZELIX_STARTUP_PROFILE_PHASE?
        | default "shell_entry"
        | into string
        | str trim
    )
    let shellhook_pid = ($nu.pid | into string)

    def profile_shellhook_step [step: string, code: closure, metadata?: record] {
        profile_startup_step "shellhook" $step $code (
            ($metadata | default {})
            | upsert phase $shellhook_phase
            | upsert pid $shellhook_pid
        )
    }

    # Keep shell entry narrow: always configure the runtime baseline plus the selected default shell.
    let shells_to_configure = ([$DEFAULT_SHELL, "bash", $default_shell] | uniq)

    # Setup logging in state directory (XDG-compliant)
    let state_dir = ($env.YAZELIX_STATE_DIR | str replace "~" $env.HOME)
    let log_dir = ($env.YAZELIX_LOGS_DIR | str replace "~" $env.HOME)
    mkdir $state_dir
    mkdir $log_dir

    # Auto-trim old logs (keep 10 most recent)
    let old_shellhook_logs = try {
        ls $"($log_dir)/shellhook_*.log"
        | sort-by modified -r
        | skip 10
        | get name
    } catch { [] }

    let old_welcome_logs = try {
        ls $"($log_dir)/welcome_*.log"
        | sort-by modified -r
        | skip 10
        | get name
    } catch { [] }

    let all_old_logs = ($old_shellhook_logs | append $old_welcome_logs)

    if not ($all_old_logs | is-empty) {
        rm ...$all_old_logs
    }

    let log_file = $"($log_dir)/shellhook_(date now | format date '%Y%m%d_%H%M%S').log"

    if not $quiet_mode {
        print $"📝 Logging to: ($log_file)"
    }

    # Generate shell initializers for configured shells only
    profile_shellhook_step "generate_initializers" {
        let yzx_control_bin = (resolve_yzx_control_path $yazelix_dir)
        with-env {YAZELIX_QUIET_MODE: (if $quiet_mode { "true" } else { "false" })} {
            ^$yzx_control_bin generate_shell_initializers ($shells_to_configure | str join ",")
        }
    } {
        shells: $shells_to_configure
    }
    profile_shellhook_step "sync_yzx_extern_bridge" {
        sync_generated_yzx_extern_bridge $yazelix_dir
    }
    # Editor setup is now handled in the shellHook

    profile_shellhook_step "ensure_runtime_scripts_executable" {
        ensure_runtime_scripts_executable $yazelix_dir
    }

    let zjstatus_target = $"($yazelix_dir)/configs/zellij/plugins/zjstatus.wasm"
    if not ($zjstatus_target | path exists) {
        print $"❌ Error: Vendored zjstatus wasm not found at: ($zjstatus_target)"
        exit 1
    }

    if not $quiet_mode {
        print "✅ Yazelix environment setup complete!"
    }
}
