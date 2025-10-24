#!/usr/bin/env nu
# Welcome Message Module
# Handles ASCII art display and welcome message generation

use ../utils/ascii_art.nu *
use ../utils/config_parser.nu parse_yazelix_config

# Show ASCII art based on mode (animated or static)
export def show_ascii_art [
    ascii_art_mode: string
    show_macchina_on_welcome: bool
]: nothing -> nothing {
    if $ascii_art_mode == "animated" {
        print ""
        play_animation 0.5sec
    } else if $ascii_art_mode == "static" {
        let ascii_art = get_welcome_ascii_art
        for $line in $ascii_art {
            print $line
        }
        print ""
    }

    # Show macchina if enabled and available
    if $show_macchina_on_welcome {
        macchina -o machine -o distribution -o desktop-environment -o processor -o gpu -o terminal
    }
}

# Get flake last updated info
export def get_flake_info [yazelix_dir: string, colors: record]: nothing -> string {
    let flake_path = $"($yazelix_dir)/flake.nix"
    let flake_days_ago = if ($flake_path | path exists) {
        try {
            let file_info = (ls $flake_path | first)
            let now_seconds = (date now | format date %s | into int)
            let mod_seconds = ($file_info.modified | format date %s | into int)
            let diff_seconds = ($now_seconds - $mod_seconds)
            ($diff_seconds / 86400 | math floor)
        } catch {
            0
        }
    } else {
        0
    }

    if ($flake_days_ago | describe) == 'int' {
        $"($colors.cyan)🕒 Flake last updated: ($flake_days_ago) day\(s\) ago($colors.reset)"
    } else {
        $"($colors.cyan)🕒 Flake last updated: unknown($colors.reset)"
    }
}

# Get Helix mode info
export def get_helix_info [helix_mode: string, colors: record]: nothing -> string {
    if $helix_mode == "source" {
        $"($colors.cyan)🔄 Using Helix flake from repository for latest features($colors.reset)"
    } else if $helix_mode == "release" {
        $"($colors.cyan)📦 Using latest Helix release from nixpkgs \(fast setup\)($colors.reset)"
    } else {
        $"($colors.cyan)📝 Using stable nixpkgs Helix($colors.reset)"
    }
}

# Get persistent session info
export def get_session_info [colors: record]: nothing -> string {
    try {
        let config = parse_yazelix_config
        if ($config.persistent_sessions == "true") {
            $"($colors.green)🔗 Using persistent session: ($config.session_name)($colors.reset)"
        } else {
            $"($colors.yellow)🆕 Creating new Zellij session($colors.reset)"
        }
    } catch {
        $"($colors.yellow)🆕 Creating new Zellij session($colors.reset)"
    }
}

# Get terminal info
export def get_terminal_info [colors: record]: nothing -> string {
    try {
        let config = parse_yazelix_config
        if ($config.include_terminal == "true") and ((which yazelix-ghostty | length) > 0) {
            $"($colors.green)🖥️  Using yazelix included terminal \(Ghostty with GPU acceleration\)($colors.reset)"
        } else {
            $"($colors.cyan)🖥️  Using external terminal: ($config.preferred_terminal)($colors.reset)"
        }
    } catch {
        # Fallback: check if we have yazelix-ghostty but no config
        if (which yazelix-ghostty | length) > 0 {
            $"($colors.green)🖥️  Using yazelix included terminal \(Ghostty with GPU acceleration\)($colors.reset)"
        } else {
            $"($colors.cyan)🖥️  Using external terminal \(configuration not found\)($colors.reset)"
        }
    }
}

# Build complete welcome message
export def build_welcome_message [
    yazelix_dir: string
    helix_mode: string
    colors: record
]: nothing -> list<string> {
    let flake_info = get_flake_info $yazelix_dir $colors
    let helix_info = get_helix_info $helix_mode $colors
    let session_info = get_session_info $colors
    let terminal_info = get_terminal_info $colors

    [
        "",
        $"($colors.purple)🎉 Welcome to Yazelix v9!($colors.reset)",
        $"($colors.blue)Lots of polish, support for any editor, home-manager config, better zellij tab navigation, persistent sessions and more!($colors.reset)",
        $flake_info,
        $"($colors.cyan)✨ Now with Nix auto-setup, lazygit, Starship, and markdown-oxide($colors.reset)",
        $helix_info,
        $session_info,
        $terminal_info,
        $"($colors.cyan)💡 Quick tips: Use 'alt hjkl' to navigate, 'Enter' in Yazi to open files, 'Alt [' or 'Alt ]' to swap layouts($colors.reset)"
    ] | where $it != ""
}

# Display welcome screen or log it based on mode
export def show_welcome [
    skip_welcome_screen: bool
    quiet_mode: bool
    ascii_art_mode: string
    show_macchina_on_welcome: bool
    welcome_message: list<string>
    log_dir: string
    colors: record
]: nothing -> nothing {
    # Check modes
    let env_only_mode = ($env.YAZELIX_ENV_ONLY? == "true")
    let test_mode = ($env.YAZELIX_SKIP_WELCOME? == "true")
    let should_skip_welcome = $skip_welcome_screen or $env_only_mode or $test_mode

    # Show ASCII art first (if not skipping)
    if (not $should_skip_welcome) and (not $quiet_mode) {
        show_ascii_art $ascii_art_mode $show_macchina_on_welcome
    }

    # Show welcome or log it
    if $should_skip_welcome {
        if $env_only_mode {
            print $"($colors.cyan)🔧 Yazelix environment loaded! All tools are available in your current shell.($colors.reset)"
            print $"($colors.cyan)💡 Use 'yzx start' or 'yzx launch' to open the full Yazelix interface when needed.($colors.reset)"
        } else if $test_mode {
            print $"($colors.cyan)🧪 Yazelix test mode - Welcome screen skipped($colors.reset)"
        } else {
            # Log welcome info
            let welcome_log_file = $"($log_dir)/welcome_(date now | format date '%Y%m%d_%H%M%S').log"
            $welcome_message | str join "\n" | save $welcome_log_file
            print $"($colors.cyan)💡 Welcome screen skipped. Welcome info logged to: ($welcome_log_file)($colors.reset)"
        }
    } else {
        # Display welcome message
        for $line in $welcome_message {
            print $line
        }

        # Prompt for enter (if interactive)
        try {
            input $"($colors.purple)Press Enter to launch Zellij and start your session... ($colors.reset)"
        } catch {
            # Non-interactive shell, just continue
        }
        print $"($colors.purple)Launching Zellij...($colors.reset)"
    }
}
