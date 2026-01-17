#!/usr/bin/env nu
# Sweep Testing - Configuration Generation Utilities
# Generates temporary Yazelix configurations for testing different combinations (TOML format)

# Helper: build TOML config structure for sweep tests
def build_sweep_config [
    shell: string,
    terminal: string,
    features: record,
    test_id: string
] : nothing -> record {
    {
        core: {
            recommended_deps: ($features.recommended_deps? | default true)
            yazi_extensions: ($features.yazi_extensions? | default true)
            yazi_media: false
            debug_mode: false
            skip_welcome_screen: true
            show_macchina_on_welcome: false
        }
        helix: {
            mode: ($features.helix_mode? | default "release")
        }
        editor: {
            command: ""
            enable_sidebar: ($features.enable_sidebar? | default true)
        }
        shell: {
            default_shell: $shell
            extra_shells: []
            enable_atuin: false
        }
        terminal: {
            terminals: ([$terminal "ghostty" "wezterm" "kitty" "alacritty" "foot"] | uniq)
            config_mode: "yazelix"
            cursor_trail: "none"
            transparency: "none"
        }
        zellij: {
            disable_tips: true
            rounded_corners: true
            persistent_sessions: ($features.persistent_sessions? | default false)
            session_name: $"sweep_test_($test_id)"
        }
        ascii: {
            mode: "static"
        }
        packs: {
            enabled: []
            declarations: {}
            user_packages: []
        }
    }
}

# Generate temporary yazelix.toml config for testing
export def generate_sweep_config [
    shell: string,
    terminal: string,
    features: record,
    test_id: string
]: nothing -> string {
    let temp_dir = $"($env.HOME)/.local/share/yazelix/sweep_tests"
    mkdir $temp_dir

    let config_path = $"($temp_dir)/yazelix_test_($test_id).toml"
    let config_content = (build_sweep_config $shell $terminal $features $test_id | to toml)

    $config_content | save --force --raw $config_path
    $config_path
}

# Clean up a single test config file
export def cleanup_test_config [config_path: string]: nothing -> nothing {
    if ($config_path | path exists) {
        rm $config_path
    }
}

# Clean up temporary test configs
export def cleanup_sweep_configs []: nothing -> nothing {
    let temp_dir = $"($env.HOME)/.local/share/yazelix/sweep_tests"
    if ($temp_dir | path exists) {
        rm -rf $temp_dir
    }
}
