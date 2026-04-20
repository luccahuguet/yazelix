#!/usr/bin/env nu
# Test lane: maintainer
# Defends: docs/specs/test_suite_governance.md

use ./yzx_test_helpers.nu [get_repo_root repo_path resolve_test_yzx_core_bin]
use ../setup/zellij_config_merger.nu [generate_merged_zellij_config]
use ../utils/nushell_externs.nu [get_generated_yzx_extern_fingerprint_path get_generated_yzx_extern_path sync_generated_yzx_extern_bridge]
use ../utils/shell_user_hooks.nu [get_yazelix_shell_user_hook_path sync_generated_nushell_user_hook_bridge]

def path_is_symlink [target: string] {
    let result = (^test -L $target | complete)
    $result.exit_code == 0
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
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


# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Defends: managed Nushell config sources the Yazelix-owned user hook via YAZELIX_CONFIG_DIR instead of a legacy hook-dir env override.
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
        let hook_path = (get_yazelix_shell_user_hook_path "nushell" $config_dir)
        mkdir ($hook_path | path dirname)
        '$env.YAZELIX_TEST_NU_HOOK = "from_managed_nu_hook"' | save --force --raw $hook_path
        "" | save --force --raw ($state_dir | path join "initializers" "nushell" "yazelix_init.nu")
        sync_generated_yzx_extern_bridge $repo_root $state_dir | ignore
        with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_STATE_DIR: $state_dir
            YAZELIX_USER_SHELL_HOOK_DIR: ($tmp_home | path join "bogus_shell_hooks")
        } {
            sync_generated_nushell_user_hook_bridge | ignore
        }

        let output = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: $state_dir
        } {
            ^nu --config ($repo_root | path join "nushell" "config" "config.nu") -c 'print ($env.YAZELIX_TEST_NU_HOOK? | default "")' | complete
        })

        if ($output.exit_code == 0) and (($output.stdout | str trim) == "from_managed_nu_hook") {
            print "  ✅ Managed Nushell config now sources the Yazelix-owned user hook from YAZELIX_CONFIG_DIR, even with a bogus legacy hook-dir env set"
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

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Regression: repo-local maintainer shells must still load the managed Nushell config when runtime identity is no longer exported ambiently.
def test_managed_nushell_config_loads_in_repo_shell_without_runtime_env [] {
    print "🧪 Testing managed Nushell config still loads in repo shells without ambient YAZELIX_RUNTIME_DIR..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_nu_repo_shell_guard_XXXXXX | str trim)
    let xdg_config_home = ($tmp_home | path join ".config")
    let config_dir = ($xdg_config_home | path join "yazelix")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")

    mkdir $xdg_config_home
    mkdir ($config_dir | path join "user_configs")
    mkdir ($state_dir | path join "initializers" "nushell")

    let result = (try {
        let hook_path = (get_yazelix_shell_user_hook_path "nushell" $config_dir)
        mkdir ($hook_path | path dirname)
        '$env.YAZELIX_TEST_NU_HOOK = "from_repo_shell_guard"' | save --force --raw $hook_path
        "" | save --force --raw ($state_dir | path join "initializers" "nushell" "yazelix_init.nu")

        with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_STATE_DIR: $state_dir
            IN_YAZELIX_SHELL: "true"
        } {
            sync_generated_yzx_extern_bridge $repo_root $state_dir | ignore
            sync_generated_nushell_user_hook_bridge | ignore
        }

        let output = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_STATE_DIR: $state_dir
            IN_YAZELIX_SHELL: "true"
        } {
            ^nu --config ($repo_root | path join "nushell" "config" "config.nu") -c 'print ($env.YAZELIX_TEST_NU_HOOK? | default "")' | complete
        })

        if ($output.exit_code == 0) and (($output.stdout | str trim) == "from_repo_shell_guard") {
            print "  ✅ Managed Nushell config now loads in repo shells via IN_YAZELIX_SHELL without requiring ambient runtime-root export"
            true
        } else {
            print $"  ❌ Unexpected repo-shell managed Nushell result: exit=($output.exit_code) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Regression: yzx extern bridge sync must reuse a current generated bridge instead of reprobes on every shell startup.
def test_yzx_extern_bridge_reuses_current_fingerprint [] {
    print "🧪 Testing yzx extern bridge reuses a current fingerprinted bridge..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_yzx_extern_reuse_XXXXXX | str trim)
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")

    let result = (try {
        let extern_path = (get_generated_yzx_extern_path $state_dir)
        let fingerprint_path = (get_generated_yzx_extern_fingerprint_path $state_dir)

        sync_generated_yzx_extern_bridge $repo_root $state_dir | ignore
        let first_content = (open --raw $extern_path)
        let first_modified = (ls -D $extern_path | get 0.modified | into string)
        let fingerprint_exists = ($fingerprint_path | path exists)

        sleep 100ms
        sync_generated_yzx_extern_bridge $repo_root $state_dir | ignore

        let second_content = (open --raw $extern_path)
        let second_modified = (ls -D $extern_path | get 0.modified | into string)

        if (
            $fingerprint_exists
            and ($first_content == $second_content)
            and ($first_modified == $second_modified)
            and ($second_content | str contains 'export extern "yzx')
        ) {
            print "  ✅ Current yzx extern bridge is now reused without rewriting the generated file"
            true
        } else {
            print $"  ❌ Unexpected extern bridge reuse result: fingerprint_exists=($fingerprint_exists) first_modified=($first_modified) second_modified=($second_modified)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Regression: Rust-owned yzx extern rendering must ignore host Nushell config so Rust-owned leaf externs do not get rendered twice.
def test_yzx_extern_bridge_probe_ignores_host_nushell_config [] {
    print "🧪 Testing Rust-owned yzx extern rendering ignores host Nushell config..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_yzx_extern_host_config_XXXXXX | str trim)
    let xdg_config_home = ($tmp_home | path join ".config")
    let nushell_config_dir = ($xdg_config_home | path join "nushell")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let initializers_dir = ($state_dir | path join "initializers" "nushell")

    let result = (try {
        mkdir $xdg_config_home
        mkdir $nushell_config_dir
        mkdir $initializers_dir

        let extern_path = (get_generated_yzx_extern_path $state_dir)
        let managed_config = ($repo_root | path join "nushell" "config" "config.nu")

        [
            "export extern \"yzx env\" ["
            "    --no-shell(-n)"
            "]"
            ""
            "export extern \"yzx run\" ["
            "    ...argv: string"
            "]"
        ] | str join "\n" | save --force --raw $extern_path

        "" | save --force --raw ($initializers_dir | path join "yazelix_init.nu")
        "" | save --force --raw ($initializers_dir | path join "yazelix_user_hook.nu")
        $"source \"($extern_path)\"\n" | save --force --raw ($nushell_config_dir | path join "config.nu")

        with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_STATE_DIR: $state_dir
        } {
            sync_generated_yzx_extern_bridge $repo_root $state_dir | ignore
        }

        let generated = (open --raw $extern_path)
        let env_count = ($generated | lines | where $it == 'export extern "yzx env" [' | length)
        let run_count = ($generated | lines | where $it == 'export extern "yzx run" [' | length)
        let startup = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_STATE_DIR: $state_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            IN_YAZELIX_SHELL: "true"
        } {
            ^nu --config $managed_config -c 'print ok' | complete
        })

        if (
            ($env_count == 1)
            and ($run_count == 1)
            and ($startup.exit_code == 0)
            and (($startup.stdout | str trim) == "ok")
        ) {
            print "  ✅ Rust-owned yzx extern rendering now ignores host Nushell config and keeps Rust-owned leaf externs unique"
            true
        } else {
            print $"  ❌ Unexpected Rust-owned extern render result: env_count=($env_count) run_count=($run_count) exit=($startup.exit_code) stdout=(($startup.stdout | str trim)) stderr=(($startup.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Regression: failed yzx extern bridge regeneration must not replace a previous valid bridge with the placeholder.
def test_yzx_extern_bridge_keeps_previous_bridge_when_refresh_fails [] {
    print "🧪 Testing yzx extern bridge keeps the previous bridge when refresh fails..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_yzx_extern_failure_XXXXXX | str trim)
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let missing_runtime = ($tmp_home | path join "missing_runtime")

    let result = (try {
        let extern_path = (get_generated_yzx_extern_path $state_dir)
        let fingerprint_path = (get_generated_yzx_extern_fingerprint_path $state_dir)

        sync_generated_yzx_extern_bridge $repo_root $state_dir | ignore
        let generated_content = (open --raw $extern_path)
        mkdir $missing_runtime
        "stale fingerprint" | save --force --raw $fingerprint_path

        sync_generated_yzx_extern_bridge $missing_runtime $state_dir | ignore
        let after_failed_refresh = (open --raw $extern_path)

        if (
            ($generated_content == $after_failed_refresh)
            and ($after_failed_refresh | str contains 'export extern "yzx')
            and not ($after_failed_refresh | str contains "generated Nushell extern bridge (empty)")
        ) {
            print "  ✅ Failed yzx extern bridge refresh now keeps the previous valid generated bridge"
            true
        } else {
            print $"  ❌ Failed refresh clobbered or changed the generated bridge: ($after_failed_refresh)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Defends: managed Bash config sources the Yazelix-owned user hook from YAZELIX_CONFIG_DIR instead of a legacy hook-dir env override.
def test_managed_bash_config_sources_optional_user_hook [] {
    print "🧪 Testing managed Bash config sources the optional Yazelix-owned user hook..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_bash_user_hook_XXXXXX | str trim)
    let xdg_config_home = ($tmp_home | path join ".config")
    let config_dir = ($xdg_config_home | path join "yazelix")

    mkdir $xdg_config_home
    mkdir ($config_dir | path join "user_configs")

    let result = (try {
        let hook_path = (get_yazelix_shell_user_hook_path "bash" $config_dir)
        mkdir ($hook_path | path dirname)
        'export YAZELIX_TEST_BASH_HOOK="from_managed_bash_hook"' | save --force --raw $hook_path

        let output = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_USER_SHELL_HOOK_DIR: ($tmp_home | path join "bogus_shell_hooks")
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^env -u YAZELIX_HELIX_MODE bash --noprofile --norc -c $"source \"($repo_root | path join 'shells' 'bash' 'yazelix_bash_config.sh')\"; printf '%s|%s' \"$YAZELIX_TEST_BASH_HOOK\" \"${YAZELIX_HELIX_MODE-unset}\"" | complete
        })

        if ($output.exit_code == 0) and (($output.stdout | str trim) == "from_managed_bash_hook|unset") {
            print "  ✅ Managed Bash config now sources the Yazelix-owned user hook from YAZELIX_CONFIG_DIR without exporting Helix mode"
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

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Regression: managed Fish config stays side-effect-free and does not export Helix mode on startup.
def test_managed_fish_config_does_not_export_helix_mode_env [] {
    print "🧪 Testing managed Fish config does not export Helix mode on startup..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_fish_helix_mode_XXXXXX | str trim)
    let xdg_config_home = ($tmp_home | path join ".config")
    let config_dir = ($xdg_config_home | path join "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")

    mkdir $xdg_config_home
    mkdir $config_dir
    mkdir $user_config_dir
    '[helix]
mode = "source"
' | save --force --raw ($user_config_dir | path join "yazelix.toml")

    let result = (try {
        let fish_probe = ($tmp_home | path join "probe.fish")
        [
            $"source \"($repo_root | path join "shells" "fish" "yazelix_fish_config.fish")\""
            'if set -q YAZELIX_HELIX_MODE; printf "set"; else printf "unset"; end'
            ""
        ] | str join "\n" | save --force --raw $fish_probe

        let output = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^env -u YAZELIX_HELIX_MODE fish --no-config $fish_probe | complete
        })
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "unset") {
            print "  ✅ Managed Fish config now stays side-effect-free instead of exporting Helix mode"
            true
        } else {
            print $"  ❌ Unexpected managed Fish Helix mode result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Regression: source-checkout runtime resolution must not be shadowed by an ambient installed-runtime env.
def test_source_checkout_runtime_resolution_beats_installed_runtime [] {
    print "🧪 Testing source-checkout runtime resolution beats an ambient installed runtime env..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_runtime_resolution_XXXXXX | str trim)
    let fake_installed_runtime = ($tmp_home | path join "installed_runtime")
    let fake_state_runtime = ($tmp_home | path join ".local" "share" "yazelix" "runtime" "current")

    mkdir $fake_installed_runtime
    mkdir ($fake_state_runtime | path dirname)
    "" | save --force --raw ($fake_installed_runtime | path join "yazelix_default.toml")
    ^ln -s $fake_installed_runtime $fake_state_runtime

    let result = (try {
        let output = (with-env {
            HOME: $tmp_home
            YAZELIX_RUNTIME_DIR: $fake_state_runtime
        } {
            ^nu -c $"use \"($repo_root | path join 'nushell' 'scripts' 'utils' 'common.nu')\" [get_yazelix_runtime_dir]; print \(get_yazelix_runtime_dir\)" | complete
        })
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == $repo_root) {
            print "  ✅ Source-checkout runtime resolution now prefers the inferred repo runtime over an ambient installed runtime pointer"
            true
        } else {
            print $"  ❌ Unexpected runtime resolution result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Regression: runtime-root resolution must fail fast instead of silently falling back to the config root.
def test_runtime_resolution_fails_fast_without_valid_runtime_root [] {
    print "🧪 Testing runtime-root resolution fails fast without a valid runtime root..."

    let tmp_root = (^mktemp -d /tmp/yazelix_missing_runtime_resolution_XXXXXX | str trim)
    let fake_home = ($tmp_root | path join "home")
    let fake_config_dir = ($fake_home | path join ".config" "yazelix")
    let fake_state_dir = ($tmp_root | path join "state")
    let fake_common_path = ($tmp_root | path join "common.nu")

    mkdir ($fake_home | path join ".config")
    mkdir $fake_config_dir
    mkdir $fake_state_dir
    cp ((get_repo_root) | path join "nushell" "scripts" "utils" "common.nu") $fake_common_path

    let result = (try {
        let output = (with-env {
            HOME: $fake_home
            YAZELIX_CONFIG_DIR: $fake_config_dir
            YAZELIX_STATE_DIR: $fake_state_dir
            YAZELIX_RUNTIME_DIR: null
        } {
            ^nu -c $"use \"($fake_common_path)\" [require_yazelix_runtime_dir]; require_yazelix_runtime_dir" | complete
        })
        let detail = (([$output.stderr $output.stdout] | compact | str join "\n") | str trim)

        if (
            ($output.exit_code != 0)
            and ($detail | str contains "Could not resolve a valid Yazelix runtime root")
            and not ($detail | str contains $fake_config_dir)
        ) {
            print "  ✅ Runtime-root resolution now fails fast instead of silently treating the config root as runtime code"
            true
        } else {
            print $"  ❌ Unexpected missing-runtime result: exit=($output.exit_code) detail=($detail)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_root
    $result
}

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Regression: runtime setup must not rewrite existing host shell surfaces or recreate the legacy ~/.local/bin/yzx wrapper.
def test_runtime_setup_leaves_existing_host_shell_surfaces_untouched [] {
    print "🧪 Testing runtime setup leaves existing host shell surfaces untouched..."

    let repo_root = (get_repo_root)
    let tmp_root = (^mktemp -d /tmp/yazelix_runtime_shell_surface_guard_XXXXXX | str trim)
    let tmp_home = ($tmp_root | path join "home")
    let xdg_config_home = ($tmp_home | path join ".config")
    let config_dir = ($xdg_config_home | path join "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let log_dir = ($state_dir | path join "logs")
    let bashrc_path = ($tmp_home | path join ".bashrc")
    let nushell_host_config = ($xdg_config_home | path join "nushell" "config.nu")
    let generated_nushell_init = ($state_dir | path join "initializers" "nushell" "yazelix_init.nu")
    let local_yzx = ($tmp_home | path join ".local" "bin" "yzx")
    let runtime_nu = (which nu | get -o 0.path | default "nu")
    let bashrc_original = "# existing bashrc\nexport TEST_BASHRC=1\n"
    let nushell_original = "# existing nushell config\n$env.TEST_NU_CONFIG = \"kept\"\n"

    mkdir $tmp_home
    mkdir $xdg_config_home
    mkdir $user_config_dir
    mkdir ($nushell_host_config | path dirname)
    cp (repo_path "yazelix_default.toml") ($user_config_dir | path join "yazelix.toml")
    $bashrc_original | save --force --raw $bashrc_path
    $nushell_original | save --force --raw $nushell_host_config

    let result = (try {
        let output = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_STATE_DIR: $state_dir
            YAZELIX_LOGS_DIR: $log_dir
        } {
            ^$runtime_nu ($repo_root | path join "nushell" "scripts" "setup" "environment.nu") --skip-welcome | complete
        })
        let bashrc_contents = (open --raw $bashrc_path)
        let nushell_contents = (open --raw $nushell_host_config)

        if (
            ($output.exit_code == 0)
            and ($bashrc_contents == $bashrc_original)
            and ($nushell_contents == $nushell_original)
            and not ($local_yzx | path exists)
            and ($generated_nushell_init | path exists)
        ) {
            print "  ✅ Runtime setup now stays self-contained and leaves existing host shell files untouched"
            true
        } else {
            print $"  ❌ Unexpected runtime-setup result: exit=($output.exit_code) bashrc=($bashrc_contents) nushell=($nushell_contents) local_yzx_exists=(($local_yzx | path exists)) init_exists=(($generated_nushell_init | path exists)) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_root
    $result
}

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Regression: runtime setup must ignore read-only Home-Manager-style host shell surfaces instead of trying to own them.
def test_runtime_setup_ignores_read_only_host_shell_surfaces [] {
    print "🧪 Testing runtime setup ignores read-only Home-Manager-style host shell surfaces..."

    let repo_root = (get_repo_root)
    let tmp_root = (^mktemp -d /tmp/yazelix_runtime_read_only_shell_surfaces_XXXXXX | str trim)
    let tmp_home = ($tmp_root | path join "home")
    let xdg_config_home = ($tmp_home | path join ".config")
    let config_dir = ($xdg_config_home | path join "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let log_dir = ($state_dir | path join "logs")
    let hm_files_root = ($tmp_root | path join "fake-home-manager-files")
    let hm_bashrc_target = ($hm_files_root | path join ".bashrc")
    let hm_nushell_target = ($hm_files_root | path join ".config" "nushell" "config.nu")
    let bashrc_path = ($tmp_home | path join ".bashrc")
    let nushell_host_config = ($xdg_config_home | path join "nushell" "config.nu")
    let generated_nushell_init = ($state_dir | path join "initializers" "nushell" "yazelix_init.nu")
    let local_yzx = ($tmp_home | path join ".local" "bin" "yzx")
    let runtime_nu = (which nu | get -o 0.path | default "nu")
    let bashrc_original = "# read-only hm bashrc\nexport TEST_BASHRC=hm\n"
    let nushell_original = "# read-only hm nushell config\n$env.TEST_NU_CONFIG = \"hm\"\n"

    mkdir $tmp_home
    mkdir $xdg_config_home
    mkdir $user_config_dir
    mkdir ($hm_bashrc_target | path dirname)
    mkdir ($hm_nushell_target | path dirname)
    cp (repo_path "yazelix_default.toml") ($user_config_dir | path join "yazelix.toml")
    $bashrc_original | save --force --raw $hm_bashrc_target
    $nushell_original | save --force --raw $hm_nushell_target
    ^chmod 444 $hm_bashrc_target $hm_nushell_target
    ^ln -s $hm_bashrc_target $bashrc_path
    mkdir ($nushell_host_config | path dirname)
    ^ln -s $hm_nushell_target $nushell_host_config

    let result = (try {
        let output = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_STATE_DIR: $state_dir
            YAZELIX_LOGS_DIR: $log_dir
        } {
            ^$runtime_nu ($repo_root | path join "nushell" "scripts" "setup" "environment.nu") --skip-welcome | complete
        })
        let bashrc_contents = (open --raw $bashrc_path)
        let nushell_contents = (open --raw $nushell_host_config)

        if (
            ($output.exit_code == 0)
            and (path_is_symlink $bashrc_path)
            and (path_is_symlink $nushell_host_config)
            and ($bashrc_contents == $bashrc_original)
            and ($nushell_contents == $nushell_original)
            and not ($local_yzx | path exists)
            and ($generated_nushell_init | path exists)
        ) {
            print "  ✅ Runtime setup now ignores read-only Home Manager shell surfaces and keeps host symlinks untouched"
            true
        } else {
            print $"  ❌ Unexpected read-only runtime-setup result: exit=($output.exit_code) bashrc_symlink=((path_is_symlink $bashrc_path)) nushell_symlink=((path_is_symlink $nushell_host_config)) bashrc=($bashrc_contents) nushell=($nushell_contents) local_yzx_exists=(($local_yzx | path exists)) init_exists=(($generated_nushell_init | path exists)) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_root
    $result
}

export def run_shell_managed_config_contract_tests [] {
    with-env {
        YAZELIX_YZX_CORE_BIN: (resolve_test_yzx_core_bin)
    } {
        [
            (test_generate_merged_zellij_config_wraps_nu_default_shell)
            (test_managed_nushell_config_sources_optional_user_hook)
            (test_managed_nushell_config_loads_in_repo_shell_without_runtime_env)
            (test_yzx_extern_bridge_reuses_current_fingerprint)
            (test_yzx_extern_bridge_probe_ignores_host_nushell_config)
            (test_yzx_extern_bridge_keeps_previous_bridge_when_refresh_fails)
            (test_managed_bash_config_sources_optional_user_hook)
            (test_managed_fish_config_does_not_export_helix_mode_env)
            (test_source_checkout_runtime_resolution_beats_installed_runtime)
            (test_runtime_resolution_fails_fast_without_valid_runtime_root)
            (test_runtime_setup_leaves_existing_host_shell_surfaces_untouched)
            (test_runtime_setup_ignores_read_only_host_shell_surfaces)
        ]
    }
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
