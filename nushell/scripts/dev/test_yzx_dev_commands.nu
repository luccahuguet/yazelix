#!/usr/bin/env nu

use ../core/yazelix.nu *

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

def test_tru_is_in_ai_agents [] {
    print "🧪 Testing tru is included in ai_agents..."

    try {
        let default_config = (open ~/.config/yazelix/yazelix_default.toml)
        let default_agents = ($default_config.packs.declarations.ai_agents | default [])
        let hm_module = (open --raw ~/.config/yazelix/home_manager/module.nix)

        if ("tru" in $default_agents) and ($hm_module | str contains '"tru"') {
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

def test_home_manager_desktop_entry_evaluates [] {
    print "🧪 Testing Home Manager desktop entry evaluates with StartupWMClass..."

    if (uname).kernel-name != "Linux" {
        print "  ⏭️  Skipping on non-Linux host"
        return true
    }

    try {
        let flake_dir = ("~/.config/yazelix/home_manager" | path expand)
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

export def run_dev_tests [] {
    [
        (test_dev_update_canary_set)
        (test_gemini_cli_is_reactivated)
        (test_tru_is_in_ai_agents)
        (test_home_manager_desktop_entry_evaluates)
    ]
}
