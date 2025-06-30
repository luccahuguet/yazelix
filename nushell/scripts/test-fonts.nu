#!/usr/bin/env nu

# Test 15 best fonts for VHS recording - each with separate output files
def test_fonts [] {
    print "🎨 Testing 15 Best Fonts for VHS Recording"
    print "=========================================="
    print ""

    let demo_file = "assets/demos/yazelix-v7-quick-demo.tape"

    if not ($demo_file | path exists) {
        print $"(ansi red)❌ Error: Demo file '($demo_file)' not found(ansi reset)"
        return
    }

    # Ensure assets/font-tests directory exists
    mkdir assets/font-tests
    
    # Define 15 best fonts with their nix packages and output names
    let fonts = [
        {name: "JetBrains Mono Nerd Font", package: "nerd-fonts.jetbrains-mono", output: "assets/font-tests/jetbrains-mono-test.gif"},
        {name: "Cascadia Code Nerd Font", package: "nerd-fonts-caskaydia-cove", output: "assets/font-tests/cascadia-code-test.gif"},
        {name: "Fira Code Nerd Font", package: "nerd-fonts.fira-code", output: "assets/font-tests/fira-code-test.gif"},
        {name: "Hack Nerd Font", package: "nerd-fonts.hack", output: "assets/font-tests/hack-test.gif"},
        {name: "Iosevka Nerd Font", package: "nerd-fonts.iosevka", output: "assets/font-tests/iosevka-test.gif"},
        {name: "Ubuntu Mono Nerd Font", package: "nerd-fonts.ubuntu-mono", output: "assets/font-tests/ubuntu-mono-test.gif"},
        {name: "Source Code Pro Nerd Font", package: "nerd-fonts.sauce-code-pro", output: "assets/font-tests/source-code-pro-test.gif"},
        {name: "Roboto Mono Nerd Font", package: "nerd-fonts.roboto-mono", output: "assets/font-tests/roboto-mono-test.gif"},
        {name: "Inconsolata Nerd Font", package: "nerd-fonts.inconsolata", output: "assets/font-tests/inconsolata-test.gif"},
        {name: "DejaVu Sans Mono Nerd Font", package: "nerd-fonts.dejavu-sans-mono", output: "assets/font-tests/dejavu-sans-mono-test.gif"},
        {name: "Liberation Mono Nerd Font", package: "nerd-fonts.liberation", output: "assets/font-tests/liberation-mono-test.gif"},
        {name: "Meslo Nerd Font", package: "nerd-fonts.meslo-lg", output: "assets/font-tests/meslo-test.gif"},
        {name: "Space Mono Nerd Font", package: "nerd-fonts.space-mono", output: "assets/font-tests/space-mono-test.gif"},
        {name: "Victor Mono Nerd Font", package: "nerd-fonts.victor-mono", output: "assets/font-tests/victor-mono-test.gif"},
        {name: "Noto Nerd Font", package: "nerd-fonts.noto", output: "assets/font-tests/noto-test.gif"}
    ]

    print $"🚀 Testing ($fonts | length) fonts..."
    print ""

    # Test each font
    for font in $fonts {
        let counter = ($fonts | enumerate | where item.name == $font.name | get index | first) + 1

        print $"[($counter)/15] 🎬 Testing: ($font.name)"
        print $"📁 Creating: ($font.output)"

        # Create custom tape file for this font
        let custom_tape = $"($font.name | str replace ' ' '-' | str downcase)-test.tape"

        # Generate tape file content with proper output filename and font
        let tape_content = (open $demo_file | str replace "Output yazelix-v7-quick-demo.gif" $"Output ($font.output)" | str replace "Set Theme \"Catppuccin Mocha\"" $"Set Theme \"Catppuccin Mocha\"\nSet FontFamily \"($font.name)\"")

        # Write custom tape file
        $tape_content | save --force $custom_tape

        # Record with this font
        print $"📦 Using nix package: ($font.package)"
        let result = (do {
            nix shell $"nixpkgs#($font.package)" nixpkgs#vhs --command vhs $custom_tape
        } | complete)

        if $result.exit_code == 0 {
            print $"(ansi green)✅ Success: ($font.output)(ansi reset)"

            # Show file size if it exists
            if ($font.output | path exists) {
                let size = (ls $font.output | get size | first)
                print $"(ansi blue)📊 Size: ($size)(ansi reset)"
            }
        } else {
            print $"(ansi red)❌ Failed: ($font.name)(ansi reset)"
            print $"(ansi red)Error: ($result.stderr)(ansi reset)"
        }

        # Clean up temporary tape file
        if ($custom_tape | path exists) {
            rm $custom_tape
        }

        print ""
    }

    print "🎉 FONT TESTING COMPLETE!"
    print ""
    print "📊 Results:"
    print "Generated GIF files:"

    # List all generated test files
    for font in $fonts {
        if ($font.output | path exists) {
            let size = (ls $font.output | get size | first)
            print $"  ✅ ($font.output) - ($size)"
        } else {
            print $"  ❌ ($font.output) - FAILED"
        }
    }

    print ""
    print "💡 RECOMMENDATIONS:"
    print "1. JetBrains Mono - Best overall for coding"
    print "2. Cascadia Code - Microsoft's modern choice"
    print "3. Fira Code - Great ligatures"
    print "4. Iosevka - Terminal optimized"
    print "5. Source Code Pro - Adobe's clean design"
}

# Clean up old test files
def "main clean" [] {
    print "🧹 Cleaning up old font test files..."
    
    let test_files = (ls assets/font-tests/*.gif 2>/dev/null | where name =~ "test.gif")
    
    if ($test_files | length) > 0 {
        for file in $test_files {
            print $"Removing: ($file.name)"
            rm $file.name
        }
        print $"(ansi green)✅ Cleaned up ($test_files | length) test files(ansi reset)"
    } else {
        print "No test files found to clean in assets/font-tests/"
    }
}

# Show help
def "main help" [] {
    print "🎨 Font Testing Script for VHS"
    print "Usage:"
    print "  nu test-fonts.nu        # Test all 15 fonts"
    print "  nu test-fonts.nu clean  # Clean up old test files"
    print "  nu test-fonts.nu help   # Show this help"
    print ""
    print "This script tests 15 different Nerd Fonts and creates separate GIF files for each."
    print "Each font gets its own output file in assets/font-tests/ (e.g., jetbrains-mono-test.gif, etc.)"
}

# Default action
def main [] {
    test_fonts
}