#!/usr/bin/env nu
# Dynamic Yazelix Config Schema Validator
# Uses yazelix_default.toml as the reference for validation

use constants.nu [SUPPORTED_TERMINALS, CURSOR_TRAIL_SHADERS, GHOSTTY_CURSOR_EFFECTS]

# Helper: Compare two records (default vs user config), only at the top level
# No recursion into nested configs
export def compare_configs [default: record, user: record] {
    mut warnings = []
    let default_keys = ($default | columns)
    let user_keys = ($user | columns)

    # Warn about unknown fields in user config
    for key in $user_keys {
        if not ($key in $default_keys) {
            $warnings = ($warnings | append $"⚠️  Unknown config field: ($key)")
        }
    }

    # Warn about missing fields in user config
    for key in $default_keys {
        if not ($key in $user_keys) {
            $warnings = ($warnings | append $"⚠️  Missing config field: ($key)")
        }
    }
    $warnings
}

# Helper: Safely retrieve nested values from a record
def get_nested_value [data: any, path: list<string>] {
    mut current = $data
    for segment in $path {
        let current = (try {
            $current | get $segment
        } catch {
            return null
        })
    }
    $current
}

# Helper: Validate enum values for key fields
export def validate_enum_values [user: record] {
    mut warnings = []
    let cursor_trail_allowed = (($CURSOR_TRAIL_SHADERS | columns | where $it != "none") | append ["random" "none"])
    let enums = [
        { path: ["shell", "default_shell"], label: "shell.default_shell", allowed: ["nu", "bash", "fish", "zsh"] },
        { path: ["helix", "mode"], label: "helix.mode", allowed: ["release", "source"] },
        { path: ["core", "refresh_output"], label: "core.refresh_output", allowed: ["quiet", "normal", "full"] },
        { path: ["terminal", "terminals"], label: "terminal.terminals", allowed: $SUPPORTED_TERMINALS },
        { path: ["terminal", "cursor_trail"], label: "terminal.cursor_trail", allowed: $cursor_trail_allowed },
        { path: ["terminal", "ghostty_cursor_effects"], label: "terminal.ghostty_cursor_effects", allowed: $GHOSTTY_CURSOR_EFFECTS },
        { path: ["ascii", "mode"], label: "ascii.mode", allowed: ["static", "animated"] },
        { path: ["zellij", "widget_tray"], label: "zellij.widget_tray", allowed: ["layout", "editor", "shell", "term", "cpu", "ram"] }
    ]
    for enum in $enums {
        let value = (get_nested_value $user $enum.path)
        if $value == null {
            continue
        }
        if (($enum.label == "terminal.cursor_trail") or ($enum.label == "terminal.ghostty_cursor_effects")) and (value | describe | str contains "list") {
            # Validate each list entry
            for v in $value {
                if not ($v in $enum.allowed) {
                    let allowed_str = ($enum.allowed | str join ", ")
                    let msg = '⚠️  Invalid value for ' + $enum.label + ': ' + $v + ' (allowed: [' + $allowed_str + '])'
                    $warnings = ($warnings | append $msg)
                }
            }
        } else if ($enum.label == "terminal.terminals") and (value | describe | str contains "list") {
            for v in $value {
                if not ($v in $enum.allowed) {
                    let allowed_str = ($enum.allowed | str join ", ")
                    let msg = '⚠️  Invalid value for terminal.terminals: ' + $v + ' (allowed: [' + $allowed_str + '])'
                    $warnings = ($warnings | append $msg)
                }
            }
        } else if ($enum.label == "zellij.widget_tray") and (value | describe | str contains "list") {
            for v in $value {
                if not ($v in $enum.allowed) {
                    let allowed_str = ($enum.allowed | str join ", ")
                    let msg = '⚠️  Invalid value for zellij.widget_tray: ' + $v + ' (allowed: [' + $allowed_str + '])'
                    $warnings = ($warnings | append $msg)
                }
            }
        } else {
            if not ($value in $enum.allowed) {
                let allowed_str = ($enum.allowed | str join ", ")
                let msg = '⚠️  Invalid value for ' + $enum.label + ': ' + $value + ' (allowed: [' + $allowed_str + '])'
                $warnings = ($warnings | append $msg)
            }
        }
    }
    $warnings
}

# Main exported function: validate user config against yazelix_default.toml
export def validate_config_against_default [yazelix_dir: string] {
    let default_path = ($yazelix_dir | path expand | path join "yazelix_default.toml")
    let user_path = ($yazelix_dir | path expand | path join "yazelix.toml")
    if not ($default_path | path exists) {
        print $"❌ yazelix_default.toml not found at ($default_path)"
        return
    }
    if not ($user_path | path exists) {
        print $"⚠️  yazelix.toml not found at ($user_path) - using defaults only"
        return
    }
    # Read TOML files directly
    let default_config = open $default_path
    let user_config = open $user_path
    let warnings = compare_configs $default_config $user_config
    let enum_warnings = validate_enum_values $user_config
    let all_warnings = ($warnings | append $enum_warnings)
    if ($all_warnings | is-empty) {
        print "✅ User config matches yazelix_default.toml (all sections present, no unknowns, all values valid)"
    } else {
        print "🔧 Yazelix Config Validation:"
        for warning in $all_warnings {
            print $"   ($warning)"
        }
    }
}
