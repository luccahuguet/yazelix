#!/usr/bin/env nu

use ../core/yazelix.nu *
use ./test_yzx_helpers.nu [get_repo_config_dir repo_path]
use ../utils/shell_config_generation.nu [get_yazelix_section_content]
use ../utils/config_manager.nu [check_config_versions]

def setup_relocated_runtime_fixture [] {
    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_relocated_runtime_XXXXXX | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    let config_dir = ($tmp_home | path join ".config" "yazelix")

    mkdir $runtime_dir
    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir

    for entry in ["nushell", "shells", "configs", "devenv.lock", "yazelix_default.toml"] {
        ^ln -s (repo_path $entry) ($runtime_dir | path join $entry)
    }

    cp (repo_path "yazelix_default.toml") ($config_dir | path join "yazelix.toml")

    {
        repo_root: $repo_root
        tmp_home: $tmp_home
        runtime_dir: $runtime_dir
        config_dir: $config_dir
        yzx_script: ($runtime_dir | path join "nushell" "scripts" "core" "yazelix.nu")
        startup_script: ($runtime_dir | path join "shells" "posix" "start_yazelix.sh")
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

def test_yzx_status_versions_uses_invoking_path_for_versions [] {
    print "🧪 Testing yzx status --versions resolves tool versions from the invoking PATH..."

    let fixture = (setup_relocated_runtime_fixture)

    let result = (try {
        let fake_bin = ($fixture.tmp_home | path join "bin")
        mkdir $fake_bin
        let nu_bin = (which nu | get 0.path)

        '#!/bin/sh
echo "zellij 9.9.9"
' | save --force --raw ($fake_bin | path join "zellij")
        ^chmod +x ($fake_bin | path join "zellij")
        ^ln -s $nu_bin ($fake_bin | path join "nu")

        let env_overlay = {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            PATH: ([$fake_bin] | append $env.PATH)
        }

        let output = with-env $env_overlay {
            do {
                cd $fixture.runtime_dir
                ^nu -c $"use \"($fixture.yzx_script)\" *; yzx status --versions" | complete
            }
        }
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Yazelix Tool Versions") and ($stdout | str contains "9.9.9") {
            print "  ✅ yzx status --versions uses the invoking PATH for version resolution"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

def test_yzx_config_view [] {
    print "🧪 Testing yzx config..."

    try {
        let yzx_script = (repo_path "nushell" "scripts" "core" "yazelix.nu")
        let output = (
            ^nu -c $"use \"($yzx_script)\" *; yzx config | columns | str join ','" | complete
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
        let yzx_script = (repo_path "nushell" "scripts" "core" "yazelix.nu")
        let yazi_merger = (repo_path "nushell" "scripts" "setup" "yazi_config_merger.nu")
        let zellij_merger = (repo_path "nushell" "scripts" "setup" "zellij_config_merger.nu")
        let root = (get_repo_config_dir)
        ^nu -c $"use \"($yazi_merger)\" *; generate_merged_yazi_config \"($root)\" --quiet | ignore" | complete | ignore
        let hx_output = (^nu -c $"use \"($yzx_script)\" *; yzx config hx | columns | str join ','" | complete).stdout | str trim
        let yazi_output = (^nu -c $"use \"($yzx_script)\" *; yzx config yazi | columns | str join ','" | complete).stdout | str trim
        if (which zellij | is-empty) {
            if ($hx_output | str contains "config_path") and ($yazi_output | str contains "manager") {
                print "  ℹ️  Skipping zellij config section check because zellij is not available"
                print "  ✅ yzx config section commands return focused sections"
                return true
            }
        }

        ^nu -c $"use \"($zellij_merger)\" *; generate_merged_zellij_config \"($root)\" | ignore" | complete | ignore
        let zellij_output = (^nu -c $"use \"($yzx_script)\" *; yzx config zellij" | complete).stdout | str trim

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

def test_yzx_config_reset_replaces_with_backup [] {
    print "🧪 Testing yzx config reset replaces the config with backup..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_config_reset_XXXXXX | str trim)
    let temp_yazelix_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir $temp_yazelix_dir

    let result = (try {
        ^ln -s ($repo_root | path join "nushell") ($temp_yazelix_dir | path join "nushell")
        cp ($repo_root | path join "yazelix_default.toml") ($temp_yazelix_dir | path join "yazelix_default.toml")
        '[shell]
default_shell = "bash"
' | save --force --raw ($temp_yazelix_dir | path join "yazelix.toml")

        let temp_yzx_script = ($temp_yazelix_dir | path join "nushell" "scripts" "core" "yazelix.nu")
        let output = with-env { HOME: $tmp_home, YAZELIX_DIR: $temp_yazelix_dir } {
            ^nu -c $"use \"($temp_yzx_script)\" *; yzx config reset --yes" | complete
        }
        let stdout = ($output.stdout | str trim)
        let new_config = (open --raw ($temp_yazelix_dir | path join "yazelix.toml"))
        let default_config = (open --raw ($temp_yazelix_dir | path join "yazelix_default.toml"))
        let backups = (
            ls $temp_yazelix_dir
            | where name =~ 'yazelix\.toml\.backup-'
        )
        let backup_content = if ($backups | is-empty) { "" } else { open --raw (($backups | first).name) }

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Backed up previous config")
            and ($stdout | str contains "Replaced yazelix.toml with a fresh template")
            and ($new_config == $default_config)
            and ($backup_content | str contains 'default_shell = "bash"')
        ) {
            print "  ✅ yzx config reset backs up the current config and restores the template"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) backups=(($backups | length))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_invalid_config_is_classified_as_config_problem [] {
    print "🧪 Testing invalid config values are classified as config problems..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_invalid_config_XXXXXX | str trim)
    let temp_yazelix_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir $temp_yazelix_dir

    let result = (try {
        ^ln -s ($repo_root | path join "nushell") ($temp_yazelix_dir | path join "nushell")
        cp ($repo_root | path join "yazelix_default.toml") ($temp_yazelix_dir | path join "yazelix_default.toml")

        let invalid_config = (
            open ($repo_root | path join "yazelix_default.toml")
            | upsert core.refresh_output "loud"
        )
        $invalid_config | to toml | save ($temp_yazelix_dir | path join "yazelix.toml")

        let parser_script = ($temp_yazelix_dir | path join "nushell" "scripts" "utils" "config_parser.nu")
        let output = with-env { HOME: $tmp_home, YAZELIX_DIR: $temp_yazelix_dir } {
            ^nu -c $"source \"($parser_script)\"; try { parse_yazelix_config | ignore } catch {|err| print $err.msg }" | complete
        }
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Invalid core.refresh_output value")
            and ($stdout | str contains "Failure class: config problem.")
            and ($stdout | str contains "yzx config reset --yes")
        ) {
            print "  ✅ Invalid config values are classified as config problems"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_config_state_supports_split_config_and_runtime_dirs [] {
    print "🧪 Testing config state supports split config and runtime directories..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_split_roots_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir ($tmp_home | path join ".config")
    mkdir $temp_config_dir

    let result = (try {
        cp ($repo_root | path join "yazelix_default.toml") ($temp_config_dir | path join "yazelix.toml")
        let state_script = ($repo_root | path join "nushell" "scripts" "utils" "config_state.nu")
        let snippet = ([
            $"source \"($state_script)\""
            'let state = (compute_config_state)'
            'print ({'
            '    config_file: $state.config_file'
            '    lock_hash_empty: (($state.lock_hash | default "") | is-empty)'
            '    runtime_lock_path: ($env.YAZELIX_RUNTIME_DIR | path join "devenv.lock")'
            '} | to json -r)'
        ] | str join "\n")
        let output = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $snippet | complete
        }
        let stdout = ($output.stdout | str trim)
        let resolved = ($stdout | lines | last | from json)

        if (
            ($output.exit_code == 0)
            and ($resolved.config_file == ($temp_config_dir | path join "yazelix.toml"))
            and ($resolved.lock_hash_empty == false)
            and (($resolved.runtime_lock_path | path exists))
        ) {
            print "  ✅ Config state reads config from the config dir and hashes inputs from the runtime dir"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_config_reset_supports_split_config_and_runtime_dirs [] {
    print "🧪 Testing yzx config reset supports split config and runtime directories..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_split_reset_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir ($tmp_home | path join ".config")
    mkdir $temp_config_dir

    let result = (try {
        '[shell]
default_shell = "bash"
' | save --force --raw ($temp_config_dir | path join "yazelix.toml")

        let yzx_script = ($repo_root | path join "nushell" "scripts" "core" "yazelix.nu")
        let output = with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $"use \"($yzx_script)\" *; yzx config reset --yes" | complete
        }
        let stdout = ($output.stdout | str trim)
        let new_config = (open --raw ($temp_config_dir | path join "yazelix.toml"))
        let default_config = (open --raw ($repo_root | path join "yazelix_default.toml"))

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Replaced yazelix.toml with a fresh template")
            and ($new_config == $default_config)
        ) {
            print "  ✅ yzx config reset reads the template from the runtime root and writes to the config root"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_shell_hook_generation_uses_runtime_root [] {
    print "🧪 Testing shell hook generation uses the configured runtime root..."

    let runtime_dir = "/tmp/yazelix-runtime"

    let result = (try {
        let bash_section = (get_yazelix_section_content "bash" $runtime_dir)
        let nu_section = (get_yazelix_section_content "nushell" $runtime_dir)

        if (
            ($bash_section | str contains $"source \"($runtime_dir)/shells/bash/yazelix_bash_config.sh\"")
            and ($bash_section | str contains $"use ($runtime_dir)/nushell/scripts/core/yazelix.nu *; yzx $*")
            and ($nu_section | str contains $"source \"($runtime_dir)/nushell/config/config.nu\"")
            and ($nu_section | str contains $"use ($runtime_dir)/nushell/scripts/core/yazelix.nu *")
            and not ($bash_section | str contains "~/.config/yazelix")
            and not ($nu_section | str contains "~/.config/yazelix")
        ) {
            print "  ✅ Shell hook generation resolves sourced Yazelix files from the runtime root"
            true
        } else {
            print "  ❌ Shell hook generation still contains stale repo-shaped runtime paths"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    $result
}

def test_shell_hook_version_check_accepts_runtime_root_hooks [] {
    print "🧪 Testing shell hook version checks accept relocated runtime paths..."

    let tmp_home = (^mktemp -d /tmp/yazelix_shell_hooks_XXXXXX | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    mkdir $runtime_dir

    let result = (try {
        let bashrc = ($tmp_home | path join ".bashrc")
        let config_nu = ($tmp_home | path join ".config" "nushell" "config.nu")
        mkdir ($tmp_home | path join ".config")
        mkdir ($tmp_home | path join ".config" "nushell")

        (get_yazelix_section_content "bash" $runtime_dir) | save --force --raw $bashrc
        (get_yazelix_section_content "nushell" $runtime_dir) | save --force --raw $config_nu

        let statuses = (with-env { HOME: $tmp_home } { check_config_versions $runtime_dir })
        let bash_status = ($statuses | where shell == "bash" | first)
        let nu_status = ($statuses | where shell == "nushell" | first)

        if ($bash_status.status == "current") and ($nu_status.status == "current") {
            print "  ✅ Shell hook version checks treat relocated runtime hooks as current"
            true
        } else {
            print $"  ❌ Unexpected statuses: bash=($bash_status.status) nushell=($nu_status.status)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_bash_runtime_config_uses_its_own_runtime_root [] {
    print "🧪 Testing bash runtime config derives its own Yazelix runtime root..."

    let tmp_home = (^mktemp -d /tmp/yazelix_bash_runtime_XXXXXX | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    let bash_runtime_dir = ($runtime_dir | path join "shells" "bash")
    mkdir $bash_runtime_dir

    let result = (try {
        cp (repo_path "shells" "bash" "yazelix_bash_config.sh") ($bash_runtime_dir | path join "yazelix_bash_config.sh")
        mkdir ($tmp_home | path join ".local" "share" "yazelix" "initializers" "bash")

        let output = with-env { HOME: $tmp_home, YAZELIX_HELIX_MODE: "release" } {
            ^bash -lc $"source \"($bash_runtime_dir | path join "yazelix_bash_config.sh")\"; alias yazelix" | complete
        }
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains $"alias yazelix='nu ($runtime_dir)/nushell/scripts/core/launch_yazelix.nu'") {
            print "  ✅ Bash runtime config routes the yazelix alias through its own runtime root"
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

def test_runtime_shell_assets_avoid_repo_shaped_runtime_paths [] {
    print "🧪 Testing runtime shell assets avoid repo-shaped internal script paths..."

    try {
        let bash_config = (open --raw (repo_path "shells" "bash" "yazelix_bash_config.sh"))
        let fish_config = (open --raw (repo_path "shells" "fish" "yazelix_fish_config.fish"))
        let zsh_config = (open --raw (repo_path "shells" "zsh" "yazelix_zsh_config.zsh"))

        if (
            not ($bash_config | str contains "~/.config/yazelix/nushell/scripts")
            and not ($fish_config | str contains "~/.config/yazelix/nushell/scripts")
            and not ($zsh_config | str contains "~/.config/yazelix/nushell/scripts")
            and ($bash_config | str contains 'YAZELIX_RUNTIME_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"')
            and ($fish_config | str contains 'set -l YAZELIX_RUNTIME_DIR')
            and ($zsh_config | str contains 'YAZELIX_RUNTIME_DIR="$(cd "$(dirname "${(%):-%N}")/../.." && pwd)"')
        ) {
            print "  ✅ Runtime shell assets derive internal script paths from their own runtime root"
            true
        } else {
            print "  ❌ One or more runtime shell assets still hardcode repo-shaped internal script paths"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_pane_orchestrator_tracked_path_defaults_to_runtime_root [] {
    print "🧪 Testing pane orchestrator tracked path defaults to the runtime root..."

    let runtime_dir = "/tmp/yazelix-runtime"

    try {
        let helper_script = (repo_path "nushell" "scripts" "setup" "zellij_plugin_paths.nu")
        let output = with-env { YAZELIX_RUNTIME_DIR: $runtime_dir } {
            ^nu -c $"source \"($helper_script)\"; get_tracked_pane_orchestrator_wasm_path" | complete
        }
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == ($runtime_dir | path join "configs" "zellij" "plugins" "yazelix_pane_orchestrator.wasm")) {
            print "  ✅ Pane orchestrator helpers default to the configured runtime root"
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

def test_pane_orchestrator_permission_cache_is_preserved_for_stable_runtime_path [] {
    print "🧪 Testing pane orchestrator sync preserves granted permissions for the stable runtime path..."

    let tmp_home = (^mktemp -d /tmp/yazelix_plugin_permissions_XXXXXX | str trim)
    let tracked_path = ($tmp_home | path join ".config" "yazelix" "configs" "zellij" "plugins" "yazelix_pane_orchestrator.wasm")
    let runtime_path = ($tmp_home | path join ".local" "share" "yazelix" "configs" "zellij" "plugins" "yazelix_pane_orchestrator.wasm")
    let permissions_dir = ($tmp_home | path join ".cache" "zellij")
    let permissions_path = ($permissions_dir | path join "permissions.kdl")

    let result = (try {
        mkdir ($tracked_path | path dirname)
        mkdir ($runtime_path | path dirname)
        mkdir $permissions_dir

        let existing_block = [
            $"\"($tmp_home | path join ".local" "share" "yazelix" "configs" "zellij" "plugins" "yazelix_pane_orchestrator_deadbeef1234.wasm")\" {"
            "    ReadApplicationState"
            "    OpenTerminalsOrPlugins"
            "    ChangeApplicationState"
            "    WriteToStdin"
            "    ReadCliPipes"
            "}"
        ] | str join "\n"
        $existing_block | save --force --raw $permissions_path

        let helper_script = (repo_path "nushell" "scripts" "setup" "zellij_plugin_paths.nu")
        let snippet = ([
            $"source '($helper_script)'"
            ("let result = (preserve_pane_orchestrator_permissions '"
                + $tracked_path
                + "' '"
                + $runtime_path
                + "')")
            "print ($result | to json -r)"
        ] | str join "\n")
        let output = with-env { HOME: $tmp_home } {
            ^nu -c $snippet | complete
        }
        let stdout = ($output.stdout | str trim)
        let cache_contents = (open --raw $permissions_path)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains '"status":"updated"')
            and ($cache_contents | str contains $tracked_path)
            and ($cache_contents | str contains $runtime_path)
            and (($cache_contents | str contains "OpenTerminalsOrPlugins"))
            and (($cache_contents | str contains "WriteToStdin"))
        ) {
            print "  ✅ Granted pane-orchestrator permissions are preserved onto the stable runtime path"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim) cache=($cache_contents)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_popup_runner_tracked_path_defaults_to_runtime_root [] {
    print "🧪 Testing popup runner tracked path defaults to the runtime root..."

    let tmp_home = (^mktemp -d /tmp/yazelix_popup_runner_paths_XXXXXX | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    mkdir ($runtime_dir | path join "configs" "zellij" "plugins")

    let result = (try {
        let helper_script = (repo_path "nushell" "scripts" "setup" "zellij_plugin_paths.nu")
        let output = with-env { YAZELIX_RUNTIME_DIR: $runtime_dir } {
            ^nu -c $"source \"($helper_script)\"; get_tracked_popup_runner_wasm_path" | complete
        }
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == ($runtime_dir | path join "configs" "zellij" "plugins" "yazelix_popup_runner.wasm")) {
            print "  ✅ Popup runner helpers default to the configured runtime root"
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

def test_popup_runner_permission_cache_is_preserved_for_stable_runtime_path [] {
    print "🧪 Testing popup runner sync preserves granted permissions for the stable runtime path..."

    let tmp_home = (^mktemp -d /tmp/yazelix_popup_permissions_XXXXXX | str trim)
    let tracked_path = ($tmp_home | path join ".config" "yazelix" "configs" "zellij" "plugins" "yazelix_popup_runner.wasm")
    let runtime_path = ($tmp_home | path join ".local" "share" "yazelix" "configs" "zellij" "plugins" "yazelix_popup_runner.wasm")
    let permissions_dir = ($tmp_home | path join ".cache" "zellij")
    let permissions_path = ($permissions_dir | path join "permissions.kdl")

    let result = (try {
        mkdir ($tracked_path | path dirname)
        mkdir ($runtime_path | path dirname)
        mkdir $permissions_dir

        let existing_block = [
            $"\"($tmp_home | path join ".local" "share" "yazelix" "configs" "zellij" "plugins" "yazelix_popup_runner_deadbeef1234.wasm")\" {"
            "    ReadApplicationState"
            "    ChangeApplicationState"
            "    ReadCliPipes"
            "}"
        ] | str join "\n"
        $existing_block | save --force --raw $permissions_path

        let helper_script = (repo_path "nushell" "scripts" "setup" "zellij_plugin_paths.nu")
        let snippet = ([
            $"source '($helper_script)'"
            ("let result = (sync_popup_runner_runtime_wasm '" + ($tmp_home | path join ".config" "yazelix") + "')")
            "print $result"
        ] | str join "\n")

        # materialize tracked file so sync can copy it
        "popup" | save --force --raw $tracked_path

        let output = with-env { HOME: $tmp_home } {
            ^nu -c $snippet | complete
        }
        let stdout = ($output.stdout | str trim)
        let cache_contents = (open --raw $permissions_path)

        if (
            ($output.exit_code == 0)
            and ($stdout == $runtime_path)
            and ($cache_contents | str contains $tracked_path)
            and ($cache_contents | str contains $runtime_path)
            and ($cache_contents | str contains "ReadCliPipes")
            and not ($cache_contents | str contains "OpenTerminalsOrPlugins")
            and not ($cache_contents | str contains "RunCommands")
        ) {
            print "  ✅ Granted popup-runner permissions are preserved onto the stable runtime path"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim) cache=($cache_contents)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_packs_helper_uses_runtime_root_for_devenv_links [] {
    print "🧪 Testing yzx packs helper reads .devenv links from the runtime root..."

    let tmp_home = (^mktemp -d /tmp/yazelix_packs_runtime_XXXXXX | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    let fake_store_target = ($tmp_home | path join "fake-shell")
    mkdir ($runtime_dir | path join ".devenv" "gc")
    mkdir $fake_store_target

    let result = (try {
        ^ln -s $fake_store_target ($runtime_dir | path join ".devenv" "gc" "shell")
        let packs_script = (repo_path "nushell" "scripts" "yzx" "packs.nu")
        let output = with-env { YAZELIX_RUNTIME_DIR: $runtime_dir } {
            ^nu -c $"source \"($packs_script)\"; get_devenv_shell" | complete
        }
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == $fake_store_target) {
            print "  ✅ yzx packs resolves .devenv links from the runtime root"
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

def test_relocated_runtime_smoke_supports_status_and_terminal_config_rendering [] {
    print "🧪 Testing relocated runtime smoke path supports status and terminal-config rendering..."

    let fixture = (setup_relocated_runtime_fixture)

    let result = (try {
        let env_overlay = {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
        }

        let status_output = with-env $env_overlay {
            ^nu -c $"use \"($fixture.yzx_script)\" *; yzx status" | complete
        }
        let gen_output = with-env $env_overlay { ^nu -c $"use \"($fixture.runtime_dir | path join "nushell" "scripts" "yzx" "gen_config.nu")\" [render_terminal_config]; render_terminal_config ghostty" | complete }

        let status_stdout = ($status_output.stdout | str trim)
        let gen_stdout = ($gen_output.stdout | str trim)

        if (
            ($status_output.exit_code == 0)
            and ($gen_output.exit_code == 0)
            and ($status_stdout | str contains $"Config File: ($fixture.config_dir | path join "yazelix.toml")")
            and ($status_stdout | str contains $"Directory: ($fixture.runtime_dir)")
            and ($status_stdout | str contains $"Logs: ($fixture.runtime_dir | path join "logs")")
            and ($gen_stdout | str contains $"exec ($fixture.startup_script)")
            and not ($gen_stdout | str contains $fixture.repo_root)
        ) {
            print "  ✅ Relocated runtime smoke path resolves config, runtime, and internal terminal launchers from split roots"
            true
        } else {
            print $"  ❌ Unexpected result: status_exit=($status_output.exit_code) gen_exit=($gen_output.exit_code)"
            print $"     status=($status_stdout)"
            print $"     gen=($gen_stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

export def run_core_canonical_tests [] {
    [
        (test_yzx_status)
        (test_yzx_status_versions_uses_invoking_path_for_versions)
        (test_invalid_config_is_classified_as_config_problem)
        (test_config_state_supports_split_config_and_runtime_dirs)
        (test_relocated_runtime_smoke_supports_status_and_terminal_config_rendering)
    ]
}

export def run_core_noncanonical_tests [] {
    [
        (test_yzx_config_reset_replaces_with_backup)
    ]
}

export def run_core_tests [] {
    [
        (run_core_canonical_tests)
        (run_core_noncanonical_tests)
    ] | flatten
}
