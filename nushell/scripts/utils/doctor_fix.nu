#!/usr/bin/env nu

use common.nu [
    get_yazelix_config_dir
    get_yazelix_runtime_dir
    get_yazelix_state_dir
    require_yazelix_runtime_dir
]
use ./config_surfaces.nu [copy_default_config_surfaces get_main_user_config_path load_active_config_surface]
use ./doctor_helix.nu fix_helix_runtime_conflicts
use ./failure_classes.nu format_failure_classification
use ./yzx_core_bridge.nu [build_default_yzx_core_error_surface build_record_yzx_core_error_surface run_yzx_core_json_command]

const ZELLIJ_MATERIALIZATION_COMMAND = "zellij-materialization.generate"
const RUNTIME_MATERIALIZATION_REPAIR_COMMAND = "runtime-materialization.repair"

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

def repair_generated_runtime_state [
    --force(-f)
    --verbose(-v)
] {
    let runtime_dir = (require_yazelix_runtime_dir)
    mut helper_args = [$RUNTIME_MATERIALIZATION_REPAIR_COMMAND "--from-env"]
    if $force {
        $helper_args = ($helper_args | append "--force")
    }

    let data = (run_yzx_core_json_command
        $runtime_dir
        (build_default_yzx_core_error_surface)
        $helper_args
        "Yazelix Rust runtime-materialization repair helper returned invalid JSON.")
    let repair = ($data.repair? | default {})

    if (($data.status? | default "") == "noop") {
        for line in ($repair.lines? | default []) {
            print $line
        }
        return {
            status: "noop"
            applied_state: ($data.plan? | default {})
        }
    }

    if $verbose {
        let progress_message = ($repair.progress_message? | default "")
        if ($progress_message | is-not-empty) {
            print $progress_message
        }
        let detail = ($repair.missing_artifacts_detail_line? | default "")
        if ($detail | is-not-empty) {
            print $detail
        }
    }

    let materialization = ($data.materialization? | default null)
    if $materialization == null {
        let classification = (format_failure_classification "generated-state" "Run `yzx doctor` to inspect the generated-state contract, then rerun the repair after fixing the reported problem.")
        error make {msg: $"Rust runtime-materialization repair returned no materialization result for a non-noop repair.\n($classification)"}
    }

    for line in ($repair.success_lines? | default []) {
        print $line
    }

    {
        status: ($data.status? | default "repaired")
        applied_state: ($materialization.plan? | default {})
    }
}

def fix_create_config [] {
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

def load_fix_results [] {
    let payload = (
        $env.YAZELIX_DOCTOR_RESULTS_JSON?
        | default ""
        | into string
        | str trim
    )

    if ($payload | is-empty) {
        error make {msg: "YAZELIX_DOCTOR_RESULTS_JSON is not set for the internal doctor fix helper."}
    }

    try {
        $payload | from json
    } catch {|err|
        error make {msg: $"Internal doctor fix payload was not valid JSON: ($err.msg)"}
    }
}

export def apply_doctor_fixes_internal [
    --verbose(-v)
] {
    let results = (load_fix_results)

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
