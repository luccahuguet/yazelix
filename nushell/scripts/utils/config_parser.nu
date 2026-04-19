#!/usr/bin/env nu
# Configuration parser for yazelix TOML files

use config_diagnostics.nu render_startup_config_error
use failure_classes.nu [format_failure_classification]
use config_surfaces.nu load_active_config_surface
use common.nu [require_yazelix_runtime_dir]

const YZX_CORE_HELPER_RELATIVE_PATH = ["libexec" "yzx_core"]

def get_runtime_yzx_core_helper_path [runtime_dir: string] {
    $YZX_CORE_HELPER_RELATIVE_PATH | prepend $runtime_dir | path join
}

def get_yzx_core_contract_path [runtime_dir: string] {
    $runtime_dir | path join "config_metadata" "main_config_contract.toml"
}

def get_explicit_yzx_core_helper_path [] {
    let explicit = (
        $env.YAZELIX_YZX_CORE_BIN?
        | default ""
        | into string
        | str trim
    )

    if ($explicit | is-empty) {
        return null
    }

    let expanded = ($explicit | path expand)
    if not ($expanded | path exists) {
        error make {
            msg: (
                [
                    $"YAZELIX_YZX_CORE_BIN points to a missing helper: ($expanded)"
                    ""
                    (format_failure_classification "host-dependency" "Enter the Yazelix maintainer shell, rebuild the local yzx_core helper, or unset YAZELIX_YZX_CORE_BIN.")
                ] | str join "\n"
            )
        }
    }

    $expanded
}

def get_source_checkout_yzx_core_helper_path [runtime_dir: string] {
    for candidate in [
        ($runtime_dir | path join "rust_core" "target" "release" "yzx_core")
        ($runtime_dir | path join "rust_core" "target" "debug" "yzx_core")
    ] {
        if ($candidate | path exists) {
            return $candidate
        }
    }

    null
}

export def resolve_yzx_core_helper_path [runtime_dir: string] {
    let runtime_helper_path = (get_runtime_yzx_core_helper_path $runtime_dir)
    if ($runtime_helper_path | path exists) {
        return $runtime_helper_path
    }

    let explicit_helper_path = (get_explicit_yzx_core_helper_path)
    if $explicit_helper_path != null {
        return $explicit_helper_path
    }

    let source_helper_path = (get_source_checkout_yzx_core_helper_path $runtime_dir)
    if $source_helper_path != null {
        return $source_helper_path
    }

    error make {
        msg: (
            [
                $"Yazelix runtime is missing the Rust config helper at ($runtime_helper_path)."
                ""
                (format_failure_classification "host-dependency" "Reinstall Yazelix so packaged runtimes include libexec/yzx_core. For source checkouts, enter the maintainer shell or set YAZELIX_YZX_CORE_BIN to a built yzx_core helper.")
            ] | str join "\n"
        )
    }
}

export def render_yzx_core_error [config_surface: record, stderr: string] {
    let trimmed_stderr = ($stderr | default "" | str trim)
    let envelope = (
        try {
            $trimmed_stderr | from json
        } catch {
            null
        }
    )

    if $envelope == null {
        return (
            [
                "Yazelix Rust config normalization failed before it could report a structured error."
                $"Raw helper stderr: ($trimmed_stderr)"
                ""
                (format_failure_classification "host-dependency" "Reinstall Yazelix so the runtime includes a working yzx_core helper, then retry.")
            ] | str join "\n"
        )
    }

    let error = ($envelope.error? | default {})
    let error_class = ($error.class? | default "runtime")
    let code = ($error.code? | default "unknown")
    let message = ($error.message? | default "Yazelix Rust config normalization failed.")
    let remediation = ($error.remediation? | default "Fix the reported Yazelix config issue and retry.")
    let details = ($error.details? | default {})

    if ($error_class == "config") and ($code == "unsupported_config") and (($details | describe) | str contains "record") {
        render_startup_config_error ($details | upsert config_path $config_surface.display_config_path)
    } else {
        let failure_class = if $error_class == "config" { "config" } else { "host-dependency" }
        [
            $message
            $"Helper code: ($code)"
            ""
            (format_failure_classification $failure_class $remediation)
        ] | str join "\n"
    }
}

def execute_yzx_core_command [runtime_dir: string, helper_args] {
    let helper_path = resolve_yzx_core_helper_path $runtime_dir
    let command_name = ($helper_args | get -o 0 | default "unknown")

    try {
        do { ^$helper_path ...$helper_args } | complete
    } catch {|err|
        error make {
            msg: (
                [
                    $"Could not execute Yazelix Rust helper command `($command_name)` at ($helper_path)."
                    $err.msg
                    ""
                    (format_failure_classification "host-dependency" "Reinstall Yazelix so the runtime includes an executable yzx_core helper, then retry.")
                ] | str join "\n"
            )
        }
    }
}

export def run_yzx_core_json_command_with_error_surface [
    runtime_dir: string
    error_surface: record
    helper_args
    invalid_json_message: string
] {
    let result = (execute_yzx_core_command $runtime_dir $helper_args)
    if $result.exit_code != 0 {
        error make {msg: (render_yzx_core_error $error_surface $result.stderr)}
    }

    let envelope = (
        try {
            $result.stdout | from json
        } catch {|err|
            error make {
                msg: (
                    [
                        $invalid_json_message
                        $err.msg
                        ""
                        (format_failure_classification "host-dependency" "Reinstall Yazelix so the runtime includes a compatible yzx_core helper, then retry.")
                    ] | str join "\n"
                )
            }
        }
    )

    let status = ($envelope | get -o status | default "")
    if $status != "ok" {
        error make {msg: (render_yzx_core_error $error_surface ($result.stdout | default ""))}
    }

    $envelope | get data
}

export def run_yzx_core_json_command [
    runtime_dir: string
    config_surface: record
    helper_args
    invalid_json_message: string
] {
    run_yzx_core_json_command_with_error_surface $runtime_dir $config_surface $helper_args $invalid_json_message
}

export def run_yzx_core_command_with_error_surface [
    runtime_dir: string
    error_surface: record
    helper_args
] {
    let result = (execute_yzx_core_command $runtime_dir $helper_args)

    if $result.exit_code != 0 {
        error make {msg: (render_yzx_core_error $error_surface $result.stderr)}
    }

    $result
}

export def run_yzx_core_command [
    runtime_dir: string
    config_surface: record
    helper_args
] {
    run_yzx_core_command_with_error_surface $runtime_dir $config_surface $helper_args
}

def parse_yazelix_config_with_rust [
    runtime_dir: string
    config_surface: record
] {
    run_yzx_core_json_command $runtime_dir $config_surface [
        "config.normalize"
        "--config"
        $config_surface.config_file
        "--default-config"
        $config_surface.default_config_path
        "--contract"
        (get_yzx_core_contract_path $runtime_dir)
    ] "Yazelix Rust config helper returned invalid JSON."
    | get normalized_config
}

# Parse yazelix configuration file and extract settings
export def parse_yazelix_config [] {
    let config_surface = load_active_config_surface
    let runtime_dir = require_yazelix_runtime_dir
    parse_yazelix_config_with_rust $runtime_dir $config_surface
}
