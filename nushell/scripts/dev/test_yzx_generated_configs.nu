#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md
# Defends: docs/specs/runtime_dependency_preflight_contract.md
# Defends: docs/specs/terminal_launch_contract.md

use ./yzx_test_helpers.nu [get_repo_root repo_path resolve_test_yzx_core_bin setup_managed_config_fixture]
use ./config_normalize_test_helpers.nu [load_normalized_active_config]
use ./materialization_dev_helpers.nu [generate_merged_yazi_config generate_merged_zellij_config regenerate_runtime_configs]
use ../utils/yzx_core_bridge.nu [record_materialized_state_via_yzx_core]
use ../core/launch_yazelix.nu [generate_all_terminal_configs]
use ../utils/yzx_core_bridge.nu [build_default_yzx_core_error_surface resolve_yzx_core_helper_path run_yzx_core_request_json_command]
use ../utils/safe_remove.nu remove_path_within_root
use ../utils/terminal_launcher.nu [build_launch_command resolve_terminal_config]

def check_schema_rejects_removed_enum_value [case: record] {
    let tmpdir = (^mktemp -d $"/tmp/($case.label)_XXXXXX" | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        $case.config | save --force --raw $config_path

        let findings = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            use ../utils/config_schema.nu [validate_enum_values]
            validate_enum_values (open $config_path)
        })
        let matching_findings = ($findings | where path == $case.path)

        if (
            (($matching_findings | length) == 1)
            and (($matching_findings | get 0.kind) == "invalid_enum")
            and ((($matching_findings | get 0.message) | str contains $case.expected_value))
        ) {
            print $"  ✅ ($case.name)"
            true
        } else {
            print $"  ❌ ($case.name): findings=($matching_findings | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ ($case.name): ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def setup_home_manager_symlinked_main_config_fixture [label: string] {
    let repo_root = (get_repo_root)
    let tmpdir = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let fake_home = ($tmpdir | path join "home")
    let config_dir = ($fake_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let hm_store_dir = ($tmpdir | path join "hm-store")
    let symlinked_main = ($user_config_dir | path join "yazelix.toml")
    let state_path = ($fake_home | path join ".local" "share" "yazelix" "state" "rebuild_hash")
    let store_main = ($hm_store_dir | path join "yazelix.toml")

    mkdir $fake_home
    mkdir ($fake_home | path join ".config")
    mkdir $config_dir
    mkdir $user_config_dir
    mkdir $hm_store_dir

    cp (repo_path "yazelix_default.toml") $store_main
    ^ln -s $store_main $symlinked_main

    {
        repo_root: $repo_root
        tmpdir: $tmpdir
        fake_home: $fake_home
        config_dir: $config_dir
        user_config_dir: $user_config_dir
        symlinked_main: $symlinked_main
        state_path: $state_path
    }
}

def setup_fake_packaged_runtime_fixture [label: string] {
    let repo_root = (get_repo_root)
    let tmpdir = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let runtime_root = ($tmpdir | path join "runtime")
    let metadata_dir = ($runtime_root | path join "config_metadata")
    let libexec_dir = ($runtime_root | path join "libexec")

    mkdir $metadata_dir
    mkdir $libexec_dir
    cp (repo_path "yazelix_default.toml") ($runtime_root | path join "yazelix_default.toml")
    cp (repo_path ".taplo.toml") ($runtime_root | path join ".taplo.toml")
    cp (repo_path "config_metadata" "main_config_contract.toml") ($metadata_dir | path join "main_config_contract.toml")

    {
        repo_root: $repo_root
        tmpdir: $tmpdir
        runtime_root: $runtime_root
        helper_path: ($libexec_dir | path join "yzx_core")
        args_log: ($tmpdir | path join "yzx_core_args.log")
    }
}

def setup_fake_source_checkout_runtime_fixture [label: string] {
    let runtime = (setup_fake_packaged_runtime_fixture $label)
    mkdir ($runtime.runtime_root | path join "rust_core")
    "[workspace]\n" | save --force --raw ($runtime.runtime_root | path join "rust_core" "Cargo.toml")
    $runtime
}

def install_fake_yzx_core_helper [runtime_fixture: record, helper_script: string] {
    $helper_script | save --force --raw $runtime_fixture.helper_path
    ^chmod +x $runtime_fixture.helper_path
    $runtime_fixture
}

def install_fake_source_checkout_yzx_core_candidate [runtime_fixture: record, profile: string, helper_script: string] {
    let helper_path = ($runtime_fixture.runtime_root | path join "rust_core" "target" $profile "yzx_core")
    mkdir ($helper_path | path dirname)
    $helper_script | save --force --raw $helper_path
    ^chmod +x $helper_path
    $helper_path
}

# Defends: generated terminal configs do not silently take over user overrides or create backup churn in Yazelix-owned generated paths.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_generate_all_terminal_configs_keeps_terminal_overrides_opt_in [] {
    print "🧪 Testing bundled terminal config generation keeps user terminal overrides opt-in..."

    let tmpdir = (^mktemp -d /tmp/yazelix_terminal_override_scaffold_XXXXXX | str trim)
    let fake_home = ($tmpdir | path join "home")
    let config_path = ($tmpdir | path join "yazelix.toml")
    let runtime_root = (pwd)
    mkdir $fake_home

    let result = (try {
        '[terminal]
terminals = ["ghostty", "kitty", "alacritty", "wezterm", "foot"]
ghostty_trail_color = "reef"
ghostty_trail_effect = "tail"
ghostty_mode_effect = "ripple_rectangle"
' | save --force --raw $config_path

        with-env {
            HOME: $fake_home
            XDG_CONFIG_HOME: ($fake_home | path join ".config")
            XDG_DATA_HOME: ($fake_home | path join ".local" "share")
            YAZELIX_CONFIG_DIR: ($fake_home | path join ".config" "yazelix")
            YAZELIX_STATE_DIR: ($fake_home | path join ".local" "share" "yazelix")
            YAZELIX_CONFIG_OVERRIDE: $config_path
        } {
            generate_all_terminal_configs $runtime_root
            generate_all_terminal_configs $runtime_root
        }

        let override_root = ($fake_home | path join ".config" "yazelix" "user_configs" "terminal")
        let generated_root = ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators")
        let ghostty_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "ghostty" "config"))
        let kitty_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "kitty" "kitty.conf"))
        let alacritty_entrypoint = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "alacritty" "alacritty.toml"))
        let wezterm_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "wezterm" ".wezterm.lua"))
        let foot_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "foot" "foot.ini"))
        let ghostty_root = ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "ghostty")
        let tail_shader = ($ghostty_root | path join "shaders" "generated_effects" "tail.glsl")
        let ripple_shader = ($ghostty_root | path join "shaders" "generated_effects" "ripple_rectangle.glsl")
        let tail_shader_text = (open --raw $tail_shader)
        let ripple_shader_text = (open --raw $ripple_shader)
        let backup_find = (^find $generated_root -name '*.yazelix-backup' | complete)
        let temp_find = (^find $generated_root | complete)
        let backup_noise = if $backup_find.exit_code == 0 {
            $backup_find.stdout | lines | where {|path| $path | str trim | is-not-empty}
        } else {
            []
        }
        let temp_noise = if $temp_find.exit_code == 0 {
            $temp_find.stdout | lines | where {|path| ($path | str trim | is-not-empty) and ($path | str contains ".yazelix-tmp-")}
        } else {
            []
        }

        if (
            not ($override_root | path exists)
            and ($ghostty_config | str contains $"config-file = ?\"($override_root | path join "ghostty")\"")
            and ($ghostty_config | str contains "custom-shader = ./shaders/generated_effects/tail.glsl")
            and ($ghostty_config | str contains "custom-shader = ./shaders/generated_effects/ripple_rectangle.glsl")
            and not ($kitty_config | str contains "include ~/.config/yazelix")
            and ($kitty_config | str contains $"Create ($override_root | path join "kitty.conf") if you want terminal-native Kitty tweaks.")
            and ($alacritty_entrypoint | str contains $"\"($fake_home)/.local/share/yazelix/configs/terminal_emulators/alacritty/alacritty_base.toml\"")
            and not ($alacritty_entrypoint | str contains $"\"($override_root | path join "alacritty.toml")\"")
            and ($alacritty_entrypoint | str contains $"Create ($override_root | path join "alacritty.toml") if you want terminal-native Alacritty tweaks.")
            and not ($ghostty_config | str contains "start_yazelix.sh")
            and not ($kitty_config | str contains "start_yazelix.sh")
            and not ($alacritty_entrypoint | str contains "start_yazelix.sh")
            and not ($wezterm_config | str contains "start_yazelix.sh")
            and not ($foot_config | str contains "start_yazelix.sh")
            and ($foot_config | str contains "[colors-dark]")
            and not ($foot_config | str contains "[colors]\n")
            and ($tail_shader | path exists)
            and ($ripple_shader | path exists)
            and ($tail_shader_text | str contains "vec4 TRAIL_COLOR = vec4(0.0, 0.902, 1.0, 1.0);")
            and ($ripple_shader_text | str contains "vec4 COLOR = vec4(0.0, 0.902, 1.0, 1.0);")
            and ($backup_noise | is-empty)
            and ($temp_noise | is-empty)
        ) {
            print "  ✅ Terminal config generation keeps user terminal overrides opt-in, rewrites generated files in place, keeps startup out of generated terminal configs, and points Ghostty at real generated shaders"
            true
        } else {
            print "  ❌ Terminal config generation still scaffolded user overrides or left backup churn in generated paths"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: terminal override imports must ignore Yazelix runtime roots.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
def test_terminal_override_imports_ignore_yazelix_dir_runtime_root [] {
    print "🧪 Testing terminal override imports ignore YAZELIX_DIR runtime roots..."

    let tmpdir = (^mktemp -d /tmp/yazelix_terminal_override_path_boundary_XXXXXX | str trim)
    let fake_home = ($tmpdir | path join "home")
    let fake_runtime_root = ($tmpdir | path join "runtime_root")
    let fake_config_dir = ($fake_home | path join ".config" "yazelix")
    let config_path = ($tmpdir | path join "yazelix.toml")
    let runtime_root = (pwd)
    let terminal_materialization_script = ($runtime_root | path join "nushell" "scripts" "core" "launch_yazelix.nu")
    mkdir $fake_home
    mkdir $fake_runtime_root
    mkdir ($fake_config_dir | path join "user_configs" "terminal")

    let result = (try {
        '[terminal]
terminals = ["ghostty", "kitty", "alacritty"]
' | save --force --raw $config_path

        '# user-owned ghostty override
' | save --force --raw ($fake_home | path join ".config" "yazelix" "user_configs" "terminal" "ghostty")
        '# user-owned kitty override
' | save --force --raw ($fake_home | path join ".config" "yazelix" "user_configs" "terminal" "kitty.conf")
        '# user-owned alacritty override
' | save --force --raw ($fake_home | path join ".config" "yazelix" "user_configs" "terminal" "alacritty.toml")

        let command_output = (with-env {
            HOME: $fake_home
            XDG_CONFIG_HOME: ($fake_home | path join ".config")
            XDG_DATA_HOME: ($fake_home | path join ".local" "share")
            YAZELIX_CONFIG_DIR: $fake_config_dir
            YAZELIX_STATE_DIR: ($fake_home | path join ".local" "share" "yazelix")
            YAZELIX_DIR: $fake_runtime_root
            YAZELIX_RUNTIME_DIR: $runtime_root
            YAZELIX_CONFIG_OVERRIDE: $config_path
        } {
            ^nu -c $"use \"($terminal_materialization_script)\" [generate_all_terminal_configs]; generate_all_terminal_configs \"($runtime_root)\"" | complete
        })

        let expected_override_root = ($fake_home | path join ".config" "yazelix" "user_configs" "terminal")
        let misplaced_override_root = ($fake_runtime_root | path join "user_configs" "terminal")
        let ghostty_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "ghostty" "config"))
        let kitty_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "kitty" "kitty.conf"))
        let alacritty_entrypoint = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "alacritty" "alacritty.toml"))

        if (
            ($command_output.exit_code == 0)
            and ($expected_override_root | path exists)
            and (($expected_override_root | path join "ghostty") | path exists)
            and (($expected_override_root | path join "kitty.conf") | path exists)
            and (($expected_override_root | path join "alacritty.toml") | path exists)
            and ($ghostty_config | str contains $"config-file = ?\"($expected_override_root | path join "ghostty")\"")
            and ($kitty_config | str contains $"include ($expected_override_root | path join "kitty.conf")")
            and ($alacritty_entrypoint | str contains $"\"($expected_override_root | path join "alacritty.toml")\"")
            and not ($misplaced_override_root | path exists)
        ) {
            print "  ✅ Terminal override imports stay under HOME/.config/yazelix/user_configs even when YAZELIX_DIR points elsewhere"
            true
        } else {
            print $"  ❌ Unexpected override destinations: exit=($command_output.exit_code) expected_root_exists=(($expected_override_root | path exists)) misplaced_root_exists=(($misplaced_override_root | path exists)) expected_root=($expected_override_root) misplaced_root=($misplaced_override_root) stderr=(($command_output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: direct terminal launch commands must keep Yazelix-only config-mode details internal.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_managed_wrapper_launch_command_does_not_forward_config_mode_flag [] {
    print "🧪 Testing direct terminal launch command keeps config-mode internal..."

    try {
        let launch_cmd = (build_launch_command {
            terminal: "ghostty"
            name: "Ghostty"
            command: "ghostty"
        } "/tmp/ghostty-config" "/tmp" false)

        if (
            ($launch_cmd | str contains 'ghostty')
            and not ($launch_cmd | str contains '--config-mode')
            and ($launch_cmd | str contains '--working-directory=/tmp')
            and not ($launch_cmd | str contains 'yazelix-ghostty')
        ) {
            print "  ✅ Direct terminal launch command now keeps config-mode internal to Yazelix"
            true
        } else {
            print $"  ❌ Unexpected managed wrapper launch command: ($launch_cmd)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Regression: Linux Ghostty launch keeps the GTK/X11 flags Yazelix relies on there.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_ghostty_linux_launch_command_keeps_linux_specific_flags [] {
    print "🧪 Testing Ghostty launch keeps Linux-specific GTK/X11 flags on Linux..."

    try {
        let launch_cmd = (with-env {YAZELIX_TEST_OS: "linux"} {
            build_launch_command {
                terminal: "ghostty"
                name: "Ghostty"
                command: "ghostty"
            } "/tmp/ghostty-config" "/tmp" false
        })

        if (
            ($launch_cmd | str contains 'shells/posix/yazelix_ghostty.sh')
            and
            ($launch_cmd | str contains '--gtk-single-instance=false')
            and ($launch_cmd | str contains '--class=com.yazelix.Yazelix')
            and ($launch_cmd | str contains '--x11-instance-name=yazelix')
        ) {
            print "  ✅ Linux Ghostty launch now routes through the runtime Ghostty env wrapper and keeps the GTK/X11 flags Yazelix expects there"
            true
        } else {
            print $"  ❌ Unexpected Linux Ghostty launch command: ($launch_cmd)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Regression: Linux Ghostty launch must use a runtime-owned nixGL wrapper when one is shipped.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_ghostty_linux_launch_command_prefers_runtime_owned_nixgl_wrapper [] {
    print "🧪 Testing Ghostty launch prefers the runtime-owned nixGL wrapper on Linux..."

    let tmpdir = (^mktemp -d /tmp/yazelix_linux_nixgl_launch_XXXXXX | str trim)
    let fake_runtime = ($tmpdir | path join "runtime")
    let fake_wrapper = ($fake_runtime | path join "libexec" "nixGLMesa")
    mkdir ($fake_runtime | path join "libexec")
    '{}' | save --force --raw ($fake_runtime | path join "yazelix_default.toml")

    let result = (try {
        '#!/bin/sh
exit 0
' | save --force --raw $fake_wrapper
        ^chmod +x $fake_wrapper

        let launch_cmd = (with-env {
            YAZELIX_TEST_OS: "linux"
            YAZELIX_RUNTIME_DIR: $fake_runtime
        } {
            build_launch_command {
                terminal: "ghostty"
                name: "Ghostty"
                command: "ghostty"
            } "/tmp/ghostty-config" "/tmp" false
        })

        if (
            ($launch_cmd | str contains ($fake_runtime | path join "shells" "posix" "yazelix_ghostty.sh"))
            and ($launch_cmd | str contains $fake_wrapper)
            and ($launch_cmd | str contains ' ghostty --config-default-files=false')
            and ($launch_cmd | str contains '--gtk-single-instance=false')
            and ($launch_cmd | str contains '--x11-instance-name=yazelix')
        ) {
            print "  ✅ Linux Ghostty launch now prefers the runtime-owned nixGL wrapper when Yazelix ships one"
            true
        } else {
            print $"  ❌ Unexpected Linux Ghostty nixGL launch command: ($launch_cmd)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: the Ghostty env wrapper must fall back to GTK_IM_MODULE=simple when Wayland IM state is stale.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_ghostty_wayland_wrapper_falls_back_to_simple_im_without_active_daemon [] {
    print "🧪 Testing Ghostty Wayland wrapper falls back to GTK_IM_MODULE=simple when no IM daemon is active..."

    let tmpdir = (^mktemp -d /tmp/yazelix_ghostty_im_fallback_XXXXXX | str trim)
    let result = (try {
        let fake_bin = ($tmpdir | path join "bin")
        let fake_pgrep = ($fake_bin | path join "pgrep")
        let probe = ($tmpdir | path join "probe-env.sh")
        let wrapper = ($env.PWD | path join "shells" "posix" "yazelix_ghostty.sh")

        mkdir $fake_bin
        '#!/bin/sh
exit 1
' | save --force --raw $fake_pgrep
        ^chmod +x $fake_pgrep

        '#!/bin/sh
printf "GTK_IM_MODULE=%s\n" "${GTK_IM_MODULE-unset}"
printf "QT_IM_MODULE=%s\n" "${QT_IM_MODULE-unset}"
printf "XMODIFIERS=%s\n" "${XMODIFIERS-unset}"
' | save --force --raw $probe
        ^chmod +x $probe

        let output = (with-env {
            PATH: ([$fake_bin] | append $env.PATH)
            WAYLAND_DISPLAY: "wayland-0"
            GTK_IM_MODULE: "ibus"
            QT_IM_MODULE: "ibus"
            XMODIFIERS: "@im=ibus"
        } {
            ^sh $wrapper $probe | complete
        })
        let env_lines = ($output.stdout | lines)

        if (
            ($output.exit_code == 0)
            and ($env_lines == [
                "GTK_IM_MODULE=simple"
                "QT_IM_MODULE=unset"
                "XMODIFIERS=unset"
            ])
        ) {
            print "  ✅ Ghostty Wayland fallback now restores dead keys by switching stale IM state to GTK_IM_MODULE=simple"
            true
        } else {
            print $"  ❌ Unexpected Ghostty Wayland IM fallback behavior: exit=($output.exit_code) env=($env_lines | to json -r) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: the Ghostty env wrapper must not clobber a valid running Wayland IM daemon setup.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_ghostty_wayland_wrapper_preserves_active_ibus_env [] {
    print "🧪 Testing Ghostty Wayland wrapper preserves a live ibus input-method setup..."

    let tmpdir = (^mktemp -d /tmp/yazelix_ghostty_im_preserve_XXXXXX | str trim)
    let result = (try {
        let fake_bin = ($tmpdir | path join "bin")
        let fake_pgrep = ($fake_bin | path join "pgrep")
        let probe = ($tmpdir | path join "probe-env.sh")
        let wrapper = ($env.PWD | path join "shells" "posix" "yazelix_ghostty.sh")

        mkdir $fake_bin
        '#!/bin/sh
if [ "$1" = "-x" ] && [ "$2" = "ibus-daemon" ]; then
  exit 0
fi
exit 1
' | save --force --raw $fake_pgrep
        ^chmod +x $fake_pgrep

        '#!/bin/sh
printf "GTK_IM_MODULE=%s\n" "${GTK_IM_MODULE-unset}"
printf "QT_IM_MODULE=%s\n" "${QT_IM_MODULE-unset}"
printf "XMODIFIERS=%s\n" "${XMODIFIERS-unset}"
' | save --force --raw $probe
        ^chmod +x $probe

        let output = (with-env {
            PATH: ([$fake_bin] | append $env.PATH)
            WAYLAND_DISPLAY: "wayland-0"
            GTK_IM_MODULE: "ibus"
            QT_IM_MODULE: "ibus"
            XMODIFIERS: "@im=ibus"
        } {
            ^sh $wrapper $probe | complete
        })
        let env_lines = ($output.stdout | lines)

        if (
            ($output.exit_code == 0)
            and ($env_lines == [
                "GTK_IM_MODULE=ibus"
                "QT_IM_MODULE=ibus"
                "XMODIFIERS=@im=ibus"
            ])
        ) {
            print "  ✅ Ghostty Wayland wrapper preserves a valid live ibus setup instead of clobbering it"
            true
        } else {
            print $"  ❌ Unexpected Ghostty Wayland IM preservation behavior: exit=($output.exit_code) env=($env_lines | to json -r) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: macOS Ghostty launch must not inherit Linux GTK/X11 flags.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_ghostty_macos_launch_command_omits_linux_specific_flags [] {
    print "🧪 Testing Ghostty launch drops Linux-specific GTK/X11 flags on macOS..."

    try {
        let launch_cmd = (with-env {YAZELIX_TEST_OS: "macos"} {
            build_launch_command {
                terminal: "ghostty"
                name: "Ghostty"
                command: "ghostty"
            } "/tmp/ghostty-config" "/tmp" false
        })

        if (
            ($launch_cmd | str contains '--config-default-files=false')
            and ($launch_cmd | str contains '--config-file=/tmp/ghostty-config')
            and ($launch_cmd | str contains '--title=Yazelix - Ghostty')
            and ($launch_cmd | str contains '--working-directory=/tmp')
            and not ($launch_cmd | str contains '--gtk-single-instance=false')
            and not ($launch_cmd | str contains '--class=com.yazelix.Yazelix')
            and not ($launch_cmd | str contains '--x11-instance-name=yazelix')
        ) {
            print "  ✅ macOS Ghostty launch now avoids the Linux-only GTK/X11 flags"
            true
        } else {
            print $"  ❌ Unexpected macOS Ghostty launch command: ($launch_cmd)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Defends: installed runtimes use the packaged Rust config helper instead of a deleted product-side parser shim.
# Contract: CRCP-001, CRCP-003
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_active_config_normalize_uses_runtime_yzx_core_helper_when_present [] {
    print "🧪 Testing the active-config normalize probe uses runtime-local yzx_core when present..."

    let fixture = (setup_managed_config_fixture "yazelix_rust_config_helper_success" "[shell]\ndefault_shell = \"fish\"\n")
    let runtime_base = (setup_fake_packaged_runtime_fixture "yazelix_rust_config_helper_runtime")
    let runtime = (install_fake_yzx_core_helper $runtime_base $'#!/bin/sh
printf "%s\n" "$@" > "$YAZELIX_TEST_YZX_CORE_ARGS_LOG"
case "$1" in
  config-surface.resolve)
    cat <<JSON
{"schema_version":1,"command":"config-surface.resolve","status":"ok","data":{"config_file":"($fixture.config_path)","default_config_path":"(($runtime_base.runtime_root | path join "yazelix_default.toml"))","contract_path":"(($runtime_base.runtime_root | path join "config_metadata" "main_config_contract.toml"))"},"warnings":[]}
JSON
    ;;
  config.normalize)
    cat <<JSON
{"schema_version":1,"command":"config.normalize","status":"ok","data":{"normalized_config":{"default_shell":"from_rust_helper","config_file":"from-helper"},"config_file":"from-helper","diagnostic_report":{}},"warnings":[]}
JSON
    ;;
  *)
    echo "unexpected command: $1" >&2
    exit 64
    ;;
esac
')

    let result = (try {
        let parsed = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $runtime.runtime_root
            YAZELIX_TEST_YZX_CORE_ARGS_LOG: $runtime.args_log
            YAZELIX_YZX_CORE_BIN: null
        } {
            load_normalized_active_config
        })
        let helper_args = (open --raw $runtime.args_log)

        if (
            (($parsed.default_shell? | default "") == "from_rust_helper")
            and (($parsed.config_file? | default "") == "from-helper")
            and ($helper_args | str contains "config.normalize")
            and ($helper_args | str contains "--config")
            and ($helper_args | str contains $fixture.config_path)
            and ($helper_args | str contains "--default-config")
            and ($helper_args | str contains ($runtime.runtime_root | path join "yazelix_default.toml"))
            and ($helper_args | str contains "--contract")
            and ($helper_args | str contains ($runtime.runtime_root | path join "config_metadata" "main_config_contract.toml"))
        ) {
            print "  ✅ Installed runtime config normalization routes through the runtime-local Rust helper"
            true
        } else {
            print $"  ❌ Unexpected helper routing result: parsed=($parsed | to json -r) args=($helper_args)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    rm -rf $runtime.tmpdir
    $result
}

# Regression: packaged helper failures must be visible and must not silently fall back through a deleted Nu parser seam.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_active_config_normalize_surfaces_yzx_core_config_errors_without_fallback [] {
    print "🧪 Testing yzx_core config errors stay visible without fallback..."

    let fixture = (setup_managed_config_fixture "yazelix_rust_config_helper_error" "[shell]\ndefault_shell = \"fish\"\n")
    let runtime_base = (setup_fake_packaged_runtime_fixture "yazelix_rust_config_helper_error_runtime")
    let runtime = (install_fake_yzx_core_helper $runtime_base $'#!/bin/sh
case "$1" in
  config-surface.resolve)
    cat <<JSON
{"schema_version":1,"command":"config-surface.resolve","status":"ok","data":{"config_file":"($fixture.config_path)","default_config_path":"(($runtime_base.runtime_root | path join "yazelix_default.toml"))","contract_path":"(($runtime_base.runtime_root | path join "config_metadata" "main_config_contract.toml"))"},"warnings":[]}
JSON
    ;;
  config.normalize)
    cat >&2 <<JSON
{"schema_version":1,"command":"config.normalize","status":"error","error":{"class":"config","code":"invalid_config_value","message":"helper rejected the config","remediation":"fix the config value","details":{"field":"shell.default_shell"}}}
JSON
    exit 65
    ;;
  *)
    echo "unexpected command: $1" >&2
    exit 64
    ;;
esac
')

    let result = (try {
        let message = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $runtime.runtime_root
            YAZELIX_YZX_CORE_BIN: null
        } {
            try {
                load_normalized_active_config | ignore
                ""
            } catch {|err|
                $err.msg
            }
        })

        if (
            ($message | str contains "helper rejected the config")
            and ($message | str contains "Helper code: invalid_config_value")
            and ($message | str contains "Failure class: config problem.")
            and ($message | str contains "Recovery: fix the config value")
        ) {
            print "  ✅ Rust helper config failures are surfaced as config-class errors"
            true
        } else {
            print $"  ❌ Unexpected helper error message: ($message)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    rm -rf $runtime.tmpdir
    $result
}

# Defends: packaged runtimes must include yzx_core; missing helper is not masked by a deleted Nu parser seam.
# Contract: CRCP-003
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_active_config_normalize_rejects_packaged_runtime_missing_yzx_core [] {
    print "🧪 Testing packaged runtimes missing yzx_core fail clearly..."

    let fixture = (setup_managed_config_fixture "yazelix_rust_config_helper_missing" "[shell]\ndefault_shell = \"fish\"\n")
    let runtime = (setup_fake_packaged_runtime_fixture "yazelix_rust_config_helper_missing_runtime")

    let result = (try {
        let message = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $runtime.runtime_root
            YAZELIX_YZX_CORE_BIN: null
        } {
            try {
                load_normalized_active_config | ignore
                ""
            } catch {|err|
                $err.msg
            }
        })

        if (
            ($message | str contains "runtime is missing the Rust config helper")
            and ($message | str contains ($runtime.runtime_root | path join "libexec" "yzx_core"))
            and ($message | str contains "Failure class: host-dependency problem.")
        ) {
            print "  ✅ Packaged runtime helper absence is explicit and does not fall back"
            true
        } else {
            print $"  ❌ Unexpected missing-helper message: ($message)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    rm -rf $runtime.tmpdir
    $result
}

# Defends: source checkouts can use an explicit yzx_core helper without reviving a deleted Nu parser owner.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_active_config_normalize_source_checkout_uses_explicit_yzx_core_helper [] {
    print "🧪 Testing source checkout config normalization uses an explicit yzx_core helper..."

    let fixture = (setup_managed_config_fixture "yazelix_source_config_explicit_helper" "[shell]\ndefault_shell = \"fish\"\n")
    let helper_path = (resolve_test_yzx_core_bin)

    let result = (try {
        let parsed = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.repo_root
            YAZELIX_YZX_CORE_BIN: $helper_path
        } {
            load_normalized_active_config
        })

        if (
            (($parsed.default_shell? | default "") == "fish")
            and (($parsed.config_file? | default "") == $fixture.config_path)
        ) {
            print "  ✅ Source checkout parsing routes through the explicit Rust helper"
            true
        } else {
            print $"  ❌ Unexpected source helper parse result: ($parsed | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: source checkouts without yzx_core must fail clearly instead of silently reviving the deleted Nu parser seam.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_active_config_normalize_source_checkout_missing_helper_does_not_fallback [] {
    print "🧪 Testing source checkout config normalization fails clearly without yzx_core..."

    let fixture = (setup_managed_config_fixture "yazelix_source_config_no_helper" "[zellij]\ncustom_text = \"  [hello]  world demo  \"\n")
    let runtime = (setup_fake_source_checkout_runtime_fixture "yazelix_source_config_no_helper_runtime")

    let result = (try {
        let message = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $runtime.runtime_root
            YAZELIX_YZX_CORE_BIN: null
        } {
            try {
                load_normalized_active_config | ignore
                ""
            } catch {|err|
                $err.msg
            }
        })

        if (
            ($message | str contains "runtime is missing the Rust config helper")
            and ($message | str contains "YAZELIX_YZX_CORE_BIN")
            and ($message | str contains "Failure class: host-dependency problem.")
            and not ($message | str contains "hello wo")
        ) {
            print "  ✅ Source checkouts without yzx_core now fail explicitly instead of using the deleted Nushell parser"
            true
        } else {
            print $"  ❌ Unexpected source missing-helper message: ($message)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    rm -rf $runtime.tmpdir
    $result
}

# Regression: source-checkout helper fallback must prefer the freshest local yzx_core build instead of blindly taking target/release.
# Contract: CRCP-003
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_run_yzx_core_request_prefers_newer_source_checkout_helper_over_stale_release [] {
    print "🧪 Testing source checkout helper selection prefers a newer debug yzx_core over a stale release build..."

    let runtime = (setup_fake_source_checkout_runtime_fixture "yazelix_source_helper_prefers_newer_debug")
    let call_log = ($runtime.tmpdir | path join "helper_calls.log")
    let release_helper = (install_fake_source_checkout_yzx_core_candidate $runtime "release" $'#!/bin/sh
printf "%s\n" release >> "($call_log)"
cat >&2 <<JSON
{"schema_version":1,"command":"runtime-materialization.materialize","status":"error","error":{"class":"usage","code":"invalid_arguments","message":"Unsupported helper command: runtime-materialization.materialize","remediation":"Run the helper with a supported command and required flags.","details":{}}}
JSON
exit 64
')
    let debug_helper = (install_fake_source_checkout_yzx_core_candidate $runtime "debug" $'#!/bin/sh
printf "%s\n" debug >> "($call_log)"
cat <<JSON
{"schema_version":1,"command":"runtime-materialization.materialize","status":"ok","data":{"plan":{"status":"noop","selected_helper":"debug"}}}
JSON
')

    ^touch -t 202604210101 $release_helper
    ^touch -t 202604210102 $debug_helper

    let result = (try {
        let outcome = (with-env {
            YAZELIX_YZX_CORE_BIN: null
        } {
            {
                helper_path: (resolve_yzx_core_helper_path $runtime.runtime_root)
                data: (run_yzx_core_request_json_command
                    $runtime.runtime_root
                    (build_default_yzx_core_error_surface)
                    "runtime-materialization.materialize"
                    {}
                    "Yazelix Rust runtime-materialization helper returned invalid JSON.")
            }
        })
        let calls = if ($call_log | path exists) {
            open --raw $call_log | lines | where {|line| ($line | str trim | is-not-empty)}
        } else {
            []
        }

        if (
            ($outcome.helper_path == $debug_helper)
            and (($outcome.data.plan.status? | default "") == "noop")
            and (($outcome.data.plan.selected_helper? | default "") == "debug")
            and ($calls == ["debug"])
        ) {
            print "  ✅ Source checkout helper fallback now picks the freshest compatible local yzx_core build instead of a stale release artifact"
            true
        } else {
            print $"  ❌ Unexpected helper selection result: helper=($outcome.helper_path) data=(($outcome.data | to json -r)) calls=(($calls | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $runtime.tmpdir
    $result
}

# Regression: Home Manager symlinked managed configs must still record canonical rebuild state.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_record_materialized_state_accepts_symlinked_managed_main_config [] {
    print "🧪 Testing record_materialized_state treats a symlinked Home Manager main config as the canonical managed surface..."

    let fixture = (setup_home_manager_symlinked_main_config_fixture "yazelix_hm_symlink_state_recording")

    let result = (try {
        with-env {
            HOME: $fixture.fake_home
            XDG_CONFIG_HOME: ($fixture.fake_home | path join ".config")
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.repo_root
            YAZELIX_STATE_DIR: ($fixture.fake_home | path join ".local" "share" "yazelix")
        } {
            record_materialized_state_via_yzx_core {
                config_file: $fixture.symlinked_main
                config_hash: "cfg"
                runtime_hash: "runtime"
            }
        }

        let recorded = if ($fixture.state_path | path exists) {
            open --raw $fixture.state_path | from json
        } else {
            null
        }
        let recorded_config_hash = if $recorded == null { "" } else { $recorded | get -o config_hash | default "" }
        let recorded_runtime_hash = if $recorded == null { "" } else { $recorded | get -o runtime_hash | default "" }

        if (
            ($recorded != null)
            and ($recorded_config_hash == "cfg")
            and ($recorded_runtime_hash == "runtime")
        ) {
            print "  ✅ Symlinked Home Manager managed configs still record canonical rebuild state"
            true
        } else {
            print $"  ❌ Unexpected result: state_exists=(($fixture.state_path | path exists)) recorded=($recorded)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

# Defends: user terminal mode requires a real terminal config path.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_user_mode_requires_real_terminal_config [] {
    print "🧪 Testing terminal.config_mode = user fails fast when the user terminal config is missing..."

    let fake_home = (^mktemp -d /tmp/yazelix_user_mode_missing_XXXXXX | str trim)

    let result = (try {
        let message = (with-env { HOME: $fake_home } {
            try {
                resolve_terminal_config "ghostty" "user"
                "unexpected-success"
            } catch {|err|
                $err.msg
            }
        })

        if ($message | str contains "terminal.config_mode = user requires a real ghostty user config") {
            print "  ✅ user mode fails clearly instead of silently falling back to Yazelix-managed config"
            true
        } else {
            print $"  ❌ Unexpected message: ($message)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fake_home
    $result
}

# Defends: removed enum values are rejected by schema validation.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_config_schema_rejects_removed_enum_values [] {
    print "🧪 Testing config schema rejects removed enum values..."

    let cases = [
        {
            name: "terminal.config_mode = auto"
            label: "yazelix_terminal_mode_schema"
            config: "[terminal]\nconfig_mode = \"auto\"\n"
            path: "terminal.config_mode"
            expected_value: "auto"
        }
        {
            name: "zellij.widget_tray contains layout"
            label: "yazelix_widget_tray_schema"
            config: "[zellij]\nwidget_tray = [\"layout\", \"editor\"]\n"
            path: "zellij.widget_tray"
            expected_value: "layout"
        }
    ]

    let results = ($cases | each {|case| check_schema_rejects_removed_enum_value $case })
    ($results | all {|result| $result })
}

def write_minimal_user_zellij_config [fake_home: string] {
    let zellij_config_dir = ($fake_home | path join ".config" "yazelix" "user_configs" "zellij")
    let zellij_config_path = ($zellij_config_dir | path join "config.kdl")
    mkdir $zellij_config_dir
    'keybinds { normal { bind "f1" { WriteChars "fixture"; } } }'
        | save --force --raw $zellij_config_path
}

def write_conflicting_user_zellij_config [fake_home: string] {
    let zellij_config_dir = ($fake_home | path join ".config" "yazelix" "user_configs" "zellij")
    let zellij_config_path = ($zellij_config_dir | path join "config.kdl")
    mkdir $zellij_config_dir
    'show_release_notes true
session_serialization false
serialize_pane_viewport false
ui {
    pane_frames {
        rounded_corners false
        hide_session_name true
    }
}
keybinds { normal { bind "f1" { WriteChars "fixture"; } } }'
        | save --force --raw $zellij_config_path
}

def write_legacy_native_zellij_config [fake_home: string] {
    let zellij_config_dir = ($fake_home | path join ".config" "zellij")
    let zellij_config_path = ($zellij_config_dir | path join "config.kdl")
    mkdir $zellij_config_dir
    'scroll_buffer_size 12345'
        | save --force --raw $zellij_config_path
}

def run_merged_zellij_config_in_fake_home [tmpdir: string, extra_env: record = {}, extra_output?: closure] {
    let out_dir = ($tmpdir | path join "out")
    let fake_home = ($tmpdir | path join "home")
    let fake_config_dir = ($fake_home | path join ".config" "yazelix")

    with-env ({
        HOME: $fake_home
        XDG_CONFIG_HOME: ($fake_home | path join ".config")
        YAZELIX_CONFIG_DIR: $fake_config_dir
        YAZELIX_TEST_OUT_DIR: $out_dir
    } | merge $extra_env) {
        let root = (get_repo_root)
        generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore
        {
            config: (open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "config.kdl"))
        } | merge (if $extra_output == null { {} } else { do $extra_output })
    }
}

# Regression: warm startup should reuse generated Zellij state when inputs are unchanged and invalidate cleanly when a real input changes.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_generate_merged_zellij_config_reuses_unchanged_state_and_invalidates_on_input_change [] {
    print "🧪 Testing merged Zellij config reuses unchanged generated state and invalidates when a real input changes..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_generation_reuse_XXXXXX | str trim)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        let fake_config_dir = ($fake_home | path join ".config" "yazelix")
        let user_config_dir = ($fake_config_dir | path join "user_configs")
        let user_config_path = ($user_config_dir | path join "yazelix.toml")
        let out_dir = ($tmpdir | path join "out")
        let metadata_path = ($out_dir | path join ".yazelix_generation.json")
        let layout_path = ($out_dir | path join "layouts" "yzx_side.swap.kdl")

        mkdir ($fake_home | path join ".config")
        mkdir $user_config_dir
        write_minimal_user_zellij_config $fake_home

        '[editor]
sidebar_width_percent = 25
' | save --force --raw $user_config_path

        let first_output = (with-env {
            HOME: $fake_home
            XDG_CONFIG_HOME: ($fake_home | path join ".config")
            YAZELIX_CONFIG_DIR: $fake_config_dir
            YAZELIX_TEST_OUT_DIR: $out_dir
        } {
            let root = (get_repo_root)
            generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore
            {
                config: (open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "config.kdl"))
                metadata: (open --raw $metadata_path)
                layout: (open --raw $layout_path)
            }
        })

        sleep 10ms

        let second_output = (with-env {
            HOME: $fake_home
            XDG_CONFIG_HOME: ($fake_home | path join ".config")
            YAZELIX_CONFIG_DIR: $fake_config_dir
            YAZELIX_TEST_OUT_DIR: $out_dir
        } {
            let root = (get_repo_root)
            generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore
            {
                config: (open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "config.kdl"))
                metadata: (open --raw $metadata_path)
                layout: (open --raw $layout_path)
            }
        })

        sleep 10ms
        '[editor]
sidebar_width_percent = 35
' | save --force --raw $user_config_path

        let third_output = (with-env {
            HOME: $fake_home
            XDG_CONFIG_HOME: ($fake_home | path join ".config")
            YAZELIX_CONFIG_DIR: $fake_config_dir
            YAZELIX_TEST_OUT_DIR: $out_dir
        } {
            let root = (get_repo_root)
            generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore
            {
                config: (open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "config.kdl"))
                metadata: (open --raw $metadata_path)
                layout: (open --raw $layout_path)
            }
        })
        let temp_find = (^find $out_dir | complete)
        let temp_noise = if $temp_find.exit_code == 0 {
            $temp_find.stdout | lines | where {|path| ($path | str trim | is-not-empty) and ($path | str contains ".yazelix-tmp-")}
        } else {
            []
        }

        if (
            ($second_output.config == $first_output.config)
            and ($second_output.metadata == $first_output.metadata)
            and ($second_output.layout == $first_output.layout)
            and ($third_output.metadata != $first_output.metadata)
            and ($third_output.config | str contains 'sidebar_width_percent "35"')
            and ($third_output.layout | str contains 'size "35%"')
            and not ($third_output.layout | str contains 'size "25%"')
            and ($temp_noise | is-empty)
        ) {
            print "  ✅ Merged Zellij config now reuses unchanged generated state and regenerates when a real config input changes"
            true
        } else {
            print "  ❌ Unexpected reuse or invalidation behavior in generated Zellij state"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Defends: bounded generated-artifact cleanup refuses root and outside targets while still deleting managed children.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_remove_path_within_root_refuses_root_and_outside_targets [] {
    print "🧪 Testing bounded generated-artifact cleanup refuses root and outside targets..."

    let tmpdir = (^mktemp -d /tmp/yazelix_safe_remove_XXXXXX | str trim)
    let managed_root = ($tmpdir | path join "managed")
    let managed_child = ($managed_root | path join "child.txt")
    let external_root = ($tmpdir | path join "external")
    let external_file = ($external_root | path join "external.txt")
    let managed_symlink = ($managed_root | path join "child-link.txt")
    let outside_file = ($tmpdir | path join "outside.txt")

    let result = (try {
        mkdir $managed_root
        mkdir $external_root
        "child" | save --force --raw $managed_child
        "external" | save --force --raw $external_file
        ^ln -s $external_file $managed_symlink
        "outside" | save --force --raw $outside_file

        let remove_child = (try {
            remove_path_within_root $managed_child $managed_root "managed child"
            {ok: true, msg: ""}
        } catch {|err|
            {ok: false, msg: $err.msg}
        })
        let remove_symlink = (try {
            remove_path_within_root $managed_symlink $managed_root "managed symlink"
            {ok: true, msg: ""}
        } catch {|err|
            {ok: false, msg: $err.msg}
        })
        let remove_root = (try {
            remove_path_within_root $managed_root $managed_root "managed root" --recursive
            {ok: true, msg: ""}
        } catch {|err|
            {ok: false, msg: $err.msg}
        })
        let remove_outside = (try {
            remove_path_within_root $outside_file $managed_root "outside target"
            {ok: true, msg: ""}
        } catch {|err|
            {ok: false, msg: $err.msg}
        })

        if (
            $remove_child.ok
            and (not ($managed_child | path exists))
            and $remove_symlink.ok
            and (not ($managed_symlink | path exists))
            and ($external_file | path exists)
            and (not $remove_root.ok)
            and ($remove_root.msg | str contains "Refusing to remove")
            and ($managed_root | path exists)
            and (not $remove_outside.ok)
            and ($remove_outside.msg | str contains "Refusing to remove")
            and ($outside_file | path exists)
        ) {
            print "  ✅ Managed cleanup now deletes bounded children and managed symlinks while refusing root or outside targets"
            true
        } else {
            print $"  ❌ Unexpected bounded cleanup result: child=($remove_child | to json -r) symlink=($remove_symlink | to json -r) root=($remove_root | to json -r) outside=($remove_outside | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: recursive managed cleanup must tolerate read-only copied store artifacts before removal.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_remove_path_within_root_relaxes_read_only_managed_directories_before_recursive_cleanup [] {
    print "🧪 Testing bounded recursive cleanup relaxes read-only managed directories before removal..."

    let tmpdir = (^mktemp -d /tmp/yazelix_safe_remove_read_only_XXXXXX | str trim)
    let managed_root = ($tmpdir | path join "managed")
    let managed_dir = ($managed_root | path join "bundled-plugin")
    let managed_file = ($managed_dir | path join "main.lua")

    let result = (try {
        mkdir $managed_root
        mkdir $managed_dir
        "__YAZELIX_RUNTIME_DIR__" | save --force --raw $managed_file

        let chmod_result = (^chmod -R a-w $managed_dir | complete)
        if $chmod_result.exit_code != 0 {
            error make {msg: $"Failed to make managed fixture read-only: ($chmod_result.stderr | str trim)"}
        }

        let remove_dir = (try {
            remove_path_within_root $managed_dir $managed_root "bundled plugin" --recursive
            {ok: true, msg: ""}
        } catch {|err|
            {ok: false, msg: $err.msg}
        })

        if (
            $remove_dir.ok
            and (not ($managed_dir | path exists))
            and ($managed_root | path exists)
        ) {
            print "  ✅ Managed recursive cleanup now removes read-only generated directories"
            true
        } else {
            print $"  ❌ Unexpected recursive managed cleanup result: dir=($remove_dir | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    chmod -R u+w $tmpdir
    rm -rf $tmpdir
    $result
}

# Regression: legacy Yazi override paths now fail fast instead of being relocated during generation.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_generate_merged_yazi_config_rejects_legacy_user_overrides [] {
    print "🧪 Testing merged Yazi config rejects legacy user overrides and points to the import flow..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_yazi_user_configs_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    let legacy_user_dir = ($temp_config_dir | path join "configs" "yazi" "user")
    mkdir ($tmp_home | path join ".config")
    mkdir $temp_config_dir
    mkdir $legacy_user_dir

    let result = (try {
        '-- legacy user code
return "yazi-user-marker"
' | save --force --raw ($legacy_user_dir | path join "init.lua")

        let generation_result = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: ($tmp_home | path join ".config")
            XDG_DATA_HOME: ($tmp_home | path join ".local" "share")
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            try {
                generate_merged_yazi_config $repo_root --quiet | ignore
                {ok: true}
            } catch {|err|
                {ok: false, msg: $err.msg}
            }
        })

        if (
            (not $generation_result.ok)
            and (($generation_result.msg | default "") | str contains "yzx import yazi")
            and (($generation_result.msg | default "") | str contains "~/.config/yazelix/user_configs/yazi/")
            and (($legacy_user_dir | path join "init.lua") | path exists)
        ) {
            print "  ✅ Legacy Yazi user overrides now fail fast and point to the explicit import flow"
            true
        } else {
            print $"  ❌ Unexpected result: ($generation_result | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Regression: generated Yazi Starship plugin config must stay writable so repeated Yazi regeneration does not crash at startup.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_generate_merged_yazi_config_syncs_starship_plugin_config [] {
    print "🧪 Testing merged Yazi config syncs the bundled Starship plugin config into the managed Yazi surface across repeated regenerations..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_yazi_starship_config_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir ($tmp_home | path join ".config")

    let result = (try {
        let generated = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: ($tmp_home | path join ".config")
            XDG_DATA_HOME: ($tmp_home | path join ".local" "share")
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
            YAZELIX_LOGS_DIR: ($tmp_home | path join ".local" "share" "yazelix" "logs")
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            let merged_dir = (generate_merged_yazi_config $repo_root --quiet)
            generate_merged_yazi_config $repo_root --quiet | ignore
            let mode_result = (^stat -c '%A' ($merged_dir | path join "yazelix_starship.toml") | complete)
            let temp_find = (^find $merged_dir | complete)
            {
                merged_dir: $merged_dir
                init_lua: (open --raw ($merged_dir | path join "init.lua"))
                starship_config: (open --raw ($merged_dir | path join "yazelix_starship.toml"))
                starship_config_mode: (if $mode_result.exit_code == 0 { $mode_result.stdout | str trim } else { "" })
                temp_noise: (if $temp_find.exit_code == 0 {
                    $temp_find.stdout | lines | where {|path| ($path | str trim | is-not-empty) and ($path | str contains ".yazelix-tmp-")}
                } else {
                    []
                })
            }
        })

        let expected_starship_config_path = ($generated.merged_dir | path join "yazelix_starship.toml")

        if (
            ($expected_starship_config_path | path exists)
            and ($generated.init_lua | str contains $"config_file = \"($expected_starship_config_path)\"")
            and ($generated.starship_config | str contains "# YAZELIX STARSHIP CONFIG FOR YAZI SIDEBAR")
            and ($generated.starship_config_mode != "-r--r--r--")
            and ($generated.temp_noise | is-empty)
        ) {
            print "  ✅ Yazi Starship plugin now points at a managed sidebar-specific config that survives repeated regeneration"
            true
        } else {
            print $"  ❌ Missing stable managed Yazi Starship config wiring: path=($expected_starship_config_path) exists=(($expected_starship_config_path | path exists)) mode=($generated.starship_config_mode)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Regression: generated bundled Yazi plugins must render the runtime root instead of leaking the template placeholder.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_generate_merged_yazi_config_renders_runtime_placeholders_in_plugins [] {
    print "🧪 Testing merged Yazi config renders runtime placeholders inside bundled plugins..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_yazi_plugin_runtime_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir ($tmp_home | path join ".config")

    let result = (try {
        let generated = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: ($tmp_home | path join ".config")
            XDG_DATA_HOME: ($tmp_home | path join ".local" "share")
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
            YAZELIX_LOGS_DIR: ($tmp_home | path join ".local" "share" "yazelix" "logs")
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            regenerate_runtime_configs $repo_root --quiet | ignore
            let merged_dir = ($tmp_home | path join ".local" "share" "yazelix" "configs" "yazi")
            let zoxide_plugin = ($merged_dir | path join "plugins" "zoxide-editor.yazi" "main.lua")
            let warm_sentinel = ($merged_dir | path join "plugins" "zoxide-editor.yazi" "warm_skip_sentinel")
            "warm asset marker" | save --force --raw $warm_sentinel
            regenerate_runtime_configs $repo_root --quiet
            let sentinel_after_warm_skip = ($warm_sentinel | path exists)
            rm --force $zoxide_plugin
            regenerate_runtime_configs $repo_root --quiet
            {
                zoxide_plugin: (open --raw $zoxide_plugin)
                sentinel_after_warm_skip: $sentinel_after_warm_skip
            }
        })

        if (
            ($generated.zoxide_plugin | str contains ($repo_root | path join "nushell" "scripts" "integrations" "zoxide_open_in_editor.nu"))
            and not ($generated.zoxide_plugin | str contains "__YAZELIX_RUNTIME_DIR__")
            and $generated.sentinel_after_warm_skip
        ) {
            print "  ✅ Generated Yazi plugins render real runtime paths, skip static recopy on warm paths, and self-heal missing bundled files"
            true
        } else {
            print "  ❌ Generated Zoxide Yazi plugin still leaked the runtime placeholder or recopied warm assets unnecessarily"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Regression: warm Yazi generation should leave unchanged managed files untouched instead of rewriting them on every startup.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_generate_merged_yazi_config_skips_unchanged_managed_file_rewrites [] {
    print "🧪 Testing merged Yazi config skips rewriting unchanged managed files on warm runs..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_yazi_unchanged_reuse_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($temp_config_dir | path join "user_configs")
    let user_config_path = ($user_config_dir | path join "yazelix.toml")
    mkdir ($tmp_home | path join ".config")
    mkdir $user_config_dir

    let result = (try {
        '[yazi]
theme = "tokyo-night"
plugins = ["git"]
' | save --force --raw $user_config_path

        let generated = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: ($tmp_home | path join ".config")
            XDG_DATA_HOME: ($tmp_home | path join ".local" "share")
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
            YAZELIX_LOGS_DIR: ($tmp_home | path join ".local" "share" "yazelix" "logs")
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            let merged_dir = (generate_merged_yazi_config $repo_root --quiet)
            let tracked_paths = [
                ($merged_dir | path join "yazi.toml")
                ($merged_dir | path join "theme.toml")
                ($merged_dir | path join "keymap.toml")
                ($merged_dir | path join "init.lua")
            ]
            let before = (
                $tracked_paths
                | each {|path_value|
                    {
                        path: $path_value
                        modified: ((ls -D $path_value | get modified.0) | into datetime)
                    }
                }
            )

            sleep 50ms
            generate_merged_yazi_config $repo_root --quiet | ignore

            let after = (
                $tracked_paths
                | each {|path_value|
                    {
                        path: $path_value
                        modified: ((ls -D $path_value | get modified.0) | into datetime)
                    }
                }
            )

            {
                before: $before
                after: $after
            }
        })

        let changed_paths = (
            $generated.before
            | each {|before_entry|
                let after_entry = ($generated.after | where path == $before_entry.path | get 0)
                if $after_entry.modified != $before_entry.modified {
                    $before_entry.path
                } else {
                    null
                }
            }
            | where {|path_value| $path_value != null }
        )

        if ($changed_paths | is-empty) {
            print "  ✅ Warm Yazi generation now leaves unchanged managed files untouched"
            true
        } else {
            print $"  ❌ Warm Yazi generation still rewrote unchanged files: ($changed_paths | str join ', ')"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Regression: source-checkout sessions must generate runtime-owned Yazi and Zellij artifacts against the active runtime, not a stale installed-runtime reference.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_generated_runtime_configs_prefer_active_runtime_over_installed_reference [] {
    print "🧪 Testing generated Yazi and Zellij runtime configs prefer the active runtime over a stale installed-runtime reference..."

    let repo_root = (get_repo_root)
    let tmpdir = (^mktemp -d /tmp/yazelix_runtime_identity_split_XXXXXX | str trim)
    let fake_home = ($tmpdir | path join "home")
    let fake_state_dir = ($tmpdir | path join "state")
    let fake_installed_runtime = ($tmpdir | path join "fake_installed_runtime")
    let out_dir = ($tmpdir | path join "out")
    mkdir ($fake_state_dir | path join "runtime")
    mkdir ($fake_installed_runtime | path join "configs" "helix")
    mkdir ($fake_home | path join ".config")
    ^ln -s $fake_installed_runtime ($fake_state_dir | path join "runtime" "current")

    let result = (try {
        write_minimal_user_zellij_config $fake_home

        let generated = (with-env {
            HOME: $fake_home
            XDG_CONFIG_HOME: ($fake_home | path join ".config")
            XDG_DATA_HOME: ($fake_home | path join ".local" "share")
            YAZELIX_CONFIG_DIR: ($fake_home | path join ".config" "yazelix")
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: $fake_state_dir
        } {
            generate_merged_yazi_config $repo_root --quiet | ignore
            generate_merged_zellij_config $repo_root $out_dir | ignore
            {
                yazi_toml: (open --raw ($fake_state_dir | path join "configs" "yazi" "yazi.toml"))
                zellij_config: (open --raw ($out_dir | path join "config.kdl"))
                zellij_layout: (open --raw ($out_dir | path join "layouts" "yzx_side.kdl"))
            }
        })

        if (
            ($generated.yazi_toml | str contains $"nu ($repo_root | path join "nushell" "scripts" "integrations" "open_file.nu")")
            and not ($generated.yazi_toml | str contains $fake_installed_runtime)
            and ($generated.zellij_config | str contains $repo_root)
            and not ($generated.zellij_config | str contains $fake_installed_runtime)
            and ($generated.zellij_layout | str contains $repo_root)
            and not ($generated.zellij_layout | str contains $fake_installed_runtime)
        ) {
            print "  ✅ Generated runtime-owned configs now stay pinned to the active runtime in source-checkout sessions"
            true
        } else {
            print $"  ❌ Runtime-owned generated configs still leaked the legacy installed-runtime reference: fake_installed_runtime=($fake_installed_runtime)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: recursive managed cleanup must remove bounded symlinks without chmodding immutable external targets.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_remove_path_within_root_recursive_cleanup_removes_managed_symlinks_without_touching_targets [] {
    print "🧪 Testing bounded recursive cleanup removes managed symlinks without chmodding immutable external targets..."

    let tmpdir = (^mktemp -d /tmp/yazelix_safe_remove_symlink_recursive_XXXXXX | str trim)
    let managed_root = ($tmpdir | path join "managed")
    let managed_symlink = ($managed_root | path join "runtime-entry")
    let external_target = "/etc/passwd"

    let result = (try {
        if not ($external_target | path exists) {
            error make {msg: $"Expected immutable external target to exist for this regression test: ($external_target)"}
        }

        mkdir $managed_root
        ^ln -s $external_target $managed_symlink

        let remove_symlink = (try {
            remove_path_within_root $managed_symlink $managed_root "runtime project symlink" --recursive
            {ok: true, msg: ""}
        } catch {|err|
            {ok: false, msg: $err.msg}
        })

        if (
            $remove_symlink.ok
            and (not ($managed_symlink | path exists))
            and ($external_target | path exists)
            and ($managed_root | path exists)
        ) {
            print "  ✅ Managed recursive cleanup now removes symlinks without touching immutable external targets"
            true
        } else {
            print $"  ❌ Unexpected recursive symlink cleanup result: symlink=($remove_symlink | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Defends: sidebar width propagates into generated Zellij layouts and plugin config.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_generate_merged_zellij_config_carries_sidebar_width_to_layouts_and_plugin_config [] {
    print "🧪 Testing merged Zellij config carries editor.sidebar_width_percent into layouts and plugin config..."

    if (which zellij | is-empty) {
        print "  ℹ️  Skipping Zellij config merge test because zellij is not available"
        return true
    }

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_sidebar_width_test_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        let out_dir = ($tmpdir | path join "out")
        let fake_home = ($tmpdir | path join "home")
        write_minimal_user_zellij_config $fake_home
        '[editor]
sidebar_width_percent = 25
' | save --force --raw $config_path

        let output = (with-env {
            HOME: $fake_home
            YAZELIX_CONFIG_OVERRIDE: $config_path
            YAZELIX_TEST_OUT_DIR: $out_dir
        } {
            let root = (get_repo_root)
            generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore
            {
                config: (open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "config.kdl"))
                layout: (open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "layouts" "yzx_side.swap.kdl"))
            }
        })
        let generated_config = ($output.config | str trim)
        let generated_layout = ($output.layout | str trim)

        if (
            ($generated_config | str contains 'sidebar_width_percent "25"')
            and ($generated_layout | str contains 'size "25%"')
            and ($generated_layout | str contains 'size "75%"')
            and ($generated_layout | str contains 'size "45%"')
            and ($generated_layout | str contains 'size "30%"')
        ) {
            print "  ✅ Merged config and generated layouts carry the configured sidebar width end to end"
            true
        } else {
            print "  ❌ Sidebar width did not propagate through merged Zellij config generation"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: generated zjstatus tab bars must cap the rendered tab window and show overflow markers before the bar breaks.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_generate_merged_zellij_config_caps_zjstatus_tab_window_with_overflow_markers [] {
    print "🧪 Testing merged Zellij layouts cap the zjstatus tab window and keep visible tab indexes..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_tab_window_XXXXXX | str trim)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        write_minimal_user_zellij_config $fake_home

        let output = (run_merged_zellij_config_in_fake_home $tmpdir {} {
            {
                layout: (open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "layouts" "yzx_side.kdl"))
            }
        })
        let generated_layout = ($output.layout | str trim)

        if (
            ($generated_layout | str contains 'tab_display_count "6"')
            and ($generated_layout | str contains 'tab_truncate_start_format "#[fg=#ff6600,bold]< +{count} ... "')
            and ($generated_layout | str contains 'tab_truncate_end_format   "#[fg=#ff6600,bold]... +{count} > "')
            and ($generated_layout | str contains 'tab_normal   "#[fg=#ffff00] [{index}] {name} "')
            and ($generated_layout | str contains 'tab_active   "#[bg=#ff6600,fg=#000000,bold] [{index}] {name} {floating_indicator}"')
        ) {
            print "  ✅ Generated zjstatus layouts now keep visible tab indexes while truncating overflowing tab windows"
            true
        } else {
            print $"  ❌ Generated layout is missing the compact zjstatus tab-window policy: ($generated_layout)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: Ctrl+y should bind directly to the pane orchestrator instead of spawning a transient Nushell helper pane.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_generate_merged_zellij_config_binds_ctrl_y_directly_to_pane_orchestrator_toggle [] {
    print "🧪 Testing merged Zellij config binds Ctrl+y directly to the pane orchestrator toggle action..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_ctrl_y_helper_name_XXXXXX | str trim)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        write_minimal_user_zellij_config $fake_home

        let output = (run_merged_zellij_config_in_fake_home $tmpdir {})
        let generated_config = ($output.config | str trim)

        if (
            ($generated_config | str contains 'bind "Ctrl y" {')
            and ($generated_config | str contains 'MessagePlugin "yazelix_pane_orchestrator" {')
            and ($generated_config | str contains 'name "toggle_editor_sidebar_focus"')
            and not ($generated_config | str contains 'configs/zellij/scripts/toggle_editor_sidebar_focus.nu')
            and not ($generated_config | str contains 'yzx_toggle_editor_sidebar_focus')
        ) {
            print "  ✅ Ctrl+y now routes straight to the pane orchestrator without the transient Nushell helper-pane path"
            true
        } else {
            print $"  ❌ Generated Zellij config is missing the direct Ctrl+y pane-orchestrator binding contract: ($generated_config)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: direct pane-orchestrator messages must stay session-local and must not resupply runtime_dir on Alt+m.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_generate_merged_zellij_config_keeps_alt_m_pane_orchestrator_message_session_local [] {
    print "🧪 Testing merged Zellij config keeps Alt+m pane-orchestrator messages session-local..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_alt_m_session_local_XXXXXX | str trim)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        write_minimal_user_zellij_config $fake_home

        let output = (run_merged_zellij_config_in_fake_home $tmpdir {})
        let generated_config = ($output.config | str trim)
        let runtime_dir_lines = (
            $generated_config
            | lines
            | where {|line| $line | str contains 'runtime_dir "'}
            | length
        )

        if (
            ($generated_config | str contains "bind \"Alt m\" {\n            MessagePlugin \"yazelix_pane_orchestrator\" {\n                name \"open_workspace_terminal\"\n            }\n        }")
            and ($runtime_dir_lines == 1)
        ) {
            print "  ✅ Alt+m now targets the session-loaded pane orchestrator without resupplying runtime ownership"
            true
        } else {
            print $"  ❌ Generated Zellij config still leaks runtime ownership through the Alt+m pane-orchestrator message path: ($generated_config)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: popup and menu transient panes must route through the pane orchestrator and share one configured floating geometry contract.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_generate_merged_zellij_config_routes_popup_and_menu_through_shared_transient_pane_contract [] {
    print "🧪 Testing merged Zellij config routes popup/menu through the shared pane-orchestrator transient-pane contract..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_transient_contract_XXXXXX | str trim)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        let config_path = ($tmpdir | path join "yazelix.toml")

        write_minimal_user_zellij_config $fake_home
        '[zellij]
popup_width_percent = 82
popup_height_percent = 76
' | save --force --raw $config_path

        let output = (run_merged_zellij_config_in_fake_home $tmpdir {
            YAZELIX_CONFIG_OVERRIDE: $config_path
        })
        let generated_config = ($output.config | str trim)
        let repo_root = (get_repo_root)
        let runtime_dir_lines = (
            $generated_config
            | lines
            | where {|line| $line | str contains $"runtime_dir \"($repo_root)\""}
            | length
        )

        if (
            ($generated_config | str contains 'bind "Alt t" {')
            and ($generated_config | str contains 'bind "Alt Shift M" {')
            and (($generated_config | str contains 'name "toggle_transient_pane"') and (($generated_config | lines | where {|line| $line | str contains 'name "toggle_transient_pane"'} | length) >= 2))
            and ($generated_config | str contains 'payload "popup"')
            and ($generated_config | str contains 'payload "menu"')
            and ($generated_config | str contains "bind \"Alt t\" {\n            MessagePlugin \"yazelix_pane_orchestrator\" {\n                name \"toggle_transient_pane\"\n                payload \"popup\"\n            }\n        }")
            and ($generated_config | str contains "bind \"Alt Shift M\" {\n            MessagePlugin \"yazelix_pane_orchestrator\" {\n                name \"toggle_transient_pane\"\n                payload \"menu\"\n            }\n        }")
            and ($runtime_dir_lines == 1)
            and not ($generated_config | str contains "yazelix_popup_runner.wasm")
            and not ($generated_config | str contains 'configs/zellij/scripts/yzx_toggle_popup.nu')
            and not ($generated_config | str contains 'width "70%"')
            and not ($generated_config | str contains 'height "70%"')
            and ($generated_config | str contains $"runtime_dir \"($repo_root)\"")
            and ($generated_config | str contains 'popup_width_percent "82"')
            and ($generated_config | str contains 'popup_height_percent "76"')
        ) {
            print "  ✅ popup and menu now bind directly to the pane orchestrator and carry one shared transient-pane geometry contract in plugin config"
            true
        } else {
            print $"  ❌ Generated Zellij config is missing the shared transient-pane contract: ($generated_config)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: non-persistent Zellij sessions must quit on terminal close while persistent sessions may detach.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_generate_merged_zellij_config_sets_on_force_close_by_session_mode [] {
    print "🧪 Testing merged Zellij config sets on_force_close from Yazelix session mode..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_force_close_mode_XXXXXX | str trim)

    let result = (try {
        let nonpersistent_config = ($tmpdir | path join "nonpersistent.toml")
        let persistent_config = ($tmpdir | path join "persistent.toml")
        let nonpersistent_home = ($tmpdir | path join "nonpersistent" "home")
        let persistent_home = ($tmpdir | path join "persistent" "home")

        '[zellij]
persistent_sessions = false
' | save --force --raw $nonpersistent_config

        '[zellij]
persistent_sessions = true
session_name = "fixture"
' | save --force --raw $persistent_config

        write_minimal_user_zellij_config $nonpersistent_home
        write_minimal_user_zellij_config $persistent_home

        let nonpersistent_output = (run_merged_zellij_config_in_fake_home ($tmpdir | path join "nonpersistent") {
            YAZELIX_CONFIG_OVERRIDE: $nonpersistent_config
        })
        let persistent_output = (run_merged_zellij_config_in_fake_home ($tmpdir | path join "persistent") {
            YAZELIX_CONFIG_OVERRIDE: $persistent_config
        })

        if (
            (($nonpersistent_output.config | lines | where {|line| ($line | str trim) == 'on_force_close "quit"'} | length) == 1)
            and (($nonpersistent_output.config | lines | where {|line| ($line | str trim) == 'on_force_close "detach"'} | length) == 0)
            and (($persistent_output.config | lines | where {|line| ($line | str trim) == 'on_force_close "detach"'} | length) == 1)
            and (($persistent_output.config | lines | where {|line| ($line | str trim) == 'on_force_close "quit"'} | length) == 0)
        ) {
            print "  ✅ Merged Zellij config now quits default sessions on terminal close while preserving detach semantics for persistent sessions"
            true
        } else {
            print "  ❌ Unexpected on_force_close policy in generated Zellij config"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: first-match KDL parsing must not let earlier user ui/serialization/release-note settings override Yazelix-owned output.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_generate_merged_zellij_config_replaces_conflicting_ui_and_serialization_settings [] {
    print "🧪 Testing merged Zellij config replaces conflicting user ui and serialization settings before first-match parsing can ignore Yazelix overrides..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_first_match_XXXXXX | str trim)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        let config_path = ($tmpdir | path join "yazelix.toml")
        write_conflicting_user_zellij_config $fake_home

        '[zellij]
rounded_corners = true
' | save --force --raw $config_path

        let output = (run_merged_zellij_config_in_fake_home $tmpdir {
            YAZELIX_CONFIG_OVERRIDE: $config_path
        })
        let generated_config = ($output.config | str trim)

        if (
            (($generated_config | lines | where {|line| ($line | str trim) == 'show_release_notes false'} | length) == 1)
            and (($generated_config | lines | where {|line| ($line | str trim) == 'show_release_notes true'} | length) == 0)
            and (($generated_config | lines | where {|line| ($line | str trim) == 'session_serialization true'} | length) == 1)
            and (($generated_config | lines | where {|line| ($line | str trim) == 'session_serialization false'} | length) == 0)
            and (($generated_config | lines | where {|line| ($line | str trim) == 'serialize_pane_viewport true'} | length) == 1)
            and (($generated_config | lines | where {|line| ($line | str trim) == 'serialize_pane_viewport false'} | length) == 0)
            and not ($generated_config | str contains 'pane_viewport_serialization')
            and (($generated_config | lines | where {|line| ($line | str trim) == 'rounded_corners true'} | length) == 1)
            and (($generated_config | lines | where {|line| ($line | str trim) == 'rounded_corners false'} | length) == 0)
            and (($generated_config | lines | where {|line| ($line | str trim) == 'hide_session_name true'} | length) == 1)
        ) {
            print "  ✅ Merged Zellij config now replaces conflicting user ui and serialization settings before Zellij first-match parsing can ignore Yazelix-owned values"
            true
        } else {
            print "  ❌ Generated Zellij config still leaves conflicting first-match settings ahead of Yazelix-owned output"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Defends: native Zellij config can still be used without Yazelix taking ownership of it.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_generate_merged_zellij_config_uses_native_user_config_without_relocating_it [] {
    print "🧪 Testing merged Zellij config uses native Zellij config as a fallback without relocating it..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_user_cfg_relocate_XXXXXX | str trim)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        write_legacy_native_zellij_config $fake_home

        let output = (run_merged_zellij_config_in_fake_home $tmpdir {} {||
            {
                managed_exists: ((($fake_home | path join ".config" "yazelix" "user_configs" "zellij" "config.kdl") | path exists))
                legacy_exists: ((($fake_home | path join ".config" "zellij" "config.kdl") | path exists))
            }
        })
        let config_stdout = ($output.config | str trim)

        if (
            ($config_stdout | str contains 'scroll_buffer_size 12345')
            and (not $output.managed_exists)
            and $output.legacy_exists
        ) {
            print "  ✅ Merged Zellij config uses the native Zellij config as a fallback without moving it"
            true
        } else {
            print "  ❌ Unexpected result: native Zellij config was not preserved as a fallback correctly"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: managed Zellij config wins cleanly when both native and managed files exist.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_generate_merged_zellij_config_prefers_managed_user_config_when_native_config_also_exists [] {
    print "🧪 Testing merged Zellij config prefers the managed user config and leaves native Zellij config alone..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_dual_config_XXXXXX | str trim)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        write_minimal_user_zellij_config $fake_home
        write_legacy_native_zellij_config $fake_home

        let output = (run_merged_zellij_config_in_fake_home $tmpdir {} {||
            {
                managed_exists: ((($fake_home | path join ".config" "yazelix" "user_configs" "zellij" "config.kdl") | path exists))
                native_exists: ((($fake_home | path join ".config" "zellij" "config.kdl") | path exists))
            }
        })
        let config_stdout = ($output.config | str trim)

        if (
            ($config_stdout | str contains 'bind "f1" { WriteChars "fixture"; }')
            and not ($config_stdout | str contains 'scroll_buffer_size 12345')
            and $output.managed_exists
            and $output.native_exists
        ) {
            print "  ✅ Merged Zellij config keeps the managed user config canonical without deleting the native Zellij config"
            true
        } else {
            print "  ❌ Unexpected result: managed/native Zellij config ownership was not preserved correctly"
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
    with-env {
        YAZELIX_RUNTIME_DIR: (get_repo_root)
        YAZELIX_YZX_CORE_BIN: (resolve_test_yzx_core_bin)
    } {
        [
            (test_generate_all_terminal_configs_keeps_terminal_overrides_opt_in)
            (test_terminal_override_imports_ignore_yazelix_dir_runtime_root)
            (test_managed_wrapper_launch_command_does_not_forward_config_mode_flag)
            (test_ghostty_linux_launch_command_keeps_linux_specific_flags)
            (test_ghostty_linux_launch_command_prefers_runtime_owned_nixgl_wrapper)
            (test_ghostty_wayland_wrapper_falls_back_to_simple_im_without_active_daemon)
            (test_ghostty_wayland_wrapper_preserves_active_ibus_env)
            (test_ghostty_macos_launch_command_omits_linux_specific_flags)
            (test_active_config_normalize_uses_runtime_yzx_core_helper_when_present)
            (test_active_config_normalize_surfaces_yzx_core_config_errors_without_fallback)
            (test_active_config_normalize_rejects_packaged_runtime_missing_yzx_core)
            (test_active_config_normalize_source_checkout_uses_explicit_yzx_core_helper)
            (test_active_config_normalize_source_checkout_missing_helper_does_not_fallback)
            (test_run_yzx_core_request_prefers_newer_source_checkout_helper_over_stale_release)
            (test_record_materialized_state_accepts_symlinked_managed_main_config)
            (test_user_mode_requires_real_terminal_config)
            (test_config_schema_rejects_removed_enum_values)
            (test_generate_merged_yazi_config_rejects_legacy_user_overrides)
            (test_generate_merged_yazi_config_syncs_starship_plugin_config)
            (test_generate_merged_yazi_config_renders_runtime_placeholders_in_plugins)
            (test_generate_merged_yazi_config_skips_unchanged_managed_file_rewrites)
            (test_generated_runtime_configs_prefer_active_runtime_over_installed_reference)
            (test_generate_merged_zellij_config_uses_native_user_config_without_relocating_it)
            (test_generate_merged_zellij_config_prefers_managed_user_config_when_native_config_also_exists)
            (test_generate_merged_zellij_config_reuses_unchanged_state_and_invalidates_on_input_change)
            (test_remove_path_within_root_refuses_root_and_outside_targets)
            (test_remove_path_within_root_relaxes_read_only_managed_directories_before_recursive_cleanup)
            (test_remove_path_within_root_recursive_cleanup_removes_managed_symlinks_without_touching_targets)
            (test_generate_merged_zellij_config_carries_sidebar_width_to_layouts_and_plugin_config)
            (test_generate_merged_zellij_config_caps_zjstatus_tab_window_with_overflow_markers)
            (test_generate_merged_zellij_config_binds_ctrl_y_directly_to_pane_orchestrator_toggle)
            (test_generate_merged_zellij_config_keeps_alt_m_pane_orchestrator_message_session_local)
            (test_generate_merged_zellij_config_routes_popup_and_menu_through_shared_transient_pane_contract)
            (test_generate_merged_zellij_config_sets_on_force_close_by_session_mode)
            (test_generate_merged_zellij_config_replaces_conflicting_ui_and_serialization_settings)
        ]
    }
}
