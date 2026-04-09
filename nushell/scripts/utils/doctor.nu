#!/usr/bin/env nu
# Yazelix Doctor - Health check utilities

use logging.nu log_to_file
use constants.nu [PINNED_NIX_VERSION]
use common.nu [get_yazelix_config_dir get_yazelix_runtime_dir require_yazelix_runtime_dir]
use config_state.nu compute_config_state
use config_migration_transactions.nu [recover_stale_managed_config_transactions]
use config_surfaces.nu [get_main_user_config_path load_active_config_surface reconcile_primary_config_surfaces]
use config_diagnostics.nu [apply_doctor_config_fixes build_config_diagnostic_report render_doctor_config_details]
use config_parser.nu parse_yazelix_config
use devenv_cli.nu [get_preferred_devenv_version_line is_preferred_devenv_available resolve_preferred_devenv_path]
use doctor_helix.nu [
    check_helix_runtime_conflicts
    check_helix_runtime_health
    check_managed_helix_integration
    fix_helix_runtime_conflicts
]
use doctor_install_artifacts.nu check_desktop_entry_freshness
use launch_state.nu [describe_launch_profile_freshness resolve_runtime_owned_profile]
use runtime_distribution_capabilities.nu get_runtime_distribution_capability_profile
use runtime_contract_checker.nu [
    check_generated_layout
    check_launch_terminal_support
    check_launch_working_dir
    check_runtime_script
    resolve_expected_layout_path
    runtime_check_to_doctor_result
]
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
                try {
                    let result = (^nix --version | complete)
                    if $result.exit_code != 0 { "error" } else { extract_last_semver ($result.stdout | lines | first) }
                } catch { "error" }
            }
        }
        "devenv" => {
            if not (is_preferred_devenv_available) { "not installed" } else {
                try { extract_first_semver (get_preferred_devenv_version_line) } catch { "error" }
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

def get_version_drift_results [] {
    let nix_runtime = get_runtime_tool_version "nix"

    [
        (build_version_drift_result "nix" $PINNED_NIX_VERSION $nix_runtime)
    ]
}

export def print_runtime_version_drift_warning [] {
    let drift_results = (get_version_drift_results | where status == "warning")
    if ($drift_results | is-empty) {
        return
    }

    let nix_drift = ($drift_results | where message =~ '^nix version drift' | get -o 0)

    if ($nix_drift != null) {
        print $"⚠️  ($nix_drift.message)"
    }
}

def build_runtime_distribution_doctor_result [profile: record] {
    {
        status: "info"
        message: $profile.doctor_message
        details: $profile.doctor_details
        fix_available: false
        capability_tier: $profile.tier
        capability_mode: $profile.mode
    }
}

# Check environment variables
def check_environment_variables [] {
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
def check_configuration [--recover-interrupted-transactions] {
    let config_dir = (get_yazelix_config_dir)
    let runtime_dir = (require_yazelix_runtime_dir)
    let yazelix_legacy = ($config_dir | path join "yazelix.nix")
    let surface_paths = (try {
        {
            paths: (reconcile_primary_config_surfaces $config_dir $runtime_dir)
            error: null
        }
    } catch {|err|
        {
            paths: null
            error: $err.msg
        }
    })
    
    mut results = []

    if ($surface_paths.error | is-not-empty) {
        return [{
            status: "error"
            message: "Could not reconcile Yazelix config surfaces"
            details: $surface_paths.error
            fix_available: false
        }]
    }

    let yazelix_config = $surface_paths.paths.user_config
    let yazelix_default = $surface_paths.paths.default_config
    
    if ($yazelix_config | path expand | path exists) {
        $results = ($results | append {
            status: "ok"
            message: "Using custom yazelix.toml configuration"
            details: ($yazelix_config | path expand)
            fix_available: false
        })

        if $recover_interrupted_transactions {
            let recovery = (recover_stale_managed_config_transactions $yazelix_config)
            if $recovery.recovered_count > 0 {
                $results = ($results | append {
                    status: "info"
                    message: $"Recovered ($recovery.recovered_count) interrupted managed-config transaction\(s\)"
                    details: $yazelix_config
                    fix_available: false
                })
            }
        }

        let validation_result = (try {
            {
                report: (build_config_diagnostic_report $yazelix_config $yazelix_default)
                error: null
            }
        } catch {|err|
            {
                report: null
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
        } else if ($validation_result.report.issue_count > 0) {
            let issue_count = $validation_result.report.issue_count
            $results = ($results | append {
                status: "warning"
                message: $"Stale, unsupported, or migration-aware yazelix.toml entries detected \(($issue_count) issues\)"
                details: (render_doctor_config_details $validation_result.report)
                fix_available: $validation_result.report.has_fixable_migrations
                config_diagnostic_report: $validation_result.report
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

def check_shared_runtime_preflight [] {
    let config_result = (try {
        {config: (parse_yazelix_config), error: null}
    } catch {|err|
        {config: null, error: $err.msg}
    })
    if ($config_result.error | is-not-empty) {
        return []
    }

    let config = $config_result.config
    let runtime_dir = (get_yazelix_runtime_dir)
    let current_dir = (try { pwd } catch { null })
    let terminals = ($config.terminals? | default ["ghostty"] | uniq)
    let manage_terminals = ($config.manage_terminals? | default true)
    let layout_path = (resolve_expected_layout_path $config)
    let built_profile = (resolve_runtime_owned_profile)
    let terminal_check = if $manage_terminals and ($built_profile | is-not-empty) {
        with-env {DEVENV_PROFILE: $built_profile} {
            check_launch_terminal_support "" $terminals $manage_terminals
        }
    } else {
        check_launch_terminal_support "" $terminals $manage_terminals
    }

    mut checks = [
        (check_runtime_script ($runtime_dir | path join "nushell" "scripts" "core" "start_yazelix_inner.nu") "startup_runtime_script" "startup script" "doctor")
        (check_runtime_script ($runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu") "launch_runtime_script" "launch script" "doctor")
        (check_generated_layout $layout_path "doctor")
        $terminal_check
    ]

    if $current_dir != null {
        $checks = ($checks | prepend (check_launch_working_dir $current_dir))
    }

    $checks | each {|check| runtime_check_to_doctor_result $check }
}

def check_launch_profile_freshness [] {
    let config_state_result = (try {
        {state: (compute_config_state), error: ""}
    } catch {|err|
        {state: null, error: $err.msg}
    })
    if ($config_state_result.error | is-not-empty) {
        return {
            status: "info"
            message: "Launch-profile freshness check skipped until the active config parses cleanly"
            details: $config_state_result.error
            fix_available: false
        }
    }

    let config_state = $config_state_result.state
    let freshness = (describe_launch_profile_freshness $config_state)
    let recorded_profile = ($freshness.recorded_profile | default "")
    let recovery = "Run `yzx refresh` before relying on `yzx enter --reuse` or other cached launch-profile flows."

    match $freshness.kind {
        "healthy" => {
            {
                status: "ok"
                message: "Cached launch profile matches the current rebuild-relevant config and tracked inputs"
                details: (if ($recorded_profile | is-not-empty) {
                    $"Recorded profile: ($recorded_profile)"
                } else {
                    null
                })
                fix_available: false
            }
        }
        "stale_config_and_inputs" => {
            {
                status: "warning"
                message: "Cached launch profile is stale because rebuild-relevant config and tracked runtime/devenv inputs changed"
                details: $"($recovery)\nRecorded profile: ($recorded_profile | default '<missing>')"
                fix_available: false
            }
        }
        "stale_config" => {
            {
                status: "warning"
                message: "Cached launch profile is stale because rebuild-relevant config changed"
                details: $"($recovery)\nRecorded profile: ($recorded_profile | default '<missing>')"
                fix_available: false
            }
        }
        "stale_inputs" => {
            {
                status: "warning"
                message: "Cached launch profile is stale because tracked runtime/devenv inputs changed"
                details: $"($recovery)\nRecorded profile: ($recorded_profile | default '<missing>')"
                fix_available: false
            }
        }
        _ => {
            {
                status: "warning"
                message: "No verified cached launch profile exists for the current rebuild-relevant config and tracked inputs"
                details: $"($recovery)\nRecorded profile: ($recorded_profile | default '<missing>')"
                fix_available: false
            }
        }
    }
}

def get_desktop_applications_dir [] {
}

# Check log files
def check_log_files [] {
    let logs_dir = ((get_yazelix_runtime_dir) | path join "logs")
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
    is_preferred_devenv_available
}

# Check devenv availability inside the installed Yazelix runtime contract
def check_devenv_installation [capability_profile?: record] {
    let profile = if $capability_profile == null {
        get_runtime_distribution_capability_profile
    } else {
        $capability_profile
    }

    if (is_devenv_installed) {
        let version = try { (get_preferred_devenv_version_line | str trim) } catch { "unknown" }
        let path = try { resolve_preferred_devenv_path } catch { "unknown" }
        {
            status: "ok"
            message: $"devenv available: ($version)"
            details: $"Selected CLI: ($path)"
            fix_available: false
        }
    } else {
        let missing_result = match $profile.mode {
            "installer_managed" => {
                {
                    status: "warning"
                    message: "devenv missing from the installed Yazelix runtime"
                    details: "Repair by rerunning `nix run github:luccahuguet/yazelix#install` or by switching to a package-managed update flow, then rerun the affected launch or refresh command."
                    fix_available: false
                }
            }
            "home_manager_managed" => {
                {
                    status: "warning"
                    message: "devenv missing from the Home Manager-provided Yazelix runtime"
                    details: "Repair by reapplying or upgrading the Home Manager configuration that provides Yazelix, then rerun the affected launch or refresh command."
                    fix_available: false
                }
            }
            "package_runtime" => {
                {
                    status: "warning"
                    message: "devenv missing from the packaged Yazelix runtime"
                    details: "Repair by upgrading or reinstalling the package that provides Yazelix, then rerun the affected launch or refresh command."
                    fix_available: false
                }
            }
            _ => {
                {
                    status: "warning"
                    message: "devenv not available for this runtime-root-only Yazelix session"
                    details: "This mode does not own runtime repair. Provide `devenv` through the current runtime or PATH, or materialize a full install with `nix run github:luccahuguet/yazelix#install`."
                    fix_available: false
                }
            }
        }

        $missing_result
    }
}

def check_zellij_plugin_health [] {
    if ($env.ZELLIJ? | is-empty) {
        return [{
            status: "info"
            message: "Zellij plugin health check skipped (not inside Zellij)"
            details: "Run `yzx doctor` from inside the affected Yazelix session to verify Yazelix orchestrator permissions and managed pane detection."
            fix_available: false
        }]
    }

    let sidebar_enabled = (try {
        let config_surface = (load_active_config_surface)
        (($config_surface.merged_config.editor? | default {}) | get -o enable_sidebar | default true)
    } catch {
        true
    })

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
    build_zellij_plugin_health_results $plugin_state $sidebar_enabled
}

def build_zellij_plugin_health_results [plugin_state: record, sidebar_enabled: bool] {
    mut results = []

    if not ($plugin_state.permissions_granted? | default false) {
        $results = ($results | append {
            status: "error"
            message: "Yazelix pane-orchestrator plugin permissions not granted"
            details: "Grant the required Yazelix Zellij plugin permissions: focus the top zjstatus bar and press `y` if it prompts, and also answer yes to the Yazelix orchestrator permission popup. If permission state gets out of sync after an update, run `yzx repair zellij-permissions` and restart Yazelix. Yazelix workspace bindings like `Alt+m`, `Alt+y`, `Ctrl+y`, `Alt+r`, `Alt+[`, and `Alt+]` depend on the orchestrator."
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

# Clean large log files
def fix_large_logs [] {
    let logs_dir = ((get_yazelix_runtime_dir) | path join "logs")
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
def fix_create_config [] {
    use ./config_surfaces.nu [copy_default_config_surfaces]
    let yazelix_config_dir = (get_yazelix_config_dir)
    let yazelix_runtime_dir = (get_yazelix_runtime_dir)
    let yazelix_config = (get_main_user_config_path $yazelix_config_dir)
    let yazelix_default = ($yazelix_runtime_dir | path join "yazelix_default.toml")

    try {
        copy_default_config_surfaces ($yazelix_default | path expand) ($yazelix_config | path expand) | ignore
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
    let runtime_distribution_profile = (get_runtime_distribution_capability_profile)

    $all_results = ($all_results | append (build_runtime_distribution_doctor_result $runtime_distribution_profile))

    # Runtime conflicts check
    $all_results = ($all_results | append (check_helix_runtime_conflicts))

    # Environment variables
    $all_results = ($all_results | append (check_environment_variables))

    # Managed Helix contract
    $all_results = ($all_results | append (check_managed_helix_integration))

    # Configuration
    $all_results = ($all_results | append (check_configuration --recover-interrupted-transactions=$fix))

    # Shared runtime preflight overlap with launch-facing checks
    $all_results = ($all_results | append (check_shared_runtime_preflight))

    # Launch-profile freshness
    $all_results = ($all_results | append (check_launch_profile_freshness))

    # Desktop entry freshness
    $all_results = ($all_results | append (check_desktop_entry_freshness))

    # Log files
    $all_results = ($all_results | append (check_log_files))

    # devenv installation (performance optimization)
    $all_results = ($all_results | append (check_devenv_installation $runtime_distribution_profile))

    # Runtime drift against Yazelix pinned expectations
    $all_results = ($all_results | append (get_version_drift_results))

    # Zellij session-local plugin health
    $all_results = ($all_results | append (check_zellij_plugin_health))

    # Display results
    let errors = ($all_results | where status == "error")
    let warnings = ($all_results | where status == "warning") 
    
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

        let migration_issues = ($all_results | where config_diagnostic_report? != null)
        for $issue in $migration_issues {
            let report = $issue.config_diagnostic_report
            if $report.has_fixable_migrations {
                let apply_result = (apply_doctor_config_fixes $report)
                if $apply_result.status == "applied" {
                    print $"✅ Applied ($apply_result.applied_count) config migration fix\(es\) with backup: ($apply_result.backup_path)"
                    if ($apply_result.pack_backup_path? | is-not-empty) {
                        print $"✅ Backed up previous pack config to: ($apply_result.pack_backup_path)"
                    }
                    if ($apply_result.pack_config_path? | is-not-empty) and ($apply_result.pack_backup_path? | is-empty) and (($apply_result.pack_config_path | path exists)) {
                        print $"✅ Wrote pack config to: ($apply_result.pack_config_path)"
                    }
                }
            }
        }

        print "\n✅ Auto-fix completed. Run 'yzx doctor' again to verify."
    } else if (($all_results | where {|result| $result.fix_available } | is-not-empty)) {
        print "\n💡 Some issues can be auto-fixed. Run 'yzx doctor --fix' to resolve them."
    }
}
