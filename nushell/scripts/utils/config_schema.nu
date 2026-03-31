#!/usr/bin/env nu
# Dynamic Yazelix Config Schema Validator
# Uses the canonical main-config contract plus the default config surfaces as the
# reference for validation.

use config_contract.nu [load_main_config_contract]
use config_surfaces.nu [load_config_surface_from_main get_main_user_config_path]

const OPEN_RECORD_PATHS = [
    ["packs", "declarations"]
]

const FLEXIBLE_NUMERIC_PATHS = [
    ["core", "welcome_duration_seconds"]
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

def is_flexible_numeric_path [path: list<string>] {
    let target = (format_config_path $path)
    $FLEXIBLE_NUMERIC_PATHS | any {|candidate| (format_config_path $candidate) == $target }
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
                let nested_findings = (compare_configs ($default | get -o $key) ($user | get -o $key) ($path | append $key))
                if not ($nested_findings | is-empty) {
                    $findings = ($findings | append $nested_findings)
                }
            }
        }

        return $findings
    }

    if $default_kind != $user_kind {
        if (
            (is_flexible_numeric_path $path)
            and ($default_kind in ["int", "float"])
            and ($user_kind in ["int", "float"])
        ) {
            return []
        }

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
        $current = ($current | get -o $segment)
        if $current == null {
            return null
        }
    }
    $current
}

def set_nested_value [record: record, path: list<string>, value: any] {
    if ($path | is-empty) {
        return $record
    }

    let head = ($path | first)
    if ($path | length) == 1 {
        return ($record | upsert $head $value)
    }

    let tail = ($path | skip 1)
    let nested = ($record | get -o $head | default {})
    $record | upsert $head (set_nested_value $nested $tail $value)
}

export def apply_main_contract_to_reference_config [reference: record] {
    let contract = (load_main_config_contract)
    mut merged = $reference

    for field_path in ($contract.fields | columns) {
        let field = ($contract.fields | get -o $field_path | default {})
        let default_value = ($field.default? | default null)
        $merged = (set_nested_value $merged ($field_path | split row ".") $default_value)
    }

    $merged
}

# Helper: Validate enum values for key fields
export def validate_enum_values [user: record] {
    let contract = (load_main_config_contract)
    mut findings = []
    for field_path in ($contract.fields | columns) {
        let field = ($contract.fields | get -o $field_path | default {})
        let validation = ($field.validation? | default "")
        if ($validation != "enum") and ($validation != "enum_string_list") {
            continue
        }

        let path = ($field_path | split row ".")
        let allowed = ($field.allowed_values? | default [])
        let value = (get_nested_value $user $path)
        if $value == null {
            continue
        }

        if ($validation == "enum_string_list") and (($value | describe) | str contains "list") {
            for v in $value {
                if not ($v in $allowed) {
                    let allowed_str = ($allowed | str join ", ")
                    $findings = ($findings | append [
                        (make_finding
                            "invalid_enum"
                            $path
                            ('Invalid value for ' + $field_path + ': ' + $v + ' (allowed: [' + $allowed_str + '])')
                        )
                    ])
                }
            }
        } else {
            if not ($value in $allowed) {
                let allowed_str = ($allowed | str join ", ")
                $findings = ($findings | append [
                    (make_finding
                        "invalid_enum"
                        $path
                        ('Invalid value for ' + $field_path + ': ' + $value + ' (allowed: [' + $allowed_str + '])')
                    )
                ])
            }
        }
    }
    $findings
}

export def get_config_validation_findings [yazelix_dir: string] {
    let default_path = ($yazelix_dir | path expand | path join "yazelix_default.toml")
    let user_path = (get_main_user_config_path $yazelix_dir)

    if not ($default_path | path exists) {
        error make {msg: $"yazelix_default.toml not found at ($default_path)"}
    }

    if not ($user_path | path exists) {
        []
    } else {
        let default_config = (apply_main_contract_to_reference_config ((load_config_surface_from_main $default_path).merged_config))
        let user_config = ((load_config_surface_from_main $user_path).merged_config)
        let schema_findings = (compare_configs $default_config $user_config)
        let enum_findings = (validate_enum_values $user_config)
        [ $schema_findings $enum_findings ] | flatten
    }
}

# Main exported function: validate user config against yazelix_default.toml
export def validate_config_against_default [yazelix_dir: string] {
    let default_path = ($yazelix_dir | path expand | path join "yazelix_default.toml")
    let user_path = (get_main_user_config_path $yazelix_dir)
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
