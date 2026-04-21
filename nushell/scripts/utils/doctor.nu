#!/usr/bin/env nu
# Yazelix Doctor - Health check utilities

use common.nu [
    get_yazelix_config_dir
    get_yazelix_runtime_dir
    get_yazelix_state_dir
    require_yazelix_runtime_dir
]
use ./yzx_core_bridge.nu [build_default_yzx_core_error_surface build_record_yzx_core_error_surface run_yzx_core_request_json_command run_yzx_core_json_command]
use config_surfaces.nu [get_main_user_config_path load_active_config_surface]
use doctor_helix.nu fix_helix_runtime_conflicts
use doctor_helix_report.nu collect_helix_doctor_results
use doctor_runtime_report.nu collect_runtime_doctor_results
use install_ownership_report.nu evaluate_install_ownership_report
use ../core/materialization_orchestrator.nu repair_generated_runtime_state
use ../integrations/zellij.nu get_active_tab_session_state

const DOCTOR_CONFIG_EVALUATE_COMMAND = "doctor-config.evaluate"
const ZELLIJ_MATERIALIZATION_COMMAND = "zellij-materialization.generate"

def seed_yazelix_plugin_permissions [] {
    let runtime_dir = (require_yazelix_runtime_dir)
    let config_surface = (load_active_config_surface)
    let zellij_config_dir = (get_yazelix_state_dir | path join "configs" "zellij")
    run_yzx_core_json_command $runtime_dir (
        build_record_yzx_core_error_surface {config_file: $config_surface.config_file}
    ) [
        $ZELLIJ_MATERIALIZATION_COMMAND
        "--config"
        $config_surface.config_file
        "--default-config"
        $config_surface.default_config_path
        "--contract"
        ($runtime_dir | path join "config_metadata" "main_config_contract.toml")
        "--runtime-dir"
        $runtime_dir
        "--zellij-config-dir"
        $zellij_config_dir
        "--seed-plugin-permissions"
    ] "Yazelix Rust zellij-materialization helper returned invalid JSON." | ignore
    {
        permissions_cache_path: ($env.HOME | path join ".cache" "zellij" "permissions.kdl")
    }
}

def build_doctor_summary [results: list<record>] {
    let error_count = ($results | where status == "error" | length)
    let warning_count = ($results | where status == "warning" | length)
    let info_count = ($results | where status == "info" | length)
    let ok_count = ($results | where status == "ok" | length)
    let fixable_count = ($results | where {|result| $result.fix_available? | default false } | length)

    {
        error_count: $error_count
        warning_count: $warning_count
        info_count: $info_count
        ok_count: $ok_count
        fixable_count: $fixable_count
        healthy: (($error_count == 0) and ($warning_count == 0) and ($fixable_count == 0))
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

    let session = try {
        get_active_tab_session_state
    } catch {|err|
        return [{
            status: "warning"
            message: "Could not contact the Yazelix pane-orchestrator plugin"
            details: $"Run this from inside the affected Yazelix session after fully restarting it. Underlying error: ($err.msg)"
            fix_available: false
        }]
    }

    if ($session.raw? | is-not-empty) {
        let raw = ($session.raw | into string | str trim)
        if $raw == "permissions_denied" {
            return (build_zellij_plugin_health_results {
                permissions_granted: false
                active_tab_position: null
                sidebar_pane_id: ""
                editor_pane_id: ""
                active_swap_layout_name: null
            } $sidebar_enabled)
        }
        if $raw in ["not_ready", "missing"] {
            return [{
                status: "warning"
                message: "Yazelix pane-orchestrator session state is not ready yet"
                details: "The plugin responded before tab/workspace state was available. Wait a moment and rerun `yzx doctor` inside this Yazelix session."
                fix_available: false
            }]
        }
        return [{
            status: "warning"
            message: "Yazelix pane-orchestrator returned an unexpected response"
            details: $"Unexpected payload: ($raw)"
            fix_available: false
        }]
    }

    let plugin_state = {
        permissions_granted: true
        active_tab_position: ($session | get -o active_tab_position | default null)
        sidebar_pane_id: (
            $session.managed_panes? | default {} | get -o sidebar_pane_id | default "" | into string
        )
        editor_pane_id: (
            $session.managed_panes? | default {} | get -o editor_pane_id | default "" | into string
        )
        active_swap_layout_name: ($session.layout? | default {} | get -o active_swap_layout_name | default null)
    }
    build_zellij_plugin_health_results $plugin_state $sidebar_enabled
}

def build_zellij_plugin_health_results [plugin_state: record, sidebar_enabled: bool] {
    mut results = []

    if not ($plugin_state.permissions_granted? | default false) {
        $results = ($results | append {
            status: "error"
            message: "Yazelix pane-orchestrator plugin permissions not granted"
            details: "Grant the required Yazelix Zellij plugin permissions: focus the top zjstatus bar and press `y` if it prompts, and also answer yes to the Yazelix orchestrator permission popup. If permission state gets out of sync after an update, run `yzx doctor --fix` and restart Yazelix. Yazelix workspace bindings like `Alt+m`, `Alt+y`, `Ctrl+y`, `Alt+r`, `Alt+[`, and `Alt+]` depend on the orchestrator."
            fix_available: true
            fix_action: "seed_zellij_plugin_permissions"
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

def collect_config_doctor_results [] {
    let rd = require_yazelix_runtime_dir
    let data = (run_yzx_core_request_json_command
        $rd
        (build_default_yzx_core_error_surface)
        $DOCTOR_CONFIG_EVALUATE_COMMAND
        {
            config_dir: (get_yazelix_config_dir)
            runtime_dir: $rd
        }
        "Yazelix Rust doctor-config helper returned invalid JSON.")

    $data.findings
}

export def collect_doctor_report [] {
    mut results = []
    let install_report = (evaluate_install_ownership_report)
    let runtime_pack = (collect_runtime_doctor_results $install_report)

    $results = ($results | append $runtime_pack.distribution)

    let helix_pack = (collect_helix_doctor_results)
    $results = ($results | append $helix_pack.runtime_conflicts)
    if $helix_pack.runtime_health != null {
        $results = ($results | append $helix_pack.runtime_health)
    }
    for finding in $helix_pack.managed_integration {
        $results = ($results | append $finding)
    }
    $results = ($results | append (collect_config_doctor_results))
    for r in $runtime_pack.shared_runtime_preflight {
        $results = ($results | append $r)
    }
    for w in ($install_report.wrapper_shadowing? | default []) {
        $results = ($results | append $w)
    }
    $results = ($results | append $install_report.desktop_entry_freshness)
    $results = ($results | append (check_zellij_plugin_health))

    {
        title: "Yazelix Health Checks"
        results: $results
        summary: (build_doctor_summary $results)
    }
}

def render_doctor_result [result: record, verbose: bool] {
    match $result.status {
        "ok" => { print $"✅ ($result.message)" }
        "info" => { print $"ℹ️  ($result.message)" }
        "warning" => { print $"⚠️  ($result.message)" }
        "error" => { print $"❌ ($result.message)" }
        _ => { print $"• ($result.message)" }
    }

    if $verbose and (($result.details? | default "") | is-not-empty) {
        print $"   ($result.details)"
    }
}

export def render_doctor_report [report: record, --verbose] {
    print "🔍 Running Yazelix Health Checks...\n"

    for result in ($report.results? | default []) {
        render_doctor_result $result $verbose
    }

    print ""

    let summary = ($report.summary? | default {})
    let error_count = ($summary.error_count? | default 0)
    let warning_count = ($summary.warning_count? | default 0)

    if $error_count > 0 {
        print $"❌ Found ($error_count) errors"
    }

    if $warning_count > 0 {
        print $"⚠️  Found ($warning_count) warnings"
    }

    if ($summary.healthy? | default false) {
        print "🎉 All checks passed! Yazelix is healthy."
    }
}

def print_runtime_conflict_fix_commands [results: list<record>] {
    let runtime_conflicts = ($results | where status == "error" and message =~ "runtime")
    if ($runtime_conflicts | is-empty) {
        return
    }

    for conflict in $runtime_conflicts {
        if ($conflict.fix_commands? | is-not-empty) {
            print "\n🔧 To fix runtime conflicts, run these commands:"
            for cmd in $conflict.fix_commands {
                print $"  ($cmd)"
            }
        }
    }
}

def apply_doctor_fixes [results: list<record>, verbose: bool] {
    print "\n🔧 Attempting to auto-fix issues...\n"

    let runtime_conflicts = ($results | where status in ["error", "warning"] and message =~ "runtime")
    for conflict in $runtime_conflicts {
        if ($conflict.fix_available? | default false) and ($conflict.conflicts? | is-not-empty) {
            fix_helix_runtime_conflicts $conflict.conflicts
        }
    }

    let config_issues = ($results | where status == "info" and message =~ "default")
    if not ($config_issues | is-empty) {
        fix_create_config
    }

    let generated_state_issues = ($results | where {|result| ($result.fix_action? | default "") == "repair_generated_runtime_state" })
    if not ($generated_state_issues | is-empty) {
        try {
            repair_generated_runtime_state --verbose=$verbose | ignore
        } catch {|err|
            print $"❌ Failed to repair generated runtime state: ($err.msg)"
        }
    }

    let plugin_permission_issues = ($results | where {|result| ($result.fix_action? | default "") == "seed_zellij_plugin_permissions" })
    if not ($plugin_permission_issues | is-empty) {
        try {
            let repair_result = (seed_yazelix_plugin_permissions)
            print $"✅ Seeded Yazelix plugin permissions in: ($repair_result.permissions_cache_path)"
        } catch {|err|
            print $"❌ Failed to seed Yazelix plugin permissions: ($err.msg)"
        }
    }

    print "\n✅ Auto-fix completed. Run 'yzx doctor' again to verify."
}


# Main doctor function
export def run_doctor_checks [verbose: bool = false, fix: bool = false] {
    let report = (collect_doctor_report)
    let results = ($report.results? | default [])
    let summary = ($report.summary? | default {})

    render_doctor_report $report --verbose=$verbose

    if ($summary.healthy? | default false) {
        return
    }

    print_runtime_conflict_fix_commands $results

    if $fix {
        apply_doctor_fixes $results $verbose
    } else if (($summary.fixable_count? | default 0) > 0) {
        print "\n💡 Some issues can be auto-fixed. Run 'yzx doctor --fix' to resolve them."
    }
}
