#!/usr/bin/env nu
# yzx gen_config command - Generate terminal config output

use ../utils/common.nu [get_yazelix_runtime_dir]
use ../utils/constants.nu [SUPPORTED_TERMINALS]
use ../utils/terminal_configs.nu [
    generate_ghostty_config
    generate_wezterm_config
    generate_kitty_config
    generate_alacritty_config
    generate_foot_config
]

export def "yzx gen_config" [terminal: string] {
    let selected = ($terminal | str downcase | str trim)
    if ($selected | is-empty) {
        print "Usage: yzx gen_config <terminal>"
        return
    }

    if $selected not-in $SUPPORTED_TERMINALS {
        let supported = ($SUPPORTED_TERMINALS | str join ", ")
        error make {msg: $"Unsupported terminal: ($terminal). Supported: ($supported)"}
    }

    let runtime_dir = (get_yazelix_runtime_dir)
    let default_config = ($runtime_dir | path join "yazelix_default.toml")
    if not ($default_config | path exists) {
        error make {msg: $"Default config not found: ($default_config)"}
    }

    with-env {YAZELIX_CONFIG_OVERRIDE: $default_config} {
        match $selected {
            "ghostty" => (generate_ghostty_config $runtime_dir)
            "wezterm" => (generate_wezterm_config $runtime_dir)
            "kitty" => (generate_kitty_config $runtime_dir)
            "alacritty" => (generate_alacritty_config $runtime_dir)
            "foot" => (generate_foot_config $runtime_dir)
            _ => (error make {msg: $"Unsupported terminal: ($terminal)"})
        }
    }
}
