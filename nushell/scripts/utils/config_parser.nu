#!/usr/bin/env nu
# Configuration parser for yazelix TOML files

use common.nu [get_yazelix_config_dir get_yazelix_runtime_dir]
use config_diagnostics.nu [build_config_diagnostic_report render_startup_config_error]
use failure_classes.nu [format_failure_classification]

def parse_refresh_output [raw_config: record] {
    let refresh_output = ($raw_config.core?.refresh_output? | default "normal" | into string | str downcase)
    let allowed = ["quiet", "normal", "full"]

    if not ($refresh_output in $allowed) {
        let allowed_text = ($allowed | str join ", ")
        let classification = (format_failure_classification "config" "Update yazelix.toml with a supported value, or run `yzx config reset --yes` to restore the template.")
        error make {msg: $"Invalid core.refresh_output value '($refresh_output)'. Expected one of: ($allowed_text)\n($classification)"}
    }

    $refresh_output
}

def parse_zellij_default_mode [raw_config: record] {
    let default_mode = ($raw_config.zellij?.default_mode? | default "normal" | into string | str downcase)
    let allowed = ["normal", "locked"]

    if not ($default_mode in $allowed) {
        let allowed_text = ($allowed | str join ", ")
        let classification = (format_failure_classification "config" "Update yazelix.toml with a supported value, or run `yzx config reset --yes` to restore the template.")
        error make {msg: $"Invalid zellij.default_mode value '($default_mode)'. Expected one of: ($allowed_text)\n($classification)"}
    }

    $default_mode
}

def parse_zjstatus_custom_text [raw_config: record] {
    let raw_text = ($raw_config.zellij?.custom_text? | default "" | into string)
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

def parse_positive_parallel_setting [value: any, label: string, allowed_symbols: list<string>, default_value: string] {
    let normalized = ($value | default $default_value | into string | str downcase)

    if $normalized in $allowed_symbols {
        return $normalized
    }

    let parsed = (try { $normalized | into int } catch { null })
    if $parsed == null {
        let allowed_text = ($allowed_symbols | str join ", ")
        let classification = (format_failure_classification "config" "Update yazelix.toml with a supported value, or run `yzx config reset --yes` to restore the template.")
        error make {msg: $"Invalid ($label) value '($normalized)'. Expected one of: ($allowed_text), or a positive integer.\n($classification)"}
    }
    if $parsed < 1 {
        let classification = (format_failure_classification "config" "Update yazelix.toml with a supported value, or run `yzx config reset --yes` to restore the template.")
        error make {msg: $"Invalid ($label) value '($normalized)'. Expected a positive integer.\n($classification)"}
    }
    $normalized
}

def parse_percentage_setting [value: any, label: string, default_value: int] {
    let normalized = ($value | default $default_value | into string | str trim)
    let parsed = (try { $normalized | into int } catch { null })

    if $parsed == null {
        let classification = (format_failure_classification "config" "Update yazelix.toml with a value from 1 to 100, or run `yzx config reset --yes` to restore the template.")
        error make {msg: $"Invalid ($label) value '($normalized)'. Expected an integer from 1 to 100.\n($classification)"}
    }

    if ($parsed < 1) or ($parsed > 100) {
        let classification = (format_failure_classification "config" "Update yazelix.toml with a value from 1 to 100, or run `yzx config reset --yes` to restore the template.")
        error make {msg: $"Invalid ($label) value '($normalized)'. Expected an integer from 1 to 100.\n($classification)"}
    }

    $parsed
}

# Parse yazelix configuration file and extract settings
export def parse_yazelix_config [] {
    let yazelix_config_dir = get_yazelix_config_dir
    let yazelix_runtime_dir = get_yazelix_runtime_dir

    # Check for config override first (for testing)
    let config_to_read = if ($env.YAZELIX_CONFIG_OVERRIDE? | is-not-empty) {
        $env.YAZELIX_CONFIG_OVERRIDE
    } else {
        # Determine which config file to use
        let toml_file = ($yazelix_config_dir | path join "yazelix.toml")
        let default_toml = ($yazelix_runtime_dir | path join "yazelix_default.toml")

        if ($toml_file | path exists) {
            $toml_file
        } else if ($default_toml | path exists) {
            # Auto-create yazelix.toml from default (copy raw to preserve comments)
            print "📝 Creating yazelix.toml from yazelix_default.toml..."
            mkdir $yazelix_config_dir
            cp $default_toml $toml_file
            print "✅ yazelix.toml created\n"
            $toml_file
        } else {
            let classification = (format_failure_classification "config" "Restore yazelix_default.toml, or reinstall Yazelix if the default config is missing from the runtime.")
            error make {msg: $"No yazelix configuration file found \(yazelix_default.toml is missing\)\n($classification)"}
        }
    }

    # Parse TOML configuration (Nushell auto-parses TOML files)
    let raw_config = open $config_to_read
    let default_config_path = ($yazelix_runtime_dir | path join "yazelix_default.toml")

    if ($config_to_read | path basename) == "yazelix.toml" and ($default_config_path | path exists) {
        let diagnostic_report = (build_config_diagnostic_report $config_to_read $default_config_path)
        if $diagnostic_report.has_blocking {
            error make {msg: (render_startup_config_error $diagnostic_report)}
        }
    }

    let editor_cmd = ($raw_config.editor?.command? | default "" | into string)
    let editor_command = if ($editor_cmd | is-empty) { null } else { $editor_cmd }

    # Extract and return values
    {
        recommended_deps: ($raw_config.core?.recommended_deps? | default true),
        yazi_extensions: ($raw_config.core?.yazi_extensions? | default true),
        yazi_media: ($raw_config.core?.yazi_media? | default false),
        debug_mode: ($raw_config.core?.debug_mode? | default false),
        skip_welcome_screen: ($raw_config.core?.skip_welcome_screen? | default false),
        show_macchina_on_welcome: ($raw_config.core?.show_macchina_on_welcome? | default true),
        refresh_output: (parse_refresh_output $raw_config),
        max_jobs: (parse_positive_parallel_setting $raw_config.core?.max_jobs? "core.max_jobs" ["auto", "max", "max_minus_one", "half", "quarter"] "half"),
        build_cores: (parse_positive_parallel_setting $raw_config.core?.build_cores? "core.build_cores" ["max", "max_minus_one", "half", "quarter"] "2"),
        ascii_art_mode: ($raw_config.ascii?.mode? | default "static"),
        persistent_sessions: ($raw_config.zellij?.persistent_sessions? | default false | into string),
        session_name: ($raw_config.zellij?.session_name? | default "yazelix"),
        zellij_default_mode: (parse_zellij_default_mode $raw_config),
        zellij_theme: ($raw_config.zellij?.theme? | default "default"),
        zellij_widget_tray: ($raw_config.zellij?.widget_tray? | default ["editor", "shell", "term", "cpu", "ram"]),
        zellij_custom_text: (parse_zjstatus_custom_text $raw_config),
        popup_program: ($raw_config.zellij?.popup_program? | default ["lazygit"]),
        popup_width_percent: (parse_percentage_setting $raw_config.zellij?.popup_width_percent? "zellij.popup_width_percent" 90),
        popup_height_percent: (parse_percentage_setting $raw_config.zellij?.popup_height_percent? "zellij.popup_height_percent" 90),
        support_kitty_keyboard_protocol: ($raw_config.zellij?.support_kitty_keyboard_protocol? | default false | into string),
        terminals: ($raw_config.terminal?.terminals? | default ["ghostty"]),
        manage_terminals: ($raw_config.terminal?.manage_terminals? | default true),
        terminal_config_mode: ($raw_config.terminal?.config_mode? | default "yazelix"),
        ghostty_trail_color: ($raw_config.terminal?.ghostty_trail_color? | default "random"),
        ghostty_trail_effect: ($raw_config.terminal?.ghostty_trail_effect? | default "random"),
        ghostty_mode_effect: ($raw_config.terminal?.ghostty_mode_effect? | default "random"),
        ghostty_trail_glow: ($raw_config.terminal?.ghostty_trail_glow? | default "medium"),
        transparency: ($raw_config.terminal?.transparency? | default "medium"),
        default_shell: ($raw_config.shell?.default_shell? | default "nu"),
        extra_shells: ($raw_config.shell?.extra_shells? | default []),
        helix_mode: ($raw_config.helix?.mode? | default "release"),
        helix_runtime_path: ($raw_config.helix?.runtime_path? | default null),
        editor_command: $editor_command,
        enable_sidebar: ($raw_config.editor?.enable_sidebar? | default true),
        disable_zellij_tips: ($raw_config.zellij?.disable_tips? | default true | into string),
        zellij_rounded_corners: ($raw_config.zellij?.rounded_corners? | default true | into string),
        yazi_plugins: ($raw_config.yazi?.plugins? | default ["git"]),
        yazi_theme: ($raw_config.yazi?.theme? | default "default"),
        yazi_sort_by: ($raw_config.yazi?.sort_by? | default "alphabetical"),
        pack_names: ($raw_config.packs?.enabled? | default []),
        pack_declarations: ($raw_config.packs?.declarations? | default {}),
        user_packages: ($raw_config.packs?.user_packages? | default []),
        config_file: $config_to_read
    }
}
