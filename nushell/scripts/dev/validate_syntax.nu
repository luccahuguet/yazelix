#!/usr/bin/env nu
# Yazelix Syntax Validator
# Uses nu-check to validate syntax of all Nushell scripts

# Validate syntax of all Nushell scripts in yazelix
export def main [
    --verbose(-v)  # Show detailed output for each file
    --quiet(-q)    # Only show errors
] {
    let yazelix_dir = $"($env.HOME)/.config/yazelix"

    if not $quiet {
        print "🔍 Validating Nushell script syntax..."
        print ""
    }

    # Define script directories to check
    let script_dirs = [
        "nushell/scripts/core"
        "nushell/scripts/integrations"
        "nushell/scripts/setup"
        "nushell/scripts/utils"
        "nushell/scripts/dev"
        "nushell/scripts/dev/sweep"
        "nushell/config"
        "assets/macos"  # includes create_icns.nu
    ]

    # Collect all .nu files
    let all_files = $script_dirs
        | each { |dir|
            let full_path = ($yazelix_dir | path join $dir)
            if ($full_path | path exists) {
                glob ($full_path | path join "*.nu")
            } else {
                []
            }
        }
        | flatten

    if ($all_files | is-empty) {
        print "❌ No Nushell scripts found to validate"
        return
    }

    if $verbose and not $quiet {
        print $"Found ($all_files | length) script\(s\) to validate"
        print ""
    }

    # Validate each file
    let results = $all_files | each { |file|
        let file_name = ($file | path relative-to $yazelix_dir)

        if $verbose and not $quiet {
            print $"Checking ($file_name)..."
        }

        let result = (do { nu-check $file } | complete)

        if $result.exit_code == 0 {
            if $verbose and not $quiet {
                print $"  ✅ Valid"
            }
            {
                file: $file_name
                valid: true
                error: null
            }
        } else {
            if not $quiet {
                print $"  ❌ Syntax error in ($file_name)"
                if not ($result.stderr | is-empty) {
                    print $"     ($result.stderr)"
                }
            }
            {
                file: $file_name
                valid: false
                error: $result.stderr
            }
        }
    }

    # Summary
    let total = ($results | length)
    let passed = ($results | where valid == true | length)
    let failed = ($results | where valid == false | length)

    if not $quiet {
        print ""
        print "=== Syntax Validation Summary ==="
        print $"Total files: ($total)"
        print $"Valid: ($passed)"
        print $"Failed: ($failed)"
        print ""
    }

    if $failed > 0 {
        if not $quiet {
            print "❌ Syntax validation failed"
            print ""
            print "Failed files:"
            $results | where valid == false | each { |f|
                print $"  - ($f.file)"
                if not ($f.error | is-empty) {
                    print $"    ($f.error)"
                }
            }
        }
        error make { msg: "Syntax validation failed" }
    } else {
        if not $quiet {
            print $"✅ All ($total) scripts passed syntax validation!"
        }
    }
}
