#!/usr/bin/env nu

use runtime_paths.nu require_yazelix_runtime_dir
use yzx_core_bridge.nu [build_default_yzx_core_error_surface run_yzx_core_json_command]

const UPGRADE_SUMMARY_HEADLINE_COMMAND = "upgrade-summary.headline"
const UPGRADE_SUMMARY_FIRST_RUN_COMMAND = "upgrade-summary.first-run"

def yzx_cli_path [runtime_dir: string] {
    $runtime_dir | path join "shells" "posix" "yzx_cli.sh"
}

export def get_current_release_headline [] {
    let runtime_dir = (require_yazelix_runtime_dir)
    let data = (run_yzx_core_json_command
        $runtime_dir
        (build_default_yzx_core_error_surface)
        [$UPGRADE_SUMMARY_HEADLINE_COMMAND]
        "Yazelix Rust upgrade-summary headline helper returned invalid JSON.")
    ($data.headline? | default "" | into string | str trim)
}

export def maybe_show_first_run_upgrade_summary [] {
    let runtime_dir = (require_yazelix_runtime_dir)
    (run_yzx_core_json_command
        $runtime_dir
        (build_default_yzx_core_error_surface)
        [$UPGRADE_SUMMARY_FIRST_RUN_COMMAND]
        "Yazelix Rust first-run upgrade-summary helper returned invalid JSON.")
}

export def play_welcome_style_runtime [
    welcome_style: string
    welcome_duration_seconds: float
] {
    let runtime_dir = (require_yazelix_runtime_dir)
    let yzx_cli = (yzx_cli_path $runtime_dir)
    let duration_ms = (($welcome_duration_seconds * 1000.0) | math round | into int)
    do {
        ^sh $yzx_cli screen --internal-welcome --duration-ms ($duration_ms | into string) $welcome_style
    }
    let exit_code = ($env.LAST_EXIT_CODE? | default 0)
    if $exit_code != 0 {
        error make {msg: $"Rust-owned welcome renderer failed for style `($welcome_style)` with exit code ($exit_code)."}
    }
}
