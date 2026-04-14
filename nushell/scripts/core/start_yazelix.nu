#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu

use ../utils/config_parser.nu parse_yazelix_config
use ../utils/common.nu [require_yazelix_runtime_dir resolve_yazelix_nu_bin]
use ../utils/runtime_env.nu get_runtime_env
use ../utils/runtime_contract_checker.nu [
    check_runtime_script
    check_startup_working_dir
    require_runtime_check
]

def validate_startup_working_dir [working_dir: string] {
    let check = (check_startup_working_dir $working_dir)
    require_runtime_check $check | ignore
    $check.path
}

def require_runtime_script [script_path: string, label: string] {
    let check = (check_runtime_script $script_path "startup_runtime_script" $label "startup")
    require_runtime_check $check | ignore
    $check.path
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

    let config = parse_yazelix_config
    let runtime_env = (get_runtime_env $config)

    if $verbose_mode {
        print "🔍 Startup config parsed"
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
    let working_dir = (validate_startup_working_dir $requested_working_dir)
    let inner_script = (require_runtime_script $"($yazelix_dir)/nushell/scripts/core/start_yazelix_inner.nu" "startup script")
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

export def start_yazelix_session [cwd_override?: string, --verbose, --setup-only] {
    if ($cwd_override | is-not-empty) {
        _start_yazelix_impl $cwd_override --verbose=$verbose --setup-only=$setup_only
    } else {
        _start_yazelix_impl --verbose=$verbose --setup-only=$setup_only
    }
}

export def main [cwd_override?: string, --verbose, --setup-only] {
    if ($cwd_override | is-not-empty) {
        start_yazelix_session $cwd_override --verbose=$verbose --setup-only=$setup_only
    } else {
        start_yazelix_session --verbose=$verbose --setup-only=$setup_only
    }
}
