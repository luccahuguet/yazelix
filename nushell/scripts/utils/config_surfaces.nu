#!/usr/bin/env nu
# Shared helpers for loading Yazelix config surfaces.

use atomic_writes.nu write_text_atomic
use common.nu [get_yazelix_config_dir get_yazelix_user_config_dir require_yazelix_runtime_dir]
use failure_classes.nu [format_failure_classification]

export const MAIN_CONFIG_FILENAME = "yazelix.toml"
export const TAPLO_SUPPORT_FILENAME = ".taplo.toml"

def make_surface_error [headline: string, details: list<string>, recovery_hint: string] {
    error make {
        msg: (
            [
                $headline
                ...$details
                ""
                (format_failure_classification "config" $recovery_hint)
            ] | str join "\n"
        )
    }
}

def ensure_record_surface [value: any, label: string, path: string] {
    if (($value | describe) | str contains "record") {
        $value
    } else {
        (make_surface_error
            $"Invalid ($label) format at ($path)."
            [
                $"($label) must be a TOML record."
            ]
            $"Rewrite ($path) as a TOML table file, or remove it if you do not want to use that config surface."
        )
    }
}

export def get_main_user_config_path [config_root?: string] {
    (get_yazelix_user_config_dir $config_root) | path join $MAIN_CONFIG_FILENAME
}

export def normalize_config_surface_path [surface_path: string] {
    $surface_path | path expand --no-symlink
}

def get_legacy_main_config_path [config_root?: string] {
    let root = if $config_root == null { get_yazelix_config_dir } else { $config_root | path expand }
    ($root | path join $MAIN_CONFIG_FILENAME)
}

export def get_managed_taplo_support_path [config_root?: string] {
    let root = if $config_root == null { get_yazelix_config_dir } else { $config_root | path expand }
    ($root | path join $TAPLO_SUPPORT_FILENAME)
}

def get_runtime_taplo_support_path [runtime_root: string] {
    ($runtime_root | path join $TAPLO_SUPPORT_FILENAME)
}

export def copy_default_config_surfaces [
    default_config_path: string
    target_config_path: string
] {
    mkdir ($target_config_path | path dirname)
    cp $default_config_path $target_config_path
    ^chmod u+w $target_config_path

    {
        config_path: $target_config_path
    }
}

export def ensure_current_primary_config_surface [paths: record] {
    let current_exists = ($paths.user_config | path exists)
    let legacy_exists = ($paths.legacy_user_config | path exists)

    if $current_exists and $legacy_exists {
        (make_surface_error
            "Yazelix found duplicate config surfaces in both the repo root and user_configs."
            [
                $"user_configs main: ($paths.user_config)"
                $"legacy main: ($paths.legacy_user_config)"
            ]
            "Keep only the user_configs copies. Move or delete the legacy root-level config files so Yazelix has one clear config owner."
        )
    }

    if $legacy_exists {
        (make_surface_error
            "Yazelix found an unsupported legacy root-level config surface."
            [
                $"legacy main: ($paths.legacy_user_config)"
                $"current main: ($paths.user_config)"
            ]
            "Move your current Yazelix config to user_configs/yazelix.toml manually, or run `yzx config reset` to create a fresh v15 config template."
        )
    }
}

export def reconcile_primary_config_surfaces [config_root?: string, runtime_root?: string] {
    let paths = (get_primary_config_paths $config_root $runtime_root)
    ensure_current_primary_config_surface $paths
    ensure_managed_taplo_support $paths.config_dir $paths.runtime_root | ignore
    $paths
}

export def get_primary_config_paths [config_root?: string, runtime_root?: string] {
    let resolved_config_root = if $config_root == null { get_yazelix_config_dir } else { $config_root | path expand }
    let resolved_runtime_root = if $runtime_root == null { require_yazelix_runtime_dir } else { $runtime_root | path expand }
    let user_config_dir = (get_yazelix_user_config_dir $resolved_config_root)

    {
        config_dir: $resolved_config_root
        runtime_root: $resolved_runtime_root
        user_config_dir: $user_config_dir
        user_config: (get_main_user_config_path $resolved_config_root)
        legacy_user_config: (get_legacy_main_config_path $resolved_config_root)
        default_config: ($resolved_runtime_root | path join "yazelix_default.toml")
    }
}

def ensure_managed_taplo_support [config_root?: string, runtime_root?: string] {
    let resolved_config_root = if $config_root == null { get_yazelix_config_dir } else { $config_root | path expand }
    let resolved_runtime_root = if $runtime_root == null { require_yazelix_runtime_dir } else { $runtime_root | path expand }
    let source_path = (get_runtime_taplo_support_path $resolved_runtime_root)
    let target_path = (get_managed_taplo_support_path $resolved_config_root)

    if not ($source_path | path exists) {
        (make_surface_error
            "Yazelix runtime is missing the Taplo formatter config."
            [
                $"runtime support file: ($source_path)"
            ]
            "Reinstall Yazelix so the runtime includes the managed Taplo formatter config."
        )
    }

    mkdir $resolved_config_root
    let source_content = (open --raw $source_path)
    let should_write = if ($target_path | path exists) {
        (open --raw $target_path) != $source_content
    } else {
        true
    }

    if $should_write {
        if ($target_path | path exists) {
            ^chmod u+w $target_path
        }
        write_text_atomic $target_path $source_content --raw | ignore
        ^chmod u+w $target_path
    }

    $target_path
}

export def load_config_surface_pair [config_file: string] {
    let main_config = (ensure_record_surface (open $config_file) "main config" $config_file)

    {
        config_file: $config_file
        main_config: $main_config
        merged_config: $main_config
        display_config_path: $config_file
    }
}

export def load_config_surface_from_main [config_file: string] {
    load_config_surface_pair $config_file
}

def resolve_active_config_paths [] {
    let paths = (reconcile_primary_config_surfaces)

    let config_file = if ($env.YAZELIX_CONFIG_OVERRIDE? | is-not-empty) {
        $env.YAZELIX_CONFIG_OVERRIDE
    } else {
        if ($paths.user_config | path exists) {
            $paths.user_config
        } else if ($paths.default_config | path exists) {
            print "📝 Creating yazelix.toml from yazelix_default.toml..."
            let copy_result = (copy_default_config_surfaces $paths.default_config $paths.user_config)
            print "✅ yazelix.toml created\n"
            $copy_result.config_path
        } else {
            (make_surface_error
                "No yazelix configuration file found."
                [
                    "yazelix_default.toml is missing from the runtime."
                ]
                "Restore yazelix_default.toml, or reinstall Yazelix if the default config is missing from the runtime."
            )
        }
    }

    {
        config_file: $config_file
        default_config_path: $paths.default_config
    }
}

export def load_active_config_surface [] {
    let resolved = resolve_active_config_paths
    let loaded = (load_config_surface_from_main $resolved.config_file)
    $loaded | merge {
        default_config_path: $resolved.default_config_path
    }
}
