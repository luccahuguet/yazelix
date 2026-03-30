#!/usr/bin/env nu
# Configuration parser for yazelix TOML files

use config_diagnostics.nu [build_config_diagnostic_report_from_records render_startup_config_error]
use failure_classes.nu [format_failure_classification]
use config_surfaces.nu [load_active_config_surface load_config_surface_from_main]

def parse_refresh_output [raw_config: record] {
    let refresh_output = ($raw_config.core?.refresh_output? | default "normal" | into string | str downcase)
    let allowed = ["quiet", "normal", "full"]

    if not ($refresh_output in $allowed) {
        let allowed_text = ($allowed | str join ", ")
        let classification = (format_failure_classification "config" "Update yazelix.toml with a supported value, or run `yzx config reset` to restore the template.")
        error make {msg: $"Invalid core.refresh_output value '($refresh_output)'. Expected one of: ($allowed_text)\n($classification)"}
    }

    $refresh_output
}

def parse_zellij_default_mode [raw_config: record] {
    let default_mode = ($raw_config.zellij?.default_mode? | default "normal" | into string | str downcase)
    let allowed = ["normal", "locked"]

    if not ($default_mode in $allowed) {
        let allowed_text = ($allowed | str join ", ")
        let classification = (format_failure_classification "config" "Update yazelix.toml with a supported value, or run `yzx config reset` to restore the template.")
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
        let classification = (format_failure_classification "config" "Update yazelix.toml with a supported value, or run `yzx config reset` to restore the template.")
        error make {msg: $"Invalid ($label) value '($normalized)'. Expected one of: ($allowed_text), or a positive integer.\n($classification)"}
    }
    if $parsed < 1 {
        let classification = (format_failure_classification "config" "Update yazelix.toml with a supported value, or run `yzx config reset` to restore the template.")
        error make {msg: $"Invalid ($label) value '($normalized)'. Expected a positive integer.\n($classification)"}
    }
    $normalized
}

def parse_percentage_setting [value: any, label: string, default_value: int] {
    let normalized = ($value | default $default_value | into string | str trim)
    let parsed = (try { $normalized | into int } catch { null })

    if $parsed == null {
        let classification = (format_failure_classification "config" "Update yazelix.toml with a value from 1 to 100, or run `yzx config reset` to restore the template.")
        error make {msg: $"Invalid ($label) value '($normalized)'. Expected an integer from 1 to 100.\n($classification)"}
    }

    if ($parsed < 1) or ($parsed > 100) {
        let classification = (format_failure_classification "config" "Update yazelix.toml with a value from 1 to 100, or run `yzx config reset` to restore the template.")
        error make {msg: $"Invalid ($label) value '($normalized)'. Expected an integer from 1 to 100.\n($classification)"}
    }

    $parsed
}

def parse_sidebar_width_percent [raw_config: record] {
    let normalized = ($raw_config.editor?.sidebar_width_percent? | default 20 | into string | str trim)
    let parsed = (try { $normalized | into int } catch { null })

    if $parsed == null {
        let classification = (format_failure_classification "config" "Update editor.sidebar_width_percent to an integer from 10 to 40, or run `yzx config reset` to restore the template.")
        error make {msg: $"Invalid editor.sidebar_width_percent value '($normalized)'. Expected an integer from 10 to 40.\n($classification)"}
    }

    if ($parsed < 10) or ($parsed > 40) {
        let classification = (format_failure_classification "config" "Update editor.sidebar_width_percent to an integer from 10 to 40, or run `yzx config reset` to restore the template.")
        error make {msg: $"Invalid editor.sidebar_width_percent value '($normalized)'. Expected an integer from 10 to 40.\n($classification)"}
    }

    $parsed
}

def parse_terminal_config_mode [raw_config: record] {
    let mode = ($raw_config.terminal?.config_mode? | default "yazelix" | into string | str downcase)
    let allowed = ["yazelix", "user"]

    if not ($mode in $allowed) {
        let allowed_text = ($allowed | str join ", ")
        let classification = (format_failure_classification "config" "Use `terminal.config_mode = \"yazelix\"` for the supported managed path, or `\"user\"` only when you want Yazelix to load the terminal's native config file.")
        error make {msg: $"Invalid terminal.config_mode value '($mode)'. Expected one of: ($allowed_text)\n($classification)"}
    }

    $mode
}

def parse_welcome_style [raw_config: record] {
    let style = ($raw_config.core?.welcome_style? | default "random" | into string | str downcase)
    let allowed = ["static", "logo", "boids", "game_of_life", "mandelbrot", "random"]

    if not ($style in $allowed) {
        let allowed_text = ($allowed | str join ", ")
        let classification = (format_failure_classification "config" "Update core.welcome_style with one of the supported values, or run `yzx config reset` to restore the template.")
        error make {msg: $"Invalid core.welcome_style value '($style)'. Expected one of: ($allowed_text)\n($classification)"}
    }

    $style
}

def parse_welcome_duration_seconds [raw_config: record] {
    let raw_value = ($raw_config.core?.welcome_duration_seconds? | default 2.0)
    let parsed = (try { $raw_value | into float } catch { null })

    if $parsed == null {
        let classification = (format_failure_classification "config" "Update core.welcome_duration_seconds to a number from 0.2 to 8.0, or run `yzx config reset` to restore the template.")
        error make {msg: $"Invalid core.welcome_duration_seconds value '($raw_value)'. Expected a number from 0.2 to 8.0.\n($classification)"}
    }

    if ($parsed < 0.2) or ($parsed > 8.0) {
        let classification = (format_failure_classification "config" "Update core.welcome_duration_seconds to a number from 0.2 to 8.0, or run `yzx config reset` to restore the template.")
        error make {msg: $"Invalid core.welcome_duration_seconds value '($raw_value)'. Expected a number from 0.2 to 8.0.\n($classification)"}
    }

    $parsed
}

# Parse yazelix configuration file and extract settings
export def parse_yazelix_config [] {
    let config_surface = load_active_config_surface
    let config_to_read = $config_surface.config_file
    let raw_config = $config_surface.merged_config
    let default_config_path = $config_surface.default_config_path

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
        welcome_style: (parse_welcome_style $raw_config),
        welcome_duration_seconds: (parse_welcome_duration_seconds $raw_config),
        refresh_output: (parse_refresh_output $raw_config),
        max_jobs: (parse_positive_parallel_setting $raw_config.core?.max_jobs? "core.max_jobs" ["auto", "max", "max_minus_one", "half", "quarter"] "half"),
        build_cores: (parse_positive_parallel_setting $raw_config.core?.build_cores? "core.build_cores" ["max", "max_minus_one", "half", "quarter"] "2"),
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
        terminal_config_mode: (parse_terminal_config_mode $raw_config),
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
        sidebar_width_percent: (parse_sidebar_width_percent $raw_config),
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
