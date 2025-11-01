#!/usr/bin/env nu
# Yazelix Doctor - Health check utilities

use logging.nu log_to_file

# Check for conflicting Helix runtime directories based on Helix's search priority
export def check_helix_runtime_conflicts [] {
    # Helix runtime search order (highest to lowest priority):
    # 1. runtime/ sibling to $CARGO_MANIFEST_DIR (dev only - skip)
    # 2. ~/.config/helix/runtime (user config directory)  
    # 3. $HELIX_RUNTIME (yazelix sets this)
    # 4. Distribution fallback (compile-time)
    # 5. runtime/ sibling to helix executable
    
    mut conflicts = []
    mut has_high_priority_conflict = false
    
    # Check user config directory runtime (highest priority conflict)
    let user_runtime = "~/.config/helix/runtime" | path expand
    if ($user_runtime | path exists) {
        $conflicts = ($conflicts | append {
            path: $user_runtime
            priority: 2
            name: "User config runtime"
            severity: "error"
        })
        $has_high_priority_conflict = true
    }
    
    # Check executable sibling runtime (lower priority but still problematic)
    let helix_exe = try { (which hx | get path.0) } catch { null }
    if ($helix_exe | is-not-empty) {
        let exe_runtime = ($helix_exe | path dirname | path join "runtime")
        if ($exe_runtime | path exists) and ($exe_runtime != ($env.HELIX_RUNTIME? | default "")) {
            $conflicts = ($conflicts | append {
                path: $exe_runtime
                priority: 5
                name: "Executable sibling runtime"
                severity: "warning"
            })
        }
    }
    
    if ($conflicts | is-empty) {
        return {
            status: "ok"
            message: "No conflicting Helix runtime directories found"
            details: "HELIX_RUNTIME will be used as intended"
            fix_available: false
            conflicts: []
        }
    }
    
    # Determine overall status based on highest priority conflict
    let status = if $has_high_priority_conflict { "error" } else { "warning" }
    
    let conflict_details = ($conflicts | each { |c| 
        $"($c.name): ($c.path) \(priority ($c.priority)\)"
    } | str join ", ")
    
    let message = if $has_high_priority_conflict {
        "HIGH PRIORITY: ~/.config/helix/runtime will override HELIX_RUNTIME"
    } else {
        "Lower priority runtime directories found"
    }
    
    let fix_commands = if $has_high_priority_conflict {
        [
            $"# Backup and remove conflicting runtime:"
            $"mv ($user_runtime) ($user_runtime).backup"
            $"# Or if you want to delete it:"
            $"rm -rf ($user_runtime)"
        ]
    } else { [] }

    {
        status: $status
        message: $message
        details: $"Conflicting runtimes: ($conflict_details). Helix searches in priority order and will use files from higher priority directories, potentially breaking syntax highlighting."
        fix_available: true   # Auto-fix with backup
        fix_commands: $fix_commands
        conflicts: $conflicts
    }
}

# Check HELIX_RUNTIME health
export def check_helix_runtime_health [] {
    if ($env.HELIX_RUNTIME? | is-empty) {
        return {
            status: "error"
            message: "HELIX_RUNTIME environment variable not set"
            details: "This is required for Helix to find grammars and themes"
            fix_available: false
        }
    }
    
    let runtime_path = $env.HELIX_RUNTIME
    
    if not ($runtime_path | path exists) {
        return {
            status: "error" 
            message: $"HELIX_RUNTIME path does not exist: ($runtime_path)"
            details: "Helix will not work properly without a valid runtime directory"
            fix_available: false
        }
    }
    
    # Check for essential directories
    let required_dirs = ["grammars", "queries", "themes"]
    let missing_dirs = ($required_dirs | where not ($"($runtime_path)/($it)" | path exists))
    
    if not ($missing_dirs | is-empty) {
        return {
            status: "error"
            message: $"Missing required directories: ($missing_dirs | str join ', ')"
            details: $"HELIX_RUNTIME at ($runtime_path) is incomplete"
            fix_available: false
        }
    }
    
    # Count grammars
    let grammar_count = try {
        (ls $"($runtime_path)/grammars" | length)
    } catch {
        0
    }
    
    if ($grammar_count < 200) {
        return {
            status: "warning"
            message: $"Only ($grammar_count) grammar files found (expected 200+)"
            details: "Some languages may not have syntax highlighting"
            fix_available: false
        }
    }
    
    # Check tutor file
    if not ($"($runtime_path)/tutor" | path exists) {
        return {
            status: "warning"
            message: "Helix tutor file missing"
            details: "Tutorial will not be available"
            fix_available: false
        }
    }
    
    {
        status: "ok"
        message: $"HELIX_RUNTIME healthy with ($grammar_count) grammars"
        details: $"Runtime directory: ($runtime_path)"
        fix_available: false
    }
}

# Check environment variables
export def check_environment_variables [] {
    mut results = []
    
    # Check EDITOR
    if ($env.EDITOR? | is-empty) {
        $results = ($results | append {
            status: "warning"
            message: "EDITOR environment variable not set"
            details: "Some tools may not know which editor to use"
            fix_available: false
        })
    } else {
        $results = ($results | append {
            status: "ok" 
            message: $"EDITOR set to: ($env.EDITOR)"
            details: null
            fix_available: false
        })
    }
    
    # Check if using Helix and verify HELIX_RUNTIME
    if ($env.EDITOR? | default "" | str contains "hx") {
        $results = ($results | append (check_helix_runtime_health))
    }
    
    $results
}

# Check configuration files
export def check_configuration [] {
    let yazelix_config = "~/.config/yazelix/yazelix.nix"
    let yazelix_default = "~/.config/yazelix/yazelix_default.nix"
    
    mut results = []
    
    if ($yazelix_config | path expand | path exists) {
        $results = ($results | append {
            status: "ok"
            message: "Using custom yazelix.nix configuration"
            details: ($yazelix_config | path expand)
            fix_available: false
        })
    } else if ($yazelix_default | path expand | path exists) {
        $results = ($results | append {
            status: "info"
            message: "Using default configuration (yazelix_default.nix)"
            details: "Consider copying to yazelix.nix for customization"
            fix_available: true
        })
    } else {
        $results = ($results | append {
            status: "error"
            message: "No configuration file found"
            details: "Neither yazelix.nix nor yazelix_default.nix exists"
            fix_available: false
        })
    }
    
    $results
}

# Check shell integration
export def check_shell_integration [] {
    let yzx_available = try {
        (which yzx | is-not-empty)
    } catch {
        false
    }
    
    if $yzx_available {
        {
            status: "ok"
            message: "yzx commands available"
            details: "Shell integration working properly"
            fix_available: false
        }
    } else {
        {
            status: "warning"
            message: "yzx commands not found in PATH"
            details: "Shell integration may not be properly configured"
            fix_available: false
        }
    }
}

# Check log files
export def check_log_files [] {
    let logs_dir = "~/.config/yazelix/logs"
    let logs_path = ($logs_dir | path expand)
    
    if not ($logs_path | path exists) {
        return {
            status: "info"
            message: "No logs directory found"
            details: "Logs will be created when needed"
            fix_available: false
        }
    }
    
    let large_logs = try {
        (ls $logs_path | where type == file and size > 10MB)
    } catch {
        []
    }
    
    if not ($large_logs | is-empty) {
        let large_files = ($large_logs | get name | path basename | str join ", ")
        {
            status: "warning"
            message: $"Large log files found: ($large_files)"
            details: "Consider cleaning up logs to improve performance"
            fix_available: true
        }
    } else {
        {
            status: "ok"
            message: "Log files are reasonable size"
            details: $"Logs directory: ($logs_path)"
            fix_available: false
        }
    }
}

# Fix conflicting Helix runtime
export def fix_helix_runtime_conflicts [conflicts: list] {
    mut success = true
    
    for $conflict in $conflicts {
        if $conflict.severity == "error" {
            let backup_path = $"($conflict.path).backup"
            
            let move_result = try {
                mv $conflict.path $backup_path
                print $"✅ Moved ($conflict.name) from ($conflict.path) to ($backup_path)"
                true
            } catch {
                print $"❌ Failed to move ($conflict.name) from ($conflict.path)"
                false
            }
            
            if not $move_result {
                $success = false
            }
        }
    }
    
    $success
}

# Clean large log files
export def fix_large_logs [] {
    let logs_dir = "~/.config/yazelix/logs"
    let logs_path = ($logs_dir | path expand)
    
    if not ($logs_path | path exists) {
        return true
    }
    
    try {
        let large_logs = (ls $logs_path | where type == file and size > 10MB)
        
        for $log in $large_logs {
            rm $log.name
            print $"✅ Removed large log file: ($log.name | path basename)"
        }
        
        return true
    } catch {
        print "❌ Failed to clean log files"
        return false
    }
}

# Create yazelix.nix from default
export def fix_create_config [] {
    let yazelix_config = "~/.config/yazelix/yazelix.nix"
    let yazelix_default = "~/.config/yazelix/yazelix_default.nix"
    
    try {
        cp ($yazelix_default | path expand) ($yazelix_config | path expand)
        print $"✅ Created yazelix.nix from template"
        return true
    } catch {
        print "❌ Failed to create yazelix.nix"
        return false
    }
}

# Fix direnv setup by allowing .envrc
export def fix_direnv_setup [] {
    let yazelix_dir = "~/.config/yazelix" | path expand
    let envrc_path = ($yazelix_dir | path join ".envrc")

    try {
        bash -c $"direnv allow ($envrc_path)"
        print $"✅ Allowed .envrc for Yazelix directory"
        return true
    } catch {
        print "❌ Failed to allow .envrc - make sure direnv is installed and in PATH"
        return false
    }
}

# Check direnv setup for faster launches
export def check_direnv_setup [] {
    # Check if direnv is available
    let direnv_available = (which direnv | is-not-empty)

    if not $direnv_available {
        return {
            status: "info"
            message: "direnv not found - launches from regular shells will take ~4s"
            details: "Install direnv and nix-direnv for <100ms launches. Run: direnv allow ~/.config/yazelix"
            fix_available: false
        }
    }

    # Check if .envrc is allowed
    let yazelix_dir = "~/.config/yazelix" | path expand
    let envrc_path = ($yazelix_dir | path join ".envrc")

    # Try to check direnv status for the yazelix directory
    let is_allowed = try {
        cd $yazelix_dir
        let output = (bash -c "direnv status 2>&1" | complete)
        ($output.stdout | str contains "Found RC allowed true")
    } catch { false }

    if not $is_allowed {
        return {
            status: "warning"
            message: "direnv is installed but .envrc not allowed"
            details: $"Run: direnv allow ($envrc_path)"
            fix_available: true
            fix_command: $"direnv allow ($envrc_path)"
        }
    }

    {
        status: "ok"
        message: "direnv is set up correctly - enjoy fast launches!"
        details: "Launches from this directory will be <100ms instead of ~4s"
        fix_available: false
    }
}

# Main doctor function
export def run_doctor_checks [verbose: bool = false, fix: bool = false] {
    print "🔍 Running Yazelix Health Checks...\n"
    
    # Collect all checks
    mut all_results = []

    # Runtime conflicts check
    $all_results = ($all_results | append (check_helix_runtime_conflicts))

    # Environment variables
    $all_results = ($all_results | append (check_environment_variables))

    # Configuration
    $all_results = ($all_results | append (check_configuration))

    # Shell integration
    $all_results = ($all_results | append (check_shell_integration))

    # Log files
    $all_results = ($all_results | append (check_log_files))

    # direnv setup
    $all_results = ($all_results | append (check_direnv_setup))
    
    # Display results
    let errors = ($all_results | where status == "error")
    let warnings = ($all_results | where status == "warning") 
    let infos = ($all_results | where status == "info")
    let oks = ($all_results | where status == "ok")
    
    # Show results
    for $result in $all_results {
        match $result.status {
            "ok" => { print $"✅ ($result.message)" }
            "info" => { print $"ℹ️  ($result.message)" }
            "warning" => { print $"⚠️  ($result.message)" }
            "error" => { print $"❌ ($result.message)" }
        }
        
        if $verbose and ($result.details | is-not-empty) {
            print $"   ($result.details)"
        }
    }
    
    print ""
    
    # Summary
    if not ($errors | is-empty) {
        print $"❌ Found ($errors | length) errors"
    }
    
    if not ($warnings | is-empty) {
        print $"⚠️  Found ($warnings | length) warnings"
    }
    
    if ($errors | is-empty) and ($warnings | is-empty) {
        print "🎉 All checks passed! Yazelix is healthy."
        return
    }
    
    # Show manual fix commands for critical issues
    let runtime_conflicts = ($all_results | where status == "error" and message =~ "runtime")
    if not ($runtime_conflicts | is-empty) {
        for $conflict in $runtime_conflicts {
            if ($conflict.fix_commands? | is-not-empty) {
                print "\n🔧 To fix runtime conflicts, run these commands:"
                for $cmd in $conflict.fix_commands {
                    print $"  ($cmd)"
                }
            }
        }
    }
    
    # Auto-fix if requested
    if $fix {
        print "\n🔧 Attempting to auto-fix issues...\n"
        
        # Fix runtime conflicts (with backup)
        let runtime_conflicts = ($all_results | where status in ["error", "warning"] and message =~ "runtime")
        for $conflict in $runtime_conflicts {
            if $conflict.fix_available and ($conflict.conflicts? | is-not-empty) {
                fix_helix_runtime_conflicts $conflict.conflicts
            }
        }
        
        # Fix large logs
        let log_issues = ($all_results | where status == "warning" and message =~ "log")
        if not ($log_issues | is-empty) {
            fix_large_logs
        }
        
        # Fix missing config
        let config_issues = ($all_results | where status == "info" and message =~ "default")
        if not ($config_issues | is-empty) {
            fix_create_config
        }

        # Fix direnv setup
        let direnv_issues = ($all_results | where status == "warning" and message =~ "direnv")
        if not ($direnv_issues | is-empty) {
            fix_direnv_setup
        }

        print "\n✅ Auto-fix completed. Run 'yzx doctor' again to verify."
    } else if (($all_results | where fix_available == true) | is-not-empty) {
        print "\n💡 Some issues can be auto-fixed. Run 'yzx doctor --fix' to resolve them."
    }
}