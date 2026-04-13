#!/usr/bin/env nu

use config_state.nu [compute_config_state record_materialized_state]
use environment_bootstrap.nu prepare_environment
use common.nu [get_yazelix_state_dir require_yazelix_runtime_dir]
use runtime_contract_checker.nu resolve_expected_layout_path
use ../setup/yazi_config_merger.nu generate_merged_yazi_config
use ../setup/zellij_config_merger.nu generate_merged_zellij_config

export def regenerate_runtime_configs [runtime_dir: string, --quiet] {
    let quiet_mode = $quiet
    let config_state = compute_config_state

    try {
        if $quiet_mode {
            generate_merged_yazi_config $runtime_dir --quiet --sync-static-assets=($config_state.needs_refresh? | default true) | ignore
        } else {
            print "🔧 Regenerating managed Yazi configuration..."
            generate_merged_yazi_config $runtime_dir --sync-static-assets=($config_state.needs_refresh? | default true) | ignore
        }
    } catch {|err|
        error make {msg: $"Failed to regenerate Yazi configuration: ($err.msg)"}
    }

    try {
        let state_dir = (get_yazelix_state_dir)
        let zellij_config_dir = ($state_dir | path join "configs" "zellij")
        if not $quiet_mode {
            print "🔧 Regenerating managed Zellij configuration..."
        }
        generate_merged_zellij_config $runtime_dir $zellij_config_dir | ignore
    } catch {|err|
        error make {msg: $"Failed to regenerate Zellij configuration: ($err.msg)"}
    }
}

export def list_missing_runtime_config_artifacts [config: record] {
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

export def record_current_materialized_state [] {
    let applied_state = (compute_config_state)
    record_materialized_state $applied_state
    $applied_state
}

export def repair_generated_runtime_state [
    --force(-f)    # Force regeneration even when config/runtime inputs already match
    --verbose(-v)  # Print concise generated-state repair progress
] {
    let env_prep = prepare_environment
    let config = $env_prep.config
    let config_state = $env_prep.config_state
    let runtime_dir = (require_yazelix_runtime_dir)
    let show_progress = $verbose
    let missing_runtime_artifacts = (list_missing_runtime_config_artifacts $config)

    if (not $force) and (not $config_state.needs_refresh) {
        if ($missing_runtime_artifacts | is-not-empty) {
            print "🩹 Yazelix generated state is current, but some derived runtime files are missing."
            if $show_progress {
                let missing_labels = ($missing_runtime_artifacts | get label | str join ", ")
                print $"   Repairing missing artifacts: ($missing_labels)"
            }
            regenerate_runtime_configs $runtime_dir --quiet=(not $show_progress)
            let applied_state = (record_current_materialized_state)
            print "✅ Repaired the missing generated runtime artifacts."
            return {
                status: "repaired_missing_artifacts"
                applied_state: $applied_state
            }
        }

        print "✅ Yazelix generated state is already up to date."
        print "   Nothing to repair."
        return {
            status: "noop"
            applied_state: $config_state
        }
    }

    let repair_reason = if $force {
        "manual repair requested"
    } else {
        $config_state.refresh_reason? | default "config or runtime inputs changed since last generated-state repair"
    }

    if $show_progress {
        print $"♻️  Repairing generated runtime state \(($repair_reason)\)..."
    }

    regenerate_runtime_configs $runtime_dir --quiet=(not $show_progress)
    let applied_state = (record_current_materialized_state)

    print "✅ Generated runtime state repaired."
    print "   Generated Yazi/Zellij state now matches the active runtime config."

    {
        status: "repaired"
        applied_state: $applied_state
    }
}
