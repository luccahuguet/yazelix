#!/usr/bin/env nu
# Test lane: maintainer
# Defends: docs/specs/test_suite_governance.md

use ./yzx_test_helpers.nu [get_repo_root repo_path]
use ../setup/helix_config_merger.nu [generate_managed_helix_config get_helix_import_notice_marker_path]
use ../setup/zellij_config_merger.nu [generate_merged_zellij_config]
use ../setup/zellij_plugin_paths.nu [get_tracked_zjstatus_wasm_path get_zjstatus_wasm_path]
use ../utils/launch_state.nu [get_launch_env]
use ../utils/nushell_externs.nu [get_generated_yzx_extern_path sync_generated_yzx_extern_bridge]
use ../utils/shell_user_hooks.nu [get_yazelix_shell_user_hook_path sync_generated_nushell_user_hook_bridge]

def test_generate_managed_helix_config_merges_user_config_and_enforces_reveal [] {
    print "🧪 Testing managed Helix config generation keeps user settings while enforcing Yazelix reveal..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_managed_helix_config_XXXXXX | str trim)
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir $user_config_dir

    let result = (try {
        let helix_user_dir = ($user_config_dir | path join "helix")
        mkdir $helix_user_dir
        '[editor]
line-number = "relative"

[keys.normal]
g = "goto_file_start"
A-r = ":noop"
' | save --force --raw ($helix_user_dir | path join "config.toml")

        let merged = (with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
        } {
            let output_path = (generate_managed_helix_config)
            {
                output_path: $output_path
                config: (open $output_path)
            }
        })

        let normal_keys = ($merged.config.keys | get normal)

        let expected_output_path = ($tmp_home | path join ".local" "share" "yazelix" "configs" "helix" "config.toml")

        if (
            ($merged.output_path == $expected_output_path)
            and (($merged.config.editor | get "line-number") == "relative")
            and (($normal_keys | get g) == "goto_file_start")
            and (($normal_keys | get "A-r") == ':sh yzx reveal "%{buffer_name}"')
        ) {
            print "  ✅ Managed Helix config preserves user overrides while forcing the Yazelix reveal binding"
            true
        } else {
            print $"  ❌ Unexpected managed Helix config: (($merged | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_get_launch_env_wraps_helix_with_managed_wrapper [] {
    print "🧪 Testing launch env wraps Helix with the Yazelix-managed wrapper and records the real binary..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_managed_helix_launch_env_XXXXXX | str trim)
    let profile_path = ($tmp_home | path join "profile")
    let profile_bin = ($profile_path | path join "bin")
    mkdir $profile_bin
    "" | save --force --raw ($profile_bin | path join "hx")

    let result = (try {
        let launch_env = (with-env {
            HOME: $tmp_home
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
        } {
            get_launch_env {} $profile_path
        })

        let expected_wrapper = ($repo_root | path join "shells" "posix" "yazelix_hx.sh")
        let expected_binary = ($profile_bin | path join "hx")

        if (
            ($launch_env.EDITOR == $expected_wrapper)
            and (($launch_env | get YAZELIX_MANAGED_EDITOR_KIND) == "helix")
            and (($launch_env | get YAZELIX_MANAGED_HELIX_BINARY) == $expected_binary)
        ) {
            print "  ✅ Launch env now routes managed Helix sessions through the Yazelix wrapper while preserving the real Helix binary"
            true
        } else {
            print $"  ❌ Unexpected managed Helix launch env: (($launch_env | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_generate_merged_zellij_config_wraps_nu_default_shell [] {
    print "🧪 Testing merged Zellij config rewrites default_shell = \"nu\" to the managed Nushell wrapper..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_managed_nu_zellij_XXXXXX | str trim)
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let out_dir = ($tmp_home | path join "out")

    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir $user_config_dir

    let result = (try {
        '[shell]
default_shell = "nu"
' | save --force --raw ($user_config_dir | path join "yazelix.toml")

        let generated = (with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
        } {
            generate_merged_zellij_config $repo_root $out_dir | ignore
            open --raw ($out_dir | path join "config.kdl")
        })

        let expected_shell = ($repo_root | path join "shells" "posix" "yazelix_nu.sh")

        if (
            ($generated | str contains $"default_shell \"($expected_shell)\"")
            and not ($generated | str contains 'default_shell "nu"')
        ) {
            print "  ✅ Merged Zellij config now routes managed Nushell panes through the Yazelix wrapper"
            true
        } else {
            print $"  ❌ Unexpected merged default_shell output: ($generated)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_generate_merged_zellij_layouts_use_stable_zjstatus_plugin_path [] {
    print "🧪 Testing generated Zellij layouts load zjstatus from the stable Yazelix plugin path..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_stable_zjstatus_layouts_XXXXXX | str trim)
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let out_dir = ($tmp_home | path join "out")

    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir $user_config_dir

    let result = (try {
        let generated = (with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: $state_dir
        } {
            generate_merged_zellij_config $repo_root $out_dir | ignore
            {
                layout: (open --raw ($out_dir | path join "layouts" "yzx_side.kdl"))
                stable_plugin_path: ($state_dir | path join "configs" "zellij" "plugins" "zjstatus.wasm")
            }
        })

        let expected_plugin_url = $"file:($generated.stable_plugin_path)"

        if (
            ($generated.layout | str contains $"plugin location=\"($expected_plugin_url)\"")
            and not ($generated.layout | str contains "/nix/store/")
            and ($generated.stable_plugin_path | path exists)
        ) {
            print "  ✅ Generated Zellij layouts now point zjstatus at the stable Yazelix plugin path instead of a runtime store path"
            true
        } else {
            print $"  ❌ Unexpected zjstatus plugin layout output: (($generated | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_zjstatus_permission_cache_migrates_to_tracked_and_stable_paths [] {
    print "🧪 Testing zjstatus permission grants migrate onto the tracked and stable Yazelix plugin paths..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_zjstatus_permission_cache_XXXXXX | str trim)
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let cache_path = ($tmp_home | path join ".cache" "zellij" "permissions.kdl")

    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir ($cache_path | path dirname)

    let result = (try {
        '"/tmp/legacy/zjstatus.wasm" {
    ReadApplicationState
    ChangeApplicationState
    RunCommands
}
' | save --force --raw $cache_path

        let migrated = (with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: $state_dir
        } {
            let stable_path = (get_zjstatus_wasm_path $repo_root)
            let tracked_path = (get_tracked_zjstatus_wasm_path $repo_root)
            {
                stable_path: $stable_path
                tracked_path: $tracked_path
                cache: (open --raw $cache_path)
            }
        })

        if (
            ($migrated.stable_path | path exists)
            and ($migrated.cache | str contains $"\"($migrated.tracked_path)\"")
            and ($migrated.cache | str contains $"\"($migrated.stable_path)\"")
            and ($migrated.cache | str contains "RunCommands")
        ) {
            print "  ✅ zjstatus permission grants now migrate onto both the tracked runtime path and the stable Yazelix plugin path"
            true
        } else {
            print $"  ❌ Unexpected zjstatus permission cache state: (($migrated | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_generate_nushell_initializer_removes_starship_right_prompt [] {
    print "🧪 Testing generated Nushell initializer removes the Starship right prompt path..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_nu_initializer_XXXXXX | str trim)

    let result = (try {
        let output = (with-env {
            HOME: $tmp_home
            YAZELIX_QUIET_MODE: "true"
        } {
            do { ^nu (repo_path "nushell" "scripts" "setup" "initializers.nu") $repo_root "nu" } | complete
        })
        let aggregate = (open --raw ($tmp_home | path join ".local" "share" "yazelix" "initializers" "nushell" "yazelix_init.nu"))

        if (
            ($output.exit_code == 0)
            and not ($aggregate | str contains "PROMPT_COMMAND_RIGHT")
            and not ($aggregate | str contains "render_right_prompt_on_last_line")
            and ($aggregate | str contains "PROMPT_COMMAND:")
        ) {
            print "  ✅ Generated Nushell initializer keeps the main Starship prompt but removes the right-prompt path"
            true
        } else {
            print $"  ❌ Unexpected generated Nushell initializer result: exit=($output.exit_code) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim)) contents=($aggregate)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_managed_nushell_config_sources_optional_user_hook [] {
    print "🧪 Testing managed Nushell config sources the optional Yazelix-owned user hook..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_nu_user_hook_XXXXXX | str trim)
    let xdg_config_home = ($tmp_home | path join ".config")
    let config_dir = ($xdg_config_home | path join "yazelix")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")

    mkdir $xdg_config_home
    mkdir ($config_dir | path join "user_configs")
    mkdir ($state_dir | path join "initializers" "nushell")

    let result = (try {
        let hook_path = (with-env {YAZELIX_CONFIG_DIR: $config_dir} {
            get_yazelix_shell_user_hook_path "nushell"
        })
        mkdir ($hook_path | path dirname)
        '$env.YAZELIX_TEST_NU_HOOK = "from_managed_nu_hook"' | save --force --raw $hook_path
        "" | save --force --raw ($state_dir | path join "initializers" "nushell" "yazelix_init.nu")
        sync_generated_yzx_extern_bridge $repo_root $state_dir | ignore
        with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_STATE_DIR: $state_dir
        } {
            sync_generated_nushell_user_hook_bridge | ignore
        }

        let output = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_DIR: $repo_root
            YAZELIX_STATE_DIR: $state_dir
        } {
            ^nu --config ($repo_root | path join "nushell" "config" "config.nu") -c 'print ($env.YAZELIX_TEST_NU_HOOK? | default "")' | complete
        })

        if ($output.exit_code == 0) and (($output.stdout | str trim) == "from_managed_nu_hook") {
            print "  ✅ Managed Nushell config can source a Yazelix-owned user hook without touching personal config"
            true
        } else {
            print $"  ❌ Unexpected managed Nushell user-hook result: exit=($output.exit_code) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_nushell_user_hook_bridge_stays_present_and_safe_when_hook_is_absent [] {
    print "🧪 Testing the managed Nushell user-hook bridge stays present and harmless when no managed Nushell hook exists..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_nu_user_hook_bridge_XXXXXX | str trim)
    let xdg_config_home = ($tmp_home | path join ".config")
    let config_dir = ($xdg_config_home | path join "yazelix")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")

    mkdir $xdg_config_home
    mkdir ($config_dir | path join "user_configs")
    mkdir ($state_dir | path join "initializers" "nushell")

    let result = (try {
        let hook_path = (with-env {YAZELIX_CONFIG_DIR: $config_dir} {
            get_yazelix_shell_user_hook_path "nushell"
        })
        "" | save --force --raw ($state_dir | path join "initializers" "nushell" "yazelix_init.nu")

        let bridge_path = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_STATE_DIR: $state_dir
        } {
            sync_generated_nushell_user_hook_bridge
        })

        let bridge_contents_after_empty_sync = if ($bridge_path | path exists) {
            open --raw $bridge_path
        } else {
            ""
        }

        mkdir ($hook_path | path dirname)
        '$env.YAZELIX_TEST_NU_HOOK = "bridge_created_after_hook_exists"' | save --force --raw $hook_path

        let bridge_contents_after_hook = (
            with-env {
                HOME: $tmp_home
                XDG_CONFIG_HOME: $xdg_config_home
                YAZELIX_CONFIG_DIR: $config_dir
                YAZELIX_STATE_DIR: $state_dir
            } {
                sync_generated_nushell_user_hook_bridge | ignore
                open --raw $bridge_path
            }
        )

        rm -f $hook_path

        let bridge_contents_after_removal = (
            with-env {
                HOME: $tmp_home
                XDG_CONFIG_HOME: $xdg_config_home
                YAZELIX_CONFIG_DIR: $config_dir
                YAZELIX_STATE_DIR: $state_dir
            } {
                sync_generated_nushell_user_hook_bridge | ignore
                open --raw $bridge_path
            }
        )

        if (
            (($bridge_contents_after_empty_sync | str trim) == "# Yazelix managed Nushell user hook bridge (empty)")
            and ($bridge_contents_after_hook | str contains 'source "')
            and (($bridge_contents_after_removal | str trim) == "# Yazelix managed Nushell user hook bridge (empty)")
        ) {
            print "  ✅ The managed Nushell bridge always exists, but becomes an empty no-op when no Yazelix-owned hook is present"
            true
        } else {
            print $"  ❌ Unexpected managed Nushell bridge lifecycle: empty=(($bridge_contents_after_empty_sync | str trim)) after_hook=(($bridge_contents_after_hook | str trim)) after_removal=(($bridge_contents_after_removal | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_managed_nushell_config_loads_generated_yzx_extern_bridge [] {
    print "🧪 Testing managed Nushell config loads the generated yzx extern bridge..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_nu_yzx_extern_XXXXXX | str trim)
    let xdg_config_home = ($tmp_home | path join ".config")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let init_dir = ($state_dir | path join "initializers" "nushell")

    mkdir $xdg_config_home
    mkdir $init_dir

    let result = (try {
        "" | save --force --raw ($init_dir | path join "yazelix_init.nu")
        "" | save --force --raw ($init_dir | path join "yazelix_user_hook.nu")

        let output = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_DIR: $repo_root
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: $state_dir
        } {
            sync_generated_yzx_extern_bridge $repo_root $state_dir | ignore
            ^nu -c $"source \"($repo_root | path join "nushell" "config" "config.nu")\"; scope commands | where name == \"yzx update runtime\" | to json -r" | complete
        })

        let stdout = ($output.stdout | default "")
        let stderr_text = ($output.stderr | default "" | str trim)
        let extern_path = (with-env {YAZELIX_STATE_DIR: $state_dir} { get_generated_yzx_extern_path $state_dir })
        let extern_contents = (open --raw $extern_path)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "\"name\":\"yzx update runtime\"")
            and ($stdout | str contains "\"type\":\"external\"")
            and ($extern_contents | str contains 'export extern "yzx update runtime"')
            and ($extern_contents | str contains '--restart(-r)')
            and ($extern_contents | str contains '--verbose')
        ) {
            print "  ✅ Managed Nushell config now loads a generated yzx extern bridge built from the real command tree"
            true
        } else {
            print $"  ❌ Unexpected managed Nushell extern bridge output: exit=($output.exit_code) stdout=($stdout) stderr=($stderr_text) extern_path=($extern_path)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_managed_bash_config_sources_optional_user_hook [] {
    print "🧪 Testing managed Bash config sources the optional Yazelix-owned user hook..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_bash_user_hook_XXXXXX | str trim)
    let xdg_config_home = ($tmp_home | path join ".config")
    let config_dir = ($xdg_config_home | path join "yazelix")

    mkdir $xdg_config_home
    mkdir ($config_dir | path join "user_configs")

    let result = (try {
        let hook_path = (with-env {YAZELIX_CONFIG_DIR: $config_dir} {
            get_yazelix_shell_user_hook_path "bash"
        })
        mkdir ($hook_path | path dirname)
        'export YAZELIX_TEST_BASH_HOOK="from_managed_bash_hook"' | save --force --raw $hook_path

        let output = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_DIR: $repo_root
            YAZELIX_HELIX_MODE: "release"
        } {
            ^bash --noprofile --norc -c $"source \"($repo_root | path join 'shells' 'bash' 'yazelix_bash_config.sh')\"; printf '%s' \"$YAZELIX_TEST_BASH_HOOK\"" | complete
        })

        if ($output.exit_code == 0) and (($output.stdout | str trim) == "from_managed_bash_hook") {
            print "  ✅ Managed Bash config can source a Yazelix-owned user hook without touching personal dotfiles"
            true
        } else {
            print $"  ❌ Unexpected managed Bash user-hook result: exit=($output.exit_code) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_yzx_import_helix_copies_personal_config_with_force_backups [] {
    print "🧪 Testing yzx import helix copies personal Helix config and backs up managed overrides on --force..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_import_helix_XXXXXX | str trim)
    let xdg_config_home = ($tmp_home | path join ".config")
    let native_helix_dir = ($xdg_config_home | path join "helix")
    let yazelix_config_dir = ($xdg_config_home | path join "yazelix")
    let managed_helix_dir = ($yazelix_config_dir | path join "user_configs" "helix")
    mkdir $native_helix_dir
    mkdir ($yazelix_config_dir | path join "user_configs")
    mkdir $managed_helix_dir

    let result = (try {
        '[editor]
cursorline = true
' | save --force --raw ($native_helix_dir | path join "config.toml")

        let import_script = ($repo_root | path join "nushell" "scripts" "core" "yazelix.nu")
        let first_import = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $yazelix_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
        } {
            ^nu -c $"use \"($import_script)\" *; yzx import helix" | complete
        })

        '[editor]
cursorline = false
' | save --force --raw ($managed_helix_dir | path join "config.toml")
        '[editor]
line-number = "relative"
' | save --force --raw ($native_helix_dir | path join "config.toml")

        let forced_import = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $yazelix_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
        } {
            ^nu -c $"use \"($import_script)\" *; yzx import helix --force" | complete
        })

        let managed_config_path = ($managed_helix_dir | path join "config.toml")
        let managed_config = (open $managed_config_path)
        let backups = (ls $managed_helix_dir | where name =~ 'config\.toml\.backup-')

        if (
            ($first_import.exit_code == 0)
            and ($forced_import.exit_code == 0)
            and (($managed_config.editor | get "line-number") == "relative")
            and (($backups | length) == 1)
        ) {
            print "  ✅ yzx import helix copies personal Helix config into user_configs/helix and backs up the previous managed file on --force"
            true
        } else {
            print $"  ❌ Unexpected helix import result: first_exit=($first_import.exit_code) force_exit=($forced_import.exit_code) managed=(($managed_config | to json -r)) backups=(($backups | length)) stderr=(($forced_import.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_generate_managed_helix_config_shows_import_notice_once [] {
    print "🧪 Testing managed Helix config generation shows a one-time import notice for personal Helix config..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_managed_helix_import_notice_XXXXXX | str trim)
    let xdg_config_home = ($tmp_home | path join ".config")
    let config_dir = ($xdg_config_home | path join "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let native_helix_dir = ($xdg_config_home | path join "helix")

    mkdir $xdg_config_home
    mkdir $config_dir
    mkdir $user_config_dir
    mkdir $native_helix_dir

    let result = (try {
        '[editor]
theme = "ayu_evolve"
' | save --force --raw ($native_helix_dir | path join "config.toml")

        let merger_script = ($repo_root | path join "nushell" "scripts" "setup" "helix_config_merger.nu")
        let first_run = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
        } {
            ^nu $merger_script --print-path | complete
        })

        let second_run = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
        } {
            ^nu $merger_script --print-path | complete
        })

        let notice_marker_path = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
        } {
            get_helix_import_notice_marker_path
        })
        let generated_config_path = ($tmp_home | path join ".local" "share" "yazelix" "configs" "helix" "config.toml")

        if (
            ($first_run.exit_code == 0)
            and ($first_run.stderr | str contains "yzx import helix")
            and ($second_run.exit_code == 0)
            and (($second_run.stderr | str trim) == "")
            and ($notice_marker_path | path exists)
            and ($generated_config_path | path exists)
        ) {
            print "  ✅ Managed Helix config generation shows the personal-config import guidance once and stays quiet after that"
            true
        } else {
            print $"  ❌ Unexpected managed Helix import-notice behavior: first=(($first_run | to json -r)) second=(($second_run | to json -r)) marker_exists=(($notice_marker_path | path exists)) generated_exists=(($generated_config_path | path exists))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

export def run_managed_config_contract_tests [] {
    [
        (test_generate_managed_helix_config_merges_user_config_and_enforces_reveal)
        (test_get_launch_env_wraps_helix_with_managed_wrapper)
        (test_generate_merged_zellij_config_wraps_nu_default_shell)
        (test_generate_merged_zellij_layouts_use_stable_zjstatus_plugin_path)
        (test_zjstatus_permission_cache_migrates_to_tracked_and_stable_paths)
        (test_generate_nushell_initializer_removes_starship_right_prompt)
        (test_managed_nushell_config_sources_optional_user_hook)
        (test_nushell_user_hook_bridge_stays_present_and_safe_when_hook_is_absent)
        (test_managed_nushell_config_loads_generated_yzx_extern_bridge)
        (test_managed_bash_config_sources_optional_user_hook)
        (test_generate_managed_helix_config_shows_import_notice_once)
        (test_yzx_import_helix_copies_personal_config_with_force_backups)
    ]
}

export def main [] {
    let results = (run_managed_config_contract_tests)
    let passed = ($results | where {|result| $result } | length)
    let total = ($results | length)

    if $passed == $total {
        print $"✅ All managed config contract tests passed \(($passed)/($total)\)"
    } else {
        error make { msg: $"Managed config contract tests failed \(($passed)/($total)\)" }
    }
}
