#!/usr/bin/env nu
# Internal terminal config rendering helper

use ../utils/common.nu [get_yazelix_runtime_dir]
use ../utils/constants.nu [SUPPORTED_TERMINALS]
use ../utils/terminal_configs.nu [
    generate_ghostty_config
    generate_wezterm_config
    generate_kitty_config
    generate_alacritty_config
    generate_foot_config
]

export def render_terminal_config [terminal: string, runtime_dir?: string] {
    let selected = ($terminal | str downcase | str trim)
    if ($selected | is-empty) {
        error make {msg: "Terminal name is required."}
    }

    if $selected not-in $SUPPORTED_TERMINALS {
        let supported = ($SUPPORTED_TERMINALS | str join ", ")
        error make {msg: $"Unsupported terminal: ($terminal). Supported: ($supported)"}
    }

    let resolved_runtime_dir = (($runtime_dir | default (get_yazelix_runtime_dir)) | path expand)
    let default_config = ($resolved_runtime_dir | path join "yazelix_default.toml")
    if not ($default_config | path exists) {
        error make {msg: $"Default config not found: ($default_config)"}
    }

    with-env {YAZELIX_CONFIG_OVERRIDE: $default_config, YAZELIX_RUNTIME_DIR: $resolved_runtime_dir} {
        match $selected {
            "ghostty" => (generate_ghostty_config)
            "wezterm" => (generate_wezterm_config)
            "kitty" => (generate_kitty_config)
            "alacritty" => (generate_alacritty_config)
            "foot" => (generate_foot_config)
            _ => (error make {msg: $"Unsupported terminal: ($terminal)"})
        }
    }
}
