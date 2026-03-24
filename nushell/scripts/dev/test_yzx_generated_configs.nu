#!/usr/bin/env nu

use ./test_yzx_helpers.nu [get_repo_config_dir repo_path]
use ../utils/launch_state.nu [get_launch_env]
use ../setup/yazi_config_merger.nu [generate_merged_yazi_config]
use ../setup/zellij_config_merger.nu [generate_merged_zellij_config]
use ../utils/terminal_configs.nu [
    generate_alacritty_config
    generate_foot_config
    generate_ghostty_config
    generate_kitty_config
    generate_wezterm_config
]

def test_layout_generator_discovers_custom_top_level_layouts [] {
    print "🧪 Testing layout generator discovers custom top-level layouts..."

    let tmpdir = (^mktemp -d /tmp/yazelix_layout_generator_XXXXXX | str trim)

    let result = (try {
        let source_dir = ($tmpdir | path join "source")
        let target_dir = ($tmpdir | path join "target")
        let fragments_dir = ($source_dir | path join "fragments")
        let repo_fragments_dir = (repo_path "configs" "zellij" "layouts" "fragments")

        mkdir $source_dir
        mkdir $fragments_dir

        for fragment in [
            "zjstatus_tab_template.kdl"
            "keybinds_common.kdl"
            "swap_sidebar_open.kdl"
            "swap_sidebar_closed.kdl"
        ] {
            ^cp ($repo_fragments_dir | path join $fragment) ($fragments_dir | path join $fragment)
        }

        let custom_layout_path = ($source_dir | path join "custom_layout.kdl")
        'layout { pane }' | save --force --raw $custom_layout_path

        use ../utils/layout_generator.nu *
        generate_all_layouts $source_dir $target_dir ["layout", "editor"] "" "file:/tmp/yazelix_pane_orchestrator.wasm" $source_dir

        let generated_layout_path = ($target_dir | path join "custom_layout.kdl")
        let generated_fragments_dir = ($target_dir | path join "fragments")

        if ($generated_layout_path | path exists) and not ($generated_fragments_dir | path exists) {
            print "  ✅ Layout generator copies custom top-level layouts without copying fragments"
            true
        } else {
            print $"  ❌ Unexpected result: custom_exists=(($generated_layout_path | path exists)) fragments_copied=(($generated_fragments_dir | path exists))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_layout_generator_rewrites_runtime_paths [] {
    print "🧪 Testing layout generator rewrites runtime-root placeholders..."

    let tmpdir = (^mktemp -d /tmp/yazelix_layout_runtime_XXXXXX | str trim)

    let result = (try {
        let source_dir = ($tmpdir | path join "source")
        let target_dir = ($tmpdir | path join "target")
        let repo_layouts_dir = (repo_path "configs" "zellij" "layouts")
        let runtime_dir = ($tmpdir | path join "runtime")

        mkdir $source_dir
        mkdir $runtime_dir
        for entry in (ls $repo_layouts_dir) {
            let target_path = ($source_dir | path join ($entry.name | path basename))
            if $entry.type == dir {
                ^cp -R $entry.name $target_path
            } else {
                ^cp $entry.name $target_path
            }
        }

        use ../utils/layout_generator.nu *
        generate_all_layouts $source_dir $target_dir ["layout", "editor"] "" "file:/tmp/yazelix_pane_orchestrator.wasm" $runtime_dir

        let generated_layout = (open --raw ($target_dir | path join "yzx_side.kdl"))

        if (
            ($generated_layout | str contains $"($runtime_dir)/configs/zellij/scripts/launch_sidebar_yazi.nu")
            and ($generated_layout | str contains $"file:($runtime_dir)/configs/zellij/plugins/zjstatus.wasm")
            and ($generated_layout | str contains $"nu ($runtime_dir)/nushell/scripts/utils/zjstatus_widget.nu shell")
            and not ($generated_layout | str contains "~/.config/yazelix")
        ) {
            print "  ✅ Generated layouts stamp the configured runtime root into wrapper and widget paths"
            true
        } else {
            print "  ❌ Generated layouts still contain stale repo-shaped runtime paths"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_launch_env_omits_default_helix_runtime [] {
    print "🧪 Testing launch env omits HELIX_RUNTIME by default..."

    try {
        let cfg = {
            editor_command: "hx"
            helix_runtime_path: null
            terminals: ["ghostty"]
            default_shell: "nu"
            debug_mode: false
            enable_sidebar: true
            ascii_art_mode: "static"
            terminal_config_mode: "yazelix"
        }
        let stdout = (get_launch_env $cfg "/tmp/yazelix-profile" | get -o HELIX_RUNTIME | default "" | str trim)

        if $stdout == "" {
            print "  ✅ HELIX_RUNTIME is omitted unless explicitly configured"
            true
        } else {
            print $"  ❌ Unexpected result: stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_terminal_config_generation_rewrites_runtime_root [] {
    print "🧪 Testing terminal config generation rewrites runtime-root launch paths..."

    let runtime_dir = "/tmp/yazelix-runtime"

    try {
        let ghostty_config = (generate_ghostty_config $runtime_dir)
        let wezterm_config = (generate_wezterm_config $runtime_dir)
        let kitty_config = (generate_kitty_config $runtime_dir)
        let alacritty_config = (generate_alacritty_config $runtime_dir)
        let foot_config = (generate_foot_config $runtime_dir)

        if (
            ($ghostty_config | str contains $"exec ($runtime_dir)/shells/posix/start_yazelix.sh")
            and ($wezterm_config | str contains $"exec ($runtime_dir)/shells/posix/start_yazelix.sh")
            and ($kitty_config | str contains $"exec ($runtime_dir)/shells/posix/start_yazelix.sh")
            and ($alacritty_config | str contains $"exec ($runtime_dir)/shells/posix/start_yazelix.sh")
            and ($foot_config | str contains $"exec ($runtime_dir)/shells/posix/start_yazelix.sh")
            and not ($ghostty_config | str contains "$HOME/.config/yazelix")
            and not ($wezterm_config | str contains "$HOME/.config/yazelix")
            and not ($kitty_config | str contains "$HOME/.config/yazelix")
            and not ($alacritty_config | str contains "$HOME/.config/yazelix")
            and not ($foot_config | str contains "$HOME/.config/yazelix")
        ) {
            print "  ✅ Terminal config generation stamps the runtime-root launcher path into every supported terminal config"
            true
        } else {
            print "  ❌ One or more terminal configs still contain stale repo-shaped launch paths"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_terminal_config_renderer_uses_runtime_root_default_template [] {
    print "🧪 Testing the internal terminal config renderer uses the runtime-root default template..."

    let tmp_home = (^mktemp -d /tmp/yazelix_gen_config_XXXXXX | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    mkdir $runtime_dir

    let result = (try {
        cp (repo_path "yazelix_default.toml") ($runtime_dir | path join "yazelix_default.toml")
        let output = with-env {
            HOME: $tmp_home
            YAZELIX_RUNTIME_DIR: $runtime_dir
        } {
            ^nu -c $"use \"(repo_path "nushell" "scripts" "yzx" "gen_config.nu")\" [render_terminal_config]; render_terminal_config ghostty" | complete
        }
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains $"exec ($runtime_dir)/shells/posix/start_yazelix.sh")
            and not ($stdout | str contains "$HOME/.config/yazelix")
        ) {
            print "  ✅ The internal terminal config renderer reads the template from the runtime root and emits runtime-root launch paths"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_launch_env_keeps_custom_helix_runtime_override [] {
    print "🧪 Testing launch env preserves custom Helix runtime override..."

    try {
        let cfg = {
            editor_command: "hx"
            helix_runtime_path: "/tmp/custom-helix-runtime"
            terminals: ["ghostty"]
            default_shell: "nu"
            debug_mode: false
            enable_sidebar: true
            ascii_art_mode: "static"
            terminal_config_mode: "yazelix"
        }
        let stdout = (get_launch_env $cfg "/tmp/yazelix-profile" | get HELIX_RUNTIME | str trim)

        if $stdout == "/tmp/custom-helix-runtime" {
            print "  ✅ Custom helix_runtime_path is still exported"
            true
        } else {
            print $"  ❌ Unexpected result: stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_launch_env_omits_yazelix_default_shell [] {
    print "🧪 Testing launch env omits YAZELIX_DEFAULT_SHELL..."

    try {
        let cfg = {
            editor_command: "hx"
            helix_runtime_path: null
            terminals: ["ghostty"]
            default_shell: "fish"
            debug_mode: false
            enable_sidebar: true
            ascii_art_mode: "static"
            terminal_config_mode: "yazelix"
        }
        let stdout = (get_launch_env $cfg "/tmp/yazelix-profile" | get -o YAZELIX_DEFAULT_SHELL | default "" | str trim)

        if $stdout == "" {
            print "  ✅ YAZELIX_DEFAULT_SHELL is no longer part of the launch env"
            true
        } else {
            print $"  ❌ Unexpected result: stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_zjstatus_widget_reads_shell_from_config [] {
    print "🧪 Testing zjstatus shell widget reads current config..."

    try {
        let widget_script = (repo_path "nushell" "scripts" "utils" "zjstatus_widget.nu")
        let tmpdir = (^mktemp -d /tmp/yazelix_widget_shell_XXXXXX | str trim)
        '[shell]
default_shell = "nu"
' | save --force --raw ($tmpdir | path join "yazelix.toml")
        let output = (with-env {
            YAZELIX_CONFIG_OVERRIDE: ($tmpdir | path join "yazelix.toml")
            YAZELIX_DEFAULT_SHELL: "fish"
        } {
            ^nu $widget_script shell | complete
        })
        rm -rf $tmpdir
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "nu") {
            print "  ✅ Shell widget ignores stale YAZELIX_DEFAULT_SHELL env"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_zjstatus_widget_reads_editor_from_config [] {
    print "🧪 Testing zjstatus editor widget reads current config..."

    try {
        let widget_script = (repo_path "nushell" "scripts" "utils" "zjstatus_widget.nu")
        let tmpdir = (^mktemp -d /tmp/yazelix_widget_editor_XXXXXX | str trim)
        '[editor]
command = "nvim --headless"
' | save --force --raw ($tmpdir | path join "yazelix.toml")
        let output = (with-env {
            YAZELIX_CONFIG_OVERRIDE: ($tmpdir | path join "yazelix.toml")
            EDITOR: "fish"
        } {
            ^nu $widget_script editor | complete
        })
        rm -rf $tmpdir
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "nvim") {
            print "  ✅ Editor widget ignores stale EDITOR env"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_zjstatus_custom_text_is_trimmed_and_truncated_in_config_parser [] {
    print "🧪 Testing zjstatus custom text is normalized in config parsing..."

    let tmpdir = (^mktemp -d /tmp/yazelix_widget_custom_text_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        '[zellij]
custom_text = "  notes[]{}12345  "
' | save --force --raw $config_path

        let parsed = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            use ../utils/config_parser.nu [parse_yazelix_config]
            parse_yazelix_config
        })
        let normalized = ($parsed | get zellij_custom_text)

        if $normalized == "notes123" {
            print "  ✅ Config parser trims, sanitizes, and caps zjstatus custom text to 8 characters"
            true
        } else {
            print $"  ❌ Unexpected result: normalized=($normalized)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def write_minimal_user_zellij_config [fake_home: string] {
    let zellij_config_dir = ($fake_home | path join ".config" "zellij")
    let zellij_config_path = ($zellij_config_dir | path join "config.kdl")
    mkdir $zellij_config_dir
    'keybinds { normal { bind "f1" { WriteChars "fixture"; } } }'
        | save --force --raw $zellij_config_path
}

def test_generated_zellij_layout_omits_empty_custom_text_badge [] {
    print "🧪 Testing generated Zellij layout omits empty zjstatus custom text..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_badge_empty_XXXXXX | str trim)

    let result = (try {
        let out_dir = ($tmpdir | path join "out")
        let fake_home = ($tmpdir | path join "home")
        write_minimal_user_zellij_config $fake_home

        let generated_layout = (with-env {
            HOME: $fake_home
            YAZELIX_TEST_OUT_DIR: $out_dir
        } {
            let root = (get_repo_config_dir)
            generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore
            open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "layouts" "yzx_side.kdl")
        })
        let stdout = ($generated_layout | str trim)

        if (
            ($stdout | str contains '#[fg=#00ccff,bold]YAZELIX {command_version}')
            and not ($stdout | str contains '#[fg=#ffff00,bold][')
        ) {
            print "  ✅ Empty zjstatus custom text stays invisible in the generated layout"
            true
        } else {
            print "  ❌ Unexpected result: empty custom text still rendered a badge"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_generated_zellij_layout_renders_capped_custom_text_before_branding [] {
    print "🧪 Testing generated Zellij layout renders capped zjstatus custom text before branding..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_badge_render_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        let out_dir = ($tmpdir | path join "out")
        let fake_home = ($tmpdir | path join "home")
        write_minimal_user_zellij_config $fake_home
        '[zellij]
custom_text = "  roadmap-2026  "
' | save --force --raw $config_path

        let generated_layout = (with-env {
            HOME: $fake_home
            YAZELIX_CONFIG_OVERRIDE: $config_path
            YAZELIX_TEST_OUT_DIR: $out_dir
        } {
            let root = (get_repo_config_dir)
            generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore
            open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "layouts" "yzx_side.kdl")
        })
        let stdout = ($generated_layout | str trim)
        let expected_segment = '#[fg=#ffff00,bold][roadmap-] #[fg=#00ccff,bold]YAZELIX {command_version}'

        if ($stdout | str contains $expected_segment) {
            print "  ✅ Rendered zjstatus custom text is capped to 8 characters and placed before YAZELIX"
            true
        } else {
            print $"  ❌ Unexpected result: expected_segment=($expected_segment)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_sidebar_state_plugin_generated [] {
    print "🧪 Testing generated Yazi init includes sidebar-state..."

    try {
        let root = (get_repo_config_dir)
        generate_merged_yazi_config $root --quiet
        let stdout = (open --raw ($env.HOME | path join ".local" "share" "yazelix" "configs" "yazi" "init.lua") | str trim)

        if ($stdout | str contains 'require("sidebar-state"):setup()') {
            print "  ✅ Generated Yazi init loads the sidebar-state core plugin"
            true
        } else {
            print "  ❌ Unexpected result: generated init is missing sidebar-state"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_zellij_default_mode_is_enforced_in_merged_config [] {
    print "🧪 Testing merged Zellij config enforces default_mode..."

    if (which zellij | is-empty) {
        print "  ℹ️  Skipping Zellij config merge test because zellij is not available"
        return true
    }

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_mode_test_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        let out_dir = ($tmpdir | path join "out")
        '[zellij]
default_mode = "locked"
' | save --force --raw $config_path

        let output = (with-env {
            YAZELIX_CONFIG_OVERRIDE: $config_path
            YAZELIX_TEST_OUT_DIR: $out_dir
        } {
            let root = (get_repo_config_dir)
            generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore
            open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "config.kdl")
        })
        let stdout = ($output | str trim)

        if ($stdout | str contains 'default_mode "locked"') {
            print "  ✅ Generated Zellij config enforces the configured default_mode"
            true
        } else {
            print "  ❌ Unexpected result: generated config is missing default_mode"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_zellij_horizontal_walking_is_plugin_owned [] {
    print "🧪 Testing Yazelix-owned Zellij session keybinds are emitted in merged config..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_walk_test_XXXXXX | str trim)

    let result = (try {
        let out_dir = ($tmpdir | path join "out")
        let fake_home = ($tmpdir | path join "home")
        let zellij_config_dir = ($fake_home | path join ".config" "zellij")
        let zellij_config_path = ($zellij_config_dir | path join "config.kdl")
        mkdir $zellij_config_dir
        'keybinds { normal { bind "f1" { WriteChars "fixture"; } } }'
            | save --force --raw $zellij_config_path

        let output = (with-env { HOME: $fake_home, YAZELIX_TEST_OUT_DIR: $out_dir } {
            let root = (get_repo_config_dir)
            generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore
            {
                config: (open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "config.kdl"))
                layout: (open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "layouts" "yzx_side.kdl"))
            }
        })
        let config_stdout = ($output.config | str trim)
        let layout_stdout = ($output.layout | str trim)

        if (
            ($config_stdout | str contains 'bind "Alt h" "Alt Left" {')
            and ($config_stdout | str contains 'name "move_focus_left_or_tab"')
            and ($config_stdout | str contains 'bind "Alt l" "Alt Right" {')
            and ($config_stdout | str contains 'name "move_focus_right_or_tab"')
            and ($config_stdout | str contains 'bind "Alt t" {')
            and ($config_stdout | str contains 'yzx_toggle_popup.nu')
            and ($config_stdout | str contains 'yazelix_popup_runner.wasm')
            and not ($layout_stdout | str contains 'bind "Alt h" "Alt Left" {')
            and not ($layout_stdout | str contains 'bind "Alt t" {')
        ) {
            print "  ✅ Yazelix-owned session keybinds are emitted in merged config instead of layout-local keybind blocks"
            true
        } else {
            print "  ❌ Unexpected result: merged config/layout ownership for Yazelix keybinds is wrong"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

export def run_generated_config_canonical_tests [] {
    [
        (test_layout_generator_rewrites_runtime_paths)
        (test_zellij_default_mode_is_enforced_in_merged_config)
    ]
}

export def run_generated_config_noncanonical_tests [] {
    [
        (test_generated_zellij_layout_renders_capped_custom_text_before_branding)
        (test_zellij_horizontal_walking_is_plugin_owned)
    ]
}

export def run_generated_config_tests [] {
    [
        (run_generated_config_canonical_tests)
        (run_generated_config_noncanonical_tests)
    ] | flatten
}
