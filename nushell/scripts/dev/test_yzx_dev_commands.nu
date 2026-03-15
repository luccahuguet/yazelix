#!/usr/bin/env nu

use ../core/yazelix.nu *

def test_yzx_dev_exists [] {
    print "🧪 Testing yzx dev command exists..."

    try {
        let output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx" | complete).stdout | str trim

        if ($output | str contains "yzx dev") {
            print "  ✅ yzx dev command is documented in help"
            true
        } else {
            print "  ❌ yzx dev command not found in help"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

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

def test_dev_update_defaults_to_verbose_mode [] {
    print "🧪 Testing yzx dev update defaults to verbose mode..."

    try {
        let output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; help 'yzx dev update'" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "--quiet") and ($stdout | str contains "verbose by default") {
            print "  ✅ yzx dev update documents verbose-by-default behavior and the quiet opt-out"
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

def test_dev_update_help_mentions_optional_input_name [] {
    print "🧪 Testing yzx dev update documents the optional input name..."

    try {
        let output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; help 'yzx dev update'" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "input_name <string>") and ($stdout | str contains "devenv update") {
            print "  ✅ yzx dev update documents the optional input passthrough"
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

def test_runtime_pin_versions_use_repo_shell [] {
    print "🧪 Testing runtime pin versions come from the repo shell..."

    try {
        if (($env.YAZELIX_RUN_MAINTAINER_TESTS? | default "false") != "true") {
            print "  ℹ️  Skipping maintainer-only runtime pin test by default"
            return true
        }

        if (which nix | is-empty) or (which devenv | is-empty) {
            print "  ℹ️  Skipping runtime pin test because nix/devenv are not available"
            return true
        }

        let output = (^nu -c 'source ~/.config/yazelix/nushell/scripts/yzx/dev.nu; let versions = (get_runtime_pin_versions); print ({ nix_version: $versions.nix_version, devenv_version: $versions.devenv_version, nix_raw: (get_tool_version_from_repo_shell "nix"), devenv_raw: (get_tool_version_from_repo_shell "devenv") } | to json -r)' | complete)
        let stdout = ($output.stdout | str trim)
        let resolved = ($stdout | from json)

        if ($output.exit_code == 0) and ($resolved.nix_raw | str contains $resolved.nix_version) and ($resolved.devenv_raw | str contains $resolved.devenv_version) {
            print "  ✅ Runtime pins are derived from the repo shell versions"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) resolved=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

export def run_dev_tests [] {
    [
        (test_yzx_dev_exists)
        (test_dev_update_canary_set)
        (test_dev_update_defaults_to_verbose_mode)
        (test_dev_update_help_mentions_optional_input_name)
        (test_gemini_cli_is_reactivated)
        (test_tru_is_in_ai_agents)
        (test_runtime_pin_versions_use_repo_shell)
    ]
}
