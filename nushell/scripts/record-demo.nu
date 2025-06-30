#!/usr/bin/env nu

# Record VHS demos with proper font support using nushell
def record_demo [demo_file: string, output_name: string, font_package?: string] {
    # Ensure output directory exists
    mkdir ($output_name | path dirname)

    if not ($demo_file | path exists) {
        print $"(ansi red)❌ Error: Demo file '($demo_file)' not found(ansi reset)"
        exit 1
    }

    # Use specified font package or no font (system default)
    let font_pkg = $font_package

    print $"(ansi blue)🎬 Recording demo: ($demo_file)(ansi reset)"
    print $"(ansi yellow)📝 Output will be: ($output_name)(ansi reset)"
    let font_display = if ($font_pkg | is-empty) { "system default" } else { $font_pkg }
    print $"(ansi cyan)🔤 Using font: ($font_display)(ansi reset)"
    print ""
    print $"(ansi yellow)⏳ This may take a few minutes...(ansi reset)"

        # Record the demo using nix shell with font support (if specified)
    let result = if ($font_pkg | is-empty) {
        # No font specified - use system default
        (do { 
            nix shell nixpkgs#vhs --command bash -c $"vhs '($demo_file)'"
        } | complete)
    } else {
        # Use specified font
        (do { 
            nix shell $"nixpkgs#($font_pkg)" nixpkgs#vhs --command bash -c $"vhs '($demo_file)'"
        } | complete)
    }

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
    record_demo "assets/demos/yazelix-v7-quick-demo.tape" "assets/demos/yazelix-v7-quick-demo.gif" $font
}

# Long demo recording  
def "main long" [font?: string] {
    record_demo "assets/demos/yazelix-v7-demo.tape" "assets/demos/yazelix-v7-demo.gif" $font
}

# Custom demo recording
def "main custom" [demo_file: string, font?: string] {
    # Extract just the filename and save to demos folder
    let filename = ($demo_file | path basename | str replace ".tape" ".gif")
    let output_name = $"assets/demos/($filename)"
    record_demo $demo_file $output_name $font
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
    print "  (none)                    # System default font (default)"
    print "  nerd-fonts.hack           # Hack Nerd Font"
    print "  nerd-fonts.jetbrains-mono # JetBrains Mono Nerd Font"
    print "  nerd-fonts.ubuntu-mono    # Ubuntu Mono Nerd Font"
    print "  nerd-fonts.fira-code      # Fira Code Nerd Font"
    print "  nerd-fonts.iosevka        # Iosevka Nerd Font"
    print "  jetbrains-mono            # JetBrains Mono (regular)"
    print "  ubuntu-mono               # Ubuntu Mono (regular)"
    print "  # Or any other nixpkgs font package"
    print ""
    print "Examples:"
    print "  nu record-demo.nu quick                      # Use system default font"
    print "  nu record-demo.nu quick nerd-fonts.ubuntu-mono"
    print "  nu record-demo.nu custom assets/demos/my-demo.tape nerd-fonts.jetbrains-mono"
    print "  nu record-demo.nu long jetbrains-mono"
    print ""
    print "Note: All outputs are saved to assets/demos/ by default"
}

# Default to help if no args
def main [] {
    main help
}