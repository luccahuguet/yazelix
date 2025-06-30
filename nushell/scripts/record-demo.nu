#!/usr/bin/env nu

# Record VHS demos with proper font support using nushell
def record_demo [demo_file: string, output_name: string, font_package?: string] {
    # Ensure output directory exists
    mkdir ($output_name | path dirname)

    if not ($demo_file | path exists) {
        print $"(ansi red)❌ Error: Demo file '($demo_file)' not found(ansi reset)"
        exit 1
    }

    # Default to hack nerd font if no font specified
    let font_pkg = if ($font_package | is-empty) { "nerd-fonts.hack" } else { $font_package }

    print $"(ansi blue)🎬 Recording demo: ($demo_file)(ansi reset)"
    print $"(ansi yellow)📝 Output will be: ($output_name)(ansi reset)"
    print $"(ansi cyan)🔤 Using font package: ($font_pkg)(ansi reset)"
    print ""
    print $"(ansi yellow)⏳ This may take a few minutes...(ansi reset)"

    # Record the demo using nix shell with specified font support
    let result = (do {
        nix shell $"nixpkgs#($font_pkg)" nixpkgs#vhs --command bash -c $"vhs '($demo_file)'"
    } | complete)

    if $result.exit_code == 0 {
        print ""
        print $"(ansi green)✅ Demo recorded successfully!(ansi reset)"
        print $"(ansi green)📹 Output: ($output_name)(ansi reset)"

        # Show file info if it exists
        if ($output_name | path exists) {
            let file_size = (ls $output_name | get size | first | into string)
            print $"(ansi blue)📊 File size: ($file_size)(ansi reset)"
        }
    } else {
        print ""
        print $"(ansi red)❌ Recording failed!(ansi reset)"
        print $"(ansi red)Error: ($result.stderr)(ansi reset)"
    }
}

# Quick demo recording
def "main quick" [font?: string] {
    let font_pkg = if ($font | is-empty) { "nerd-fonts.hack" } else { $font }
    record_demo "assets/demos/yazelix-v7-quick-demo.tape" "assets/demos/yazelix-v7-quick-demo.gif" $font_pkg
}

# Long demo recording
def "main long" [font?: string] {
    let font_pkg = if ($font | is-empty) { "nerd-fonts.hack" } else { $font }
    record_demo "assets/demos/yazelix-v7-demo.tape" "assets/demos/yazelix-v7-demo.gif" $font_pkg
}

# Custom demo recording
def "main custom" [demo_file: string, font?: string] {
    let output_name = ($demo_file | str replace ".tape" ".gif")
    let font_pkg = if ($font | is-empty) { "nerd-fonts.hack" } else { $font }
    record_demo $demo_file $output_name $font_pkg
}

# Show help
def "main help" [] {
    print "🎬 VHS Demo Recording Script"
    print "Usage:"
    print "  nu record-demo.nu quick [font]          # Record quick demo"
    print "  nu record-demo.nu long [font]           # Record long demo"
    print "  nu record-demo.nu custom <file> [font]  # Record custom demo"
    print "  nu record-demo.nu help                  # Show this help"
    print ""
    print "Font options (nixpkgs package names):"
    print "  nerd-fonts.hack           # Hack Nerd Font (default)"
    print "  nerd-fonts.jetbrains-mono # JetBrains Mono Nerd Font"
    print "  nerd-fonts.ubuntu-mono    # Ubuntu Mono Nerd Font"
    print "  nerd-fonts.fira-code      # Fira Code Nerd Font"
    print "  nerd-fonts.iosevka        # Iosevka Nerd Font"
    print "  jetbrains-mono            # JetBrains Mono (regular)"
    print "  ubuntu-mono               # Ubuntu Mono (regular)"
    print "  # Or any other nixpkgs font package"
    print ""
    print "Examples:"
    print "  nu record-demo.nu quick"
    print "  nu record-demo.nu quick nerd-fonts.ubuntu-mono"
    print "  nu record-demo.nu custom assets/demos/my-demo.tape nerd-fonts.jetbrains-mono"
    print "  nu record-demo.nu long jetbrains-mono"
}

# Default to help if no args
def main [] {
    main help
}