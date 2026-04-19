#!/usr/bin/env nu
# Shared config diagnostics for startup, refresh, and doctor.

use config_surfaces.nu load_config_surface_from_main
use config_parser.nu collect_config_diagnostic_report
use common.nu require_yazelix_runtime_dir

export def build_config_diagnostic_report [
    config_path: string
    default_path: string
    --include-missing
] {
    let runtime_dir = (require_yazelix_runtime_dir)
    let config_surface = (
        load_config_surface_from_main $config_path
        | upsert default_config_path $default_path
    )

    collect_config_diagnostic_report $runtime_dir $config_surface --include-missing=$include_missing
}
