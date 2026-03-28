#!/usr/bin/env nu
# Shared helpers for loading Yazelix config surfaces.

use common.nu [get_yazelix_config_dir get_yazelix_runtime_dir]
use failure_classes.nu [format_failure_classification]

export const PACK_SIDECAR_FILENAME = "yazelix_packs.toml"

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

export def get_pack_sidecar_path [config_file: string] {
    ($config_file | path dirname | path join $PACK_SIDECAR_FILENAME)
}

export def copy_default_config_surfaces [
    default_config_path: string
    target_config_path: string
] {
    let default_pack_path = (get_pack_sidecar_path $default_config_path)
    let target_pack_path = (get_pack_sidecar_path $target_config_path)

    mkdir ($target_config_path | path dirname)
    cp $default_config_path $target_config_path

    if ($default_pack_path | path exists) {
        cp $default_pack_path $target_pack_path
    } else if ($target_pack_path | path exists) {
        rm $target_pack_path
    }

    {
        config_path: $target_config_path
        pack_config_path: $target_pack_path
        pack_config_copied: ($default_pack_path | path exists)
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
            "Move every [packs] entry out of yazelix.toml and into yazelix_packs.toml, or delete yazelix_packs.toml if you want to keep packs in the main file."
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
    let pack_config_file = (get_pack_sidecar_path $config_file)
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
    let yazelix_config_dir = get_yazelix_config_dir
    let yazelix_runtime_dir = get_yazelix_runtime_dir

    let config_file = if ($env.YAZELIX_CONFIG_OVERRIDE? | is-not-empty) {
        $env.YAZELIX_CONFIG_OVERRIDE
    } else {
        let toml_file = ($yazelix_config_dir | path join "yazelix.toml")
        let default_toml = ($yazelix_runtime_dir | path join "yazelix_default.toml")

        if ($toml_file | path exists) {
            $toml_file
        } else if ($default_toml | path exists) {
            print "📝 Creating yazelix.toml from yazelix_default.toml..."
            let copy_result = (copy_default_config_surfaces $default_toml $toml_file)
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
        default_config_path: ($yazelix_runtime_dir | path join "yazelix_default.toml")
    }
}

export def load_active_config_surface [] {
    let resolved = resolve_active_config_paths
    let loaded = (load_config_surface_from_main $resolved.config_file)
    $loaded | merge {
        default_config_path: $resolved.default_config_path
    }
}
