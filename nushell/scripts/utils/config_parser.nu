#!/usr/bin/env nu
# Configuration parser for yazelix TOML files

use config_contract.nu [load_main_config_contract]
use config_diagnostics.nu [build_config_diagnostic_report_from_records render_startup_config_error]
use failure_classes.nu [format_failure_classification]
use config_surfaces.nu [load_active_config_surface load_config_surface_from_main]
use common.nu [require_yazelix_runtime_dir]

const YZX_CORE_HELPER_RELATIVE_PATH = ["libexec" "yzx_core"]

def bool_to_string [value: bool] {
    if $value { "true" } else { "false" }
}

def runtime_allows_nushell_config_parser_fallback [runtime_dir: string] {
    let has_git_dir = (($runtime_dir | path join ".git") | path exists)
    let has_rust_workspace = (($runtime_dir | path join "rust_core" "Cargo.toml") | path exists)
    $has_git_dir or $has_rust_workspace
}

def get_yzx_core_helper_path [runtime_dir: string] {
    $YZX_CORE_HELPER_RELATIVE_PATH | prepend $runtime_dir | path join
}

def get_yzx_core_contract_path [runtime_dir: string] {
    $runtime_dir | path join "config_metadata" "main_config_contract.toml"
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

def get_contract_field [contract: record, field_path: string] {
    let field = (
        $contract.fields
        | transpose key value
        | where key == $field_path
        | get -o value.0
        | default null
    )
    if $field == null {
        error make {msg: $"Unknown config contract field: ($field_path)"}
    }
    $field
}

def get_nested_config_value [raw_config: record, field_path: string] {
    mut current = $raw_config
    for segment in ($field_path | split row ".") {
        $current = ($current | get -o $segment)
        if $current == null {
            return null
        }
    }
    $current
}

def get_contract_value_or_default [contract: record, raw_config: record, field_path: string] {
    let raw_value = (get_nested_config_value $raw_config $field_path)
    if $raw_value == null {
        (get_contract_field $contract $field_path).default? | default null
    } else {
        $raw_value
    }
}

def make_contract_value_error [field_path: string, actual_value: string, expectation: string, remediation: string] {
    let classification = (format_failure_classification "config" $remediation)
    error make {msg: $"Invalid ($field_path) value '($actual_value)'. Expected ($expectation).\n($classification)"}
}

def parse_contract_enum_field [contract: record, raw_config: record, field_path: string, remediation: string] {
    let field = (get_contract_field $contract $field_path)
    let allowed = ($field.allowed_values? | default [])
    let normalized = (get_contract_value_or_default $contract $raw_config $field_path | into string | str downcase)

    if not ($normalized in $allowed) {
        let allowed_text = ($allowed | str join ", ")
        make_contract_value_error $field_path $normalized $"one of: ($allowed_text)" $remediation
    }

    $normalized
}

def parse_contract_badge_text [contract: record, raw_config: record, field_path: string] {
    let raw_text = (get_contract_value_or_default $contract $raw_config $field_path | default "" | into string)
    let compact = (
        $raw_text
        | str replace -ar '\s+' ' '
        | str trim
        | str replace -ar '[\[\]\{\}"\\]' ''
    )

    if ($compact | is-empty) {
        return ""
    }

    if (($compact | str length) > 8) {
        $compact | str substring 0..7
    } else {
        $compact
    }
}

def parse_contract_symbol_or_positive_int_string [contract: record, raw_config: record, field_path: string, remediation: string] {
    let field = (get_contract_field $contract $field_path)
    let allowed_symbols = ($field.allowed_symbols? | default [])
    let normalized = (get_contract_value_or_default $contract $raw_config $field_path | into string | str downcase)

    if $normalized in $allowed_symbols {
        return $normalized
    }

    let parsed = (try { $normalized | into int } catch { null })
    if $parsed == null {
        let allowed_text = ($allowed_symbols | str join ", ")
        make_contract_value_error $field_path $normalized $"one of: ($allowed_text), or a positive integer" $remediation
    }

    if $parsed < 1 {
        make_contract_value_error $field_path $normalized "a positive integer" $remediation
    }

    $normalized
}

def parse_contract_int_range_field [contract: record, raw_config: record, field_path: string, remediation: string] {
    let field = (get_contract_field $contract $field_path)
    let min = ($field.min? | default 0)
    let max = ($field.max? | default 0)
    let normalized = (get_contract_value_or_default $contract $raw_config $field_path | into string | str trim)
    let parsed = (try { $normalized | into int } catch { null })

    if $parsed == null {
        make_contract_value_error $field_path $normalized $"an integer from ($min) to ($max)" $remediation
    }

    if ($parsed < $min) or ($parsed > $max) {
        make_contract_value_error $field_path $normalized $"an integer from ($min) to ($max)" $remediation
    }

    $parsed
}

def parse_contract_float_range_field [contract: record, raw_config: record, field_path: string, remediation: string] {
    let field = (get_contract_field $contract $field_path)
    let min = ($field.min? | default 0.0)
    let max = ($field.max? | default 0.0)
    let raw_value = (get_contract_value_or_default $contract $raw_config $field_path)
    let parsed = (try { $raw_value | into float } catch { null })

    if $parsed == null {
        make_contract_value_error $field_path ($raw_value | into string) $"a number from ($min) to ($max)" $remediation
    }

    if ($parsed < $min) or ($parsed > $max) {
        make_contract_value_error $field_path ($raw_value | into string) $"a number from ($min) to ($max)" $remediation
    }

    $parsed
}

def parse_contract_enum_string_list_field [contract: record, raw_config: record, field_path: string, remediation: string] {
    let field = (get_contract_field $contract $field_path)
    let allowed = ($field.allowed_values? | default [])
    let allowed_text = ($allowed | str join ", ")
    let raw_value = (get_contract_value_or_default $contract $raw_config $field_path)
    let described = ($raw_value | describe)

    if not ($described | str contains "list") {
        make_contract_value_error $field_path ($raw_value | into string) $"a list with values from: ($allowed_text)" $remediation
    }

    for value in $raw_value {
        let normalized = ($value | into string)
        if not ($normalized in $allowed) {
            make_contract_value_error $field_path $normalized $"a list with values from: ($allowed_text)" $remediation
        }
    }

    $raw_value
}

def parse_contract_nullable_string_field [contract: record, raw_config: record, field_path: string] {
    let value = (get_contract_value_or_default $contract $raw_config $field_path | default "" | into string)
    if ($value | is-empty) { null } else { $value }
}

def parse_contract_bool_to_string_field [contract: record, raw_config: record, field_path: string] {
    bool_to_string (get_contract_value_or_default $contract $raw_config $field_path)
}

def parse_contract_direct_field [contract: record, raw_config: record, field_path: string] {
    get_contract_value_or_default $contract $raw_config $field_path
}

def upsert_parsed_field [parsed: record, contract: record, field_path: string, value: any] {
    let parser_key = ((get_contract_field $contract $field_path).parser_key? | default $field_path)
    $parsed | upsert $parser_key $value
}

def get_contract_recovery_hint [field_path: string] {
    match $field_path {
        "terminal.config_mode" => "Use `terminal.config_mode = \"yazelix\"` for the supported managed path, or `\"user\"` only when you want Yazelix to load the terminal's native config file."
        _ => "Update yazelix.toml with a supported value, or run `yzx config reset` to restore the template."
    }
}

def parse_contract_field_value [contract: record, raw_config: record, field_path: string] {
    let field = (get_contract_field $contract $field_path)
    let validation = ($field.validation? | default "")
    let behavior = ($field.parser_behavior? | default "direct")
    let remediation = (get_contract_recovery_hint $field_path)

    match $behavior {
        "compact_badge_text" => (parse_contract_badge_text $contract $raw_config $field_path)
        "empty_string_to_null" => (parse_contract_nullable_string_field $contract $raw_config $field_path)
        "bool_to_string" => (parse_contract_bool_to_string_field $contract $raw_config $field_path)
        _ => {
            match $validation {
                "enum" => (parse_contract_enum_field $contract $raw_config $field_path $remediation)
                "enum_string_list" => (parse_contract_enum_string_list_field $contract $raw_config $field_path $remediation)
                "float_range" => (parse_contract_float_range_field $contract $raw_config $field_path $remediation)
                "int_range" => (parse_contract_int_range_field $contract $raw_config $field_path $remediation)
                "symbol_or_positive_int_string" => (parse_contract_symbol_or_positive_int_string $contract $raw_config $field_path $remediation)
                _ => (parse_contract_direct_field $contract $raw_config $field_path)
            }
        }
    }
}

def parse_yazelix_config_with_nushell_fallback [config_surface: record] {
    let config_to_read = $config_surface.config_file
    let raw_config = $config_surface.merged_config
    let default_config_path = $config_surface.default_config_path
    let contract = (load_main_config_contract)

    if ($config_to_read | path basename) == "yazelix.toml" and ($default_config_path | path exists) {
        let default_surface = (load_config_surface_from_main $default_config_path)
        let diagnostic_report = (
            build_config_diagnostic_report_from_records
                $raw_config
                $default_surface.merged_config
                $config_to_read
            | upsert config_path $config_surface.display_config_path
        )
        if $diagnostic_report.has_blocking {
            error make {msg: (render_startup_config_error $diagnostic_report)}
        }
    }

    mut parsed = {}

    for field_path in ($contract.fields | columns | sort) {
        $parsed = (
            upsert_parsed_field
            $parsed
            $contract
            $field_path
            (parse_contract_field_value $contract $raw_config $field_path)
        )
    }

    $parsed | upsert config_file $config_to_read
}

# Parse yazelix configuration file and extract settings
export def parse_yazelix_config [] {
    let config_surface = load_active_config_surface
    let runtime_dir = require_yazelix_runtime_dir
    let helper_path = get_yzx_core_helper_path $runtime_dir
    let contract_path = get_yzx_core_contract_path $runtime_dir

    if ($helper_path | path exists) {
        return (parse_yazelix_config_with_rust $config_surface $config_surface.default_config_path $contract_path $helper_path)
    }

    if (runtime_allows_nushell_config_parser_fallback $runtime_dir) {
        return (parse_yazelix_config_with_nushell_fallback $config_surface)
    }

    error make {
        msg: (
            [
                $"Yazelix runtime is missing the Rust config helper at ($helper_path)."
                ""
                (format_failure_classification "host-dependency" "Reinstall Yazelix so the packaged runtime includes libexec/yzx_core. Source checkouts may keep using the Nushell fallback.")
            ] | str join "\n"
        )
    }
}
