#!/usr/bin/env nu
# Dynamic Yazelix Config Schema Validator
# Uses yazelix_default.toml as the reference for validation

use constants.nu [SUPPORTED_TERMINALS, CURSOR_TRAIL_SHADERS, GHOSTTY_TRAIL_EFFECTS, GHOSTTY_MODE_EFFECTS, GHOSTTY_TRAIL_GLOW_LEVELS]

const OPEN_RECORD_PATHS = [
    ["packs", "declarations"]
]

def format_config_path [path: list<string>] {
    if ($path | is-empty) {
        "<root>"
    } else {
        $path | str join "."
    }
}

def classify_value [value: any] {
    let description = ($value | describe)

    if ($description | str contains "record") {
        "record"
    } else if ($description | str contains "list") {
        "list"
    } else if ($description | str contains "string") {
        "string"
    } else if ($description | str contains "bool") {
        "bool"
    } else if ($description | str contains "int") {
        "int"
    } else if ($description | str contains "float") {
        "float"
    } else if ($description | str contains "nothing") {
        "nothing"
    } else {
        $description
    }
}

def make_finding [kind: string, path: list<string>, message: string] {
    {
        kind: $kind
        path: (format_config_path $path)
        message: $message
    }
}

def is_open_record_path [path: list<string>] {
    let target = (format_config_path $path)
    $OPEN_RECORD_PATHS | any {|candidate| (format_config_path $candidate) == $target }
}

# Compare two config structures recursively, except for explicitly user-extensible paths.
export def compare_configs [default: any, user: any, path: list<string> = []] {
    let default_kind = (classify_value $default)
    let user_kind = (classify_value $user)

    if $default_kind == "record" {
        if $user_kind != "record" {
            return [
                (make_finding
                    "type_mismatch"
                    $path
                    $"Type mismatch at (format_config_path $path): expected ($default_kind), found ($user_kind)"
                )
            ]
        }

        let default_keys = ($default | columns)
        let user_keys = ($user | columns)
        mut findings = []

        if not (is_open_record_path $path) {
            for key in $user_keys {
                if not ($key in $default_keys) {
                    let finding_path = ($path | append $key)
                    $findings = ($findings | append [
                        (make_finding
                            "unknown_field"
                            $finding_path
                            $"Unknown config field: (format_config_path $finding_path)"
                        )
                    ])
                }
            }

            for key in $default_keys {
                if not ($key in $user_keys) {
                    let finding_path = ($path | append $key)
                    $findings = ($findings | append [
                        (make_finding
                            "missing_field"
                            $finding_path
                            $"Missing config field: (format_config_path $finding_path)"
                        )
                    ])
                }
            }
        }

        if (is_open_record_path $path) {
            return $findings
        }

        for key in $default_keys {
            if $key in $user_keys {
                let nested_findings = (compare_configs ($default | get $key) ($user | get $key) ($path | append $key))
                if not ($nested_findings | is-empty) {
                    $findings = ($findings | append $nested_findings)
                }
            }
        }

        return $findings
    }

    if $default_kind != $user_kind {
        return [
            (make_finding
                "type_mismatch"
                $path
                $"Type mismatch at (format_config_path $path): expected ($default_kind), found ($user_kind)"
            )
        ]
    }

    []
}

# Helper: Safely retrieve nested values from a record
def get_nested_value [data: any, path: list<string>] {
    mut current = $data
    for segment in $path {
        try {
            $current = ($current | get $segment)
        } catch {
            return null
        }
    }
    $current
}

# Helper: Validate enum values for key fields
export def validate_enum_values [user: record] {
    mut findings = []
    let ghostty_trail_color_allowed = (($CURSOR_TRAIL_SHADERS | columns) | append ["random"])
    let ghostty_trail_effect_allowed = ($GHOSTTY_TRAIL_EFFECTS | append ["random"])
    let ghostty_mode_effect_allowed = ($GHOSTTY_MODE_EFFECTS | append ["random"])
    let enums = [
        { path: ["shell", "default_shell"], label: "shell.default_shell", allowed: ["nu", "bash", "fish", "zsh"] },
        { path: ["helix", "mode"], label: "helix.mode", allowed: ["release", "source"] },
        { path: ["core", "refresh_output"], label: "core.refresh_output", allowed: ["quiet", "normal", "full"] },
        { path: ["terminal", "terminals"], label: "terminal.terminals", allowed: $SUPPORTED_TERMINALS },
        { path: ["terminal", "config_mode"], label: "terminal.config_mode", allowed: ["yazelix", "user"] },
        { path: ["terminal", "ghostty_trail_color"], label: "terminal.ghostty_trail_color", allowed: $ghostty_trail_color_allowed },
        { path: ["terminal", "ghostty_trail_effect"], label: "terminal.ghostty_trail_effect", allowed: $ghostty_trail_effect_allowed },
        { path: ["terminal", "ghostty_mode_effect"], label: "terminal.ghostty_mode_effect", allowed: $ghostty_mode_effect_allowed },
        { path: ["terminal", "ghostty_trail_glow"], label: "terminal.ghostty_trail_glow", allowed: $GHOSTTY_TRAIL_GLOW_LEVELS },
        { path: ["ascii", "mode"], label: "ascii.mode", allowed: ["static", "animated"] },
        { path: ["zellij", "widget_tray"], label: "zellij.widget_tray", allowed: ["editor", "shell", "term", "cpu", "ram"] }
    ]
    for enum in $enums {
        let value = (get_nested_value $user $enum.path)
        if $value == null {
            continue
        }
        if ($enum.label == "terminal.terminals") and ($value | describe | str contains "list") {
            for v in $value {
                if not ($v in $enum.allowed) {
                    let allowed_str = ($enum.allowed | str join ", ")
                    $findings = ($findings | append [
                        (make_finding
                            "invalid_enum"
                            $enum.path
                            ('Invalid value for terminal.terminals: ' + $v + ' (allowed: [' + $allowed_str + '])')
                        )
                    ])
                }
            }
        } else if ($enum.label == "zellij.widget_tray") and ($value | describe | str contains "list") {
            for v in $value {
                if not ($v in $enum.allowed) {
                    let allowed_str = ($enum.allowed | str join ", ")
                    $findings = ($findings | append [
                        (make_finding
                            "invalid_enum"
                            $enum.path
                            ('Invalid value for zellij.widget_tray: ' + $v + ' (allowed: [' + $allowed_str + '])')
                        )
                    ])
                }
            }
        } else {
            if not ($value in $enum.allowed) {
                let allowed_str = ($enum.allowed | str join ", ")
                $findings = ($findings | append [
                    (make_finding
                        "invalid_enum"
                        $enum.path
                        ('Invalid value for ' + $enum.label + ': ' + $value + ' (allowed: [' + $allowed_str + '])')
                    )
                ])
            }
        }
    }
    $findings
}

export def get_config_validation_findings [yazelix_dir: string] {
    let default_path = ($yazelix_dir | path expand | path join "yazelix_default.toml")
    let user_path = ($yazelix_dir | path expand | path join "yazelix.toml")

    if not ($default_path | path exists) {
        error make {msg: $"yazelix_default.toml not found at ($default_path)"}
    }

    if not ($user_path | path exists) {
        []
    } else {
        let default_config = open $default_path
        let user_config = open $user_path
        let schema_findings = (compare_configs $default_config $user_config)
        let enum_findings = (validate_enum_values $user_config)
        [ $schema_findings $enum_findings ] | flatten
    }
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
    let findings = (get_config_validation_findings $yazelix_dir)
    if ($findings | is-empty) {
        print "✅ User config matches yazelix_default.toml (all fields present, no unknowns, all values valid)"
    } else {
        print "🔧 Yazelix Config Validation:"
        for finding in $findings {
            print $"   ⚠️  ($finding.message)"
        }
    }
}
