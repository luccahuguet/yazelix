#!/usr/bin/env nu
# yzx refresh command - Refresh Yazelix devenv cache/environment without launching UI

use ../utils/build_policy.nu [describe_build_parallelism]
use ../utils/environment_bootstrap.nu [prepare_environment]
use ../utils/devenv_backend.nu [format_command_failure_summary get_refresh_output_mode run_devenv_build_shell]
use ../utils/config_state.nu [compute_config_state record_materialized_state]
use ../utils/launch_state.nu record_launch_profile_state
use ../utils/common.nu [get_yazelix_state_dir require_yazelix_runtime_dir]
use ../utils/runtime_contract_checker.nu resolve_expected_layout_path
use ../setup/yazi_config_merger.nu generate_merged_yazi_config
use ../setup/zellij_config_merger.nu generate_merged_zellij_config

def summarize_values [values max_items: int] {
    let normalized = ($values | each { |value| $value | into string })
    let total = ($normalized | length)

    if $total == 0 {
        "none"
    } else if $total <= $max_items {
        $normalized | str join ", "
    } else {
        let shown = ($normalized | first $max_items | str join ", ")
        let remaining = ($total - $max_items)
        $"($shown), +($remaining) more"
    }
}

def get_requested_package_scope [config] {
    let enabled_packs = ($config.pack_names? | default [])
    let pack_declarations = ($config.pack_declarations? | default {})
    let pack_packages = (
        $enabled_packs
        | each { |pack_name|
            $pack_declarations | get -o $pack_name | default []
        }
        | flatten
    )
    let user_packages = ($config.user_packages? | default [])
    let top_level_packages = ($pack_packages | append $user_packages | flatten | uniq)

    {
        enabled_packs: $enabled_packs
        top_level_packages: $top_level_packages
    }
}

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

# Refresh devenv evaluation cache without launching Yazelix UI
export def "yzx refresh" [
    --force(-f)    # Force refresh even when no config/input changes are detected
    --verbose(-v)  # Show package scope and concise refresh progress
    --very-verbose(-V)  # Show full build logs during refresh (-vv equivalent)
] {
    use ../utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    let env_prep = prepare_environment
    let config = $env_prep.config
    let config_state = $env_prep.config_state
    let runtime_dir = (require_yazelix_runtime_dir)
    let needs_refresh = $config_state.needs_refresh
    let max_jobs = ($config.max_jobs? | default "half" | into string)
    let build_cores = ($config.build_cores? | default "2" | into string)
    let build_parallelism_description = (describe_build_parallelism $build_cores $max_jobs)
    let refresh_output = if $very_verbose {
        "full"
    } else if $verbose {
        "normal"
    } else {
        get_refresh_output_mode $config
    }
    let show_progress = ($refresh_output != "quiet")
    let missing_runtime_artifacts = (list_missing_runtime_config_artifacts $config)

    if (not $force) and (not $needs_refresh) {
        if ($missing_runtime_artifacts | is-not-empty) {
            print "🩹 Yazelix environment is up to date, but generated runtime configs are missing."
            if $show_progress {
                let missing_labels = ($missing_runtime_artifacts | get label | str join ", ")
                print $"   Repairing without rebuild: ($missing_labels)"
            }
            regenerate_runtime_configs $runtime_dir --quiet=($refresh_output == "quiet")
            print "✅ Refresh repaired the missing runtime configs."
            return
        }

        print "✅ Yazelix environment is already up to date."
        print "   Nothing to refresh."
        return
    }

    let refresh_reason = if $force {
        "manual refresh requested"
    } else {
        $config_state.refresh_reason? | default "config or devenv inputs changed since last launch"
    }

    if $show_progress {
        let scope = get_requested_package_scope $config
        let packs_text = summarize_values $scope.enabled_packs 12
        let packages_text = summarize_values $scope.top_level_packages 20
        let recommended_deps = ($config | get -o recommended_deps | default true)
        let yazi_extensions = ($config | get -o yazi_extensions | default true)
        print $"📦 Requested packs: ($packs_text)"
        print $"📦 Top-level packages: ($packages_text)"
        print $"   Optional bundles: recommended_deps=($recommended_deps), yazi_extensions=($yazi_extensions)"
        print "   Note: Nix builds transitive dependencies in addition to these top-level packages."
    }

    mut built_profile = ""
    if $needs_refresh or $force {
        print $"♻️  Refreshing Yazelix environment \(($refresh_reason), using ($build_parallelism_description)\)..."

        let refresh_result = (run_devenv_build_shell --max-jobs $max_jobs --build-cores $build_cores --refresh-eval-cache --output-mode $refresh_output --skip-shellhook-welcome)

        if $refresh_result.exit_code != 0 {
            print (format_command_failure_summary
                "Refresh failed"
                $refresh_result.command
                $refresh_result.exit_code
                $refresh_result.stderr
                "Run `yzx doctor` to inspect the runtime, then rerun `yzx refresh` after fixing the failing build command."
                --stderr-streamed=$refresh_result.stderr_streamed
            )
            exit 1
        }

        $built_profile = ($refresh_result.built_profile | default "")
        if ($built_profile | is-empty) {
            print "❌ Refresh completed the build but Yazelix could not resolve the resulting DEVENV_PROFILE from the build output."
            print "   Recovery: rerun `yzx refresh --verbose` and inspect the final `devenv build shell` result, or run `yzx doctor`."
            exit 1
        }
    }
    let applied_state = (compute_config_state)
    regenerate_runtime_configs $runtime_dir --quiet=($refresh_output == "quiet")
    record_materialized_state $applied_state
    if ($built_profile | is-not-empty) {
        record_launch_profile_state $applied_state $built_profile
    }

    print "✅ Refresh completed."
    print "⚠️  Your current Yazelix session keeps its existing environment."
    print "   Run 'yzx restart' to switch this window to the refreshed profile."
    print "   Or run 'yzx launch' to open a separate Yazelix window on the refreshed profile."
}
