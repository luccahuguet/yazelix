#!/usr/bin/env nu
# Test runner for maintainer-only yzx checks

def test_runtime_pin_versions_use_repo_shell [] {
    print "🧪 Testing runtime pin versions come from the repo shell..."

    if (which nix | is-empty) {
        print "  ❌ nix is required for maintainer tests"
        return false
    }

    if (which devenv | is-empty) {
        print "  ❌ devenv is required for maintainer tests"
        return false
    }

    try {
        let command = 'source nushell/scripts/yzx/dev.nu; let versions = (get_runtime_pin_versions); print ({ nix_version: $versions.nix_version, devenv_version: $versions.devenv_version, nix_raw: (get_tool_version_from_repo_shell "nix"), devenv_raw: (get_tool_version_from_repo_shell "devenv") } | to json -r)'
        let output = if (which timeout | is-not-empty) {
            ^timeout 30 nu -c $command | complete
        } else {
            ^nu -c $command | complete
        }
        let stdout = ($output.stdout | str trim)
        let resolved = ($stdout | lines | last | from json)

        if ($output.exit_code == 0) and ($resolved.nix_raw | str contains $resolved.nix_version) and ($resolved.devenv_raw | str contains $resolved.devenv_version) {
            print "  ✅ Runtime pins are derived from the repo shell versions"
            true
        } else if $output.exit_code == 124 {
            print "  ❌ Timed out while resolving runtime pins from the repo shell"
            false
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) resolved=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def main [] {
    print "=== Testing yzx Maintainer Commands ==="
    print ""

    let results = [
        (test_runtime_pin_versions_use_repo_shell)
    ]

    let passed = ($results | where $it == true | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ All yzx maintainer tests passed \(($passed)/($total)\)"
    } else {
        print $"❌ Some maintainer tests failed \(($passed)/($total)\)"
        error make { msg: "yzx maintainer tests failed" }
    }
}
