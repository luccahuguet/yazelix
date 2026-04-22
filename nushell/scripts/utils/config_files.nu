#!/usr/bin/env nu
# Non-owning file helpers for explicit Yazelix config paths.

use failure_classes.nu format_failure_classification

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
