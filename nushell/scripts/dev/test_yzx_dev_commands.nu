#!/usr/bin/env nu

use ./test_yzx_helpers.nu [repo_path]

def test_home_manager_desktop_entry_evaluates [] {
    print "🧪 Testing Home Manager desktop entry uses the POSIX launcher contract..."

    if (uname).kernel-name != "Linux" {
        print "  ⏭️  Skipping on non-Linux host"
        return true
    }

    try {
        let flake_dir = (repo_path "home_manager")
        let system_output = (^nix eval --impure --raw --expr "builtins.currentSystem" | complete)
        let system = ($system_output.stdout | str trim)

        if ($system_output.exit_code != 0) or ($system | is-empty) {
            print $"  ❌ Failed to resolve current Nix system: stderr=($system_output.stderr | str trim)"
            return false
        }

        let startup_attr = $"($flake_dir)#checks.($system).desktop_entry_smoke.startupWMClass"
        let exec_attr = $"($flake_dir)#checks.($system).desktop_entry_smoke.exec"
        let toml_attr = $"($flake_dir)#checks.($system).desktop_entry_smoke.yazelixToml"
        let packs_attr = $"($flake_dir)#checks.($system).desktop_entry_smoke.yazelixPacksToml"
        let startup_output = (^nix eval --raw --read-only --no-write-lock-file $startup_attr | complete)
        let exec_output = (^nix eval --raw --read-only --no-write-lock-file $exec_attr | complete)
        let toml_output = (^nix eval --raw --read-only --no-write-lock-file $toml_attr | complete)
        let packs_output = (^nix eval --raw --read-only --no-write-lock-file $packs_attr | complete)
        let startup_wm_class = ($startup_output.stdout | str trim)
        let desktop_exec = ($exec_output.stdout | str trim)
        let generated_toml = ($toml_output.stdout | str trim)
        let generated_packs = ($packs_output.stdout | str trim)
        let expected_exec = "/home/test/.config/yazelix/shells/posix/desktop_launcher.sh"

        if (
            ($startup_output.exit_code == 0)
            and ($exec_output.exit_code == 0)
            and ($toml_output.exit_code == 0)
            and ($packs_output.exit_code == 0)
            and ($startup_wm_class == "com.yazelix.Yazelix")
            and ($desktop_exec == $expected_exec)
            and (not ($generated_toml | str contains "[packs]"))
            and ($generated_toml | str contains 'welcome_style = "random"')
            and not ($generated_toml | str contains "[ascii]")
            and ($generated_toml | str contains "Pack configuration lives in yazelix_packs.toml.")
            and ($generated_packs | str contains "enabled = [")
            and ($generated_packs | str contains '"git"')
            and ($generated_packs | str contains "[declarations]")
        ) {
            print "  ✅ Home Manager emits desktop entry plus split pack config surfaces"
            true
        } else {
            print $"  ❌ Unexpected result: startup_exit=($startup_output.exit_code) startup=($startup_wm_class) exec_exit=($exec_output.exit_code) exec=($desktop_exec) toml_exit=($toml_output.exit_code) packs_exit=($packs_output.exit_code) stderr=($startup_output.stderr | str trim) ($exec_output.stderr | str trim) ($toml_output.stderr | str trim) ($packs_output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_sync_readme_version_marker_rebuilds_generated_series_block [] {
    print "🧪 Testing README sync rebuilds the generated latest-series block..."

    let tmp_root = (^mktemp -d /tmp/yazelix_readme_sync_XXXXXX | str trim)
    let fixture_root = ($tmp_root | path join "repo")

    let result = (try {
        mkdir $fixture_root
        mkdir ($fixture_root | path join "docs")
        mkdir ($fixture_root | path join "nushell")
        mkdir ($fixture_root | path join "nushell" "scripts")
        mkdir ($fixture_root | path join "nushell" "scripts" "utils")

        let readme_path = ($fixture_root | path join "README.md")
        ^cp (repo_path "README.md") $readme_path
        ^cp (repo_path "docs" "upgrade_notes.toml") ($fixture_root | path join "docs" "upgrade_notes.toml")
        ^cp (repo_path "nushell" "scripts" "utils" "common.nu") ($fixture_root | path join "nushell" "scripts" "utils" "common.nu")
        ^cp (repo_path "nushell" "scripts" "utils" "constants.nu") ($fixture_root | path join "nushell" "scripts" "utils" "constants.nu")
        ^cp (repo_path "nushell" "scripts" "utils" "readme_release_block.nu") ($fixture_root | path join "nushell" "scripts" "utils" "readme_release_block.nu")
        let broken_readme = (
            open --raw $readme_path
            | str replace -r '(?s)<!-- BEGIN GENERATED README LATEST SERIES -->.*<!-- END GENERATED README LATEST SERIES -->' $"<!-- BEGIN GENERATED README LATEST SERIES -->\n## What's New In v0\n\n- stale block\n\n<!-- END GENERATED README LATEST SERIES -->"
        )
        $broken_readme | save --force --raw $readme_path

        let helper_script = ($fixture_root | path join "nushell" "scripts" "utils" "readme_release_block.nu")

        let sync_output = (with-env {YAZELIX_DIR: $fixture_root} {
            ^nu -c $"use \"($helper_script)\" [sync_readme_surface]; sync_readme_surface | ignore" | complete
        })
        let expected_output = (with-env {YAZELIX_DIR: $fixture_root} {
            ^nu -c $"use \"($helper_script)\" [render_readme_latest_series_section]; render_readme_latest_series_section" | complete
        })
        let expected_section = ($expected_output.stdout | str trim)
        let updated_readme = (open --raw $readme_path)

        if ($sync_output.exit_code == 0) and ($expected_output.exit_code == 0) and ($updated_readme | str contains $expected_section) and (not ($updated_readme | str contains "stale block")) {
            print "  ✅ README sync restores the generated latest-series block from upgrade_notes.toml"
            true
        } else {
            print $"  ❌ Unexpected result: sync_exit=($sync_output.exit_code) expected_exit=($expected_output.exit_code) sync_stderr=($sync_output.stderr | str trim) expected_stderr=($expected_output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_root
    $result
}

export def run_dev_canonical_tests [] {
    [
        (test_sync_readme_version_marker_rebuilds_generated_series_block)
        (test_home_manager_desktop_entry_evaluates)
    ]
}

export def run_dev_noncanonical_tests [] {
    []
}

export def run_dev_tests [] {
    [
        (run_dev_canonical_tests)
        (run_dev_noncanonical_tests)
    ] | flatten
}
