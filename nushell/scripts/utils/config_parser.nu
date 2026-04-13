#!/usr/bin/env nu
# Configuration parser for yazelix TOML files

use config_contract.nu [load_main_config_contract]
use config_diagnostics.nu [build_config_diagnostic_report_from_records render_startup_config_error]
use failure_classes.nu [format_failure_classification]
use config_surfaces.nu [load_active_config_surface load_config_surface_from_main]

def bool_to_string [value: bool] {
    if $value { "true" } else { "false" }
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

# Parse yazelix configuration file and extract settings
export def parse_yazelix_config [] {
    let config_surface = load_active_config_surface
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
