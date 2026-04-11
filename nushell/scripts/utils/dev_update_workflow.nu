#!/usr/bin/env nu

use repo_checkout.nu require_yazelix_repo_root
use config_surfaces.nu [copy_default_config_surfaces load_config_surface_from_main get_main_user_config_path]
use readme_release_block.nu sync_readme_surface
use devenv_cli.nu resolve_preferred_devenv_path
use nix_detector.nu ensure_nix_available

def update_constant_value [contents: string, key: string, new_value: string] {
    let pattern = $"export const ($key) = \"[^\"]+\""
    $contents | str replace -ra $pattern $"export const ($key) = \"($new_value)\""
}

def extract_version [value: string] {
    $value | parse --regex '(\d+\.\d+\.\d+)' | get capture0 | last | default ""
}

def get_nix_version_from_repo_shell [] {
    let result = (^nix --version | complete)
    if $result.exit_code != 0 {
        let stderr = ($result.stderr | str trim)
        print $"❌ Failed to resolve nix version from the current environment: ($stderr)"
        exit 1
    }
    $result.stdout | str trim
}

def get_runtime_pin_versions [] {
    if (which nix | is-empty) {
        print "❌ nix not found in PATH."
        exit 1
    }

    print "   Resolving nix from the current environment..."
    let nix_version_raw = (get_nix_version_from_repo_shell)
    let nix_version = (extract_version $nix_version_raw)

    if ($nix_version | is-empty) {
        print $"❌ Failed to parse nix version from: ($nix_version_raw)"
        exit 1
    }

    {
        nix_version: $nix_version
    }
}

def sync_runtime_pins [] {
    let constants_path = ((require_yazelix_repo_root) | path join "nushell" "scripts" "utils" "constants.nu")
    if not ($constants_path | path exists) {
        print $"❌ Constants file not found: ($constants_path)"
        exit 1
    }

    let runtime_pins = get_runtime_pin_versions
    let contents = (open $constants_path)
    let updated = (
        update_constant_value $contents "PINNED_NIX_VERSION" $runtime_pins.nix_version
    )

    if $updated == $contents {
        print $"✅ Runtime pins unchanged: nix ($runtime_pins.nix_version)"
        return
    }

    $updated | save $constants_path --force
    print $"✅ Updated runtime pins: nix ($runtime_pins.nix_version)"
}

def sync_vendored_zjstatus [] {
    let update_script = ((require_yazelix_repo_root) | path join "nushell" "scripts" "dev" "update_zjstatus.nu")
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

def sync_vendored_yazi_plugins [] {
    let update_script = ((require_yazelix_repo_root) | path join "nushell" "scripts" "dev" "update_yazi_plugins.nu")
    if not ($update_script | path exists) {
        print $"❌ Vendored Yazi plugin refresh helper not found: ($update_script)"
        exit 1
    }

    print "🔄 Refreshing vendored Yazi plugin runtime files..."
    try {
        ^nu $update_script
    } catch {|err|
        print $"❌ Failed to refresh vendored Yazi plugin runtime files: ($err.msg)"
        exit 1
    }
}

def get_declared_yazelix_version [] {
    let constants_path = ((require_yazelix_repo_root) | path join "nushell" "scripts" "utils" "constants.nu")
    let constants = (open --raw $constants_path)
    let version_match = (
        $constants
        | parse --regex 'export const YAZELIX_VERSION = "(v[^"]+)"'
        | get -o capture0
        | first
        | default ""
    )

    if ($version_match | is-empty) {
        print $"❌ Failed to read YAZELIX_VERSION from: ($constants_path)"
        exit 1
    }

    $version_match
}

def sync_readme_version_marker [] {
    let readme_path = ((require_yazelix_repo_root) | path join "README.md")
    if not ($readme_path | path exists) {
        print $"❌ README not found: ($readme_path)"
        exit 1
    }

    let declared_version = get_declared_yazelix_version
    let sync_result = (sync_readme_surface $readme_path $declared_version)
    let title_changed = $sync_result.title_changed
    let series_changed = $sync_result.series_changed

    if (not $title_changed) and (not $series_changed) {
        print $"✅ README version marker and generated latest-series block already match ($declared_version)"
        return
    }

    print $"✅ Synced README title/version marker and generated latest-series block for ($declared_version)"
}

def resolve_update_canary_selection [requested: list<string>] {
    let available = ["default", "shell_layout"]

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

def materialize_update_canaries [selected: list<string>] {
    let default_config_path = ((require_yazelix_repo_root) | path join "yazelix_default.toml")
    if not ($default_config_path | path exists) {
        error make {msg: $"Default config not found: ($default_config_path)"}
    }

    let template_surface = (load_config_surface_from_main $default_config_path)
    let template = $template_surface.merged_config
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
                        description: "default v15 runtime config"
                    }
                }
                "shell_layout" => {
                    let config_dir = ($temp_dir | path join "shell_layout")
                    let config_path = (get_main_user_config_path $config_dir)
                    mkdir $config_dir
                    let copied = (copy_default_config_surfaces $default_config_path $config_path)
                    let config = (
                        $template
                        | upsert shell.default_shell "zsh"
                        | upsert editor.command "nvim"
                        | upsert editor.enable_sidebar false
                    )
                    $config | to toml | save --force --raw $copied.config_path
                    {
                        name: "shell_layout"
                        config_path: $config_path
                        description: "zsh entry, neovim editor, no-sidebar layout"
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

def run_update_canary [canary: record] {
    let yzx_script = ((require_yazelix_repo_root) | path join "nushell" "scripts" "core" "yazelix.nu")
    let refresh_command = $"use \"($yzx_script)\" *; yzx refresh --force --verbose"

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

def run_update_canaries [selected: list<string>] {
    let context = materialize_update_canaries $selected
    let results = try {
        (
            $context.canaries
            | each { |canary|
                print $"🧪 Canary: ($canary.name) — ($canary.description)"
                run_update_canary $canary
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
    let failures = ($results | where {|result| not $result.ok })
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

def resolve_update_activation_mode [requested: string] {
    let normalized = ($requested | default "" | into string | str trim | str downcase)
    let available = ["installer", "home_manager", "none"]

    if $normalized in $available {
        return $normalized
    }

    let available_text = ($available | str join ", ")
    error make {msg: $"Unknown activation mode: ($requested). Expected one of: ($available_text)"}
}

export def resolve_requested_update_activation_mode [requested?: string, canary_only: bool = false] {
    let normalized = ($requested | default "" | into string | str trim)

    if $canary_only {
        if ($normalized | is-not-empty) {
            resolve_update_activation_mode $normalized | ignore
        }
        return ""
    }

    if ($normalized | is-empty) {
        error make {msg: "yzx dev update now requires --activate installer|home_manager|none unless you are using --canary-only."}
    }

    resolve_update_activation_mode $normalized
}

def resolve_home_manager_flake_dir [candidate: string] {
    let expanded = ($candidate | path expand)
    let flake_file = ($expanded | path join "flake.nix")

    if not ($expanded | path exists) {
        print $"❌ Home Manager flake directory not found: ($expanded)"
        exit 1
    }

    if not ($flake_file | path exists) {
        print $"❌ Home Manager flake is missing flake.nix: ($flake_file)"
        exit 1
    }

    $expanded
}

def build_home_manager_switch_ref [flake_dir: string, attr: string = ""] {
    let normalized_attr = ($attr | default "" | into string | str trim)
    if ($normalized_attr | is-empty) {
        $flake_dir
    } else {
        $"($flake_dir)#($normalized_attr)"
    }
}

def activate_updated_installer_runtime [repo_root: string] {
    print "🔄 Installing updated local Yazelix runtime..."
    print "   Streaming local installer activation logs \(this may take a while when Nix rebuilds\)..."

    let exit_code = (do {
        cd $repo_root
        ^nix run -L .#install
        ($env.LAST_EXIT_CODE? | default 0)
    })

    if $exit_code != 0 {
        print "❌ nix run .#install failed."
        print "   Recovery: Fix the install failure, rerun `nix run .#install`, then restart Yazelix."
        exit $exit_code
    }

    print "✅ Installed runtime updated."
}

def refresh_home_manager_input_lock [flake_dir: string, input_name: string] {
    let normalized_input = ($input_name | default "" | into string | str trim)

    if ($normalized_input | is-empty) {
        print "❌ Home Manager activation requires a non-empty input name."
        exit 1
    }

    print "🔄 Refreshing Home Manager Yazelix input..."

    let result = (^nix flake update $normalized_input --flake $flake_dir | complete)
    if $result.exit_code != 0 {
        let stderr_tail = trim_output_tail ($result.stderr | default "") 25
        let stdout_tail = trim_output_tail ($result.stdout | default "") 25
        print "❌ Failed to refresh the Home Manager flake lock."
        if ($stderr_tail | is-not-empty) {
            print "   stderr tail:"
            print ($stderr_tail | lines | each { |line| $"     ($line)" } | str join "\n")
        } else if ($stdout_tail | is-not-empty) {
            print "   stdout tail:"
            print ($stdout_tail | lines | each { |line| $"     ($line)" } | str join "\n")
        }
        print $"   Recovery: Rerun `nix flake update ($normalized_input) --flake \"($flake_dir)\"` after fixing the Home Manager flake."
        exit $result.exit_code
    }

    let stdout_text = ($result.stdout | default "" | str trim)
    let stderr_text = ($result.stderr | default "" | str trim)
    if ($stdout_text | is-not-empty) {
        print $stdout_text
    }
    if ($stderr_text | is-not-empty) {
        print --stderr $stderr_text
    }

    print "✅ Home Manager flake input updated."
}

export def activate_updated_home_manager_runtime [flake_dir: string, input_name: string, attr: string = ""] {
    if (which home-manager | is-empty) {
        print "❌ home-manager not found in PATH."
        print "   Recovery: Install Home Manager first, or use `yzx dev update --activate installer` or `--activate none`."
        exit 1
    }

    let resolved_dir = (resolve_home_manager_flake_dir $flake_dir)
    let switch_ref = (build_home_manager_switch_ref $resolved_dir $attr)
    refresh_home_manager_input_lock $resolved_dir $input_name

    print "🔄 Applying updated Home Manager Yazelix configuration..."
    let result = (^home-manager switch --flake $switch_ref | complete)
    if $result.exit_code != 0 {
        let stderr_tail = trim_output_tail ($result.stderr | default "") 25
        let stdout_tail = trim_output_tail ($result.stdout | default "") 25
        print "❌ home-manager switch failed."
        if ($stderr_tail | is-not-empty) {
            print "   stderr tail:"
            print ($stderr_tail | lines | each { |line| $"     ($line)" } | str join "\n")
        } else if ($stdout_tail | is-not-empty) {
            print "   stdout tail:"
            print ($stdout_tail | lines | each { |line| $"     ($line)" } | str join "\n")
        }
        print $"   Recovery: Rerun `home-manager switch --flake \"($switch_ref)\"` after fixing the Home Manager configuration."
        exit $result.exit_code
    }

    let stdout_text = ($result.stdout | default "" | str trim)
    let stderr_text = ($result.stderr | default "" | str trim)
    if ($stdout_text | is-not-empty) {
        print $stdout_text
    }
    if ($stderr_text | is-not-empty) {
        print --stderr $stderr_text
    }

    print "✅ Home Manager configuration applied."
    {
        flake_dir: $resolved_dir
        input_name: ($input_name | str trim)
        switch_ref: $switch_ref
    }
}

export def run_dev_update_workflow [
    yes: bool = false
    no_canary: bool = false
    activate: string = ""
    home_manager_dir: string = "~/.config/home-manager"
    home_manager_input: string = "yazelix-hm"
    home_manager_attr: string = ""
    canary_only: bool = false
    canaries: list<string> = []
] {
    ensure_nix_available

    let yazelix_dir = require_yazelix_repo_root
    let selected_canaries = resolve_update_canary_selection $canaries

    if $no_canary and $canary_only {
        print "❌ --no-canary and --canary-only cannot be used together."
        exit 1
    }

    let activation_mode = try {
        resolve_requested_update_activation_mode $activate $canary_only
    } catch {|err|
        print $"❌ ($err.msg)"
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
    } else {
        print $"⚙️ Running: devenv update \(cwd: ($yazelix_dir)\)"
    }

    if not $canary_only {
        try {
            do {
                cd $yazelix_dir
                let devenv_path = (resolve_preferred_devenv_path)
                ^$devenv_path update
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
        let canary_results = run_update_canaries $selected_canaries
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
    sync_readme_version_marker
    sync_vendored_zjstatus
    sync_vendored_yazi_plugins

    match $activation_mode {
        "none" => {
            print "⚠️  No local activation was requested."
            print "✅ Inputs, canaries, runtime pins, README version marker, vendored zjstatus, and vendored Yazi plugin runtime files are in sync in the repo checkout. Review and commit the changes if everything looks good."
        }
        "installer" => {
            activate_updated_installer_runtime $yazelix_dir
            print "✅ Inputs, canaries, runtime pins, README version marker, vendored zjstatus, vendored Yazi plugin runtime files, and the local installer-owned runtime are in sync. Review and commit the changes if everything looks good."
        }
        "home_manager" => {
            let activation = (activate_updated_home_manager_runtime $home_manager_dir $home_manager_input $home_manager_attr)
            print $"✅ Inputs, canaries, runtime pins, README version marker, vendored zjstatus, vendored Yazi plugin runtime files, and the Home Manager activation at ($activation.switch_ref) are in sync. Review and commit the changes if everything looks good."
        }
    }
}
