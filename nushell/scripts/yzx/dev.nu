#!/usr/bin/env nu
# Development helper commands for maintainers

use ../utils/terminal_configs.nu generate_all_terminal_configs
use ../utils/common.nu [get_yazelix_dir]
use ../utils/readme_release_block.nu [sync_readme_surface]
use ../utils/issue_bead_contract.nu [
    build_imported_issue_description
    canonical_issue_bead_comment_body
    find_issue_bead_comment
    infer_issue_type_from_body
    load_contract_beads
    load_contract_github_issues
    load_issue_comments
    plan_issue_bead_reconciliation
    plan_issue_bead_comment_sync
]

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

def get_runtime_version_lines_from_repo_shell [] {
    let version_result = (do {
        cd (get_yazelix_dir)
        with-env {
            YAZELIX_ENV_ONLY: "true"
            YAZELIX_SHELLHOOK_SKIP_WELCOME: "true"
        } {
            ^devenv shell --no-tui -- sh -c 'printf "__YZX_NIX__\n"; nix --version; printf "__YZX_DEVENV__\n"; devenv --version' | complete
        }
    })

    if $version_result.exit_code != 0 {
        let stderr = ($version_result.stderr | str trim)
        print $"❌ Failed to resolve runtime versions from the repo shell: ($stderr)"
        exit 1
    }

    let lines = (
        $version_result.stdout
        | lines
        | where { |line|
            let trimmed = ($line | str trim)
            ($trimmed | is-not-empty) and not ($trimmed | str starts-with "Configuring shell") and not ($trimmed | str starts-with "Loading tasks") and not ($trimmed | str starts-with "Running tasks") and not ($trimmed | str starts-with "Running           ") and not ($trimmed | str starts-with "Succeeded         ") and not ($trimmed | str starts-with "No command") and not ($trimmed | str contains "Yazelix environment loaded!")
        }
    )

    mut current_tool = ""
    mut nix_version_raw = ""
    mut devenv_version_raw = ""

    for line in $lines {
        let trimmed = ($line | str trim)
        if $trimmed == "__YZX_NIX__" {
            $current_tool = "nix"
        } else if $trimmed == "__YZX_DEVENV__" {
            $current_tool = "devenv"
        } else if ($current_tool == "nix") and ($nix_version_raw | is-empty) {
            $nix_version_raw = $trimmed
        } else if ($current_tool == "devenv") and ($devenv_version_raw | is-empty) {
            $devenv_version_raw = $trimmed
        }
    }

    if ($nix_version_raw | is-empty) or ($devenv_version_raw | is-empty) {
        print "❌ Failed to capture runtime versions from the repo shell."
        exit 1
    }

    {
        nix_raw: $nix_version_raw
        devenv_raw: $devenv_version_raw
    }
}

def get_tool_version_from_repo_shell [tool: string] {
    let runtime_versions = get_runtime_version_lines_from_repo_shell

    match $tool {
        "nix" => $runtime_versions.nix_raw
        "devenv" => $runtime_versions.devenv_raw
        _ => {
            print $"❌ Unsupported runtime version request: ($tool)"
            exit 1
        }
    }
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

    print "   Resolving nix and devenv versions from the repo shell..."
    let runtime_versions = (get_runtime_version_lines_from_repo_shell)
    let nix_version_raw = $runtime_versions.nix_raw
    let devenv_version_raw = $runtime_versions.devenv_raw
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
    let constants_path = ((get_yazelix_dir) | path join "nushell" "scripts" "utils" "constants.nu")
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
    let update_script = ((get_yazelix_dir) | path join "nushell" "scripts" "dev" "update_zjstatus.nu")
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

def get_declared_yazelix_version [] {
    let constants_path = ((get_yazelix_dir) | path join "nushell" "scripts" "utils" "constants.nu")
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
    let readme_path = ((get_yazelix_dir) | path join "README.md")
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

def get_pane_orchestrator_paths [] {
    let yazelix_dir = get_yazelix_dir
    let crate_dir = ($yazelix_dir | path join "rust_plugins" "zellij_pane_orchestrator")
    let build_target = "wasm32-wasip1"
    let wasm_path = ($crate_dir | path join "target" $build_target "release" "yazelix_pane_orchestrator.wasm")
    let sync_script = ($yazelix_dir | path join "nushell" "scripts" "dev" "update_zellij_pane_orchestrator.nu")

    {
        yazelix_dir: $yazelix_dir
        crate_dir: $crate_dir
        build_target: $build_target
        wasm_path: $wasm_path
        sync_script: $sync_script
    }
}

def get_popup_runner_paths [] {
    let yazelix_dir = get_yazelix_dir
    let crate_dir = ($yazelix_dir | path join "rust_plugins" "zellij_popup_runner")
    let build_target = "wasm32-wasip1"
    let wasm_path = ($crate_dir | path join "target" $build_target "release" "yazelix_popup_runner.wasm")
    let sync_script = ($yazelix_dir | path join "nushell" "scripts" "dev" "update_zellij_popup_runner.nu")

    {
        yazelix_dir: $yazelix_dir
        crate_dir: $crate_dir
        build_target: $build_target
        wasm_path: $wasm_path
        sync_script: $sync_script
    }
}

def print_rust_wasi_enable_hint [] {
    print "   Enable the `rust_wasi` pack in ~/.config/yazelix/yazelix_packs.toml to get the pinned WASI-capable Rust toolchain."
    print '   Example: enabled = ["rust_wasi"]'
}

def get_available_update_canaries [] {
    ["default" "maximal"]
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
    let default_config_path = ((get_yazelix_dir) | path join "yazelix_default.toml")
    if not ($default_config_path | path exists) {
        error make {msg: $"Default config not found: ($default_config_path)"}
    }

    use ../utils/config_surfaces.nu [copy_default_config_surfaces load_config_surface_from_main]
    let template_surface = (load_config_surface_from_main $default_config_path)
    let template = $template_surface.merged_config
    let all_pack_names = ($template.packs.declarations | columns | sort)

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
                        description: "yazelix_default.toml + yazelix_packs_default.toml"
                    }
                }
                "maximal" => {
                    let config_dir = ($temp_dir | path join "maximal")
                    let config_path = ($config_dir | path join "yazelix.toml")
                    mkdir $config_dir
                    let copied = (copy_default_config_surfaces $default_config_path $config_path)
                    let config = ($template | upsert packs.enabled $all_pack_names)
                    ($config | reject packs) | to toml | save --force --raw $copied.config_path
                    {
                        enabled: $all_pack_names
                        declarations: ($template.packs.declarations)
                        user_packages: ($template.packs.user_packages? | default [])
                    } | to toml | save --force --raw $copied.pack_config_path
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
    let yzx_script = ((get_yazelix_dir) | path join "nushell" "scripts" "core" "yazelix.nu")
    let refresh_command = if $verbose {
        $"use \"($yzx_script)\" *; yzx refresh --force --verbose"
    } else {
        $"use \"($yzx_script)\" *; yzx refresh --force"
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
                if not $verbose {
                    print "   This might take a while. Canary output is captured unless you use --verbose."
                }
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
    input_name?: string  # Optional input name to pass through to `devenv update` (for example: devenv)
    --verbose  # Deprecated compatibility flag; maintainer update output is verbose by default
    --quiet  # Capture canary output and reduce update progress noise
    --yes      # Skip confirmation prompt
    --no-canary  # Skip canary refresh/build checks after updating devenv.lock
    --canary-only  # Run canary checks without updating devenv.lock or syncing pins
    --canaries: list<string> = []  # Canary subset: default, maximal
] {
    use ../utils/nix_detector.nu ensure_nix_available
    ensure_nix_available

    let yazelix_dir = get_yazelix_dir
    let selected_canaries = resolve_update_canary_selection $canaries
    let verbose_mode = (not $quiet)

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
    } else if $verbose_mode {
        let command_label = if ($input_name | is-not-empty) {
            $"devenv update ($input_name)"
        } else {
            "devenv update"
        }
        print $"⚙️ Running: ($command_label) \(cwd: ($yazelix_dir)\)"
    } else {
        print "🔄 Updating Yazelix inputs..."
    }

    if not $canary_only {
        try {
            do {
                cd $yazelix_dir
                if ($input_name | is-not-empty) {
                    ^devenv update $input_name
                } else {
                    ^devenv update
                }
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
        let canary_results = run_update_canaries $selected_canaries $verbose_mode
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
    print "✅ Inputs, canaries, runtime pins, README version marker, and vendored zjstatus are in sync. Review and commit the changes if everything looks good."
}

export def "yzx dev sync_terminal_configs" [] {
    let yazelix_dir = get_yazelix_dir
    let config_root = ($yazelix_dir | path join "configs/terminal_emulators")
    let generated_root = "~/.local/share/yazelix/configs/terminal_emulators" | path expand

    if not ($config_root | path exists) {
        print $"❌ Configs directory not found: ($config_root)"
        exit 1
    }

    let default_config = ($yazelix_dir | path join "yazelix_default.toml")
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

def print_issue_sync_summary [actions: list] {
    let created = ($actions | where kind == "create" | length)
    let reopened = ($actions | where kind == "reopen" | length)
    let closed = ($actions | where kind == "close" | length)
    let unchanged = ($actions | where kind == "noop" | length)

    print ""
    print "Issue sync summary:"
    print $"  Created: ($created)"
    print $"  Reopened: ($reopened)"
    print $"  Closed: ($closed)"
    print $"  Already aligned: ($unchanged)"
}

def print_issue_comment_sync_summary [actions: list] {
    let created = ($actions | where kind == "create" | length)
    let updated = ($actions | where kind == "update" | length)
    let unchanged = ($actions | where kind == "noop" | length)

    print ""
    print "Issue comment sync summary:"
    print $"  Created: ($created)"
    print $"  Updated: ($updated)"
    print $"  Already aligned: ($unchanged)"
}

def fail_issue_sync_plan [errors: list] {
    print "❌ GitHub/Beads reconciliation is blocked:"
    $errors | each { |err| print $"   - ($err)" }
    error make { msg: "issue sync plan is invalid" }
}

def create_bead_from_github_issue [issue: record] {
    let issue_type = (infer_issue_type_from_body ($issue.body? | default ""))
    let description = (build_imported_issue_description $issue)
    let output = (
        ^br create $issue.title
            --type $issue_type
            --priority 2
            --description $description
            --external-ref $issue.url
            --json
        | complete
    )

    if $output.exit_code != 0 {
        error make {
            msg: $"Failed to create bead for GitHub issue #($issue.number): ($output.stderr | str trim)"
        }
    }

    $output.stdout | from json
}

def reopen_bead_from_github_issue [action: record] {
    let issue = $action.issue
    let bead = $action.bead
    let output = (^br update $bead.id --status open --json | complete)

    if $output.exit_code != 0 {
        error make {
            msg: $"Failed to reopen bead ($bead.id) for GitHub issue #($issue.number): ($output.stderr | str trim)"
        }
    }
}

def close_bead_from_github_issue [action: record] {
    let issue = $action.issue
    let bead = $action.bead
    let output = (^br close $bead.id --reason "Closed on GitHub" --json | complete)

    if $output.exit_code != 0 {
        error make {
            msg: $"Failed to close bead ($bead.id) for GitHub issue #($issue.number): ($output.stderr | str trim)"
        }
    }
}

def format_issue_sync_action [action: record] {
    let issue = $action.issue
    match $action.kind {
        "create" => $"create bead for #($issue.number) (($issue.title))"
        "reopen" => $"reopen ($action.bead.id) for #($issue.number) (($issue.title))"
        "close" => $"close ($action.bead.id) for #($issue.number) (($issue.title))"
        _ => $"noop #($issue.number) (($issue.title))"
    }
}

def collect_issue_bead_comment_actions [github_issues: list, beads: list] {
    let issue_records = (
        $github_issues
        | where { |issue|
            let matches = (
                $beads
                | where { |bead| (($bead.external_ref? | default "") == $issue.url) }
            )
            ($matches | length) == 1
        }
    )

    $issue_records | each { |issue|
        let bead = (
            $beads
            | where { |candidate| (($candidate.external_ref? | default "") == $issue.url) }
            | first
        )
        let comments = (load_issue_comments $issue.number)
        plan_issue_bead_comment_sync $issue $bead $comments
    }
}

def create_issue_bead_comment [action: record] {
    let output = (^gh issue comment $action.issue.number --body $action.body | complete)
    if $output.exit_code != 0 {
        error make {
            msg: $"Failed to create Beads comment for GitHub issue #($action.issue.number): ($output.stderr | str trim)"
        }
    }
}

def update_issue_bead_comment [action: record] {
    let mutation = 'mutation($id: ID!, $body: String!) { updateIssueComment(input: { id: $id, body: $body }) { issueComment { id } } }'
    let output = (
        ^gh api graphql
            -f $"query=($mutation)"
            -F $"id=($action.comment.id)"
            -F $"body=($action.body)"
        | complete
    )

    if $output.exit_code != 0 {
        error make {
            msg: $"Failed to update Beads comment for GitHub issue #($action.issue.number): ($output.stderr | str trim)"
        }
    }
}

def format_issue_comment_action [action: record] {
    match $action.kind {
        "create" => $"create comment for #($action.issue.number) -> ($action.bead.id)"
        "update" => $"update comment for #($action.issue.number) -> ($action.bead.id)"
        _ => $"noop comment #($action.issue.number) -> ($action.bead.id)"
    }
}

export def "yzx dev sync_issues" [
    --dry-run  # Show the local GitHub→Beads reconciliation plan without mutating Beads
] {
    let github_issues = load_contract_github_issues
    let beads = load_contract_beads
    let plan = (plan_issue_bead_reconciliation $github_issues $beads)
    let actions = $plan.actions
    let errors = $plan.errors

    if not ($errors | is-empty) {
        fail_issue_sync_plan $errors
    }

    let mutating_actions = ($actions | where kind in ["create" "reopen" "close"])

    let initial_comment_actions = (collect_issue_bead_comment_actions $github_issues $beads)
    let mutating_comment_actions = ($initial_comment_actions | where kind in ["create" "update"])

    if $dry_run {
        print "GitHub→Beads local sync plan:"
        if ($mutating_actions | is-empty) {
            print "  No changes needed."
        } else {
            $mutating_actions | each { |action| print $"  - (format_issue_sync_action $action)" }
        }
        print ""
        print "GitHub issue comment plan:"
        if ($mutating_comment_actions | is-empty) {
            print "  No changes needed."
        } else {
            $mutating_comment_actions | each { |action| print $"  - (format_issue_comment_action $action)" }
        }
        print_issue_sync_summary $actions
        print_issue_comment_sync_summary $initial_comment_actions
        return
    }

    if ($mutating_actions | is-empty) and ($mutating_comment_actions | is-empty) {
        print "✅ GitHub issues and local Beads are already aligned."
        print_issue_sync_summary $actions
        print_issue_comment_sync_summary $initial_comment_actions
        return
    }

    if not ($mutating_actions | is-empty) {
        print "🔄 Syncing GitHub issue lifecycle into local Beads..."
    }
    for action in $mutating_actions {
        match $action.kind {
            "create" => {
                let created_bead = (create_bead_from_github_issue $action.issue)
                print $"  ✅ Created ($created_bead.id) for GitHub issue #($action.issue.number)"
                if $action.issue.state != "OPEN" {
                    close_bead_from_github_issue {
                        issue: $action.issue
                        bead: $created_bead
                    }
                    print $"  ✅ Closed ($created_bead.id) to match GitHub issue #($action.issue.number)"
                }
            }
            "reopen" => {
                reopen_bead_from_github_issue $action
                print $"  ✅ Reopened ($action.bead.id) for GitHub issue #($action.issue.number)"
            }
            "close" => {
                close_bead_from_github_issue $action
                print $"  ✅ Closed ($action.bead.id) for GitHub issue #($action.issue.number)"
            }
        }
    }

    ^br sync --flush-only

    let refreshed_github_issues = load_contract_github_issues
    let refreshed_beads = load_contract_beads
    let comment_actions = (collect_issue_bead_comment_actions $refreshed_github_issues $refreshed_beads)
    let mutating_comment_actions = ($comment_actions | where kind in ["create" "update"])

    if not ($mutating_comment_actions | is-empty) {
        print "🔄 Syncing canonical Beads comments onto GitHub issues..."
        for action in $mutating_comment_actions {
            match $action.kind {
                "create" => {
                    create_issue_bead_comment $action
                    print $"  ✅ Added Beads comment to GitHub issue #($action.issue.number)"
                }
                "update" => {
                    update_issue_bead_comment $action
                    print $"  ✅ Updated Beads comment on GitHub issue #($action.issue.number)"
                }
            }
        }
    }

    let validator = (^nu ((get_yazelix_dir) | path join ".github" "scripts" "validate_issue_bead_contract.nu") | complete)
    if $validator.exit_code != 0 {
        print ($validator.stdout | str trim)
        let stderr = ($validator.stderr | str trim)
        if ($stderr | is-not-empty) {
            print $stderr
        }
        error make { msg: "Issue sync completed but contract validation failed" }
    }

    print "✅ GitHub issue lifecycle is now synced into local Beads."
    print_issue_sync_summary $actions
    print_issue_comment_sync_summary $comment_actions
}

export def "yzx dev build_pane_orchestrator" [
    --sync  # Sync the built wasm into the repo/runtime paths after a successful build
] {
    let paths = get_pane_orchestrator_paths

    if not ($paths.crate_dir | path exists) {
        print $"❌ Pane orchestrator crate not found: ($paths.crate_dir)"
        exit 1
    }

    let missing_tools = (
        ["cargo" "rustc"]
        | where { |tool| (which $tool | is-empty) }
    )
    if ($missing_tools | is-not-empty) {
        print $"❌ Missing Rust tool(s): ($missing_tools | str join ', ')"
        print_rust_wasi_enable_hint
        exit 1
    }

    print $"🦀 Building pane orchestrator for target ($paths.build_target)..."
    let result = (do {
        cd $paths.crate_dir
        ^cargo build --target $paths.build_target --profile release | complete
    })

    if ($result.stdout | default "" | str trim | is-not-empty) {
        print ($result.stdout | str trim)
    }

    if $result.exit_code != 0 {
        let stderr_text = ($result.stderr | default "" | str trim)
        if ($stderr_text | is-not-empty) {
            print $stderr_text
        }
        if (
            ($stderr_text | str contains "can't find crate for `core`")
            or ($stderr_text | str contains "can't find crate for `std`")
            or ($stderr_text | str contains "target may not be installed")
        ) {
            print ""
            print "❌ The wasm target stdlib is not available in the current Rust toolchain."
            print_rust_wasi_enable_hint
        } else {
            print ""
            print "❌ Pane orchestrator build failed."
        }
        exit $result.exit_code
    }

    if not ($paths.wasm_path | path exists) {
        print $"❌ Build reported success, but wasm output was not found at: ($paths.wasm_path)"
        exit 1
    }

    print $"✅ Built pane orchestrator wasm: ($paths.wasm_path)"

    if $sync {
        if not ($paths.sync_script | path exists) {
            print $"❌ Sync helper not found: ($paths.sync_script)"
            exit 1
        }
        print "🔄 Syncing pane orchestrator wasm into Yazelix..."
        ^nu $paths.sync_script
    }
}

export def "yzx dev build_popup_plugin" [
    --sync  # Sync the built wasm into the repo/runtime paths after a successful build
] {
    let paths = get_popup_runner_paths

    if not ($paths.crate_dir | path exists) {
        print $"❌ Popup runner crate not found: ($paths.crate_dir)"
        exit 1
    }

    let missing_tools = (
        ["cargo" "rustc"]
        | where { |tool| (which $tool | is-empty) }
    )
    if ($missing_tools | is-not-empty) {
        print $"❌ Missing Rust tool(s): ($missing_tools | str join ', ')"
        print_rust_wasi_enable_hint
        exit 1
    }

    print $"🦀 Building popup runner for target ($paths.build_target)..."
    let result = (do {
        cd $paths.crate_dir
        ^cargo build --target $paths.build_target --profile release | complete
    })

    if ($result.stdout | default "" | str trim | is-not-empty) {
        print ($result.stdout | str trim)
    }

    if $result.exit_code != 0 {
        let stderr_text = ($result.stderr | default "" | str trim)
        if ($stderr_text | is-not-empty) {
            print $stderr_text
        }
        if (
            ($stderr_text | str contains "can't find crate for `core`")
            or ($stderr_text | str contains "can't find crate for `std`")
            or ($stderr_text | str contains "target may not be installed")
        ) {
            print ""
            print "❌ The wasm target stdlib is not available in the current Rust toolchain."
            print_rust_wasi_enable_hint
        } else {
            print ""
            print "❌ Popup runner build failed."
        }
        exit $result.exit_code
    }

    if not ($paths.wasm_path | path exists) {
        print $"❌ Build reported success, but wasm output was not found at: ($paths.wasm_path)"
        exit 1
    }

    print $"✅ Built popup runner wasm: ($paths.wasm_path)"

    if $sync {
        if not ($paths.sync_script | path exists) {
            print $"❌ Sync helper not found: ($paths.sync_script)"
            exit 1
        }
        print "🔄 Syncing popup runner wasm into Yazelix..."
        ^nu $paths.sync_script
    }
}

# Run Yazelix test suite
export def "yzx dev test" [
    --verbose(-v)  # Show detailed test output
    --new-window(-n)  # Run tests in a new Yazelix window
    --lint-only  # Run only syntax validation
    --sweep  # Run the non-visual configuration sweep only
    --visual  # Run the visual terminal sweep only
    --all(-a)  # Run the default suite plus sweep + visual lanes
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

    nu ((get_yazelix_dir) | path join "nushell" "scripts" "dev" "benchmark_terminals.nu") ...$args
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
