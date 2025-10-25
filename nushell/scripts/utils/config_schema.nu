#!/usr/bin/env nu
# Dynamic Yazelix Config Schema Validator
# Uses yazelix_default.nix as the reference for validation

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

# Helper: Validate enum values for key fields
export def validate_enum_values [user: record] {
    mut warnings = []
    let enums = [
        { key: "default_shell", allowed: ["nu", "bash", "fish", "zsh"] },
        { key: "helix_mode", allowed: ["release", "source"] },
        { key: "preferred_terminal", allowed: ["wezterm", "ghostty", "kitty", "alacritty", "foot"] },
        { key: "cursor_trail", allowed: ["blaze", "snow", "cosmic", "ocean", "forest", "sunset", "neon", "party", "eclipse", "dusk", "orchid", "reef", "random", "none"] },
        { key: "ascii_art_mode", allowed: ["static", "animated"] }
    ]
    for enum in $enums {
        if not ($user | columns | any {|k| $k == $enum.key }) {
            continue
        }
        let value = ($user | get $enum.key)
        if ($enum.key == "cursor_trail") and (value | describe | str contains "list") {
            # Validate each list entry
            for v in $value {
                if not ($v in $enum.allowed) {
                    let allowed_str = ($enum.allowed | str join ", ")
                    let msg = '⚠️  Invalid value for cursor_trail: ' + $v + ' (allowed: [' + $allowed_str + '])'
                    $warnings = ($warnings | append $msg)
                }
            }
        } else {
            if not ($value in $enum.allowed) {
                let allowed_str = ($enum.allowed | str join ", ")
                let msg = '⚠️  Invalid value for ' + $enum.key + ': ' + $value + ' (allowed: [' + $allowed_str + '])'
                $warnings = ($warnings | append $msg)
            }
        }
    }
    $warnings
}

# Helper: Evaluate a Nix config file to JSON using nix eval --json --impure --expr
# Handles Nix function configs by passing a dummy pkgs argument
export def eval_nix_config_to_json [path: string] {
    let abs_path = ($path | path expand)
    let expr = $'import "' + $abs_path + '" { pkgs = import <nixpkgs> {}; }'
    (run-external "nix" "eval" "--json" "--impure" "--expr" $expr | from json)
}

# Main exported function: validate user config against yazelix_default.nix
export def validate_config_against_default [yazelix_dir: string] {
    let default_path = ($yazelix_dir | path expand | path join "yazelix_default.nix")
    let user_path = ($yazelix_dir | path expand | path join "yazelix.nix")
    if not ($default_path | path exists) {
        print $"❌ yazelix_default.nix not found at ($default_path)"
        return
    }
    if not ($user_path | path exists) {
        print $"❌ yazelix.nix not found at ($user_path)"
        return
    }
    # Use helper to evaluate both configs
    let default_config = eval_nix_config_to_json $default_path
    let user_config = eval_nix_config_to_json $user_path
    let warnings = compare_configs $default_config $user_config
    let enum_warnings = validate_enum_values $user_config
    let all_warnings = ($warnings | append $enum_warnings)
    if ($all_warnings | is-empty) {
        print "✅ User config matches yazelix_default.nix (all fields present, no unknowns, all values valid)"
    } else {
        print "🔧 Yazelix Config Validation:"
        for warning in $all_warnings {
            print $"   ($warning)"
        }
    }
}
