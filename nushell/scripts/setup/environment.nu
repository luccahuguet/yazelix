#!/usr/bin/env nu
# Main Yazelix environment setup script
# Called from flake.nix shellHook to reduce complexity

def main [
    yazelix_dir: string
    include_optional: bool
    build_helix_from_source: bool
    default_shell: string
    debug_mode: bool
] {
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

    # Generate shell initializers
    print "🔧 Generating shell initializers..."
    nu $"($yazelix_dir)/nushell/scripts/setup/initializers.nu" $yazelix_dir $include_optional

    # Setup shell configurations
    setup_bash_config $yazelix_dir
    setup_nushell_config $yazelix_dir
    setup_fish_config $yazelix_dir
    setup_zsh_config $yazelix_dir

    # Setup editor
    setup_helix_config

    # Set permissions
    chmod +x $"($yazelix_dir)/bash/launch-yazelix.sh"
    chmod +x $"($yazelix_dir)/bash/start-yazelix.sh"

    print "✅ Yazelix environment setup complete!"

    # Welcome screen with pause
    print ""
    print "🎉 Welcome to Yazelix v7!"
    print "   Your integrated terminal environment with Yazi + Zellij + Helix"
    print "   ✨ Now with Nix auto-setup, lazygit, Starship, and markdown-oxide"
    print "   🔧 All dependencies installed, shell configs updated, tools ready"
    print ""
    print "   Quick tips: Use 'alt hjkl' to navigate, 'Enter' in Yazi to open files"
    print ""
    input "   Press Enter to launch Zellij and start your session... "
}

def setup_bash_config [yazelix_dir: string] {
    let bash_config = $"($yazelix_dir)/bash/yazelix_bash_config.sh"
    let bashrc = $"($env.HOME)/.bashrc"
    let comment = "# Source Yazelix Bash configuration (added by Yazelix)"
    let source_line = $"source \"($bash_config)\""

    if not ($bash_config | path exists) {
        print $"⚠️  Bash config not found: ($bash_config)"
        return
    }

    touch $bashrc
    let bashrc_content = (open $bashrc)

    if not ($bashrc_content | str contains $comment) {
        print $"🐚 Adding Yazelix Bash config to ($bashrc)"
        $"\n($comment)\n($source_line)" | save --append $bashrc
    } else {
        print $"✅ Bash config already sourced"
    }
}

def setup_nushell_config [yazelix_dir: string] {
    let nushell_config = $"($env.HOME)/.config/nushell/config.nu"
    let yazelix_config = $"($yazelix_dir)/nushell/config/config.nu"
    let comment = "# Source Yazelix Nushell configuration (added by Yazelix)"
    let source_line = $"source \"($yazelix_config)\""

    mkdir ($nushell_config | path dirname)

    if not ($nushell_config | path exists) {
        print $"📝 Creating new Nushell config: ($nushell_config)"
        "# Nushell user configuration (created by Yazelix setup)" | save $nushell_config
    }

    let config_content = (open $nushell_config)

    if not ($config_content | str contains $comment) {
        print $"🐚 Adding Yazelix Nushell config to ($nushell_config)"
        $"\n($comment)\n($source_line)" | save --append $nushell_config
    } else {
        print $"✅ Nushell config already sourced"
    }
}

def setup_fish_config [yazelix_dir: string] {
    let fish_config = $"($env.HOME)/.config/fish/config.fish"
    let yazelix_config = $"($yazelix_dir)/fish/yazelix_fish_config.fish"
    let comment = "# Source Yazelix Fish configuration (added by Yazelix)"
    let source_line = $"source \"($yazelix_config)\""

    if not ($yazelix_config | path exists) {
        print $"⚠️  Fish config not found, skipping Fish setup"
        return
    }

    mkdir ($fish_config | path dirname)
    touch $fish_config
    let config_content = (open $fish_config)

    if not ($config_content | str contains $comment) {
        print $"🐚 Adding Yazelix Fish config to ($fish_config)"
        $"\n($comment)\n($source_line)" | save --append $fish_config
    } else {
        print $"✅ Fish config already sourced"
    }
}

def setup_zsh_config [yazelix_dir: string] {
    let zsh_config = $"($env.HOME)/.zshrc"
    let yazelix_config = $"($yazelix_dir)/zsh/yazelix_zsh_config.zsh"
    let comment = "# Source Yazelix Zsh configuration (added by Yazelix)"
    let source_line = $"source \"($yazelix_config)\""

    if not ($yazelix_config | path exists) {
        print $"⚠️  Zsh config not found, skipping Zsh setup"
        return
    }

    mkdir ($zsh_config | path dirname)
    touch $zsh_config
    let config_content = (open $zsh_config)

    if not ($config_content | str contains $comment) {
        print $"🐚 Adding Yazelix Zsh config to ($zsh_config)"
        $"\n($comment)\n($source_line)" | save --append $zsh_config
    } else {
        print $"✅ Zsh config already sourced"
    }
}

def setup_helix_config [] {
    # Detect available Helix binary
    let editor = if (which helix | is-not-empty) {
        "helix"
    } else {
        "hx"
    }

    print $"📝 Setting EDITOR to: ($editor)"
    $env.EDITOR = $editor
}