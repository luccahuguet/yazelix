#!/usr/bin/env nu
# Yazelix Doctor - Health check utilities

use logging.nu log_to_file
use constants.nu [PINNED_NIX_VERSION PINNED_DEVENV_VERSION]
use common.nu [get_yazelix_dir]
use config_parser.nu parse_yazelix_config
use config_schema.nu get_config_validation_findings
use ../integrations/zellij.nu debug_editor_state

def extract_first_semver [text: string] {
    let matches = ($text | parse --regex '(\d+\.\d+\.\d+)' | get -o capture0)
    if ($matches | is-empty) { "unknown" } else { $matches | first }
}

def extract_last_semver [text: string] {
    let matches = ($text | parse --regex '(\d+\.\d+\.\d+)' | get -o capture0)
    if ($matches | is-empty) { "unknown" } else { $matches | last }
}

def get_runtime_tool_version [tool: string] {
    match $tool {
        "nix" => {
            if (which nix | is-empty) { "not installed" } else {
                try { extract_last_semver (nix --version | lines | first) } catch { "error" }
            }
        }
        "devenv" => {
            if (which devenv | is-empty) { "not installed" } else {
                try { extract_first_semver (devenv --version | lines | first) } catch { "error" }
            }
        }
        _ => "unknown"
    }
}

def build_version_drift_result [tool: string, pinned: string, runtime: string] {
    if $runtime == "not installed" {
        {
            status: "warning"
            message: $"($tool) not installed"
            details: $"Yazelix expects ($tool) ($pinned)"
            fix_available: false
        }
    } else if $runtime == "error" or $runtime == "unknown" {
        {
            status: "warning"
            message: $"Could not determine ($tool) runtime version"
            details: $"Yazelix expects ($tool) ($pinned)"
            fix_available: false
        }
    } else if $runtime != $pinned {
        {
            status: "warning"
            message: $"($tool) version drift: runtime ($runtime), Yazelix expects ($pinned)"
            details: "Version drift can cause breakage after upstream CLI or evaluation changes"
            fix_available: false
        }
    } else {
        {
            status: "ok"
            message: $"($tool) version matches Yazelix expectation: ($runtime)"
            details: null
            fix_available: false
        }
    }
}

export def get_version_drift_results [] {
    let nix_runtime = get_runtime_tool_version "nix"
    let devenv_runtime = get_runtime_tool_version "devenv"

    [
        (build_version_drift_result "nix" $PINNED_NIX_VERSION $nix_runtime)
        (build_version_drift_result "devenv" $PINNED_DEVENV_VERSION $devenv_runtime)
    ]
}

export def print_runtime_version_drift_warning [] {
    let drift_results = (get_version_drift_results | where status == "warning")
    if ($drift_results | is-empty) {
        return
    }

    let devenv_drift = ($drift_results | where message =~ '^devenv version drift' | get -o 0)
    let nix_drift = ($drift_results | where message =~ '^nix version drift' | get -o 0)

    if ($devenv_drift != null) {
        print $"⚠️  ($devenv_drift.message)"
        print "   Yazelix may still work, but upstream devenv changes can break launch/rebuild flows."
    }
    if ($nix_drift != null) {
        print $"⚠️  ($nix_drift.message)"
    }
}

# Check for conflicting Helix runtime directories based on Helix's search priority
export def check_helix_runtime_conflicts [] {
    # Helix runtime search order (highest to lowest priority):
    # 1. runtime/ sibling to $CARGO_MANIFEST_DIR (dev only - skip)
    # 2. ~/.config/helix/runtime (user config directory)  
    # 3. Explicitly configured runtime (when present)
    # 4. Package/distribution fallback runtime
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
    let effective_runtime = (detect_effective_helix_runtime)
    if ($helix_exe | is-not-empty) {
        let exe_runtime = ($helix_exe | path dirname | path join "runtime")
        if ($exe_runtime | path exists) and ($exe_runtime != ($effective_runtime | default "")) {
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
            details: "Helix runtime search order will behave as intended"
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
        "HIGH PRIORITY: ~/.config/helix/runtime will override the intended Helix runtime"
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

# Check effective Helix runtime health
def detect_effective_helix_runtime [] {
    if (which hx | is-empty) {
        return null
    }

    try {
        let runtime_line = (
            hx --health
            | lines
            | where {|line| $line | str starts-with "Runtime directories:"}
            | first
        )
        let runtime_candidates = (
            $runtime_line
            | str replace "Runtime directories: " ""
            | split row ";"
            | each {|entry| $entry | str trim}
            | where {|entry| $entry != ""}
        )

        let detected_runtime = (
            $runtime_candidates
            | where {|candidate| $candidate | path exists}
            | get -o 0
        )

        if ($detected_runtime | is-empty) {
            null
        } else {
            $detected_runtime
        }
    } catch {
        null
    }
}

export def check_helix_runtime_health [] {
    let detected_runtime = (detect_effective_helix_runtime)
    let runtime_path = $detected_runtime

    if ($runtime_path | is-empty) {
        return {
            status: "error"
            message: "Helix runtime could not be resolved"
            details: "Helix did not report any valid runtime directory in `hx --health`"
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
            details: $"The effective Helix runtime at ($runtime_path) is incomplete"
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
        message: $"Helix runtime healthy with ($grammar_count) grammars"
        details: $"Effective runtime directory: ($runtime_path)"
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
    
    # Check if using Helix and verify its effective runtime
    if ($env.EDITOR? | default "" | str contains "hx") {
        $results = ($results | append (check_helix_runtime_health))
    }
    
    $results
}

# Check configuration files
export def check_configuration [] {
    let yazelix_dir = (get_yazelix_dir)
    let yazelix_config = ($yazelix_dir | path join "yazelix.toml")
    let yazelix_legacy = ($yazelix_dir | path join "yazelix.nix")
    let yazelix_default = ($yazelix_dir | path join "yazelix_default.toml")
    
    mut results = []
    
    if ($yazelix_config | path expand | path exists) {
        $results = ($results | append {
            status: "ok"
            message: "Using custom yazelix.toml configuration"
            details: ($yazelix_config | path expand)
            fix_available: false
        })

        let validation_result = (try {
            {
                findings: (get_config_validation_findings $yazelix_dir)
                error: null
            }
        } catch {|err|
            {
                findings: []
                error: $err.msg
            }
        })

        if ($validation_result.error | is-not-empty) {
            $results = ($results | append {
                status: "error"
                message: "Could not validate yazelix.toml against the current schema"
                details: $validation_result.error
                fix_available: false
            })
        } else if not ($validation_result.findings | is-empty) {
            let validation_findings = $validation_result.findings
            let issue_count = ($validation_findings | length)
            let detail_lines = ($validation_findings | each {|finding| $" - ($finding.message)" })
            $results = ($results | append {
                status: "warning"
                message: $"Stale or invalid yazelix.toml fields detected \(($issue_count) issues\)"
                details: (
                    [
                        "Compare your config against yazelix_default.toml."
                        "To replace it with a fresh template \(with backup\): yzx config reset --yes"
                        ...$detail_lines
                    ] | str join "\n"
                )
                fix_available: false
            })
        }
    } else if ($yazelix_legacy | path expand | path exists) {
        $results = ($results | append {
            status: "warning"
            message: "Legacy yazelix.nix configuration detected"
            details: ($yazelix_legacy | path expand)
            fix_available: false
        })
    } else if ($yazelix_default | path expand | path exists) {
        $results = ($results | append {
            status: "info"
            message: "Using default configuration (yazelix_default.toml)"
            details: "Consider copying to yazelix.toml for customization"
            fix_available: true
        })
    } else {
        $results = ($results | append {
            status: "error"
            message: "No configuration file found"
            details: "Neither yazelix.toml nor yazelix_default.toml exists"
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
    let logs_dir = ((get_yazelix_dir) | path join "logs")
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

def is_devenv_installed [] {
    (which devenv | is-not-empty)
}

# Check devenv installation for performance boost
export def check_devenv_installation [] {
    if (is_devenv_installed) {
        let version = try { (devenv version | str trim) } catch { "unknown" }
        {
            status: "ok"
            message: $"devenv installed: ($version)"
            details: "Shell startup: ~0.3s (13x faster than without devenv)"
            fix_available: false
        }
    } else {
        {
            status: "warning"
            message: "devenv not installed (optional)"
            details: "Install devenv for 13x faster shell startup (~4-5s → ~0.3s). Desktop entries and terminal sessions will launch instantly."
            fix_available: true
        }
    }
}

export def check_zellij_plugin_health [] {
    if ($env.ZELLIJ? | is-empty) {
        return [{
            status: "info"
            message: "Zellij plugin health check skipped (not inside Zellij)"
            details: "Run `yzx doctor` from inside the affected Yazelix session to verify Yazelix orchestrator permissions and managed pane detection."
            fix_available: false
        }]
    }

    let plugin_state = try {
        debug_editor_state
    } catch {|err|
        return [{
            status: "warning"
            message: "Could not contact the Yazelix pane-orchestrator plugin"
            details: $"Run this from inside the affected Yazelix session after fully restarting it. Underlying error: ($err.msg)"
            fix_available: false
        }]
    }

    if ($plugin_state.raw? | is-not-empty) {
        return [{
            status: "warning"
            message: "Yazelix pane-orchestrator returned an unexpected response"
            details: $"Unexpected payload: ($plugin_state.raw)"
            fix_available: false
        }]
    }

    let config = parse_yazelix_config
    let sidebar_enabled = ($config.enable_sidebar? | default true)
    build_zellij_plugin_health_results $plugin_state $sidebar_enabled
}

export def build_zellij_plugin_health_results [plugin_state: record, sidebar_enabled: bool] {
    mut results = []

    if not ($plugin_state.permissions_granted? | default false) {
        $results = ($results | append {
            status: "error"
            message: "Yazelix pane-orchestrator plugin permissions not granted"
            details: "Grant permissions for both Zellij plugins: focus the top zjstatus bar and press `y`, and also answer yes to the Yazelix orchestrator permission popup. Yazelix workspace bindings like `Alt+m`, `Alt+y`, `Ctrl+y`, `Alt+r`, `Alt+[`, and `Alt+]` depend on the orchestrator."
            fix_available: false
        })
    } else {
        $results = ($results | append {
            status: "ok"
            message: "Yazelix pane-orchestrator permissions granted"
            details: "The orchestrator plugin can handle Yazelix tab and pane actions in this Zellij session."
            fix_available: false
        })
    }

    if ($plugin_state.active_tab_position? | default null) == null {
        $results = ($results | append {
            status: "warning"
            message: "Yazelix pane-orchestrator does not see an active tab yet"
            details: "The plugin may still be initializing. Wait a moment and rerun `yzx doctor` inside this Yazelix session."
            fix_available: false
        })
        return $results
    }

    if $sidebar_enabled {
        if ($plugin_state.sidebar_pane_id? | is-empty) {
            $results = ($results | append {
                status: "warning"
                message: "Managed sidebar pane not detected in the current tab"
                details: "If sidebar mode is enabled, `Alt+y` and `Ctrl+y` may not work until the current tab uses a Yazelix sidebar layout."
                fix_available: false
            })
        } else {
            $results = ($results | append {
                status: "ok"
                message: $"Managed sidebar pane detected: ($plugin_state.sidebar_pane_id)"
                details: $"Layout state: ($plugin_state.active_swap_layout_name? | default 'unknown')"
                fix_available: false
            })
        }
    }

    if ($plugin_state.editor_pane_id? | is-empty) {
        $results = ($results | append {
            status: "info"
            message: "Managed editor pane not detected in the current tab"
            details: "This is normal until you open a managed Helix or Neovim editor pane in the current tab. An editor started manually from an ordinary shell pane does not count as the managed editor pane."
            fix_available: false
        })
    } else {
        $results = ($results | append {
            status: "ok"
            message: $"Managed editor pane detected: ($plugin_state.editor_pane_id)"
            details: null
            fix_available: false
        })
    }

    $results
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
    let logs_dir = ((get_yazelix_dir) | path join "logs")
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

# Create yazelix.toml from default
export def fix_create_config [] {
    let yazelix_dir = (get_yazelix_dir)
    let yazelix_config = ($yazelix_dir | path join "yazelix.toml")
    let yazelix_default = ($yazelix_dir | path join "yazelix_default.toml")

    try {
        cp ($yazelix_default | path expand) ($yazelix_config | path expand)
        print $"✅ Created yazelix.toml from template"
        return true
    } catch {
        print "❌ Failed to create yazelix.toml"
        return false
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

    # devenv installation (performance optimization)
    $all_results = ($all_results | append (check_devenv_installation))

    # Runtime drift against Yazelix pinned expectations
    $all_results = ($all_results | append (get_version_drift_results))

    # Zellij session-local plugin health
    $all_results = ($all_results | append (check_zellij_plugin_health))

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

        print "\n✅ Auto-fix completed. Run 'yzx doctor' again to verify."
    } else if (($all_results | where fix_available == true) | is-not-empty) {
        print "\n💡 Some issues can be auto-fixed. Run 'yzx doctor --fix' to resolve them."
    }
}
