#!/usr/bin/env nu
# Development helper commands for maintainers

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

def get_runtime_pin_versions [] {
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

    {
        nix_version: $nix_version
        devenv_version: $devenv_version
    }
}

def sync_runtime_pins [] {
    let constants_path = "~/.config/yazelix/nushell/scripts/utils/constants.nu" | path expand
    if not ($constants_path | path exists) {
        print $"❌ Constants file not found: ($constants_path)"
        exit 1
    }

    let runtime_pins = get_runtime_pin_versions
    let contents = (open $constants_path)
    let updated = (
        update_constant_value (
            update_constant_value $contents "PINNED_NIX_VERSION" $runtime_pins.nix_version
        ) "PINNED_DEVENV_VERSION" $runtime_pins.devenv_version
    )

    if $updated == $contents {
        print $"✅ Runtime pins unchanged: nix ($runtime_pins.nix_version), devenv ($runtime_pins.devenv_version)"
        return
    }

    $updated | save $constants_path --force
    print $"✅ Updated runtime pins: nix ($runtime_pins.nix_version), devenv ($runtime_pins.devenv_version)"
}

def sync_vendored_zjstatus [] {
    let update_script = ("~/.config/yazelix/nushell/scripts/dev/update_zjstatus.nu" | path expand)
    if not ($update_script | path exists) {
        print $"❌ zjstatus refresh helper not found: ($update_script)"
        exit 1
    }

    print "🔄 Refreshing vendored zjstatus.wasm..."
    try {
        ^nu $update_script
    } catch {|err|
        print $"❌ Failed to refresh vendored zjstatus.wasm: ($err.msg)"
        exit 1
    }
}

def get_available_update_canaries [] {
    ["default" "ai-heavy" "maximal"]
}

def resolve_update_canary_selection [requested: list<string>] {
    let available = get_available_update_canaries

    if ($requested | is-empty) {
        return $available
    }

    let normalized = ($requested | each { |name| $name | into string | str downcase })
    let invalid = ($normalized | where { |name| $name not-in $available })
    if ($invalid | is-not-empty) {
        let available_text = ($available | str join ", ")
        let invalid_text = ($invalid | str join ", ")
        error make {msg: $"Unknown canary name(s): ($invalid_text). Expected one of: ($available_text)"}
    }

    $normalized | uniq
}

def write_update_canary_config [config: record, output_path: string] {
    ($config | to toml) | save --force --raw $output_path
}

def materialize_update_canaries [selected: list<string>] {
    let default_config_path = "~/.config/yazelix/yazelix_default.toml" | path expand
    if not ($default_config_path | path exists) {
        error make {msg: $"Default config not found: ($default_config_path)"}
    }

    let template = (open $default_config_path)
    let all_pack_names = ($template.packs.declarations | columns | sort)
    let ai_heavy_packs = (
        ["ai_agents" "ai_tools" "config" "git" "nix" "python" "rust" "rust_extra" "ts"]
        | where { |name| $name in $all_pack_names }
    )

    let base_temp_dir = "~/.local/share/yazelix/update_canaries" | path expand
    mkdir $base_temp_dir
    let temp_dir = (^mktemp -d ($base_temp_dir | path join "update_XXXXXX") | str trim)

    let canaries = (
        $selected
        | each { |name|
            match $name {
                "default" => {
                    {
                        name: "default"
                        config_path: $default_config_path
                        description: "yazelix_default.toml"
                    }
                }
                "ai-heavy" => {
                    let config_path = ($temp_dir | path join "canary_ai_heavy.toml")
                    let config = ($template | upsert packs.enabled $ai_heavy_packs)
                    write_update_canary_config $config $config_path
                    {
                        name: "ai-heavy"
                        config_path: $config_path
                        description: $"packs.enabled = [($ai_heavy_packs | str join ', ')]"
                    }
                }
                "maximal" => {
                    let config_path = ($temp_dir | path join "canary_maximal.toml")
                    let config = ($template | upsert packs.enabled $all_pack_names)
                    write_update_canary_config $config $config_path
                    {
                        name: "maximal"
                        config_path: $config_path
                        description: "all pack declarations enabled"
                    }
                }
            }
        }
    )

    {
        temp_dir: $temp_dir
        canaries: $canaries
    }
}

def cleanup_update_canaries [temp_dir: string] {
    if ($temp_dir | path exists) {
        rm -rf $temp_dir
    }
}

def trim_output_tail [text: string, max_lines: int] {
    let trimmed = ($text | default "" | str trim)
    if ($trimmed | is-empty) {
        return ""
    }

    let lines = ($trimmed | lines)
    if (($lines | length) <= $max_lines) {
        $trimmed
    } else {
        $lines | last $max_lines | str join "\n"
    }
}

def run_update_canary [canary: record, verbose: bool] {
    let refresh_command = if $verbose {
        "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx refresh --force --verbose"
    } else {
        "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx refresh --force"
    }

    let result = (do {
        with-env {YAZELIX_CONFIG_OVERRIDE: $canary.config_path} {
            ^nu -c $refresh_command | complete
        }
    })

    let stdout_tail = trim_output_tail ($result.stdout | default "") 25
    let stderr_tail = trim_output_tail ($result.stderr | default "") 25

    {
        name: $canary.name
        config_path: $canary.config_path
        description: $canary.description
        exit_code: $result.exit_code
        stdout_tail: $stdout_tail
        stderr_tail: $stderr_tail
        ok: ($result.exit_code == 0)
    }
}

def run_update_canaries [selected: list<string>, verbose: bool] {
    let context = materialize_update_canaries $selected
    let results = try {
        (
            $context.canaries
            | each { |canary|
                print $"🧪 Canary: ($canary.name) — ($canary.description)"
                run_update_canary $canary $verbose
            }
        )
    } catch { |err|
        cleanup_update_canaries $context.temp_dir
        error make {msg: $err.msg}
    }
    cleanup_update_canaries $context.temp_dir
    $results
}

def print_update_canary_summary [results: list] {
    print ""
    print "Canary summary:"
    for result in $results {
        let status_icon = if $result.ok { "✅" } else { "❌" }
        print $"  ($status_icon) ($result.name) — ($result.description)"
    }
}

def print_update_canary_failure_details [results: list] {
    let failures = ($results | where ok == false)
    if ($failures | is-empty) {
        return
    }

    print ""
    print "Failed canary details:"
    for failure in $failures {
        print $"  ❌ ($failure.name)"
        print $"     Config: ($failure.config_path)"
        print $"     Exit code: ($failure.exit_code)"
        if ($failure.stderr_tail | is-not-empty) {
            print "     stderr tail:"
            print ($failure.stderr_tail | lines | each { |line| $"       ($line)" } | str join "\n")
        } else if ($failure.stdout_tail | is-not-empty) {
            print "     stdout tail:"
            print ($failure.stdout_tail | lines | each { |line| $"       ($line)" } | str join "\n")
        }
    }
}

export def "yzx dev update" [
    --verbose  # Show the underlying devenv command
    --yes      # Skip confirmation prompt
    --no-canary  # Skip canary refresh/build checks after updating devenv.lock
    --canary-only  # Run canary checks without updating devenv.lock or syncing pins
    --canaries: list<string> = []  # Canary subset: default, ai-heavy, maximal
] {
    use ~/.config/yazelix/nushell/scripts/utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    let yazelix_dir = "~/.config/yazelix" | path expand
    let selected_canaries = resolve_update_canary_selection $canaries

    if $no_canary and $canary_only {
        print "❌ --no-canary and --canary-only cannot be used together."
        exit 1
    }

    if (not $yes) and (not $canary_only) {
        print "⚠️  This updates Yazelix maintainer inputs to latest upstream versions."
        print "   The hardened flow updates devenv.lock locally, then runs canary refresh/build checks before finishing."
        print "   Broken updates should stay local and never be pushed."
        let confirm = try {
            (input "Continue? [y/N]: " | str downcase)
        } catch { "n" }
        if $confirm not-in ["y", "yes"] {
            print "Aborted."
            return
        }
    }

    if $canary_only {
        print $"🧪 Running update canaries only: ($selected_canaries | str join ', ')"
    } else if $verbose {
        print $"⚙️ Running: devenv update \(cwd: ($yazelix_dir)\)"
    } else {
        print "🔄 Updating Yazelix inputs..."
    }

    if not $canary_only {
        try {
            do {
                cd $yazelix_dir
                ^devenv update
            }
        } catch {|err|
            print $"❌ devenv update failed: ($err.msg)"
            print "   Check your network connection and devenv.yaml inputs, then try again."
            exit 1
        }
        print "✅ devenv.lock updated."
    }

    if $no_canary {
        print "⚠️  Canary checks were skipped."
    } else {
        let canary_results = run_update_canaries $selected_canaries $verbose
        print_update_canary_summary $canary_results
        if ($canary_results | any { |result| not $result.ok }) {
            print_update_canary_failure_details $canary_results
            print ""
            print "❌ One or more canaries failed."
            if not $canary_only {
                print "   Keep this lockfile update local until the failures are resolved."
            }
            exit 1
        }
        print "✅ All selected canaries passed."
    }

    if $canary_only {
        print "✅ Canary run completed. No lockfile or pin changes were made."
        return
    }

    print "🔄 Syncing pinned runtime expectations..."
    sync_runtime_pins
    sync_vendored_zjstatus
    print "✅ Inputs, canaries, runtime pins, and vendored zjstatus are in sync. Review and commit the changes if everything looks good."
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
    --lint-only  # Run only syntax validation
    --sweep  # Run only the non-visual configuration sweep
    --visual  # Run only the visual terminal sweep
    --all(-a)  # Run the full suite plus the visual terminal sweep
    --delay: int = 3  # Delay between visual terminal launches in seconds
] {
    use ../utils/test_runner.nu run_all_tests
    run_all_tests --verbose=$verbose --new-window=$new_window --lint-only=$lint_only --sweep=$sweep --visual=$visual --all=$all --delay $delay
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
