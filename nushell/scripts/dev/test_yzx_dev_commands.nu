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

def test_home_manager_desktop_entry_evaluates [] {
    print "🧪 Testing Home Manager desktop entry evaluates with StartupWMClass..."

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

        let attr = $"($flake_dir)#checks.($system).desktop_entry_smoke.startupWMClass"
        let output = (^nix eval --raw --read-only --no-write-lock-file $attr | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "com.yazelix.Yazelix") {
            print "  ✅ Home Manager desktop entry evaluates with StartupWMClass in settings"
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

def test_readme_title_matches_declared_version [] {
    print "🧪 Testing README title matches YAZELIX_VERSION..."

    try {
        let validator_script = ((get_repo_root) | path join "nushell" "scripts" "dev" "validate_readme_version.nu")
        let output = (^nu $validator_script | complete)

        if $output.exit_code == 0 {
            print "  ✅ README title/version marker matches YAZELIX_VERSION"
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

export def run_dev_tests [] {
    [
        (test_dev_update_canary_set)
        (test_gemini_cli_is_reactivated)
        (test_tru_is_in_ai_agents)
        (test_maintainer_pack_stays_in_sync)
        (test_popup_program_default_stays_in_sync)
        (test_home_manager_desktop_entry_evaluates)
        (test_readme_title_matches_declared_version)
        (test_specs_have_traceability_contract)
    ]
}
