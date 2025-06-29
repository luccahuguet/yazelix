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
    use_patchy_helix: bool
    patchy_pull_requests: string
    patchy_patches: string
    patchy_pin_commits: bool
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

    # Setup patchy Helix if enabled
    if $use_patchy_helix {
        print "🔧 Setting up patchy Helix with community PRs..."
        setup_patchy_helix $yazelix_dir $patchy_pull_requests $patchy_patches $patchy_pin_commits
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
    setup_helix_config $use_patchy_helix $yazelix_dir

    # Set permissions
    chmod +x $"($yazelix_dir)/bash/launch-yazelix.sh"
    chmod +x $"($yazelix_dir)/bash/start-yazelix.sh"

    print "✅ Yazelix environment setup complete!"

    # Prepare welcome message
    let patchy_info = if $use_patchy_helix {
        let pr_count = if ($patchy_pull_requests | is-empty) or ($patchy_pull_requests == "NONE") { 0 } else { ($patchy_pull_requests | split row "," | length) }
        $"   🧩 Patchy Helix enabled with ($pr_count) community PRs for enhanced features"
    } else { "" }
    
    let welcome_message = [
        "",
        "🎉 Welcome to Yazelix v7!",
        "   Your integrated terminal environment with Yazi + Zellij + Helix",
        "   ✨ Now with Nix auto-setup, lazygit, Starship, and markdown-oxide",
        $patchy_info,
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

def setup_helix_config [use_patchy: bool = false, yazelix_dir: string = ""] {
    let editor = if $use_patchy and ($yazelix_dir != "") {
        let patchy_hx = $"($yazelix_dir)/helix_patchy/target/release/hx"
        if ($patchy_hx | path exists) {
            print $"📝 Using patchy-built Helix: ($patchy_hx)"
            $patchy_hx
        } else {
            print $"⚠️  Patchy Helix not found, falling back to system hx"
            "hx"
        }
    } else {
        "hx"
    }
    
    print $"📝 Setting EDITOR to: ($editor)"
    $env.EDITOR = $editor
    
    # Create hx alias for patchy if available
    if $use_patchy and ($yazelix_dir != "") {
        let patchy_hx = $"($yazelix_dir)/helix_patchy/target/release/hx"
        if ($patchy_hx | path exists) {
            # This will be picked up by shell configs
            $env.YAZELIX_PATCHY_HX = $patchy_hx
        }
    }
}

def setup_patchy_helix [
    yazelix_dir: string
    pull_requests_str: string
    patches_str: string
    pin_commits: bool
] {
    let helix_patchy_dir = $"($yazelix_dir)/helix_patchy"
    let patchy_config_dir = $"($helix_patchy_dir)/.patchy"
    
    # Create helix-patchy directory if it doesn't exist
    if not ($helix_patchy_dir | path exists) {
        print $"📂 Creating patchy Helix directory: ($helix_patchy_dir)"
        mkdir $helix_patchy_dir
        
        # Initialize git repo and add helix remote
        cd $helix_patchy_dir
        git init
        git remote add origin https://github.com/helix-editor/helix.git
        print "🔄 Fetching Helix repository (this may take a moment)..."
        git fetch origin master
        git checkout -b patchy origin/master
    }
    
    # Create .patchy directory
    mkdir $patchy_config_dir
    
    # Parse pull requests and patches
    let pull_requests = if ($pull_requests_str | is-empty) or ($pull_requests_str == "NONE") { 
        [] 
    } else { 
        $pull_requests_str | split row "," | where $it != "" 
    }
    
    let patches = if ($patches_str | is-empty) or ($patches_str == "NONE") { 
        [] 
    } else { 
        $patches_str | split row "," | where $it != "" 
    }
    
    # Generate patchy config.toml
    let config_content = [
        "# Patchy configuration for Yazelix Helix build"
        "# Auto-generated from yazelix.nix configuration"
        ""
        "# Main repository to fetch from"
        "repo = \"helix-editor/helix\""
        ""
        "# The repository's branch"
        "remote-branch = \"master\""
        ""
        "# This is the branch where patchy will merge all PRs"
        "local-branch = \"patchy\""
        ""
        "# List of pull requests to merge"
        $"pull-requests = [($pull_requests | each {|pr| $'  "($pr)"'} | str join ',\n')]"
        ""
        "# List of patches to apply"
        $"patches = [($patches | each {|patch| $'  "($patch)"'} | str join ',\n')]"
    ]
    
    $config_content | str join "\n" | save -f $"($patchy_config_dir)/config.toml"
    
    print $"✅ Generated patchy config with ($pull_requests | length) PRs and ($patches | length) patches"
    
    # Run patchy to merge PRs
    cd $helix_patchy_dir
    if (which patchy | is-not-empty) {
        print "🔄 Running patchy to merge pull requests..."
        try {
            patchy run
            print "✅ Patchy completed successfully!"
            
            # Automatically build patchy Helix after successful merge
            print "🔨 Building patchy Helix (this may take a few minutes)..."
            try {
                cargo build --release
                print "✅ Patchy Helix built successfully!"
                print $"🎯 Helix binary available at: ($helix_patchy_dir)/target/release/hx"
                
                # Create symlink for user helix config to be accessible
                let user_helix_config = $"($env.HOME)/.config/helix"
                let user_helix_runtime = $"($user_helix_config)/runtime"
                let patchy_runtime = $"($helix_patchy_dir)/runtime"
                
                mkdir $user_helix_config
                
                # Remove existing runtime link/dir if it exists
                if ($user_helix_runtime | path exists) {
                    rm -rf $user_helix_runtime
                }
                
                # Create symlink from user config to patchy runtime
                try {
                    ln -sf $patchy_runtime $user_helix_runtime
                    print $"🔗 Created runtime symlink: ($user_helix_runtime) -> ($patchy_runtime)"
                } catch {
                    print $"⚠️  Could not create runtime symlink, you may need to set HELIX_RUNTIME manually"
                }
            } catch {|build_err|
                print $"⚠️  Failed to build patchy Helix: ($build_err.msg)"
                print "   You can build manually with: cargo build --release"
                print $"   Navigate to: ($helix_patchy_dir)"
            }
        } catch {|err|
            print $"⚠️  Patchy encountered issues: ($err.msg)"
            print "   You may need to resolve merge conflicts manually"
            print $"   Navigate to: ($helix_patchy_dir)"
        }
    } else {
        print "⚠️  Patchy command not found, skipping PR merge"
        print "   PRs will be available for manual merging when patchy is installed"
    }
}