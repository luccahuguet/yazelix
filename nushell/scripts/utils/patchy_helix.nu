#!/usr/bin/env nu
# Yazelix Patchy Helix Management Utility
# Helps manage community PRs for Helix editor

def main [action?: string] {
    let yazelix_dir = $env.HOME | path join ".config" "yazelix"
    let yazelix_config = $yazelix_dir | path join "yazelix.nix"
    let helix_patchy_dir = $yazelix_dir | path join "helix_patchy"
    
    if $action == null {
        show_help
        return
    }
    
    match $action {
        "status" => { show_status $yazelix_config $helix_patchy_dir }
        "list" => { list_prs $yazelix_config }
        "add" => { 
            print "Usage: nu patchy_helix.nu add <PR_NUMBER>"
            print "Example: nu patchy_helix.nu add 12345"
        }
        "remove" => {
            print "Usage: nu patchy_helix.nu remove <PR_NUMBER>"
            print "Example: nu patchy_helix.nu remove 12345"
        }
        "sync" => { sync_prs $helix_patchy_dir }
        "clean" => { clean_patchy $helix_patchy_dir }
        _ => { show_help }
    }
}

def show_help [] {
    print "🧩 Yazelix Patchy Helix Management"
    print ""
    print "Commands:"
    print "  status  - Show current patchy configuration status"
    print "  list    - List currently configured PRs"
    print "  add     - Add a new PR (interactive)"
    print "  remove  - Remove a PR (interactive)"
    print "  sync    - Sync and rebuild PRs"
    print "  clean   - Clean patchy directory"
    print ""
    print "Examples:"
    print "  nu patchy_helix.nu status"
    print "  nu patchy_helix.nu list"
    print "  nu patchy_helix.nu sync"
}

def show_status [config_file: string, patchy_dir: string] {
    print "🧩 Patchy Helix Status"
    print "====================="
    
    if not ($config_file | path exists) {
        print "❌ yazelix.nix not found. Run Yazelix first to create it."
        return
    }
    
    # This is a simplified status check - in a real implementation,
    # we'd need to parse the Nix file properly
    let config_content = open $config_file
    let patchy_enabled = ($config_content | str contains "use_patchy_helix = true")
    
    if $patchy_enabled {
        print "✅ Patchy integration: ENABLED"
        
        if ($patchy_dir | path exists) {
            print $"✅ Patchy directory: ($patchy_dir)"
            
            cd $patchy_dir
            if (git status | complete | get exit_code) == 0 {
                let current_branch = (git branch --show-current | str trim)
                print $"🌿 Current branch: ($current_branch)"
                
                let last_commit = (git log -1 --oneline | str trim)
                print $"📝 Last commit: ($last_commit)"
            }
        } else {
            print "⚠️  Patchy directory not found - run Yazelix to initialize"
        }
    } else {
        print "❌ Patchy integration: DISABLED"
        print "   Enable in yazelix.nix: use_patchy_helix = true"
    }
}

def list_prs [config_file: string] {
    if not ($config_file | path exists) {
        print "❌ yazelix.nix not found"
        return
    }
    
    print "📋 Configured Pull Requests"
    print "============================="
    
    # This is a simplified PR listing - proper implementation would parse Nix
    let config_content = open $config_file
    let pr_lines = ($config_content | lines | where ($it | str contains '"') | where ($it | str contains "#"))
    
    if ($pr_lines | is-empty) {
        print "No PRs configured"
    } else {
        for $line in $pr_lines {
            print $line
        }
    }
    
    print ""
    print "💡 To modify PRs, edit yazelix.nix patchy_helix_config section"
}

def sync_prs [patchy_dir: string] {
    if not ($patchy_dir | path exists) {
        print "❌ Patchy directory not found. Run Yazelix first to initialize."
        return
    }
    
    print "🔄 Syncing Helix PRs..."
    cd $patchy_dir
    
    if (which patchy | is-not-empty) {
        try {
            patchy run
            print "✅ Successfully synced PRs!"
        } catch {|err|
            print $"❌ Sync failed: ($err.msg)"
            print "You may need to resolve conflicts manually"
        }
    } else {
        print "❌ Patchy command not found. Ensure it's installed via Nix."
    }
}

def clean_patchy [patchy_dir: string] {
    if not ($patchy_dir | path exists) {
        print "✅ Patchy directory doesn't exist - nothing to clean"
        return
    }
    
    print "🧹 This will delete the entire patchy directory and force recreation"
    let confirm = (input "Are you sure? (y/N): ")
    
    if ($confirm | str downcase) == "y" {
        rm -rf $patchy_dir
        print "✅ Patchy directory cleaned. Run Yazelix to recreate."
    } else {
        print "❌ Cancelled"
    }
} 