#!/usr/bin/env nu
# Main Yazelix environment setup script
# Called from flake.nix shellHook to reduce complexity

def main [
    yazelix_dir: string
    recommended: bool
    build_helix_from_source: bool
    default_shell: string
    debug_mode: bool
    extra_shells_str: string
    skip_welcome_screen: bool
    helix_mode: string
    ascii_art_mode: string
    show_macchina_on_welcome: bool = false
] {
    # Import constants and utility functions
    use ../utils/constants.nu *
    use ../utils/common.nu detect_environment
    use ../utils/config_manager.nu get_yazelix_section_content

    # Detect environment first
    let env_info = (detect_environment)
    if $debug_mode {
        print $"🔍 Environment detection: ($env_info)"
    }

    # Handle different environment types
    match $env_info.environment_type {
        "home-manager" => {
            if $debug_mode {
                print "🏠 Home-manager environment detected - using read-only config approach"
            }
        }
        "read-only" => {
            print "⚠️  WARNING: Read-only configuration directory detected!"
            print "   This may indicate a managed environment or permission issue."
            print "   If using home-manager, see docs/home_manager_integration.md"
            print "   Some features may not work correctly."
        }
        "standard" => {
            # Auto-create yazelix.nix in standard environments (preserve existing behavior)
            let user_config = $"($yazelix_dir)/yazelix.nix"
            let default_config = $"($yazelix_dir)/yazelix_default.nix"
            
            if not ($user_config | path exists) and ($default_config | path exists) {
                try {
                    cp $default_config $user_config
                    print "📋 Created yazelix.nix from template. Customize it for your needs!"
                } catch {|err|
                    print $"⚠️  Could not create yazelix.nix: ($err.msg)"
                }
            }
        }
    }

    # Validate user config against schema
    use ../utils/config_schema.nu validate_config_against_default
    
    # Parse extra shells from comma-separated string
    let extra_shells = if ($extra_shells_str | is-empty) or ($extra_shells_str == "NONE") {
        []
    } else {
        $extra_shells_str | split row "," | where $it != ""
    }

    # Determine which shells to configure (always nu/bash, plus default_shell and extra_shells)
    let shells_to_configure = (["nu", "bash"] ++ [$default_shell] ++ $extra_shells) | uniq

    # Setup logging in state directory (XDG-compliant)
    let state_dir = ($YAZELIX_STATE_DIR | str replace "~" $env.HOME)
    let log_dir = ($YAZELIX_LOGS_DIR | str replace "~" $env.HOME)
    mkdir $state_dir
    mkdir $log_dir

    # Auto-trim old logs (keep 10 most recent)
    let old_shellhook_logs = try {
        ls $"($log_dir)/shellhook_*.log"
        | sort-by modified -r
        | skip 10
        | get name
    } catch { [] }

    let old_welcome_logs = try {
        ls $"($log_dir)/welcome_*.log"
        | sort-by modified -r
        | skip 10
        | get name
    } catch { [] }

    let all_old_logs = ($old_shellhook_logs | append $old_welcome_logs)

    if not ($all_old_logs | is-empty) {
        rm ...$all_old_logs
    }

    let log_file = $"($log_dir)/shellhook_(date now | format date '%Y%m%d_%H%M%S').log"

    print $"📝 Logging to: ($log_file)"


    # Generate shell initializers for configured shells only
    nu $"($yazelix_dir)/nushell/scripts/setup/initializers.nu" $yazelix_dir $recommended ($shells_to_configure | str join ",")

    # Setup shell configurations (always setup bash/nu, conditionally setup fish/zsh)
    setup_bash_config $yazelix_dir
    setup_nushell_config $yazelix_dir

    if ("fish" in $shells_to_configure) {
        setup_fish_config $yazelix_dir
    }

    if ("zsh" in $shells_to_configure) {
        setup_zsh_config $yazelix_dir
    }

    # Editor setup is now handled in the shellHook

    # Set permissions
    chmod +x $"($yazelix_dir)/shells/bash/start_yazelix.sh"
    chmod +x $"($yazelix_dir)/nushell/scripts/core/launch_yazelix.nu"
    chmod +x $"($yazelix_dir)/nushell/scripts/core/start_yazelix.nu"

    print "✅ Yazelix environment setup complete!"

    # Import ASCII art module
    use ../utils/ascii_art.nu *

    # Show ASCII art based on configuration
    if not $skip_welcome_screen {
        if $ascii_art_mode == "animated" {
            # Play animated ASCII art
            print ""
           play_animation 0.5sec
        } else if $ascii_art_mode == "static" {
            # Show static ASCII art
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

    # Get color scheme for consistent styling
    let colors = get_yazelix_colors

    # Get flake.nix last updated date dynamically (in days ago, using into duration)
    let flake_path = $"($yazelix_dir)/flake.nix"
    let flake_days_ago = if ($flake_path | path exists) {
        let last_mod = (ls $flake_path | get modified | first | into datetime)
        let now = (date now)
        let diff = ($now - $last_mod | into duration)
        let days = ($diff / 1day | math floor)
        $days
    } else {
        null
    }
    let flake_info = if ($flake_days_ago | describe) == 'int' {
        $"($colors.cyan)🕒 Flake last updated: ($flake_days_ago) day\(s\) ago($colors.reset)"
    } else if ($flake_days_ago | describe) == 'string' and ($flake_days_ago | is-empty) == false {
        $"($colors.cyan)🕒 Flake last updated: ($flake_days_ago) day\(s\) ago($colors.reset)"
    } else {
        $"($colors.cyan)🕒 Flake last updated: unknown($colors.reset)"
    }

    # Prepare welcome message with consistent colors
    let helix_info = if $helix_mode == "source" {
        $"($colors.cyan)🔄 Using Helix flake from repository for latest features($colors.reset)"
    } else if $helix_mode == "release" {
        $"($colors.cyan)📦 Using latest Helix release from nixpkgs \(fast setup\)($colors.reset)"
    } else {
        $"($colors.cyan)📝 Using stable nixpkgs Helix($colors.reset)"
    }

    # Get ASCII art
    let ascii_art = get_welcome_ascii_art

    let welcome_message = [
        "",
        $"($colors.purple)🎉 Welcome to Yazelix v7.5!($colors.reset)",
        $"($colors.blue)Your integrated terminal environment with Yazi + Zellij + Helix($colors.reset)",
        $flake_info,
        $"($colors.cyan)✨ Now with Nix auto-setup, lazygit, Starship, and markdown-oxide($colors.reset)",
        $helix_info,
        $"($colors.cyan)💡 Quick tips: Use 'alt hjkl' to navigate, 'Enter' in Yazi to open files($colors.reset)"
    ] | where $it != ""

    # Show welcome screen or log it
    if $skip_welcome_screen {
        # Log welcome info instead of displaying it
        let welcome_log_file = $"($log_dir)/welcome_(date now | format date '%Y%m%d_%H%M%S').log"
        $welcome_message | str join "\n" | save $welcome_log_file
        print $"($colors.cyan)💡 Welcome screen skipped. Welcome info logged to: ($welcome_log_file)($colors.reset)"
    } else {
        # Display the rest of the welcome message (animation already played above)
        for $line in $welcome_message {
            print $line
        }
        input $"($colors.purple)Press Enter to launch Zellij and start your session... ($colors.reset)"
    }
}

def setup_bash_config [yazelix_dir: string] {
    use ../utils/constants.nu *
    use ../utils/config_manager.nu get_yazelix_section_content

    let bash_config = $"($yazelix_dir)/shells/bash/yazelix_bash_config.sh"
    let bashrc = ($SHELL_CONFIGS | get bash | str replace "~" $env.HOME)
    let section_content = get_yazelix_section_content "bash" $yazelix_dir

    if not ($bash_config | path exists) {
        print $"⚠️  Bash config not found: ($bash_config)"
        return
    }

    touch $bashrc
    let bashrc_content = (open $bashrc)

    # Check if yazelix section already exists
    if ($bashrc_content | str contains $YAZELIX_START_MARKER) {
        print $"✅ Bash config already sourced"
        return
    }

    print $"🐚 Adding Yazelix Bash config to ($bashrc)"
    $"\n\n($section_content)" | save --append $bashrc
}

def setup_nushell_config [yazelix_dir: string] {
    use ../utils/constants.nu *
    use ../utils/config_manager.nu get_yazelix_section_content

    let nushell_config = ($SHELL_CONFIGS | get nushell | str replace "~" $env.HOME)
    let yazelix_config = $"($yazelix_dir)/nushell/config/config.nu"
    let section_content = get_yazelix_section_content "nushell" $yazelix_dir

    mkdir ($nushell_config | path dirname)

    if not ($nushell_config | path exists) {
        print $"📝 Creating new Nushell config: ($nushell_config)"
        "# Nushell user configuration (created by Yazelix setup)" | save $nushell_config
    }

    let config_content = (open $nushell_config)

    # Check if yazelix section already exists
    if ($config_content | str contains $YAZELIX_START_MARKER) {
        print $"✅ Nushell config already sourced"
        return
    }

    print $"🐚 Adding Yazelix Nushell config to ($nushell_config)"
    $"\n\n($section_content)" | save --append $nushell_config
}

def setup_fish_config [yazelix_dir: string] {
    use ../utils/constants.nu *
    use ../utils/config_manager.nu get_yazelix_section_content

    let fish_config = ($SHELL_CONFIGS | get fish | str replace "~" $env.HOME)
    let yazelix_config = $"($yazelix_dir)/shells/fish/yazelix_fish_config.fish"
    let section_content = get_yazelix_section_content "fish" $yazelix_dir

    if not ($yazelix_config | path exists) {
        print $"⚠️  Fish config not found, skipping Fish setup"
        return
    }

    mkdir ($fish_config | path dirname)
    touch $fish_config
    let config_content = (open $fish_config)

    # Check if yazelix section already exists
    if ($config_content | str contains $YAZELIX_START_MARKER) {
        print $"✅ Fish config already sourced"
        return
    }

    print $"🐚 Adding Yazelix Fish config to ($fish_config)"
    $"\n\n($section_content)" | save --append $fish_config
}

def setup_zsh_config [yazelix_dir: string] {
    use ../utils/constants.nu *
    use ../utils/config_manager.nu get_yazelix_section_content

    let zsh_config = ($SHELL_CONFIGS | get zsh | str replace "~" $env.HOME)
    let yazelix_config = $"($yazelix_dir)/shells/zsh/yazelix_zsh_config.zsh"
    let section_content = get_yazelix_section_content "zsh" $yazelix_dir

    if not ($yazelix_config | path exists) {
        print $"⚠️  Zsh config not found, skipping Zsh setup"
        return
    }

    mkdir ($zsh_config | path dirname)
    touch $zsh_config
    let config_content = (open $zsh_config)

    # Check if yazelix section already exists
    if ($config_content | str contains $YAZELIX_START_MARKER) {
        print $"✅ Zsh config already sourced"
        return
    }

    print $"🐚 Adding Yazelix Zsh config to ($zsh_config)"
    $"\n\n($section_content)" | save --append $zsh_config
}





