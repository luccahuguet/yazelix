#!/usr/bin/env nu
# Test script for yzx CLI commands

use ../core/yazelix.nu *
use ../integrations/yazi.nu [consume_bootstrap_sidebar_cwd]

const clean_zellij_env_prefix = "env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID -u ZELLIJ_TAB_NAME -u ZELLIJ_TAB_POSITION"

def test_yzx_help [] {
    print "🧪 Testing yzx help..."

    try {
        let output = (yzx | str join "\n")

        # Check for key elements in auto-generated help output
        let required_elements = [
            "Usage:",
            "Subcommands:",
            "yzx doctor",
            "yzx launch",
            "yzx dev"
        ]

        for element in $required_elements {
            if not ($output | str contains $element) {
                print $"  ❌ Missing element: ($element)"
                return false
            }
        }

        print "  ✅ Help output contains all required elements"
        true
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_status [] {
    print "🧪 Testing yzx status..."

    try {
        yzx status | ignore
        print "  ✅ yzx status runs successfully"
        true
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_status_versions [] {
    print "🧪 Testing yzx status --versions..."

    try {
        let output = (
            ^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx status --versions" | complete
        ).stdout

        # Check for core tools
        let expected_tools = [
            "zellij",
            "yazi",
            "helix",
            "nushell"
        ]

        for tool in $expected_tools {
            if not ($output | str contains $tool) {
                print $"  ❌ Missing tool: ($tool)"
                return false
            }
        }

        print "  ✅ Versions output contains expected tools"
        true
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_why [] {
    print "🧪 Testing yzx why..."

    try {
        # Just verify the command runs without error
        # (yzx why uses print, which doesn't produce pipeline output)
        yzx why | ignore
        print "  ✅ yzx why runs successfully"
        true
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_status_verbose [] {
    print "🧪 Testing yzx status --verbose..."

    try {
        let output = (
            ^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx status --verbose" | complete
        ).stdout

        # Check for shell entries
        let shells = ["bash", "nushell", "fish", "zsh"]

        for shell in $shells {
            if not ($output | str contains $shell) {
                print $"  ⚠️  Missing shell in output: ($shell)"
            }
        }

        print "  ✅ Status verbose output generated"
        true
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_dev_exists [] {
    print "🧪 Testing yzx dev command exists..."

    try {
        # Just check that help mentions the dev command surface
        let output = (yzx | str join "\n")

        if ($output | str contains "yzx dev") {
            print "  ✅ yzx dev command is documented in help"
            true
        } else {
            print "  ❌ yzx dev command not found in help"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_dev_update_canary_set [] {
    print "🧪 Testing yzx dev update canary set..."

    try {
        let output = (^nu -c "source ~/.config/yazelix/nushell/scripts/yzx/dev.nu; get_available_update_canaries | to json -r" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "[\"default\",\"maximal\"]") {
            print "  ✅ yzx dev update exposes the expected canary set"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_dev_update_defaults_to_verbose_mode [] {
    print "🧪 Testing yzx dev update defaults to verbose mode..."

    try {
        let output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; help 'yzx dev update'" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "--quiet") and ($stdout | str contains "verbose by default") {
            print "  ✅ yzx dev update documents verbose-by-default behavior and the quiet opt-out"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_dev_update_help_mentions_optional_input_name [] {
    print "🧪 Testing yzx dev update documents the optional input name..."

    try {
        let output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; help 'yzx dev update'" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "input_name <string>") and ($stdout | str contains "devenv update") {
            print "  ✅ yzx dev update documents the optional input passthrough"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_gemini_cli_is_reactivated [] {
    print "🧪 Testing Gemini CLI is reactivated in default and Home Manager configs..."

    try {
        let default_config = (open ~/.config/yazelix/yazelix_default.toml)
        let default_agents = ($default_config.packs.declarations.ai_agents | default [])
        let hm_module = (open --raw ~/.config/yazelix/home_manager/module.nix)

        if ("gemini-cli" in $default_agents) and ($hm_module | str contains '"gemini-cli"') {
            print "  ✅ Gemini CLI is present in both configuration paths"
            true
        } else {
            print "  ❌ Gemini CLI is missing from the default config or Home Manager module"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_runtime_pin_versions_use_repo_shell [] {
    print "🧪 Testing runtime pin versions come from the repo shell..."

    try {
        let output = (^nu -c 'source ~/.config/yazelix/nushell/scripts/yzx/dev.nu; let versions = (get_runtime_pin_versions); print ({ nix_version: $versions.nix_version, devenv_version: $versions.devenv_version, nix_raw: (get_tool_version_from_repo_shell "nix"), devenv_raw: (get_tool_version_from_repo_shell "devenv") } | to json -r)' | complete)
        let stdout = ($output.stdout | str trim)
        let resolved = ($stdout | from json)

        if ($output.exit_code == 0) and ($resolved.nix_raw | str contains $resolved.nix_version) and ($resolved.devenv_raw | str contains $resolved.devenv_version) {
            print "  ✅ Runtime pins are derived from the repo shell versions"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) resolved=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_consume_bootstrap_sidebar_cwd [] {
    print "🧪 Testing restart-only sidebar Yazi cwd bootstrap..."

    let tmpdir = (^mktemp -d /tmp/yazelix_sidebar_bootstrap_XXXXXX | str trim)

    let result = (try {
        let workspace_dir = ($tmpdir | path join "workspace")
        mkdir $workspace_dir
        let bootstrap_file = ($tmpdir | path join "sidebar_cwd.txt")
        $workspace_dir | save --force --raw $bootstrap_file

        let resolved = (with-env {YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE: $bootstrap_file} {
            consume_bootstrap_sidebar_cwd
        })

        if ($resolved == $workspace_dir) and (not ($bootstrap_file | path exists)) {
            print "  ✅ Sidebar Yazi bootstrap cwd is consumed exactly once"
            true
        } else {
            print $"  ❌ Unexpected result: resolved=($resolved) file_exists=(($bootstrap_file | path exists))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_restart_uses_home_for_future_tab_defaults [] {
    print "🧪 Testing restart keeps pane and tab defaults at HOME..."

    try {
        let output = (^nu -c "source ~/.config/yazelix/nushell/scripts/core/start_yazelix_inner.nu; with-env {HOME: '/tmp/yazelix-home', YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE: '/tmp/sidebar-bootstrap'} { print ({ session_default: (resolve_session_default_cwd '/tmp/restart-workspace'), launch_process: (resolve_launch_process_cwd '/tmp/restart-workspace') } | to json -r) }" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == '{"session_default":"/tmp/yazelix-home","launch_process":"/tmp/yazelix-home"}') {
            print "  ✅ Restart keeps both the launch process and future tab defaults at HOME"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_sidebar_layout_uses_wrapper_launcher [] {
    print "🧪 Testing sidebar layouts use the Yazi wrapper launcher..."

    try {
        let side_layout = (open --raw ~/.config/yazelix/configs/zellij/layouts/yzx_side.kdl)
        let no_side_layout = (open --raw ~/.config/yazelix/configs/zellij/layouts/yzx_no_side.kdl)
        let swap_fragment = (open --raw ~/.config/yazelix/configs/zellij/layouts/fragments/swap_sidebar_open.kdl)

        if ($side_layout | str contains "launch_sidebar_yazi.nu") and ($no_side_layout | str contains "launch_sidebar_yazi.nu") and ($swap_fragment | str contains "launch_sidebar_yazi.nu") {
            print "  ✅ Sidebar layouts launch Yazi through the restart-aware wrapper"
            true
        } else {
            print "  ❌ One or more sidebar layouts still launch Yazi directly"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_sidebar_wrapper_bootstraps_workspace_root [] {
    print "🧪 Testing sidebar Yazi wrapper bootstraps the tab workspace root..."

    try {
        let wrapper = (open --raw ~/.config/yazelix/configs/zellij/scripts/launch_sidebar_yazi.nu)

        if ($wrapper | str contains 'set_workspace_root') and ($wrapper | str contains 'bootstrap_workspace_root') {
            print "  ✅ Sidebar Yazi wrapper updates the tab workspace root before launch"
            true
        } else {
            print "  ❌ Sidebar Yazi wrapper is missing workspace-root bootstrap logic"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_layout_generator_discovers_custom_top_level_layouts [] {
    print "🧪 Testing layout generator discovers custom top-level layouts..."

    let tmpdir = (^mktemp -d /tmp/yazelix_layout_generator_XXXXXX | str trim)

    let result = (try {
        let source_dir = ($tmpdir | path join "source")
        let target_dir = ($tmpdir | path join "target")
        let fragments_dir = ($source_dir | path join "fragments")
        let repo_fragments_dir = ($env.HOME | path join ".config" "yazelix" "configs" "zellij" "layouts" "fragments")

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

        use ~/.config/yazelix/nushell/scripts/utils/layout_generator.nu *
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

def test_yzx_doctor_exists [] {
    print "🧪 Testing yzx doctor command exists..."

    try {
        # Just check that help mentions the doctor command
        let output = (yzx | str join "\n")

        if ($output | str contains "yzx doctor") {
            print "  ✅ yzx doctor command is documented in help"
            true
        } else {
            print "  ❌ yzx doctor command not found in help"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_doctor_reports_zellij_plugin_context [] {
    print "🧪 Testing yzx doctor reports Zellij plugin context..."

    try {
        let output = (^bash -lc $"($clean_zellij_env_prefix) nu -c 'use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx doctor --verbose'" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Zellij plugin health check skipped \(not inside Zellij\)") {
            print "  ✅ yzx doctor explains when Zellij-local plugin checks are skipped"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_launch_env_omits_default_helix_runtime [] {
    print "🧪 Testing launch env omits HELIX_RUNTIME by default..."

    try {
        let output = (^nu -c 'use ~/.config/yazelix/nushell/scripts/utils/launch_state.nu *; let cfg = { editor_command: "hx", helix_runtime_path: null, terminals: ["ghostty"], default_shell: "nu", debug_mode: false, enable_sidebar: true, ascii_art_mode: "static", terminal_config_mode: "yazelix" }; let env_map = (get_launch_env $cfg "/tmp/yazelix-profile"); print ($env_map | get -o HELIX_RUNTIME | default "")' | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "") {
            print "  ✅ HELIX_RUNTIME is omitted unless explicitly configured"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
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
        let output = (^nu -c 'use ~/.config/yazelix/nushell/scripts/utils/launch_state.nu *; let cfg = { editor_command: "hx", helix_runtime_path: "/tmp/custom-helix-runtime", terminals: ["ghostty"], default_shell: "nu", debug_mode: false, enable_sidebar: true, ascii_art_mode: "static", terminal_config_mode: "yazelix" }; let env_map = (get_launch_env $cfg "/tmp/yazelix-profile"); print ($env_map | get HELIX_RUNTIME)' | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "/tmp/custom-helix-runtime") {
            print "  ✅ Custom helix_runtime_path is still exported"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
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
        let output = (^nu -c 'use ~/.config/yazelix/nushell/scripts/utils/launch_state.nu *; let cfg = { editor_command: "hx", helix_runtime_path: null, terminals: ["ghostty"], default_shell: "fish", debug_mode: false, enable_sidebar: true, ascii_art_mode: "static", terminal_config_mode: "yazelix" }; let env_map = (get_launch_env $cfg "/tmp/yazelix-profile"); print ($env_map | get -o YAZELIX_DEFAULT_SHELL | default "")' | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "") {
            print "  ✅ YAZELIX_DEFAULT_SHELL is no longer part of the launch env"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
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
        let output = (^bash -lc 'tmpdir=$(mktemp -d); trap "rm -rf \"$tmpdir\"" EXIT; cat > "$tmpdir/yazelix.toml" <<'"'"'EOF'"'"'
[shell]
default_shell = "nu"
EOF
YAZELIX_CONFIG_OVERRIDE="$tmpdir/yazelix.toml" YAZELIX_DEFAULT_SHELL=fish nu ~/.config/yazelix/nushell/scripts/utils/zjstatus_widget.nu shell' | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "nu") {
            print "  ✅ Shell widget ignores stale YAZELIX_DEFAULT_SHELL env"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
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
        let output = (^bash -lc 'tmpdir=$(mktemp -d); trap "rm -rf \"$tmpdir\"" EXIT; cat > "$tmpdir/yazelix.toml" <<'"'"'EOF'"'"'
[editor]
command = "nvim --headless"
EOF
YAZELIX_CONFIG_OVERRIDE="$tmpdir/yazelix.toml" EDITOR=fish nu ~/.config/yazelix/nushell/scripts/utils/zjstatus_widget.nu editor' | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "nvim") {
            print "  ✅ Editor widget ignores stale EDITOR env"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_menu_exists [] {
    print "🧪 Testing yzx menu command exists..."

    try {
        let output = (yzx | str join "\n")

        if ($output | str contains "yzx menu") {
            print "  ✅ yzx menu command is documented in help"
            true
        } else {
            print "  ❌ yzx menu command not found in help"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_cwd_exists [] {
    print "🧪 Testing yzx cwd command exists..."

    try {
        let output = (yzx | str join "\n")

        if ($output | str contains "yzx cwd") {
            print "  ✅ yzx cwd command is documented in help"
            true
        } else {
            print "  ❌ yzx cwd command not found in help"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_cwd_requires_zellij [] {
    print "🧪 Testing yzx cwd outside Zellij..."

    try {
        let output = (^bash -lc $"($clean_zellij_env_prefix) nu -c 'use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx cwd .'" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 1) and ($stdout | str contains "only works inside Zellij") {
            print "  ✅ yzx cwd fails clearly outside Zellij"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_cwd_resolves_zoxide_query [] {
    print "🧪 Testing yzx cwd zoxide resolution..."

    try {
        let output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; resolve_yzx_cwd_target yazelix" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "/home/lucca/.config/yazelix") {
            print "  ✅ yzx cwd resolves zoxide queries before updating the tab directory"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_get_tab_name_uses_exact_directory [] {
    print "🧪 Testing tab naming uses the exact yzx cwd directory..."

    try {
        let output = (^nu -c "use ~/.config/yazelix/nushell/scripts/integrations/zellij.nu *; get_tab_name ~/.config/yazelix/nushell" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "nushell") {
            print "  ✅ yzx cwd tab naming matches the exact retargeted directory"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_sidebar_yazi_state_path_normalization [] {
    print "🧪 Testing sidebar Yazi state path normalization..."

    try {
        let output = (^nu -c "use ~/.config/yazelix/nushell/scripts/integrations/yazi.nu *; get_sidebar_yazi_state_path main 2" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str ends-with "main__terminal_2.txt") {
            print "  ✅ Sidebar Yazi state paths normalize pane ids consistently"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_resolve_reveal_target_path_from_relative_buffer [] {
    print "🧪 Testing reveal target resolution for relative buffer paths..."

    try {
        let output = (^bash -lc 'cd ~/.config/yazelix && nu -c "use ~/.config/yazelix/nushell/scripts/integrations/yazi.nu *; print (resolve_reveal_target_path README.md)"' | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "/home/lucca/.config/yazelix/README.md") {
            print "  ✅ Reveal target resolution expands relative buffer paths against the current cwd"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_reveal_in_yazi_fails_clearly_outside_zellij [] {
    print "🧪 Testing reveal in Yazi outside Zellij..."

    try {
        let output = (^bash -lc $"($clean_zellij_env_prefix) nu -c 'use ~/.config/yazelix/nushell/scripts/integrations/yazi.nu *; reveal_in_yazi ~/.config/yazelix/README.md'" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Reveal in Yazi only works inside a Yazelix/Zellij session.") and (not ($stdout | str contains "YAZI_ID")) {
            print "  ✅ Reveal in Yazi now fails clearly outside Zellij without relying on YAZI_ID"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
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
        let output = (^nu -c "use ~/.config/yazelix/nushell/scripts/setup/yazi_config_merger.nu *; let root = ($env.HOME | path join '.config' 'yazelix'); generate_merged_yazi_config $root --quiet; open --raw ($env.HOME | path join '.local' 'share' 'yazelix' 'configs' 'yazi' 'init.lua')" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains 'require("sidebar-state"):setup()') {
            print "  ✅ Generated Yazi init loads the sidebar-state core plugin"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_zellij_default_mode_is_enforced_in_merged_config [] {
    print "🧪 Testing merged Zellij config enforces default_mode..."

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
            ^nu -c 'use ~/.config/yazelix/nushell/scripts/setup/zellij_config_merger.nu *; let root = ($env.HOME | path join ".config" "yazelix"); generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore; open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "config.kdl")' | complete
        })
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains 'default_mode "locked"') {
            print "  ✅ Generated Zellij config enforces the configured default_mode"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_sidebar_yazi_sync_skips_outside_zellij [] {
    print "🧪 Testing sidebar Yazi sync skips outside Zellij..."

    try {
        let output = (^bash -lc $"($clean_zellij_env_prefix) nu -c 'use ~/.config/yazelix/nushell/scripts/integrations/yazi.nu *; sync_active_sidebar_yazi_to_directory . | to json -r'" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and (($stdout | str contains '"status":"skipped"') and ($stdout | str contains '"reason":"outside_zellij"')) {
            print "  ✅ Sidebar Yazi sync stays non-fatal outside Zellij"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_managed_editor_sync_skips_outside_zellij [] {
    print "🧪 Testing managed editor cwd sync skips outside Zellij..."

    try {
        let output = (^bash -lc $"($clean_zellij_env_prefix) nu -c 'use ~/.config/yazelix/nushell/scripts/integrations/yazi.nu *; sync_managed_editor_cwd . | to json -r'" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and (($stdout | str contains '"status":"skipped"') and ($stdout | str contains '"reason":"outside_zellij"')) {
            print "  ✅ Managed editor cwd sync stays non-fatal outside Zellij"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_sponsor_exists [] {
    print "🧪 Testing yzx sponsor command exists..."

    try {
        let output = (yzx | str join "\n")

        if ($output | str contains "yzx sponsor") {
            print "  ✅ yzx sponsor command is documented in help"
            true
        } else {
            print "  ❌ yzx sponsor command not found in help"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_config_open_print [] {
    print "🧪 Testing yzx config open --print..."

    try {
        let output = (yzx config open --print | into string | str trim)

        if ($output | str ends-with ".toml") and ($output | path exists) {
            print $"  ✅ Config path resolved: ($output)"
            true
        } else {
            print $"  ❌ Unexpected output: ($output)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_config_view [] {
    print "🧪 Testing yzx config..."

    try {
        let output = (
            ^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx config | columns | str join ','" | complete
        ).stdout | str trim

        if ($output | str contains "core") and ($output | str contains "terminal") and not ($output | str contains "packs") {
            print "  ✅ yzx config hides packs by default"
            true
        } else {
            print $"  ❌ Unexpected output: ($output)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_config_sections [] {
    print "🧪 Testing yzx config section views..."

    try {
        let hx_output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx config hx | columns | str join ','" | complete).stdout | str trim
        let yazi_output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx config yazi | columns | str join ','" | complete).stdout | str trim
        let zellij_output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx config zellij" | complete).stdout | str trim

        if ($hx_output | str contains "config_path") and ($yazi_output | str contains "manager") and ($zellij_output | str contains "default_layout") {
            print "  ✅ yzx config section commands return focused sections"
            true
        } else {
            print $"  ❌ Unexpected section output: hx=($hx_output) yazi=($yazi_output) zellij=($zellij_output)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def main [] {
    print "=== Testing yzx Commands ==="
    print ""

    let results = [
        (test_yzx_help),
        (test_yzx_status),
        (test_yzx_status_versions),
        (test_yzx_why),
        (test_yzx_status_verbose),
        (test_yzx_dev_exists),
        (test_dev_update_canary_set),
        (test_dev_update_defaults_to_verbose_mode),
        (test_dev_update_help_mentions_optional_input_name),
        (test_gemini_cli_is_reactivated),
        (test_runtime_pin_versions_use_repo_shell),
        (test_consume_bootstrap_sidebar_cwd),
        (test_restart_uses_home_for_future_tab_defaults),
        (test_sidebar_layout_uses_wrapper_launcher),
        (test_sidebar_wrapper_bootstraps_workspace_root),
        (test_layout_generator_discovers_custom_top_level_layouts),
        (test_yzx_doctor_exists),
        (test_yzx_doctor_reports_zellij_plugin_context),
        (test_launch_env_omits_default_helix_runtime),
        (test_launch_env_keeps_custom_helix_runtime_override),
        (test_launch_env_omits_yazelix_default_shell),
        (test_zjstatus_widget_reads_shell_from_config),
        (test_zjstatus_widget_reads_editor_from_config),
        (test_yzx_menu_exists),
        (test_yzx_cwd_exists),
        (test_yzx_cwd_requires_zellij),
        (test_yzx_cwd_resolves_zoxide_query),
        (test_get_tab_name_uses_exact_directory),
        (test_sidebar_yazi_state_path_normalization),
        (test_resolve_reveal_target_path_from_relative_buffer),
        (test_reveal_in_yazi_fails_clearly_outside_zellij),
        (test_sidebar_state_plugin_generated),
        (test_zellij_default_mode_is_enforced_in_merged_config),
        (test_sidebar_yazi_sync_skips_outside_zellij),
        (test_managed_editor_sync_skips_outside_zellij),
        (test_yzx_sponsor_exists),
        (test_yzx_config_view),
        (test_yzx_config_sections),
        (test_yzx_config_open_print)
    ]

    let passed = ($results | where $it == true | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ All yzx command tests passed \(($passed)/($total)\)"
    } else {
        print $"❌ Some tests failed \(($passed)/($total)\)"
        error make { msg: "yzx command tests failed" }
    }
}
