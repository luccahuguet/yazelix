#!/usr/bin/env nu
# Development helper commands for maintainers

use ../utils/constants.nu [PINNED_NIX_VERSION PINNED_DEVENV_VERSION YAZELIX_VERSION]

def update_constant_value [contents: string, key: string, new_value: string] {
    let pattern = $"export const ($key) = \"[^\"]+\""
    $contents | str replace -ra $pattern $"export const ($key) = \"($new_value)\""
}

def extract_version [value: string] {
    $value | parse --regex '(\d+\.\d+\.\d+)' | get capture0 | last | default ""
}

def get_latest_tag [] {
    if (which git | is-empty) {
        print "❌ git not found in PATH."
        exit 1
    }

    let result = (^git describe --tags --abbrev=0 | complete)
    if $result.exit_code != 0 {
        print $"❌ Failed to read git tag: ($result.stderr | str trim)"
        exit 1
    }

    let tag = ($result.stdout | str trim)
    if ($tag | is-empty) {
        print "❌ No git tags found."
        exit 1
    }

    $tag
}

export def "yzx dev sync_pins" [] {
    if (which nix | is-empty) {
        print "❌ nix not found in PATH."
        exit 1
    }

    if (which devenv | is-empty) {
        print "❌ devenv not found in PATH."
        exit 1
    }

    let nix_version_raw = (nix --version | lines | first)
    let devenv_version_raw = (devenv --version | lines | first)
    let nix_version = (extract_version $nix_version_raw)
    let devenv_version = (extract_version $devenv_version_raw)

    if ($nix_version | is-empty) {
        print $"❌ Failed to parse nix version from: ($nix_version_raw)"
        exit 1
    }

    if ($devenv_version | is-empty) {
        print $"❌ Failed to parse devenv version from: ($devenv_version_raw)"
        exit 1
    }

    let constants_path = "~/.config/yazelix/nushell/scripts/utils/constants.nu" | path expand
    if not ($constants_path | path exists) {
        print $"❌ Constants file not found: ($constants_path)"
        exit 1
    }

    let latest_tag = (get_latest_tag)
    let contents = (open $constants_path)
    let updated = (
        update_constant_value (
            update_constant_value (
                update_constant_value $contents "YAZELIX_VERSION" $latest_tag
            ) "PINNED_NIX_VERSION" $nix_version
        ) "PINNED_DEVENV_VERSION" $devenv_version
    )

    if $updated == $contents {
        print $"✅ Pins unchanged: yazelix ($YAZELIX_VERSION), nix ($PINNED_NIX_VERSION), devenv ($PINNED_DEVENV_VERSION)"
        return
    }

    $updated | save $constants_path --force
    print $"✅ Updated pins: yazelix ($latest_tag), nix ($nix_version), devenv ($devenv_version)"
}

export def "yzx dev update_lock" [
    --verbose  # Show the underlying devenv command
    --yes      # Skip confirmation prompt
] {
    use ~/.config/yazelix/nushell/scripts/utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    let yazelix_dir = "~/.config/yazelix" | path expand

    if not $yes {
        print "⚠️  This updates Yazelix inputs (devenv.lock) to latest upstream versions."
        print "   If upstream changes are broken, you may hit bugs before fixes land."
        print "   Prefer a safer path? The Yazelix maintainer updates the project at least once a month."
        let confirm = (input "Continue? [y/N]: " | str downcase)
        if $confirm not-in ["y", "yes"] {
            print "Aborted."
            exit 0
        }
    }

    if $verbose {
        print $"⚙️ Running: devenv update \(cwd: ($yazelix_dir)\)"
    } else {
        print "🔄 Updating Yazelix inputs (devenv.lock)..."
    }

    try {
        do {
            cd $yazelix_dir
            ^devenv update
        }
        print "✅ devenv.lock updated. Review and commit the changes if everything looks good."
    } catch {|err|
        print $"❌ devenv update failed: ($err.msg)"
        print "   Check your network connection and devenv.yaml inputs, then try again."
        exit 1
    }
}

export def "yzx dev update_nix" [
    --yes      # Skip confirmation prompt
    --verbose  # Show the underlying command
] {
    if (which determinate-nixd | is-empty) {
        print "❌ determinate-nixd not found in PATH."
        print "   Install Determinate Nix or check your PATH, then try again."
        exit 1
    }

    if not $yes {
        print "⚠️  This upgrades Determinate Nix using determinate-nixd."
        print "   If your Nix install is not based on Determinate Nix, this will not work."
        print "   It requires sudo and may prompt for your password."
        let confirm = (input "Continue? [y/N]: " | str downcase)
        if $confirm not-in ["y", "yes"] {
            print "Aborted."
            exit 0
        }
    }

    if $verbose {
        print "⚙️ Running: sudo determinate-nixd upgrade"
    } else {
        print "🔄 Upgrading Determinate Nix..."
    }

    try {
        let result = (^sudo determinate-nixd upgrade | complete)
        if $result.exit_code != 0 {
            print $"❌ Determinate Nix upgrade failed: ($result.stderr | str trim)"
            exit 1
        }
        print "✅ Determinate Nix upgraded."
    } catch {|err|
        print $"❌ Determinate Nix upgrade failed: ($err.msg)"
        exit 1
    }
}
