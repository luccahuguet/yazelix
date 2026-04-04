#!/usr/bin/env nu
# yzx open - Inspect generated or managed downstream config surfaces

use ../setup/helix_config_merger.nu [get_generated_helix_config_path get_managed_helix_user_config_path]

def show_config_section [section: string] {
    let yazi_config_path = ("~/.local/share/yazelix/configs/yazi/yazi.toml" | path expand)
    let zellij_config_path = ("~/.local/share/yazelix/configs/zellij/config.kdl" | path expand)
    let helix_config_path = (get_managed_helix_user_config_path)
    let generated_helix_config_path = (get_generated_helix_config_path)

    match $section {
        "hx" => {
            {
                config_path: $helix_config_path
                config: (if ($helix_config_path | path exists) { open $helix_config_path } else { null })
                generated_config_path: $generated_helix_config_path
                generated_config: (if ($generated_helix_config_path | path exists) { open $generated_helix_config_path } else { null })
            }
        }
        "yazi" => {
            if not ($yazi_config_path | path exists) {
                error make {msg: $"Yazi config not found at ($yazi_config_path). Launch Yazelix once to generate it."}
            }
            open $yazi_config_path
        }
        "zellij" => {
            if not ($zellij_config_path | path exists) {
                error make {msg: $"Zellij config not found at ($zellij_config_path). Launch Yazelix once to generate it."}
            }
            open --raw $zellij_config_path
        }
        _ => (error make {msg: $"Unknown config section: ($section)"})
    }
}

export def "yzx open hx" [] {
    show_config_section "hx"
}

export def "yzx open yazi" [] {
    show_config_section "yazi"
}

export def "yzx open zellij" [] {
    show_config_section "zellij"
}
