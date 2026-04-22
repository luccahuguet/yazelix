#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu

use ../utils/common.nu [require_yazelix_runtime_dir resolve_yazelix_nu_bin]
use ../utils/yzx_core_bridge.nu [build_default_yzx_core_error_surface compute_runtime_env_via_yzx_core run_yzx_core_request_json_command]

const STARTUP_LAUNCH_PREFLIGHT_EVALUATE_COMMAND = "startup-launch-preflight.evaluate"

def run_startup_preflight [working_dir: string, runtime_dir: string] {
    let data = (run_yzx_core_request_json_command
        $runtime_dir
        (build_default_yzx_core_error_surface)
        $STARTUP_LAUNCH_PREFLIGHT_EVALUATE_COMMAND
        {
            startup: {
                working_dir: ($working_dir | path expand)
                runtime_script: {
                    id: "startup_runtime_script"
                    label: "startup script"
                    owner_surface: "startup"
                    path: ($runtime_dir | path join "nushell" "scripts" "core" "start_yazelix_inner.nu" | path expand)
                }
            }
        }
        "Yazelix Rust startup-launch-preflight helper returned invalid JSON.")

    if ($data.kind? | default "") != "startup" {
        error make {msg: "Unexpected startup-launch-preflight response \(expected startup\)."}
    }

    $data
}

def validate_startup_working_dir [working_dir: string] {
    let runtime_dir = (require_yazelix_runtime_dir)
    (run_startup_preflight $working_dir $runtime_dir).working_dir
}

def run_runtime_setup [runtime_dir: string, nu_bin: string, --quiet] {
    if $quiet {
        ^$nu_bin $"($runtime_dir)/nushell/scripts/setup/environment.nu" --welcome-source start --skip-welcome
    } else {
        ^$nu_bin $"($runtime_dir)/nushell/scripts/setup/environment.nu" --welcome-source start
    }
}

def _start_yazelix_impl [cwd_override?: string, --verbose, --setup-only] {
    let original_dir = pwd
    let verbose_mode = $verbose
    if $verbose_mode {
        print "🔍 start_yazelix: verbose mode enabled"
    }

    let yazelix_dir = try {
        require_yazelix_runtime_dir
    } catch {|err|
        print $"Error: ($err.msg)"
        exit 1
    }
    let nu_bin = (resolve_yazelix_nu_bin)

    let runtime_env = (compute_runtime_env_via_yzx_core)

    if $verbose_mode {
        print "🔍 Startup runtime env computed"
    }

    if $setup_only {
        print "🔧 Setting up Yazelix generated environment files..."
        with-env $runtime_env {
            run_runtime_setup $yazelix_dir $nu_bin --quiet=false
        }
        print "✅ Setup complete."
        return
    }

    let requested_working_dir = if ($cwd_override | is-not-empty) {
        $cwd_override
    } else {
        $original_dir
    }
    let preflight = (run_startup_preflight $requested_working_dir $yazelix_dir)
    let working_dir = $preflight.working_dir
    let inner_script = ($preflight.script_path? | default "")
    let base_args = ["-i", $inner_script, $working_dir]
    let inner_args = if $verbose_mode {
        $base_args | append "--verbose"
    } else {
        $base_args
    }

    with-env $runtime_env {
        run_runtime_setup $yazelix_dir $nu_bin --quiet=true
        ^$nu_bin ...$inner_args
    }
}

def run_yzx_enter_command [path: string, home: bool, verbose: bool] {
    let verbose_mode = $verbose
    if $verbose_mode {
        print "🔍 yzx enter: verbose mode enabled"
    }

    $env.YAZELIX_ENV_ONLY = "false"

    let cwd_override = if $home {
        $env.HOME
    } else if ($path | is-not-empty) {
        $path
    } else {
        null
    }

    if ($cwd_override != null) {
        start_yazelix_session $cwd_override --verbose=$verbose_mode
    } else {
        start_yazelix_session --verbose=$verbose_mode
    }
}

export def start_yazelix_session [cwd_override?: string, --verbose, --setup-only] {
    if ($cwd_override | is-not-empty) {
        _start_yazelix_impl $cwd_override --verbose=$verbose --setup-only=$setup_only
    } else {
        _start_yazelix_impl --verbose=$verbose --setup-only=$setup_only
    }
}

export def "yzx enter" [
    --path(-p): string = "" # Start in specific directory
    --home             # Start in home directory
    --verbose          # Enable verbose logging
] {
    run_yzx_enter_command $path $home $verbose
}

export def main [cwd_override?: string, --verbose, --setup-only] {
    if ($cwd_override | is-not-empty) {
        start_yazelix_session $cwd_override --verbose=$verbose --setup-only=$setup_only
    } else {
        start_yazelix_session --verbose=$verbose --setup-only=$setup_only
    }
}
