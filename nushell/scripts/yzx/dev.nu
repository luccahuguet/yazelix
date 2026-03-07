#!/usr/bin/env nu
# Development helper commands for maintainers

use ../utils/constants.nu [PINNED_NIX_VERSION PINNED_DEVENV_VERSION YAZELIX_VERSION]
use ../utils/terminal_configs.nu generate_all_terminal_configs

# Development and maintainer commands
export def "yzx dev" [] {
    print "Run 'yzx dev --help' to see available maintainer subcommands"
}

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
        let confirm = try {
            (input "Continue? [y/N]: " | str downcase)
        } catch { "n" }
        if $confirm not-in ["y", "yes"] {
            print "Aborted."
            return
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

export def "yzx dev sync_terminal_configs" [] {
    let yazelix_dir = "~/.config/yazelix" | path expand
    let config_root = ($yazelix_dir | path join "configs/terminal_emulators")
    let generated_root = "~/.local/share/yazelix/configs/terminal_emulators" | path expand

    if not ($config_root | path exists) {
        print $"❌ Configs directory not found: ($config_root)"
        exit 1
    }

    let default_config = "~/.config/yazelix/yazelix_default.toml" | path expand
    if not ($default_config | path exists) {
        print $"❌ Default config not found: ($default_config)"
        exit 1
    }

    print "Generating terminal configs from defaults..."
    with-env {YAZELIX_CONFIG_OVERRIDE: $default_config} {
        generate_all_terminal_configs
    }

    let generated_at = (date now | format date "%Y-%m-%d %H:%M:%S %Z")
    let header_lines = [
        "# Generated by Yazelix"
        $"# Timestamp: ($generated_at)"
        $"# Source: ($generated_root)"
        $"# Config: ($default_config)"
        ""
    ]
    let header = ($header_lines | str join "\n")

    let mappings = [
        {terminal: "ghostty", source: "ghostty/config", dest: "ghostty/config"}
        {terminal: "wezterm", source: "wezterm/.wezterm.lua", dest: "wezterm/.wezterm.lua"}
        {terminal: "kitty", source: "kitty/kitty.conf", dest: "kitty/kitty.conf"}
        {terminal: "alacritty", source: "alacritty/alacritty.toml", dest: "alacritty/alacritty.toml"}
        {terminal: "foot", source: "foot/foot.ini", dest: "foot/foot.ini"}
    ]

    for entry in $mappings {
        let source_path = ($generated_root | path join $entry.source)
        if not ($source_path | path exists) {
            print $"⚠️  Skipping ($entry.terminal): no generated config at ($source_path)"
            continue
        }

        let dest_path = ($config_root | path join $entry.dest)
        let content = (open --raw $source_path)
        let final_content = $"($header)($content)"
        $final_content | save $dest_path --force
        print $"✅ Synced ($entry.terminal) → ($dest_path)"
    }
}

# Run Yazelix test suite
export def "yzx dev test" [
    --verbose(-v)  # Show detailed test output
    --new-window(-n)  # Run tests in a new Yazelix window
    --sweep  # Run only the non-visual configuration sweep
    --visual  # Run only the visual terminal sweep
    --all(-a)  # Run the full suite plus the visual terminal sweep
    --delay: int = 3  # Delay between visual terminal launches in seconds
] {
    use ../utils/test_runner.nu run_all_tests
    run_all_tests --verbose=$verbose --new-window=$new_window --sweep=$sweep --visual=$visual --all=$all --delay $delay
}

# Validate syntax of all Nushell scripts
export def "yzx dev lint" [
    --verbose(-v)  # Show detailed output for each file
] {
    if $verbose {
        nu $"($env.HOME)/.config/yazelix/nushell/scripts/dev/validate_syntax.nu" --verbose
    } else {
        nu $"($env.HOME)/.config/yazelix/nushell/scripts/dev/validate_syntax.nu"
    }
}

# Benchmark terminal launch performance
export def "yzx dev bench" [
    --iterations(-n): int = 1  # Number of iterations per terminal
    --terminal(-t): string     # Test only specific terminal
    --verbose(-v)              # Show detailed output
] {
    mut args = ["--iterations", $iterations]

    if ($terminal | is-not-empty) {
        $args = ($args | append ["--terminal", $terminal])
    }

    if $verbose {
        $args = ($args | append "--verbose")
    }

    nu $"($env.HOME)/.config/yazelix/nushell/scripts/dev/benchmark_terminals.nu" ...$args
}

# Profile launch sequence and identify bottlenecks
export def "yzx dev profile" [
    --cold(-c)        # Profile cold launch from vanilla terminal (emulates desktop entry or fresh terminal launch)
    --clear-cache     # Toggle yazelix.toml option and clear cache to force full Nix re-evaluation (simulates config change)
] {
    use ../utils/profile.nu *

    if $cold {
        profile_cold_launch --clear-cache=$clear_cache
    } else {
        profile_launch
    }
}
