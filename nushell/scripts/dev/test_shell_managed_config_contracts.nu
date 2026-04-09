#!/usr/bin/env nu
# Test lane: maintainer
# Defends: docs/specs/test_suite_governance.md

use ./yzx_test_helpers.nu [get_repo_root repo_path]
use ../setup/zellij_config_merger.nu [generate_merged_zellij_config]
use ../setup/shell_hooks.nu [setup_shell_hooks]
use ../utils/nushell_externs.nu [get_generated_yzx_extern_path sync_generated_yzx_extern_bridge]
use ../utils/shell_config_generation.nu get_yazelix_section_content
use ../utils/shell_user_hooks.nu [get_yazelix_shell_user_hook_path sync_generated_nushell_user_hook_bridge]

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
# Regression: repo-local devenv shells must still load the managed Nushell config when runtime identity is no longer exported ambiently.
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

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
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

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
# Regression: generated Nushell shell hooks must not pin yzx to a runtime-store import after runtime updates.
def test_generated_nushell_shell_hook_uses_managed_config_only [] {
    print "🧪 Testing generated Nushell shell hooks source the managed config without importing a runtime-pinned yzx command..."

    let repo_root = (get_repo_root)

    let result = (try {
        let section = (with-env {
            HOME: "/tmp"
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            get_yazelix_section_content "nushell" $repo_root
        })

        if (
            ($section | str contains 'source "')
            and ($section | str contains 'nushell/config/config.nu')
            and not ($section | str contains 'scripts/core/yazelix.nu')
            and not ($section | str contains 'use ')
        ) {
            print "  ✅ Generated Nushell shell hooks now rely on the managed config and extern bridge instead of importing a store-pinned yzx command"
            true
        } else {
            print $"  ❌ Unexpected generated Nushell shell-hook section: ($section)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

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
    let fake_state_runtime = ($tmp_home | path join ".local" "share" "yazelix" "runtime" "current")

    mkdir ($fake_state_runtime | path dirname)
    ^ln -s $repo_root $fake_state_runtime

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

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Regression: installed-runtime modules must not treat a Nix -source mirror as the active runtime when runtime/current exists.
def test_installed_runtime_resolution_prefers_runtime_current_over_nix_source_mirror [] {
    print "🧪 Testing installed-runtime resolution prefers runtime/current over a Nix -source mirror..."

    let repo_root = (get_repo_root)
    let tmp_root = (^mktemp -d /tmp/yazelix_installed_runtime_resolution_XXXXXX | str trim)
    let fake_state_dir = ($tmp_root | path join "state")
    let fake_installed_runtime = ($tmp_root | path join "installed_runtime")
    let fake_source_root = ($tmp_root | path join "fake-runtime-source")
    let fake_common_path = ($fake_source_root | path join "nushell" "scripts" "utils" "common.nu")

    mkdir ($fake_state_dir | path join "runtime")
    mkdir ($fake_installed_runtime | path join "nushell")
    mkdir ($fake_source_root | path join "nushell" "scripts" "utils")
    cp ($repo_root | path join ".taplo.toml") ($fake_installed_runtime | path join ".taplo.toml")
    "" | save --force --raw ($fake_installed_runtime | path join "yazelix_default.toml")
    ^ln -s $fake_installed_runtime ($fake_state_dir | path join "runtime" "current")
    cp ($repo_root | path join "nushell" "scripts" "utils" "common.nu") $fake_common_path

    let result = (try {
        let output = (with-env {
            YAZELIX_STATE_DIR: $fake_state_dir
        } {
            ^nu -c $"use \"($fake_common_path)\" [get_yazelix_runtime_dir]; print \(get_yazelix_runtime_dir\)" | complete
        })
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == $fake_installed_runtime) {
            print "  ✅ Installed-runtime resolution now prefers runtime/current over a Nix -source mirror"
            true
        } else {
            print $"  ❌ Unexpected installed-runtime resolution result: exit=($output.exit_code) stdout=($stdout) expected=($fake_installed_runtime) stderr=(($output.stderr | str trim))"
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
        } {
            ^nu -c $"use \"($fake_common_path)\" [require_yazelix_runtime_dir]; require_yazelix_runtime_dir" | complete
        })
        let stderr = ($output.stderr | str trim)

        if (
            ($output.exit_code != 0)
            and ($stderr | str contains "Could not resolve a valid Yazelix runtime root")
            and not ($stderr | str contains $fake_config_dir)
        ) {
            print "  ✅ Runtime-root resolution now fails fast instead of silently treating the config root as runtime code"
            true
        } else {
            print $"  ❌ Unexpected missing-runtime result: exit=($output.exit_code) stderr=($stderr)"
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
# Regression: Home Manager shellhook setup must not require host dotfiles or recreate the legacy ~/.local/bin/yzx wrapper.
def test_home_manager_shellhook_setup_skips_host_shell_surfaces [] {
    print "🧪 Testing Home Manager shellhook setup skips host dotfiles and the legacy yzx wrapper..."

    let repo_root = (get_repo_root)
    let tmp_root = (^mktemp -d /tmp/yazelix_hm_shellhook_surfaces_XXXXXX | str trim)
    let tmp_home = ($tmp_root | path join "home")
    let xdg_config_home = ($tmp_home | path join ".config")
    let config_dir = ($xdg_config_home | path join "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let log_dir = ($state_dir | path join "logs")
    let runtime_dir = ($tmp_root | path join "runtime")
    let runtime_bin_dir = ($runtime_dir | path join "bin")
    let hm_marker = ($tmp_home | path join ".nix-profile" "etc" "profile.d" "hm-session-vars.sh")
    let bashrc_path = ($tmp_home | path join ".bashrc")
    let nushell_host_config = ($xdg_config_home | path join "nushell" "config.nu")
    let local_yzx = ($tmp_home | path join ".local" "bin" "yzx")
    let runtime_nu = (which nu | get -o 0.path | default "nu")

    mkdir $xdg_config_home
    mkdir $user_config_dir
    mkdir $state_dir
    mkdir $log_dir
    mkdir ($hm_marker | path dirname)
    mkdir $runtime_dir
    mkdir $runtime_bin_dir

    for entry in [".taplo.toml", "assets", "config_metadata", "configs", "nushell", "shells", "yazelix_default.toml", "yazelix_packs_default.toml"] {
        ^ln -s (repo_path $entry) ($runtime_dir | path join $entry)
    }
    ^ln -s $runtime_nu ($runtime_bin_dir | path join "nu")
    cp (repo_path "yazelix_default.toml") ($user_config_dir | path join "yazelix.toml")
    cp (repo_path "yazelix_packs_default.toml") ($user_config_dir | path join "yazelix_packs.toml")
    "" | save --force --raw $hm_marker

    let result = (try {
        let output = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_RUNTIME_DIR: $runtime_dir
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_STATE_DIR: $state_dir
            YAZELIX_LOGS_DIR: $log_dir
        } {
            ^$runtime_nu ($runtime_dir | path join "nushell" "scripts" "setup" "environment.nu") --skip-welcome | complete
        })

        let generated_nushell_init = ($state_dir | path join "initializers" "nushell" "yazelix_init.nu")

        if (
            ($output.exit_code == 0)
            and not ($bashrc_path | path exists)
            and not ($nushell_host_config | path exists)
            and not ($local_yzx | path exists)
            and ($generated_nushell_init | path exists)
        ) {
            print "  ✅ Home Manager shellhook setup now stays self-contained instead of requiring host dotfiles or recreating ~/.local/bin/yzx"
            true
        } else {
            print $"  ❌ Unexpected Home Manager shellhook result: exit=($output.exit_code) bashrc_exists=(($bashrc_path | path exists)) nushell_config_exists=(($nushell_host_config | path exists)) local_yzx_exists=(($local_yzx | path exists)) init_exists=(($generated_nushell_init | path exists)) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim))"
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
# Regression: legacy pre-v4 shell hook blocks are rejected with one explicit manual recovery path instead of being duplicated or auto-migrated silently.
def test_setup_shell_hooks_rejects_legacy_generations [] {
    print "🧪 Testing shell hook setup rejects legacy pre-v4 managed sections with manual recovery guidance..."

    let repo_root = (get_repo_root)
    let legacy_cases = [
        {generation: "v1", marker: "# YAZELIX START - Yazelix managed configuration (do not modify this comment)", end_marker: "# YAZELIX END - Yazelix managed configuration (do not modify this comment)"}
        {generation: "v2", marker: "# YAZELIX START v2 - Yazelix managed configuration (do not modify this comment)", end_marker: "# YAZELIX END v2 - Yazelix managed configuration (do not modify this comment)"}
        {generation: "v3", marker: "# YAZELIX START v3 - Yazelix managed configuration (do not modify this comment)", end_marker: "# YAZELIX END v3 - Yazelix managed configuration (do not modify this comment)"}
    ]

    let results = ($legacy_cases | each {|legacy|
        let tmp_root = (^mktemp -d /tmp/yazelix_legacy_shell_hook_XXXXXX | str trim)
        let tmp_home = ($tmp_root | path join "home")
        let bashrc_path = ($tmp_home | path join ".bashrc")
        let state_dir = ($tmp_home | path join ".local" "share" "yazelix")

        mkdir $tmp_home
        mkdir ($tmp_home | path join ".local" "share")
        mkdir $state_dir

        [
            $legacy.marker
            "# old generated shell hook body"
            $legacy.end_marker
        ] | str join "\n" | save --force --raw $bashrc_path

        let outcome = (with-env {
            HOME: $tmp_home
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: $state_dir
        } {
            try {
                setup_shell_hooks "bash" $repo_root true true
                {ok: false, message: "setup_shell_hooks succeeded unexpectedly"}
            } catch {|err|
                {ok: true, message: ($err.msg | default "")}
            }
        })

        let bashrc_contents = (open --raw $bashrc_path)
        let passed = (
            $outcome.ok
            and ($outcome.message | str contains $"no longer auto-migrates ($legacy.generation)")
            and ($outcome.message | str contains "delete the old Yazelix-managed section")
            and ($outcome.message | str contains "yzx refresh")
            and ($bashrc_contents | str contains $legacy.marker)
            and not ($bashrc_contents | str contains "# YAZELIX START v4")
        )

        if not $passed {
            print $"  ❌ Unexpected legacy shell-hook result for ($legacy.generation): message=($outcome.message) contents=($bashrc_contents)"
        }

        rm -rf $tmp_root
        $passed
    })

    if ($results | all {|result| $result}) {
        print "  ✅ Legacy pre-v4 shell hook generations now fail fast with one explicit cleanup path"
        true
    } else {
        false
    }
}

export def run_shell_managed_config_contract_tests [] {
    [
        (test_generate_merged_zellij_config_wraps_nu_default_shell)
        (test_managed_nushell_config_sources_optional_user_hook)
        (test_managed_nushell_config_loads_in_repo_shell_without_runtime_env)
        (test_managed_nushell_config_loads_generated_yzx_extern_bridge)
        (test_generated_nushell_shell_hook_uses_managed_config_only)
        (test_managed_bash_config_sources_optional_user_hook)
        (test_managed_fish_config_does_not_export_helix_mode_env)
        (test_source_checkout_runtime_resolution_beats_installed_runtime)
        (test_installed_runtime_resolution_prefers_runtime_current_over_nix_source_mirror)
        (test_runtime_resolution_fails_fast_without_valid_runtime_root)
        (test_home_manager_shellhook_setup_skips_host_shell_surfaces)
        (test_setup_shell_hooks_rejects_legacy_generations)
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
