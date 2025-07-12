#!/usr/bin/env nu
# Main Yazelix environment setup script
# Called from flake.nix shellHook to reduce complexity

def main [
    yazelix_dir: string
    include_optional: bool
    build_helix_from_source: bool
    default_shell: string
    debug_mode: bool
    extra_shells_str: string
    skip_welcome_screen: bool
    helix_mode: string
] {
    # Parse extra shells from comma-separated string
    let extra_shells = if ($extra_shells_str | is-empty) or ($extra_shells_str == "NONE") {
        []
    } else {
        $extra_shells_str | split row "," | where $it != ""
    }

    # Determine which shells to configure (always nu/bash, plus default_shell and extra_shells)
    let shells_to_configure = (["nu", "bash"] ++ [$default_shell] ++ $extra_shells) | uniq

    # Setup logging
    let log_dir = $"($yazelix_dir)/logs"
    mkdir $log_dir

    # Auto-trim old logs (keep 10 most recent)
    let old_logs = try {
        ls $"($log_dir)/shellhook_*.log"
        | sort-by modified -r
        | skip 10
        | get name
    } catch { [] }

    if not ($old_logs | is-empty) {
        rm ...$old_logs
    }

    let log_file = $"($log_dir)/shellhook_(date now | format date '%Y%m%d_%H%M%S').log"

    print $"🚀 Yazelix Environment Setup Started"
    print $"📝 Logging to: ($log_file)"

    # Generate shell initializers for configured shells only
    print "🔧 Generating shell initializers..."
    nu $"($yazelix_dir)/nushell/scripts/setup/initializers.nu" $yazelix_dir $include_optional ($shells_to_configure | str join ",")



    # Setup Helix based on mode
    if $helix_mode == "source" {
        print "🔧 Using Helix flake from repository (always updated)..."
        # No setup needed - flake.nix handles this automatically
    } else if $helix_mode == "release" {
        print "✅ Using latest Helix release from nixpkgs (no custom build needed)"


    } else {
        print "✅ Using default nixpkgs Helix (no custom build needed)"
    }

    # Setup shell configurations (always setup bash/nu, conditionally setup fish/zsh)
    setup_bash_config $yazelix_dir
    setup_nushell_config $yazelix_dir

    if ("fish" in $shells_to_configure) {
        setup_fish_config $yazelix_dir
    }

    if ("zsh" in $shells_to_configure) {
        setup_zsh_config $yazelix_dir
    }

    # Setup editor
    setup_helix_config ($helix_mode != "default") $yazelix_dir

    # Set permissions
    chmod +x $"($yazelix_dir)/bash/start_yazelix.sh"
    chmod +x $"($yazelix_dir)/nushell/scripts/launch_yazelix.nu"
    chmod +x $"($yazelix_dir)/nushell/scripts/start_yazelix.nu"

    print "✅ Yazelix environment setup complete!"

    # Prepare welcome message
    let helix_info = if $helix_mode == "source" {
        $"   🔄 Using Helix flake from repository for latest features"
    } else if $helix_mode == "release" {
        "   📦 Using latest Helix release from nixpkgs (fast setup)"


    } else {
        $"   📝 Using stable nixpkgs Helix"
    }

    let welcome_message = [
        "",
        "🎉 Welcome to Yazelix v7!",
        "   Your integrated terminal environment with Yazi + Zellij + Helix",
        "   ✨ Now with Nix auto-setup, lazygit, Starship, and markdown-oxide",
        $helix_info,
        "   🔧 All dependencies installed, shell configs updated, tools ready",
        "",
        "   Quick tips: Use 'alt hjkl' to navigate, 'Enter' in Yazi to open files",
        ""
    ] | where $it != ""

    # Show welcome screen or log it
    if $skip_welcome_screen {
        # Log welcome info instead of displaying it
        let welcome_log_file = $"($log_dir)/welcome_(date now | format date '%Y%m%d_%H%M%S').log"
        $welcome_message | str join "\n" | save $welcome_log_file
        print $"💡 Welcome screen skipped. Welcome info logged to: ($welcome_log_file)"
    } else {
        # Display welcome screen with pause
        for $line in $welcome_message {
            print $line
        }
        input "   Press Enter to launch Zellij and start your session... "
    }
}

def setup_bash_config [yazelix_dir: string] {
    use ../utils/constants.nu *
    
    let bash_config = $"($yazelix_dir)/bash/yazelix_bash_config.sh"
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
    
    let fish_config = ($SHELL_CONFIGS | get fish | str replace "~" $env.HOME)
    let yazelix_config = $"($yazelix_dir)/fish/yazelix_fish_config.fish"
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
    
    let zsh_config = ($SHELL_CONFIGS | get zsh | str replace "~" $env.HOME)
    let yazelix_config = $"($yazelix_dir)/zsh/yazelix_zsh_config.zsh"
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

def setup_helix_config [use_custom_helix: bool = false, yazelix_dir: string = ""] {
    let editor = if $use_custom_helix and ($yazelix_dir != "") {
        let custom_hx = $"($yazelix_dir)/helix_custom/target/release/hx"
        if ($custom_hx | path exists) {
            print $"📝 Using custom-built Helix: ($custom_hx)"
            $custom_hx
        } else {
            print $"⚠️  Custom Helix not found, falling back to system hx"
            "hx"
        }
    } else {
        "hx"
    }

    print $"📝 Setting EDITOR to: ($editor)"
    $env.EDITOR = $editor

    # Create hx alias for custom build if available
    if $use_custom_helix and ($yazelix_dir != "") {
        let custom_hx = $"($yazelix_dir)/helix_custom/target/release/hx"
        if ($custom_hx | path exists) {
            # This will be picked up by shell configs
            $env.YAZELIX_CUSTOM_HELIX = $custom_hx
        }
    }
}



