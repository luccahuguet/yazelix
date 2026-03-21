#!/usr/bin/env nu

use ./test_yzx_helpers.nu [get_repo_config_dir repo_path]
use ../utils/launch_state.nu [get_launch_env]
use ../setup/yazi_config_merger.nu [generate_merged_yazi_config]
use ../setup/zellij_config_merger.nu [generate_merged_zellij_config]

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
        generate_all_layouts $source_dir $target_dir ["layout", "editor"] "file:/tmp/yazelix_pane_orchestrator.wasm"

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

export def run_generated_config_tests [] {
    [
        (test_layout_generator_discovers_custom_top_level_layouts)
        (test_launch_env_omits_default_helix_runtime)
        (test_launch_env_keeps_custom_helix_runtime_override)
        (test_launch_env_omits_yazelix_default_shell)
        (test_zjstatus_widget_reads_shell_from_config)
        (test_zjstatus_widget_reads_editor_from_config)
        (test_sidebar_state_plugin_generated)
        (test_zellij_default_mode_is_enforced_in_merged_config)
    ]
}
