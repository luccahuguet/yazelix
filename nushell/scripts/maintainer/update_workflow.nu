#!/usr/bin/env nu

use ../utils/config_files.nu [copy_default_config_surfaces load_config_surface_from_main]

def get_main_user_config_path [config_root?: string] {
    let user_config_dir = if ($config_root | is-not-empty) {
        $config_root | path expand
    } else {
        ($env.HOME | path join ".config" "yazelix" "user_configs")
    }
    $user_config_dir | path join "yazelix.toml"
}

def require_yazelix_repo_root [] {
    let repo_root = ($env.YAZELIX_REPO_ROOT? | default "" | path expand)
    if ($repo_root | is-empty) or (not ($repo_root | path exists)) {
        error make {msg: "This maintainer workflow requires YAZELIX_REPO_ROOT to point at a writable Yazelix repo checkout."}
    }
    $repo_root
}

def ensure_nix_available [] {
    if (which nix | where type == "external" | is-empty) {
        print "❌ nix not found in PATH."
        print "   Install Nix, restart the shell, or enter an environment where `nix --version` works before running the maintainer update workflow."
        exit 1
    }

    let version_result = (^nix --version | complete)
    if $version_result.exit_code != 0 {
        print "❌ nix exists in PATH, but `nix --version` failed."
        let stderr = ($version_result.stderr | str trim)
        if ($stderr | is-not-empty) {
            print $stderr
        }
        exit 1
    }

    let flake_result = (^nix flake --help | complete)
    if $flake_result.exit_code != 0 {
        print "❌ nix flakes are not available in this shell."
        print "   Enable `nix-command flakes` or use the Yazelix maintainer shell before running this workflow."
        exit 1
    }
}

def update_constant_value [contents: string, key: string, new_value: string] {
    let pattern = $"export const ($key) = \"[^\"]+\""
    $contents | str replace -ra $pattern $"export const ($key) = \"($new_value)\""
}

def extract_version [value: string] {
    $value | parse --regex '(\d+\.\d+\.\d+)' | get capture0 | last | default ""
}

def eval_locked_nixpkgs_version [repo_root: string, attr_expr: string, label: string] {
    let expr = (
        [
            "let"
            $"  flake = builtins.getFlake \"path:($repo_root)\";"
            "  system = builtins.currentSystem;"
            "  pkgs = flake.inputs.nixpkgs.legacyPackages.${system};"
            $"in ($attr_expr)"
        ] | str join "\n"
    )

    let result = (^nix eval --raw --impure --extra-experimental-features "nix-command flakes" --expr $expr | complete)
    if $result.exit_code != 0 {
        let stderr = ($result.stderr | str trim)
        print $"❌ Failed to resolve ($label) version from the locked nixpkgs input: ($stderr)"
        exit 1
    }

    let version = (extract_version ($result.stdout | str trim))
    if ($version | is-empty) {
        print $"❌ Failed to parse ($label) version from: ($result.stdout | str trim)"
        exit 1
    }

    $version
}

def get_runtime_pin_versions [repo_root: string] {
    if (which nix | is-empty) {
        print "❌ nix not found in PATH."
        exit 1
    }

    print "   Resolving runtime pins from the locked nixpkgs input..."
    let nix_version = (eval_locked_nixpkgs_version $repo_root "pkgs.nixVersions.latest.version" "Nix")
    let nushell_version = (eval_locked_nixpkgs_version $repo_root "pkgs.nushell.version" "Nushell")

    {
        nix_version: $nix_version
        nushell_version: $nushell_version
    }
}

def sync_runtime_pins [] {
    let repo_root = require_yazelix_repo_root
    let constants_path = ($repo_root | path join "nushell" "scripts" "utils" "constants.nu")
    if not ($constants_path | path exists) {
        print $"❌ Constants file not found: ($constants_path)"
        exit 1
    }

    let runtime_pins = get_runtime_pin_versions $repo_root
    let contents = (open $constants_path)
    let updated = (
        update_constant_value
            (update_constant_value $contents "PINNED_NIX_VERSION" $runtime_pins.nix_version)
            "PINNED_NUSHELL_VERSION"
            $runtime_pins.nushell_version
    )

    if $updated == $contents {
        print $"✅ Runtime pins unchanged: nix ($runtime_pins.nix_version), nushell ($runtime_pins.nushell_version)"
        return
    }

    $updated | save $constants_path --force
    print $"✅ Updated runtime pins: nix ($runtime_pins.nix_version), nushell ($runtime_pins.nushell_version)"
}

def copy_zjstatus_from_store [store_root: string, target_dir: string] {
    let store_path = ($store_root | path join "bin" "zjstatus.wasm")
    if not ($store_path | path exists) {
        print $"❌ zjstatus wasm not found at: ($store_path)"
        exit 1
    }

    let byte_len = (open --raw $store_path | length)
    if $byte_len < 1024 {
        print $"❌ Nix-provided zjstatus wasm is too small to be valid \(size=($byte_len) bytes\)"
        exit 1
    }

    let target_path = ($target_dir | path join "zjstatus.wasm")
    let tmp_path = $"($target_path).tmp"
    try { cp --force $store_path $tmp_path } catch {|err|
        print $"❌ Failed to write temporary zjstatus file: ($err.msg)"
        exit 1
    }
    mv --force $tmp_path $target_path

    {
        target_path: $target_path
        byte_len: $byte_len
    }
}

def resolve_current_system [] {
    let system_result = (^nix eval --impure --raw --expr "builtins.currentSystem" | complete)
    if $system_result.exit_code != 0 {
        print "❌ Failed to resolve current Nix system"
        let stderr = ($system_result.stderr | str trim)
        if ($stderr | is-not-empty) {
            print $stderr
        }
        exit 1
    }

    let system = ($system_result.stdout | str trim)
    if ($system | is-empty) {
        print "❌ Failed to resolve current Nix system"
        exit 1
    }

    $system
}

def resolve_locked_zjstatus_store_root [repo_root: string] {
    let lock_path = ($repo_root | path join "flake.lock")
    if not ($lock_path | path exists) {
        print $"❌ flake.lock not found at: ($lock_path)"
        exit 1
    }

    let lock = (open --raw $lock_path | from json)
    let locked_zjstatus = ($lock | get nodes.zjstatus.locked)
    let owner = ($locked_zjstatus | get owner)
    let repo = ($locked_zjstatus | get repo)
    let rev = ($locked_zjstatus | get rev)
    let system = (resolve_current_system)
    let flake_ref = $"github:($owner)/($repo)/($rev)#packages.($system).default"
    let build_result = (^nix build --no-link --print-out-paths $flake_ref | complete)
    if $build_result.exit_code != 0 {
        print $"❌ Failed to build zjstatus flake ref: ($flake_ref)"
        let stderr = ($build_result.stderr | str trim)
        if ($stderr | is-not-empty) {
            print $stderr
        }
        exit 1
    }

    {
        flake_ref: $flake_ref
        store_root: ($build_result.stdout | str trim)
    }
}

def sync_vendored_zjstatus [] {
    ensure_nix_available
    let repo_root = require_yazelix_repo_root
    let target_dir = ($repo_root | path join "configs" "zellij" "plugins")
    if not ($target_dir | path exists) {
        mkdir $target_dir
    }

    print "🔄 Refreshing vendored zjstatus.wasm..."
    let package = (resolve_locked_zjstatus_store_root $repo_root)
    let zjstatus = (copy_zjstatus_from_store $package.store_root $target_dir)
    print $"✅ Updated vendored zjstatus at: ($zjstatus.target_path) \(size=($zjstatus.byte_len) bytes, source=($package.flake_ref)\)"
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
    let repo_root = require_yazelix_repo_root
    let readme_path = ($repo_root | path join "README.md")
    if not ($readme_path | path exists) {
        print $"❌ README not found: ($readme_path)"
        exit 1
    }

    let declared_version = get_declared_yazelix_version
    let result = (
        ^nix develop -c cargo run --quiet --manifest-path ($repo_root | path join "rust_core" "Cargo.toml")
            -p yazelix_core
            --bin yzx_repo_maintainer
            --
            --repo-root $repo_root
            sync-readme-surface
            --readme-path $readme_path
            --version $declared_version
        | complete
    )
    if $result.exit_code != 0 {
        print "❌ Failed to sync README surface through Rust maintainer owner."
        let stderr = ($result.stderr | str trim)
        if ($stderr | is-not-empty) {
            print $stderr
        }
        exit 1
    }

    let sync_result = ($result.stdout | from json)
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
    let repo_root = require_yazelix_repo_root
    let bridge_script = ($repo_root | path join "nushell" "scripts" "utils" "yzx_core_bridge.nu")
    let config_parent = ($canary.config_path | path dirname)
    let config_dir = if (($config_parent | path basename) == "user_configs") {
        $config_parent | path dirname
    } else {
        $config_parent
    }
    let repair_command = (
        [
            $"use \"($bridge_script)\" [build_default_yzx_core_error_surface run_yzx_core_json_command]"
            $"run_yzx_core_json_command \"($repo_root)\" \(build_default_yzx_core_error_surface\) ['runtime-materialization.repair' '--from-env' '--force'] 'Yazelix Rust runtime-materialization repair helper returned invalid JSON.' | ignore"
        ] | str join "\n"
    )

    let result = (do {
        with-env {
            YAZELIX_CONFIG_OVERRIDE: $canary.config_path
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $repair_command | complete
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
    let available = ["profile", "home_manager", "none"]

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
        error make {msg: "yzx dev update now requires --activate profile|home_manager|none unless you are using --canary-only."}
    }

    resolve_update_activation_mode $normalized
}

def load_default_profile_elements [] {
    let result = (^nix profile list --json | complete)
    if $result.exit_code != 0 {
        error make {msg: $"Failed to inspect the default Nix profile: (($result.stderr | default $result.stdout | str trim))"}
    }

    try {
        $result.stdout | from json | get -o elements | default {}
    } catch {|err|
        error make {msg: $"Failed to parse `nix profile list --json`: ($err.msg)"}
    }
}

def is_yazelix_profile_entry [row: record] {
    let name = ($row.name | default "" | into string | str trim)
    let entry = ($row.entry | default {})
    let attr_path = ($entry | get -o attrPath | default "" | into string | str trim)
    let original_url = ($entry | get -o originalUrl | default "" | into string | str trim)
    let resolved_url = ($entry | get -o url | default "" | into string | str trim)
    let store_paths = ($entry | get -o storePaths | default [])

    (
        ($name =~ '^yazelix(-\d+)?$')
        or ($attr_path =~ '(^|\\.)yazelix$')
        or ($original_url | str contains "luccahuguet/yazelix")
        or ($resolved_url | str contains "luccahuguet/yazelix")
        or ($store_paths | any {|store_path|
            let normalized = ($store_path | into string | str trim)
            ($normalized | str contains "-yazelix-") or ($normalized | str ends-with "-yazelix")
        })
    )
}

def find_default_profile_yazelix_entries [] {
    (
        load_default_profile_elements
        | transpose name entry
        | where {|row| is_yazelix_profile_entry $row }
    )
}

def refresh_repo_runtime_inputs [repo_root: string] {
    print $"⚙️ Running: nix flake update nixpkgs \(cwd: ($repo_root)\)"
    try {
        do {
            ^nix flake update nixpkgs --flake $repo_root
        }
    } catch {|err|
        print $"❌ nix flake update nixpkgs failed: ($err.msg)"
        print "   Check your network connection and flake inputs, then try again."
        exit 1
    }
    print "✅ flake.lock nixpkgs input updated."
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

def activate_updated_profile_runtime [repo_root: string] {
    print "🔄 Activating updated local Yazelix package in the default Nix profile..."
    print "   Streaming local profile activation logs \(this may take a while when Nix rebuilds\)..."

    let existing_entries = (find_default_profile_yazelix_entries)
    if (($existing_entries | length) > 0) {
        let entry_names = ($existing_entries | get name)
        let remove_command = $"nix profile remove ($entry_names | str join ' ')"
        print $"   Removing existing Yazelix profile entries before installing the local checkout: ($entry_names | str join ', ')"
        let remove_result = (^nix profile remove ...$entry_names | complete)
        if $remove_result.exit_code != 0 {
            error make {msg: $"Failed to remove existing Yazelix profile entries with `($remove_command)`: (($remove_result.stderr | default $remove_result.stdout | str trim))"}
        }
    }

    do {
        cd $repo_root
        ^nix profile add --refresh -L .#yazelix
    }
    let exit_code = ($env.LAST_EXIT_CODE? | default 0)

    if $exit_code != 0 {
        print "❌ `nix profile add --refresh .#yazelix` failed."
        print "   Recovery: Fix the local package failure, rerun `nix profile add --refresh .#yazelix`, then restart Yazelix."
        exit $exit_code
    }

    print "✅ Default-profile Yazelix package updated from the local checkout."
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
        print "   Recovery: Install Home Manager first, or use `yzx dev update --activate profile` or `--activate none`."
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
        print "⚠️  This updates Yazelix runtime inputs to latest upstream unstable revisions."
        print "   The hardened flow updates flake.lock locally, then runs canary refresh/build checks before finishing."
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
    }

    if not $canary_only {
        refresh_repo_runtime_inputs $yazelix_dir
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
        "profile" => {
            activate_updated_profile_runtime $yazelix_dir
            print "✅ Inputs, canaries, runtime pins, README version marker, vendored zjstatus, vendored Yazi plugin runtime files, and the local default-profile Yazelix package are in sync. Review and commit the changes if everything looks good."
        }
        "home_manager" => {
            let activation = (activate_updated_home_manager_runtime $home_manager_dir $home_manager_input $home_manager_attr)
            print $"✅ Inputs, canaries, runtime pins, README version marker, vendored zjstatus, vendored Yazi plugin runtime files, and the Home Manager activation at ($activation.switch_ref) are in sync. Review and commit the changes if everything looks good."
        }
    }
}
