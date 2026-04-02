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
    let field = ($contract.fields | get -o $field_path)
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
                $config_surface.main_config
                $config_surface.pack_config
            | upsert config_path $config_surface.display_config_path
        )
        if $diagnostic_report.has_blocking {
            error make {msg: (render_startup_config_error $diagnostic_report)}
        }
    }

    let generic_recovery = "Update yazelix.toml with a supported value, or run `yzx config reset` to restore the template."
    mut parsed = {}

    $parsed = (upsert_parsed_field $parsed $contract "core.recommended_deps" (parse_contract_direct_field $contract $raw_config "core.recommended_deps"))
    $parsed = (upsert_parsed_field $parsed $contract "core.yazi_extensions" (parse_contract_direct_field $contract $raw_config "core.yazi_extensions"))
    $parsed = (upsert_parsed_field $parsed $contract "core.yazi_media" (parse_contract_direct_field $contract $raw_config "core.yazi_media"))
    $parsed = (upsert_parsed_field $parsed $contract "core.debug_mode" (parse_contract_direct_field $contract $raw_config "core.debug_mode"))
    $parsed = (upsert_parsed_field $parsed $contract "core.skip_welcome_screen" (parse_contract_direct_field $contract $raw_config "core.skip_welcome_screen"))
    $parsed = (upsert_parsed_field $parsed $contract "core.show_macchina_on_welcome" (parse_contract_direct_field $contract $raw_config "core.show_macchina_on_welcome"))
    $parsed = (upsert_parsed_field $parsed $contract "core.welcome_style" (parse_contract_enum_field $contract $raw_config "core.welcome_style" "Update core.welcome_style with one of the supported values, or run `yzx config reset` to restore the template."))
    $parsed = (upsert_parsed_field $parsed $contract "core.welcome_duration_seconds" (parse_contract_float_range_field $contract $raw_config "core.welcome_duration_seconds" "Update core.welcome_duration_seconds to a number from 0.2 to 8.0, or run `yzx config reset` to restore the template."))
    $parsed = (upsert_parsed_field $parsed $contract "core.refresh_output" (parse_contract_enum_field $contract $raw_config "core.refresh_output" $generic_recovery))
    $parsed = (upsert_parsed_field $parsed $contract "core.max_jobs" (parse_contract_symbol_or_positive_int_string $contract $raw_config "core.max_jobs" $generic_recovery))
    $parsed = (upsert_parsed_field $parsed $contract "core.build_cores" (parse_contract_symbol_or_positive_int_string $contract $raw_config "core.build_cores" $generic_recovery))

    $parsed = (upsert_parsed_field $parsed $contract "helix.mode" (parse_contract_enum_field $contract $raw_config "helix.mode" $generic_recovery))
    $parsed = (upsert_parsed_field $parsed $contract "helix.runtime_path" (parse_contract_nullable_string_field $contract $raw_config "helix.runtime_path"))

    $parsed = (upsert_parsed_field $parsed $contract "editor.command" (parse_contract_nullable_string_field $contract $raw_config "editor.command"))
    $parsed = (upsert_parsed_field $parsed $contract "editor.enable_sidebar" (parse_contract_direct_field $contract $raw_config "editor.enable_sidebar"))
    $parsed = (upsert_parsed_field $parsed $contract "editor.sidebar_width_percent" (parse_contract_int_range_field $contract $raw_config "editor.sidebar_width_percent" "Update editor.sidebar_width_percent to an integer from 10 to 40, or run `yzx config reset` to restore the template."))

    $parsed = (upsert_parsed_field $parsed $contract "shell.default_shell" (parse_contract_direct_field $contract $raw_config "shell.default_shell"))
    $parsed = (upsert_parsed_field $parsed $contract "shell.extra_shells" (parse_contract_direct_field $contract $raw_config "shell.extra_shells"))

    $parsed = (upsert_parsed_field $parsed $contract "terminal.terminals" (parse_contract_direct_field $contract $raw_config "terminal.terminals"))
    $parsed = (upsert_parsed_field $parsed $contract "terminal.manage_terminals" (parse_contract_direct_field $contract $raw_config "terminal.manage_terminals"))
    $parsed = (upsert_parsed_field $parsed $contract "terminal.config_mode" (parse_contract_enum_field $contract $raw_config "terminal.config_mode" "Use `terminal.config_mode = \"yazelix\"` for the supported managed path, or `\"user\"` only when you want Yazelix to load the terminal's native config file."))
    $parsed = (upsert_parsed_field $parsed $contract "terminal.ghostty_trail_color" (parse_contract_direct_field $contract $raw_config "terminal.ghostty_trail_color"))
    $parsed = (upsert_parsed_field $parsed $contract "terminal.ghostty_trail_effect" (parse_contract_direct_field $contract $raw_config "terminal.ghostty_trail_effect"))
    $parsed = (upsert_parsed_field $parsed $contract "terminal.ghostty_mode_effect" (parse_contract_direct_field $contract $raw_config "terminal.ghostty_mode_effect"))
    $parsed = (upsert_parsed_field $parsed $contract "terminal.ghostty_trail_glow" (parse_contract_direct_field $contract $raw_config "terminal.ghostty_trail_glow"))
    $parsed = (upsert_parsed_field $parsed $contract "terminal.transparency" (parse_contract_direct_field $contract $raw_config "terminal.transparency"))

    $parsed = (upsert_parsed_field $parsed $contract "zellij.disable_tips" (parse_contract_bool_to_string_field $contract $raw_config "zellij.disable_tips"))
    $parsed = (upsert_parsed_field $parsed $contract "zellij.pane_frames" (parse_contract_bool_to_string_field $contract $raw_config "zellij.pane_frames"))
    $parsed = (upsert_parsed_field $parsed $contract "zellij.rounded_corners" (parse_contract_bool_to_string_field $contract $raw_config "zellij.rounded_corners"))
    $parsed = (upsert_parsed_field $parsed $contract "zellij.support_kitty_keyboard_protocol" (parse_contract_bool_to_string_field $contract $raw_config "zellij.support_kitty_keyboard_protocol"))
    $parsed = (upsert_parsed_field $parsed $contract "zellij.theme" (parse_contract_direct_field $contract $raw_config "zellij.theme"))
    $parsed = (upsert_parsed_field $parsed $contract "zellij.widget_tray" (parse_contract_direct_field $contract $raw_config "zellij.widget_tray"))
    $parsed = (upsert_parsed_field $parsed $contract "zellij.custom_text" (parse_contract_badge_text $contract $raw_config "zellij.custom_text"))
    $parsed = (upsert_parsed_field $parsed $contract "zellij.popup_program" (parse_contract_direct_field $contract $raw_config "zellij.popup_program"))
    $parsed = (upsert_parsed_field $parsed $contract "zellij.popup_width_percent" (parse_contract_int_range_field $contract $raw_config "zellij.popup_width_percent" "Update yazelix.toml with a value from 1 to 100, or run `yzx config reset` to restore the template."))
    $parsed = (upsert_parsed_field $parsed $contract "zellij.popup_height_percent" (parse_contract_int_range_field $contract $raw_config "zellij.popup_height_percent" "Update yazelix.toml with a value from 1 to 100, or run `yzx config reset` to restore the template."))
    $parsed = (upsert_parsed_field $parsed $contract "zellij.persistent_sessions" (parse_contract_bool_to_string_field $contract $raw_config "zellij.persistent_sessions"))
    $parsed = (upsert_parsed_field $parsed $contract "zellij.session_name" (parse_contract_direct_field $contract $raw_config "zellij.session_name"))
    $parsed = (upsert_parsed_field $parsed $contract "zellij.default_mode" (parse_contract_enum_field $contract $raw_config "zellij.default_mode" $generic_recovery))

    $parsed = (upsert_parsed_field $parsed $contract "yazi.command" (parse_contract_nullable_string_field $contract $raw_config "yazi.command"))
    $parsed = (upsert_parsed_field $parsed $contract "yazi.ya_command" (parse_contract_nullable_string_field $contract $raw_config "yazi.ya_command"))
    $parsed = (upsert_parsed_field $parsed $contract "yazi.plugins" (parse_contract_direct_field $contract $raw_config "yazi.plugins"))
    $parsed = (upsert_parsed_field $parsed $contract "yazi.theme" (parse_contract_direct_field $contract $raw_config "yazi.theme"))
    $parsed = (upsert_parsed_field $parsed $contract "yazi.sort_by" (parse_contract_direct_field $contract $raw_config "yazi.sort_by"))

    $parsed
        | upsert pack_names ($raw_config.packs?.enabled? | default [])
        | upsert pack_declarations ($raw_config.packs?.declarations? | default {})
        | upsert user_packages ($raw_config.packs?.user_packages? | default [])
        | upsert config_file $config_to_read
}
