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

def resolve_yzx_core_helper_path [runtime_dir: string] {
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

def render_yzx_core_error [config_surface: record, stderr: string] {
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

def parse_yazelix_config_with_rust [
    config_surface: record
    default_config_path: string
    contract_path: string
    helper_path: string
] {
    let config_path = $config_surface.config_file
    let result = (
        try {
            do { ^$helper_path config.normalize --config $config_path --default-config $default_config_path --contract $contract_path } | complete
        } catch {|err|
            error make {
                msg: (
                    [
                        $"Could not execute Yazelix Rust config helper at ($helper_path)."
                        $err.msg
                        ""
                        (format_failure_classification "host-dependency" "Reinstall Yazelix so the runtime includes an executable yzx_core helper, then retry.")
                    ] | str join "\n"
                )
            }
        }
    )

    if $result.exit_code != 0 {
        error make {msg: (render_yzx_core_error $config_surface $result.stderr)}
    }

    let envelope = (
        try {
            $result.stdout | from json
        } catch {|err|
            error make {
                msg: (
                    [
                        "Yazelix Rust config helper returned invalid JSON."
                        $err.msg
                        ""
                        (format_failure_classification "host-dependency" "Reinstall Yazelix so the runtime includes a compatible yzx_core helper, then retry.")
                    ] | str join "\n"
                )
            }
        }
    )

    if (($envelope.status? | default "") != "ok") {
        error make {msg: (render_yzx_core_error $config_surface ($result.stdout | default ""))}
    }

    $envelope.data.normalized_config
}

# Parse yazelix configuration file and extract settings
export def parse_yazelix_config [] {
    let config_surface = load_active_config_surface
    let runtime_dir = require_yazelix_runtime_dir
    let helper_path = resolve_yzx_core_helper_path $runtime_dir
    let contract_path = get_yzx_core_contract_path $runtime_dir
    parse_yazelix_config_with_rust $config_surface $config_surface.default_config_path $contract_path $helper_path
}
