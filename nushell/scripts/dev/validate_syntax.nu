#!/usr/bin/env nu
# Yazelix Syntax Validator
# Uses nu-check to validate syntax of all Nushell scripts

# Validate syntax of all Nushell scripts in yazelix
export def main [
    --verbose(-v)  # Show detailed output for each file
    --quiet(-q)    # Only show errors (internal flag for test runner)
] {
    let yazelix_dir = $"($env.HOME)/.config/yazelix"

    if not $quiet {
        print "🔍 Validating Nushell script syntax..."
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

    # Validate each file
    let results = $all_files | each { |file|
        let file_name = ($file | path relative-to $yazelix_dir)

        if $verbose and not $quiet {
            print $"Checking ($file_name)..."
        }

        let result = try {
            nu-check $file
            { success: true, error: null }
        } catch { |err|
            { success: false, error: ($err | get debug) }
        }

        if $result.success {
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
                print $"❌ Syntax error in ($file_name)"
                if ($result.error != null) {
                    print $"   ($result.error)"
                }
            }
            {
                file: $file_name
                valid: false
                error: $result.error
            }
        }
    }

    # Summary
    let total = ($results | length)
    let passed = ($results | where valid == true | length)
    let failed = ($results | where valid == false | length)

    if $failed > 0 {
        if not $quiet {
            print ""
            if $verbose {
                print "=== Syntax Validation Failed ==="
                $results | where valid == false | each { |f|
                    print $"❌ ($f.file)"
                    if not ($f.error | is-empty) {
                        print $"   ($f.error)"
                    }
                }
                print ""
                print $"Failed: ($failed)/($total) scripts"
            } else {
                print $"❌ Syntax validation failed: ($failed)/($total) scripts have errors"
            }
        }
        error make { msg: "Syntax validation failed" }
    } else {
        if not $quiet {
            print $"✅ All ($total) scripts passed syntax validation"
        }
    }
}
