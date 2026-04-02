#!/usr/bin/env nu
# Test lane: maintainer
# Defends: docs/specs/test_suite_governance.md

use ./yzx_test_helpers.nu [get_repo_root repo_path]
use ../setup/zellij_config_merger.nu [generate_merged_zellij_config]
use ../utils/nushell_externs.nu [get_generated_yzx_extern_path sync_generated_yzx_extern_bridge]
use ../utils/shell_user_hooks.nu [get_yazelix_shell_user_hook_path sync_generated_nushell_user_hook_bridge]

# Strength: 7/10
# Defends: merged Zellij config routes managed Nushell panes through the Yazelix wrapper.
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


# Strength: 6/10
# Regression: generated Nushell initializer keeps Starship while removing the right-prompt path.
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

# Strength: 7/10
# Defends: managed Nushell config sources the optional Yazelix-owned user hook without touching personal config.
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

# Strength: 6/10
# Invariant: the managed Nushell user-hook bridge stays present and harmless when no hook exists.
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

# Strength: 7/10
# Defends: managed Nushell config loads the generated yzx extern bridge built from the real command tree.
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

# Strength: 7/10
# Defends: managed Bash config sources the optional Yazelix-owned user hook without touching personal dotfiles.
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

export def run_shell_managed_config_contract_tests [] {
    [
        (test_generate_merged_zellij_config_wraps_nu_default_shell)
        (test_generate_nushell_initializer_removes_starship_right_prompt)
        (test_managed_nushell_config_sources_optional_user_hook)
        (test_nushell_user_hook_bridge_stays_present_and_safe_when_hook_is_absent)
        (test_managed_nushell_config_loads_generated_yzx_extern_bridge)
        (test_managed_bash_config_sources_optional_user_hook)
    ]
}

export def main [] {
    let results = (run_shell_managed_config_contract_tests)
    let passed = ($results | where {|result| $result } | length)
    let total = ($results | length)

    if $passed == $total {
        print $"✅ All shell managed config contract tests passed \(($passed)/($total)\)"
    } else {
        error make { msg: $"Shell managed config contract tests failed \(($passed)/($total)\)" }
    }
}
