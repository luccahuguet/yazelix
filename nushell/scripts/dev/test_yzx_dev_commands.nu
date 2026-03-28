#!/usr/bin/env nu

use ../core/yazelix.nu *
use ./test_yzx_helpers.nu [get_repo_config_dir get_repo_root repo_path]

def extract_hm_pack_declaration [pack_name: string] {
    let hm_module = (open --raw (repo_path "home_manager" "module.nix"))
    let lines = ($hm_module | lines)
    let start_pattern = $"        ($pack_name) = ["
    let start_index = ($lines | enumerate | where item == $start_pattern | get -o 0.index | default null)

    if $start_index == null {
        return []
    }

    $lines
    | skip ($start_index + 1)
    | take while { |line| ($line | str trim) != "];" }
    | each { |line|
        $line
        | str trim
        | parse --regex '^"(?<pkg>.+)"$'
        | get -o 0.pkg
        | default null
    }
    | where { |pkg| $pkg != null }
}

def test_dev_update_canary_set [] {
    print "🧪 Testing yzx dev update canary set..."

    try {
        let dev_script = (repo_path "nushell" "scripts" "yzx" "dev.nu")
        let output = (^nu -c $"source \"($dev_script)\"; get_available_update_canaries | to json -r" | complete)
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

def test_gemini_cli_is_reactivated [] {
    print "🧪 Testing Gemini CLI is reactivated in default and Home Manager configs..."

    try {
        let default_config = (open (repo_path "yazelix_default.toml"))
        let default_agents = ($default_config.packs.declarations.ai_agents | default [])
        let hm_module = (open --raw (repo_path "home_manager" "module.nix"))

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

def test_tru_is_in_ai_agents [] {
    print "🧪 Testing tru is included in ai_agents..."

    try {
        let default_config = (open (repo_path "yazelix_default.toml"))
        let default_agents = ($default_config.packs.declarations.ai_agents | default [])
        let hm_agents = (extract_hm_pack_declaration "ai_agents")

        if ("tru" in $default_agents) and ("tru" in $hm_agents) {
            print "  ✅ tru is present in both ai_agents configuration paths"
            true
        } else {
            print "  ❌ tru is missing from the default config or Home Manager module"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_maintainer_pack_stays_in_sync [] {
    print "🧪 Testing maintainer pack stays in sync across config paths..."

    try {
        let expected = [
            "gh"
            "prek"
            "tru"
            "beads-rust"
            "beads-viewer"
            "rust_wasi_toolchain"
        ]
        let default_config = (open (repo_path "yazelix_default.toml"))
        let default_pack = ($default_config.packs.declarations.maintainer | default [])
        let hm_pack = (extract_hm_pack_declaration "maintainer")

        if (($default_pack | sort) == ($expected | sort)) and (($hm_pack | sort) == ($expected | sort)) {
            print "  ✅ maintainer pack matches in both the default config and Home Manager module"
            true
        } else {
            print $"  ❌ Unexpected maintainer pack contents: default=($default_pack | to json -r) hm=($hm_pack | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_ts_pack_stays_in_sync [] {
    print "🧪 Testing ts pack stays in sync across config paths..."

    try {
        let expected = [
            "nodePackages.typescript-language-server"
            "tailwindcss-language-server"
            "biome"
            "oxlint"
        ]
        let default_config = (open (repo_path "yazelix_default.toml"))
        let default_pack = ($default_config.packs.declarations.ts | default [])
        let hm_pack = (extract_hm_pack_declaration "ts")

        if (($default_pack | sort) == ($expected | sort)) and (($hm_pack | sort) == ($expected | sort)) {
            print "  ✅ ts pack matches in both the default config and Home Manager module"
            true
        } else {
            print $"  ❌ Unexpected ts pack contents: default=($default_pack | to json -r) hm=($hm_pack | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_modern_js_pack_stays_in_sync [] {
    print "🧪 Testing modern_js pack stays in sync across config paths..."

    try {
        let expected = [
            "bun"
            "deno"
        ]
        let default_config = (open (repo_path "yazelix_default.toml"))
        let default_pack = ($default_config.packs.declarations.modern_js | default [])
        let hm_pack = (extract_hm_pack_declaration "modern_js")

        if (($default_pack | sort) == ($expected | sort)) and (($hm_pack | sort) == ($expected | sort)) {
            print "  ✅ modern_js pack matches in both the default config and Home Manager module"
            true
        } else {
            print $"  ❌ Unexpected modern_js pack contents: default=($default_pack | to json -r) hm=($hm_pack | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_popup_program_default_stays_in_sync [] {
    print "🧪 Testing popup_program default stays in sync across config paths..."

    try {
        let default_config = (open (repo_path "yazelix_default.toml"))
        let default_popup = ($default_config.zellij.popup_program | default [])
        let hm_module = (open --raw (repo_path "home_manager" "module.nix"))

        if ($default_popup == ["lazygit"]) and ($hm_module | str contains 'default = [ "lazygit" ];') and ($hm_module | str contains 'popup_program = ${listToToml cfg.popup_program}') {
            print "  ✅ popup_program default matches in both the default config and Home Manager module"
            true
        } else {
            print $"  ❌ Unexpected popup_program defaults: default=($default_popup | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_zellij_custom_text_default_stays_in_sync [] {
    print "🧪 Testing zellij custom_text default stays in sync across config paths..."

    try {
        let default_config = (open (repo_path "yazelix_default.toml"))
        let default_custom_text = ($default_config.zellij.custom_text | default "")
        let hm_module = (open --raw (repo_path "home_manager" "module.nix"))

        if ($default_custom_text == "") and ($hm_module | str contains 'default = "";') and ($hm_module | str contains 'custom_text = ${escapeString cfg.zellij_custom_text}') {
            print "  ✅ zellij custom_text default matches in both the default config and Home Manager module"
            true
        } else {
            print $"  ❌ Unexpected zellij custom_text defaults: default=($default_custom_text | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

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
        let startup_output = (^nix eval --raw --read-only --no-write-lock-file $startup_attr | complete)
        let exec_output = (^nix eval --raw --read-only --no-write-lock-file $exec_attr | complete)
        let startup_wm_class = ($startup_output.stdout | str trim)
        let desktop_exec = ($exec_output.stdout | str trim)
        let expected_exec = "/home/test/.config/yazelix/shells/posix/desktop_launcher.sh"

        if ($startup_output.exit_code == 0) and ($exec_output.exit_code == 0) and ($startup_wm_class == "com.yazelix.Yazelix") and ($desktop_exec == $expected_exec) {
            print "  ✅ Home Manager desktop entry evaluates with the POSIX launcher and StartupWMClass"
            true
        } else {
            print $"  ❌ Unexpected result: startup_exit=($startup_output.exit_code) startup=($startup_wm_class) exec_exit=($exec_output.exit_code) exec=($desktop_exec) stderr=($startup_output.stderr | str trim) ($exec_output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_specs_have_traceability_contract [] {
    print "🧪 Testing real specs declare bead and regression traceability..."

    try {
        let validator_script = ((get_repo_root) | path join "nushell" "scripts" "dev" "validate_specs.nu")
        let output = (^nu $validator_script | complete)

        if $output.exit_code == 0 {
            print "  ✅ real specs declare bead and regression traceability"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_default_suite_traceability_contract [] {
    print "🧪 Testing the default test suite is tied to specs or justified regression-only entries..."

    try {
        let validator_script = ((get_repo_root) | path join "nushell" "scripts" "dev" "validate_default_test_traceability.nu")
        let output = (^nu $validator_script | complete)

        if $output.exit_code == 0 {
            print "  ✅ default-suite entrypoints are traced to specs or a tiny justified allowlist"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($output.stderr | str trim)"
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
        (test_specs_have_traceability_contract)
        (test_default_suite_traceability_contract)
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
