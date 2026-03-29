#!/usr/bin/env nu
# Shared helpers for loading Yazelix config surfaces.

use common.nu [get_yazelix_config_dir get_yazelix_runtime_dir get_yazelix_user_config_dir]
use failure_classes.nu [format_failure_classification]

export const MAIN_CONFIG_FILENAME = "yazelix.toml"
export const PACK_SIDECAR_FILENAME = "yazelix_packs.toml"
export const PACK_DEFAULT_FILENAME = "yazelix_packs_default.toml"

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

export def get_pack_sidecar_path [config_file_or_root?: string] {
    if $config_file_or_root == null {
        return ((get_yazelix_user_config_dir) | path join $PACK_SIDECAR_FILENAME)
    }

    let candidate = ($config_file_or_root | path expand)
    if ($candidate | path basename) == $MAIN_CONFIG_FILENAME {
        ($candidate | path dirname | path join $PACK_SIDECAR_FILENAME)
    } else {
        ($candidate | path join $PACK_SIDECAR_FILENAME)
    }
}

export def get_pack_default_path [default_config_path: string] {
    ($default_config_path | path dirname | path join $PACK_DEFAULT_FILENAME)
}

export def get_legacy_main_config_path [config_root?: string] {
    let root = if $config_root == null { get_yazelix_config_dir } else { $config_root | path expand }
    ($root | path join $MAIN_CONFIG_FILENAME)
}

export def get_legacy_pack_sidecar_path [config_root?: string] {
    let root = if $config_root == null { get_yazelix_config_dir } else { $config_root | path expand }
    ($root | path join $PACK_SIDECAR_FILENAME)
}

def get_associated_pack_surface_path [config_file: string] {
    if ($config_file | path basename) == "yazelix_default.toml" {
        get_pack_default_path $config_file
    } else {
        get_pack_sidecar_path $config_file
    }
}

export def copy_default_config_surfaces [
    default_config_path: string
    target_config_path: string
] {
    let default_pack_path = (get_pack_default_path $default_config_path)
    let target_pack_path = (get_pack_sidecar_path $target_config_path)

    mkdir ($target_config_path | path dirname)
    cp $default_config_path $target_config_path

    let pack_config_copied = if ($default_pack_path | path exists) and not ($target_pack_path | path exists) {
        cp $default_pack_path $target_pack_path
        true
    } else if ($target_pack_path | path exists) {
        false
    } else {
        false
    }

    {
        config_path: $target_config_path
        pack_config_path: $target_pack_path
        pack_config_copied: $pack_config_copied
    }
}

def relocate_legacy_config_surfaces_if_needed [paths: record] {
    let current_exists = ($paths.user_config | path exists)
    let current_pack_exists = ($paths.user_pack_config | path exists)
    let legacy_exists = ($paths.legacy_user_config | path exists)
    let legacy_pack_exists = ($paths.legacy_pack_config | path exists)

    if ($current_exists or $current_pack_exists) and ($legacy_exists or $legacy_pack_exists) {
        (make_surface_error
            "Yazelix found duplicate config surfaces in both the repo root and user_configs."
            [
                $"user_configs main: ($paths.user_config)"
                $"user_configs packs: ($paths.user_pack_config)"
                $"legacy main: ($paths.legacy_user_config)"
                $"legacy packs: ($paths.legacy_pack_config)"
            ]
            "Keep only the user_configs copies. Move or delete the legacy root-level config files so Yazelix has one clear config owner."
        )
    }

    if not ($legacy_exists or $legacy_pack_exists) {
        return
    }

    mkdir $paths.user_config_dir

    if $legacy_exists {
        mv $paths.legacy_user_config $paths.user_config
    }

    if $legacy_pack_exists {
        mv $paths.legacy_pack_config $paths.user_pack_config
    }
}

export def reconcile_primary_config_surfaces [config_root?: string, runtime_root?: string] {
    let paths = (get_primary_config_paths $config_root $runtime_root)
    relocate_legacy_config_surfaces_if_needed $paths
    $paths
}

export def get_primary_config_paths [config_root?: string, runtime_root?: string] {
    let resolved_config_root = if $config_root == null { get_yazelix_config_dir } else { $config_root | path expand }
    let resolved_runtime_root = if $runtime_root == null { get_yazelix_runtime_dir } else { $runtime_root | path expand }
    let user_config_dir = (get_yazelix_user_config_dir $resolved_config_root)

    {
        config_dir: $resolved_config_root
        user_config_dir: $user_config_dir
        user_config: (get_main_user_config_path $resolved_config_root)
        user_pack_config: (get_pack_sidecar_path (get_main_user_config_path $resolved_config_root))
        legacy_user_config: (get_legacy_main_config_path $resolved_config_root)
        legacy_pack_config: (get_legacy_pack_sidecar_path $resolved_config_root)
        default_config: ($resolved_runtime_root | path join "yazelix_default.toml")
        default_pack_config: ($resolved_runtime_root | path join $PACK_DEFAULT_FILENAME)
    }
}

export def merge_pack_sidecar [
    main_config: record
    config_path: string
    pack_path: string
    pack_config?: any
] {
    if $pack_config == null {
        return $main_config
    }

    let validated_pack_config = (ensure_record_surface $pack_config "pack sidecar" $pack_path)
    let main_has_packs = ("packs" in ($main_config | columns))

    if $main_has_packs {
        (make_surface_error
            "Yazelix found pack settings in both yazelix.toml and yazelix_packs.toml."
            [
                $"Main config: ($config_path)"
                $"Pack sidecar: ($pack_path)"
                "When yazelix_packs.toml exists, it fully owns pack settings."
            ]
            "Because both files already define packs, `yzx config migrate --apply` cannot safely choose which file should define the pack settings. Remove the duplicate [packs] entry from yazelix.toml or move the desired pack settings fully into yazelix_packs.toml so only yazelix_packs.toml defines packs. If you want to discard custom pack edits and restore the shipped config surfaces instead, run `yzx config reset --yes` as the blunt fallback."
        )
    }

    if ("packs" in ($validated_pack_config | columns)) {
        (make_surface_error
            "Yazelix found an invalid pack sidecar shape."
            [
                $"Pack sidecar: ($pack_path)"
                "yazelix_packs.toml is already a dedicated pack file."
                "Do not wrap it in an extra [packs] table."
            ]
            "Keep pack fields at the top level of yazelix_packs.toml, for example enabled = [...] and [declarations]."
        )
    }

    $main_config | upsert packs $validated_pack_config
}

export def load_config_surface_from_main [config_file: string] {
    let main_config = (ensure_record_surface (open $config_file) "main config" $config_file)
    let pack_config_file = (get_associated_pack_surface_path $config_file)
    let pack_config = if ($pack_config_file | path exists) {
        open $pack_config_file
    } else {
        null
    }
    let merged_config = (merge_pack_sidecar $main_config $config_file $pack_config_file $pack_config)

    {
        config_file: $config_file
        pack_config_file: $pack_config_file
        pack_config_exists: ($pack_config != null)
        main_config: $main_config
        pack_config: $pack_config
        merged_config: $merged_config
        display_config_path: (
            if ($pack_config != null) {
                $"($config_file) + ($pack_config_file)"
            } else {
                $config_file
            }
        )
    }
}

export def resolve_active_config_paths [] {
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
