#!/usr/bin/env nu
# yzx refresh command - Repair Yazelix generated runtime state without launching UI

use ../utils/environment_bootstrap.nu [prepare_environment]
use ../utils/config_state.nu [compute_config_state record_materialized_state]
use ../utils/common.nu [get_yazelix_state_dir require_yazelix_runtime_dir]
use ../utils/runtime_contract_checker.nu resolve_expected_layout_path
use ../setup/yazi_config_merger.nu generate_merged_yazi_config
use ../setup/zellij_config_merger.nu generate_merged_zellij_config

def regenerate_runtime_configs [runtime_dir: string, --quiet] {
    let quiet_mode = $quiet

    try {
        if $quiet_mode {
            generate_merged_yazi_config $runtime_dir --quiet | ignore
        } else {
            print "🔧 Regenerating managed Yazi configuration..."
            generate_merged_yazi_config $runtime_dir | ignore
        }
    } catch {|err|
        error make {msg: $"Failed to regenerate Yazi configuration during refresh: ($err.msg)"}
    }

    try {
        if not $quiet_mode {
            print "🔧 Regenerating managed Zellij configuration..."
        }
        generate_merged_zellij_config $runtime_dir | ignore
    } catch {|err|
        error make {msg: $"Failed to regenerate Zellij configuration during refresh: ($err.msg)"}
    }
}

def list_missing_runtime_config_artifacts [config: record] {
    let state_dir = (get_yazelix_state_dir)
    let yazi_dir = ($state_dir | path join "configs" "yazi")
    let zellij_dir = ($state_dir | path join "configs" "zellij")
    let expected_layout_path = (resolve_expected_layout_path $config)
    let required_artifacts = [
        { label: "generated Yazi config" path: ($yazi_dir | path join "yazi.toml") }
        { label: "generated Yazi keymap" path: ($yazi_dir | path join "keymap.toml") }
        { label: "generated Yazi init.lua" path: ($yazi_dir | path join "init.lua") }
        { label: "generated Zellij config" path: ($zellij_dir | path join "config.kdl") }
        { label: "generated Zellij layout" path: $expected_layout_path }
    ]

    $required_artifacts
    | where {|artifact|
        let resolved_path = ($artifact.path | path expand)
        not ($resolved_path | path exists)
    }
}

# Repair generated runtime configuration without launching Yazelix UI
export def "yzx refresh" [
    --force(-f)    # Force refresh even when no config/input changes are detected
    --verbose(-v)  # Show concise generated-state repair progress
    --very-verbose(-V)  # Alias for --verbose in the v15 generated-state repair flow
] {
    let env_prep = prepare_environment
    let config = $env_prep.config
    let config_state = $env_prep.config_state
    let runtime_dir = (require_yazelix_runtime_dir)
    let needs_refresh = $config_state.needs_refresh
    let show_progress = $verbose or $very_verbose
    let missing_runtime_artifacts = (list_missing_runtime_config_artifacts $config)

    if (not $force) and (not $needs_refresh) {
        if ($missing_runtime_artifacts | is-not-empty) {
            print "🩹 Yazelix environment is up to date, but generated runtime configs are missing."
            if $show_progress {
                let missing_labels = ($missing_runtime_artifacts | get label | str join ", ")
                print $"   Repairing without rebuild: ($missing_labels)"
            }
            regenerate_runtime_configs $runtime_dir --quiet=(not $show_progress)
            print "✅ Refresh repaired the missing runtime configs."
            return
        }

        print "✅ Yazelix generated state is already up to date."
        print "   Nothing to refresh."
        return
    }

    let refresh_reason = if $force {
        "manual refresh requested"
    } else {
        $config_state.refresh_reason? | default "config or runtime inputs changed since last launch"
    }

    if $show_progress {
        print $"♻️  Repairing Yazelix generated state \(($refresh_reason)\)..."
    }

    regenerate_runtime_configs $runtime_dir --quiet=(not $show_progress)
    let applied_state = (compute_config_state)
    record_materialized_state $applied_state

    print "✅ Refresh completed."
    print "   Generated Yazi/Zellij state now matches the active runtime config."
}
