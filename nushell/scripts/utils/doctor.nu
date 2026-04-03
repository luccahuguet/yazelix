#!/usr/bin/env nu
# Yazelix Doctor - Health check utilities

use logging.nu log_to_file
use constants.nu [PINNED_NIX_VERSION]
use common.nu [get_yazelix_config_dir get_yazelix_dir get_yazelix_runtime_dir get_yazelix_state_dir get_yazelix_runtime_reference_dir]
use config_migration_transactions.nu [recover_stale_managed_config_transactions]
use config_surfaces.nu [get_main_user_config_path reconcile_primary_config_surfaces]
use config_diagnostics.nu [apply_doctor_config_fixes build_config_diagnostic_report render_doctor_config_details]
use config_parser.nu parse_yazelix_config
use devenv_cli.nu [get_preferred_devenv_version_line is_preferred_devenv_available resolve_preferred_devenv_path]
use launch_state.nu [resolve_built_profile]
use runtime_contract_checker.nu [
    check_generated_layout
    check_launch_terminal_support
    check_launch_working_dir
    check_runtime_script
    resolve_expected_layout_path
    runtime_check_to_doctor_result
]
use ../setup/helix_config_merger.nu [build_managed_helix_config get_generated_helix_config_path get_managed_helix_user_config_path get_managed_reveal_command get_native_helix_config_path]
use ../integrations/zellij.nu debug_editor_state

def extract_first_semver [text: string] {
    let matches = ($text | parse --regex '(\d+\.\d+\.\d+)' | get -o capture0)
    if ($matches | is-empty) { "unknown" } else { $matches | first }
}

def extract_last_semver [text: string] {
    let matches = ($text | parse --regex '(\d+\.\d+\.\d+)' | get -o capture0)
    if ($matches | is-empty) { "unknown" } else { $matches | last }
}

def is_helix_editor_command [editor: string] {
    let normalized = ($editor | str trim)
    ($normalized | is-empty) or ($normalized | str ends-with "/hx") or ($normalized == "hx") or ($normalized | str ends-with "/helix") or ($normalized == "helix")
}

def managed_helix_config_has_reveal_binding [config: record] {
    let normal_keys = ($config.keys? | default {} | get -o normal | default {})
    (($normal_keys | get -o "A-r" | default "") == (get_managed_reveal_command))
}

def get_runtime_tool_version [tool: string] {
    match $tool {
        "nix" => {
            if (which nix | is-empty) { "not installed" } else {
                try {
                    let result = (^nix --version | complete)
                    if $result.exit_code != 0 { "error" } else { extract_last_semver ($result.stdout | lines | first) }
                } catch { "error" }
            }
        }
        "devenv" => {
            if not (is_preferred_devenv_available) { "not installed" } else {
                try { extract_first_semver (get_preferred_devenv_version_line) } catch { "error" }
            }
        }
        _ => "unknown"
    }
}

def build_version_drift_result [tool: string, pinned: string, runtime: string] {
    if $runtime == "not installed" {
        {
            status: "warning"
            message: $"($tool) not installed"
            details: $"Yazelix expects ($tool) ($pinned)"
            fix_available: false
        }
    } else if $runtime == "error" or $runtime == "unknown" {
        {
            status: "warning"
            message: $"Could not determine ($tool) runtime version"
            details: $"Yazelix expects ($tool) ($pinned)"
            fix_available: false
        }
    } else if $runtime != $pinned {
        {
            status: "warning"
            message: $"($tool) version drift: runtime ($runtime), Yazelix expects ($pinned)"
            details: "Version drift can cause breakage after upstream CLI or evaluation changes"
            fix_available: false
        }
    } else {
        {
            status: "ok"
            message: $"($tool) version matches Yazelix expectation: ($runtime)"
            details: null
            fix_available: false
        }
    }
}

export def get_version_drift_results [] {
    let nix_runtime = get_runtime_tool_version "nix"

    [
        (build_version_drift_result "nix" $PINNED_NIX_VERSION $nix_runtime)
    ]
}

export def print_runtime_version_drift_warning [] {
    let drift_results = (get_version_drift_results | where status == "warning")
    if ($drift_results | is-empty) {
        return
    }

    let nix_drift = ($drift_results | where message =~ '^nix version drift' | get -o 0)

    if ($nix_drift != null) {
        print $"⚠️  ($nix_drift.message)"
    }
}

# Check for conflicting Helix runtime directories based on Helix's search priority
export def check_helix_runtime_conflicts [] {
    # Helix runtime search order (highest to lowest priority):
    # 1. runtime/ sibling to $CARGO_MANIFEST_DIR (dev only - skip)
    # 2. ~/.config/helix/runtime (user config directory)  
    # 3. Explicitly configured runtime (when present)
    # 4. Package/distribution fallback runtime
    # 5. runtime/ sibling to helix executable
    
    mut conflicts = []
    mut has_high_priority_conflict = false
    
    # Check user config directory runtime (highest priority conflict)
    let user_runtime = "~/.config/helix/runtime" | path expand
    if ($user_runtime | path exists) {
        $conflicts = ($conflicts | append {
            path: $user_runtime
            priority: 2
            name: "User config runtime"
            severity: "error"
        })
        $has_high_priority_conflict = true
    }
    
    # Check executable sibling runtime (lower priority but still problematic)
    let helix_exe = try { (which hx | get path.0) } catch { null }
    let effective_runtime = (detect_effective_helix_runtime)
    if ($helix_exe | is-not-empty) {
        let exe_runtime = ($helix_exe | path dirname | path join "runtime")
        if ($exe_runtime | path exists) and ($exe_runtime != ($effective_runtime | default "")) {
            $conflicts = ($conflicts | append {
                path: $exe_runtime
                priority: 5
                name: "Executable sibling runtime"
                severity: "warning"
            })
        }
    }
    
    if ($conflicts | is-empty) {
        return {
            status: "ok"
            message: "No conflicting Helix runtime directories found"
            details: "Helix runtime search order will behave as intended"
            fix_available: false
            conflicts: []
        }
    }
    
    # Determine overall status based on highest priority conflict
    let status = if $has_high_priority_conflict { "error" } else { "warning" }
    
    let conflict_details = ($conflicts | each { |c| 
        $"($c.name): ($c.path) \(priority ($c.priority)\)"
    } | str join ", ")
    
    let message = if $has_high_priority_conflict {
        "HIGH PRIORITY: ~/.config/helix/runtime will override the intended Helix runtime"
    } else {
        "Lower priority runtime directories found"
    }
    
    let fix_commands = if $has_high_priority_conflict {
        [
            $"# Backup and remove conflicting runtime:"
            $"mv ($user_runtime) ($user_runtime).backup"
            $"# Or if you want to delete it:"
            $"rm -rf ($user_runtime)"
        ]
    } else { [] }

    {
        status: $status
        message: $message
        details: $"Conflicting runtimes: ($conflict_details). Helix searches in priority order and will use files from higher priority directories, potentially breaking syntax highlighting."
        fix_available: true   # Auto-fix with backup
        fix_commands: $fix_commands
        conflicts: $conflicts
    }
}

# Check effective Helix runtime health
# Returns all valid runtime directories (Nix splits runtime across multiple paths)
def detect_all_helix_runtimes [] {
    if (which hx | is-empty) {
        return []
    }

    try {
        let runtime_line = (
            hx --health
            | lines
            | where {|line| $line | str starts-with "Runtime directories:"}
            | first
        )
        let runtime_candidates = (
            $runtime_line
            | str replace "Runtime directories: " ""
            | split row ";"
            | each {|entry| $entry | str trim}
            | where {|entry| $entry != ""}
        )

        $runtime_candidates
        | where {|candidate| $candidate | path exists}
    } catch {
        []
    }
}

def detect_effective_helix_runtime [] {
    let all_runtimes = (detect_all_helix_runtimes)
    if ($all_runtimes | is-empty) {
        null
    } else {
        $all_runtimes | first
    }
}

export def check_helix_runtime_health [] {
    let all_runtimes = (detect_all_helix_runtimes)
    let primary_runtime = (detect_effective_helix_runtime)

    if ($primary_runtime | is-empty) {
        return {
            status: "error"
            message: "Helix runtime could not be resolved"
            details: "Helix did not report any valid runtime directory in `hx --health`"
            fix_available: false
        }
    }

    # Check for essential directories across ALL runtime directories
    # (Nix splits runtime across multiple store paths)
    let required_dirs = ["grammars", "queries", "themes"]
    let missing_dirs = ($required_dirs | where {|required_dir|
        # Check if this directory exists in ANY runtime path
        let found_in_any = ($all_runtimes | any {|runtime_path|
            $"($runtime_path)/($required_dir)" | path exists
        })
        not $found_in_any
    })
    
    if not ($missing_dirs | is-empty) {
        return {
            status: "error"
            message: $"Missing required directories: ($missing_dirs | str join ', ')"
            details: $"The effective Helix runtime at ($primary_runtime) is incomplete (note: Nix may split runtime across multiple paths)"
            fix_available: false
        }
    }

    # Count grammars across all runtime paths
    let grammar_count = ($all_runtimes | each {|runtime_path|
        try {
            (ls $"($runtime_path)/grammars" | length)
        } catch {
            0
        }
    } | math sum)
    
    if ($grammar_count < 200) {
        return {
            status: "warning"
            message: $"Only ($grammar_count) grammar files found (expected 200+)"
            details: "Some languages may not have syntax highlighting"
            fix_available: false
        }
    }

    # Check tutor file across all runtime paths
    let tutor_exists = ($all_runtimes | any {|runtime_path|
        $"($runtime_path)/tutor" | path exists
    })
    
    if not $tutor_exists {
        return {
            status: "warning"
            message: "Helix tutor file missing"
            details: "Tutorial will not be available"
            fix_available: false
        }
    }

    {
        status: "ok"
        message: $"Helix runtime healthy with ($grammar_count) grammars"
        details: $"Primary runtime directory: ($primary_runtime)"
        fix_available: false
    }
}

export def check_managed_helix_integration [] {
    let config = (try {
        parse_yazelix_config
    } catch {
        return []
    })

    let configured_editor = ($config.editor_command? | default "" | into string | str trim)
    if not (is_helix_editor_command $configured_editor) {
        return []
    }

    mut results = []

    let managed_user_config = (get_managed_helix_user_config_path)
    let native_helix_config = (get_native_helix_config_path)
    if (not ($managed_user_config | path exists)) and ($native_helix_config | path exists) {
        $results = ($results | append {
            status: "info"
            message: "Personal Helix config has not been imported into Yazelix-managed Helix"
            details: $"Native config: ($native_helix_config)\nManaged config: ($managed_user_config)\nRun `yzx import helix` if you want Yazelix-managed Helix sessions to reuse that personal config."
            fix_available: false
        })
    }

    let expected_config_result = (try {
        {
            config: (build_managed_helix_config)
            error: null
        }
    } catch {|err|
        {
            config: null
            error: $err.msg
        }
    })
    if ($expected_config_result.error | is-not-empty) {
        return ($results | append {
            status: "error"
            message: "Managed Helix config contract could not be built"
            details: $expected_config_result.error
            fix_available: false
        })
    }
    let expected_config = $expected_config_result.config

    if not (managed_helix_config_has_reveal_binding $expected_config) {
        return ($results | append {
            status: "error"
            message: "Managed Helix config contract lost the Yazelix reveal binding"
            details: "The expected managed Helix config no longer enforces `A-r = :sh yzx reveal \"%{buffer_name}\"`."
            fix_available: false
        })
    }

    let generated_config_path = (get_generated_helix_config_path)
    if not ($generated_config_path | path exists) {
        return ($results | append {
            status: "info"
            message: "Managed Helix config has not been materialized yet"
            details: $"Expected generated config: ($generated_config_path)\nThis is normal before the first managed Helix launch. Yazelix will generate it on demand."
            fix_available: false
        })
    }

    let generated_config_result = (try {
        {
            config: (open $generated_config_path)
            error: null
        }
    } catch {|err|
        {
            config: null
            error: $err.msg
        }
    })
    if ($generated_config_result.error | is-not-empty) {
        return ($results | append {
            status: "warning"
            message: "Managed Helix generated config could not be read"
            details: $"Generated config: ($generated_config_path)\nUnderlying error: ($generated_config_result.error)"
            fix_available: false
        })
    }
    let generated_config = $generated_config_result.config

    if not (managed_helix_config_has_reveal_binding $generated_config) {
        return ($results | append {
            status: "warning"
            message: "Managed Helix generated config is stale or invalid"
            details: $"Generated config: ($generated_config_path)\nExpected `A-r` to run `yzx reveal`.\nLaunch a managed Helix session again to regenerate it."
            fix_available: false
        })
    }

    $results | append {
        status: "ok"
        message: "Managed Helix reveal integration is healthy"
        details: $"Generated config: ($generated_config_path)"
        fix_available: false
    }
}

# Check environment variables
export def check_environment_variables [] {
    mut results = []
    
    # Check EDITOR
    if ($env.EDITOR? | is-empty) {
        $results = ($results | append {
            status: "warning"
            message: "EDITOR environment variable not set"
            details: "Some tools may not know which editor to use"
            fix_available: false
        })
    } else {
        $results = ($results | append {
            status: "ok" 
            message: $"EDITOR set to: ($env.EDITOR)"
            details: null
            fix_available: false
        })
    }
    
    # Check if using Helix and verify its effective runtime
    if ($env.EDITOR? | default "" | str contains "hx") {
        $results = ($results | append (check_helix_runtime_health))
    }
    
    $results
}

# Check configuration files
export def check_configuration [--recover-interrupted-transactions] {
    let config_dir = (get_yazelix_config_dir)
    let runtime_dir = (get_yazelix_runtime_dir)
    let yazelix_legacy = ($config_dir | path join "yazelix.nix")
    let surface_paths = (try {
        {
            paths: (reconcile_primary_config_surfaces $config_dir $runtime_dir)
            error: null
        }
    } catch {|err|
        {
            paths: null
            error: $err.msg
        }
    })
    
    mut results = []

    if ($surface_paths.error | is-not-empty) {
        return [{
            status: "error"
            message: "Could not reconcile Yazelix config surfaces"
            details: $surface_paths.error
            fix_available: false
        }]
    }

    let yazelix_config = $surface_paths.paths.user_config
    let yazelix_default = $surface_paths.paths.default_config
    
    if ($yazelix_config | path expand | path exists) {
        $results = ($results | append {
            status: "ok"
            message: "Using custom yazelix.toml configuration"
            details: ($yazelix_config | path expand)
            fix_available: false
        })

        if $recover_interrupted_transactions {
            let recovery = (recover_stale_managed_config_transactions $yazelix_config)
            if $recovery.recovered_count > 0 {
                $results = ($results | append {
                    status: "info"
                    message: $"Recovered ($recovery.recovered_count) interrupted managed-config transaction\(s\)"
                    details: $yazelix_config
                    fix_available: false
                })
            }
        }

        let validation_result = (try {
            {
                report: (build_config_diagnostic_report $yazelix_config $yazelix_default)
                error: null
            }
        } catch {|err|
            {
                report: null
                error: $err.msg
            }
        })

        if ($validation_result.error | is-not-empty) {
            $results = ($results | append {
                status: "error"
                message: "Could not validate yazelix.toml against the current schema"
                details: $validation_result.error
                fix_available: false
            })
        } else if ($validation_result.report.issue_count > 0) {
            let issue_count = $validation_result.report.issue_count
            $results = ($results | append {
                status: "warning"
                message: $"Stale, unsupported, or migration-aware yazelix.toml entries detected \(($issue_count) issues\)"
                details: (render_doctor_config_details $validation_result.report)
                fix_available: $validation_result.report.has_fixable_migrations
                config_diagnostic_report: $validation_result.report
            })
        }
    } else if ($yazelix_legacy | path expand | path exists) {
        $results = ($results | append {
            status: "warning"
            message: "Legacy yazelix.nix configuration detected"
            details: ($yazelix_legacy | path expand)
            fix_available: false
        })
    } else if ($yazelix_default | path expand | path exists) {
        $results = ($results | append {
            status: "info"
            message: "Using default configuration (yazelix_default.toml)"
            details: "Consider copying to yazelix.toml for customization"
            fix_available: true
        })
    } else {
        $results = ($results | append {
            status: "error"
            message: "No configuration file found"
            details: "Neither yazelix.toml nor yazelix_default.toml exists"
            fix_available: false
        })
    }
    
    $results
}

# Check shell integration
export def check_shell_integration [] {
    let yzx_available = try {
        (which yzx | is-not-empty)
    } catch {
        false
    }
    
    if $yzx_available {
        {
            status: "ok"
            message: "yzx commands available"
            details: "Shell integration working properly"
            fix_available: false
        }
    } else {
        {
            status: "warning"
            message: "yzx commands not found in PATH"
            details: "Shell integration may not be properly configured"
            fix_available: false
        }
    }
}

export def check_shared_runtime_preflight [] {
    let config_result = (try {
        {config: (parse_yazelix_config), error: null}
    } catch {|err|
        {config: null, error: $err.msg}
    })
    if ($config_result.error | is-not-empty) {
        return []
    }

    let config = $config_result.config
    let runtime_dir = (get_yazelix_runtime_dir)
    let current_dir = (try { pwd } catch { null })
    let terminals = ($config.terminals? | default ["ghostty"] | uniq)
    let manage_terminals = ($config.manage_terminals? | default true)
    let layout_path = (resolve_expected_layout_path $config)
    let built_profile = (resolve_built_profile)
    let terminal_check = if $manage_terminals and ($built_profile | is-not-empty) {
        with-env {DEVENV_PROFILE: $built_profile} {
            check_launch_terminal_support "" $terminals $manage_terminals
        }
    } else {
        check_launch_terminal_support "" $terminals $manage_terminals
    }

    mut checks = [
        (check_runtime_script ($runtime_dir | path join "nushell" "scripts" "core" "start_yazelix_inner.nu") "startup_runtime_script" "startup script" "doctor")
        (check_runtime_script ($runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu") "launch_runtime_script" "launch script" "doctor")
        (check_generated_layout $layout_path "doctor")
        $terminal_check
    ]

    if $current_dir != null {
        $checks = ($checks | prepend (check_launch_working_dir $current_dir))
    }

    $checks | each {|check| runtime_check_to_doctor_result $check }
}

def get_desktop_applications_dir [] {
    let data_home = (
        $env.XDG_DATA_HOME?
        | default "~/.local/share"
        | into string
        | str trim
    )

    ($data_home | path expand | path join "applications")
}

def get_desktop_entry_path [] {
    (get_desktop_applications_dir | path join "com.yazelix.Yazelix.desktop")
}

def resolve_realpath_or_null [target: string] {
    let result = (^readlink -f $target | complete)
    if $result.exit_code == 0 {
        let resolved = ($result.stdout | str trim)
        if ($resolved | is-empty) { null } else { $resolved }
    } else {
        null
    }
}

def path_is_symlink [target: string] {
    let result = (^bash -lc $"test -L ($target | into string | to json -r)" | complete)
    $result.exit_code == 0
}

def get_current_installed_runtime_target [] {
    let runtime_link = (get_yazelix_state_dir | path join "runtime" "current")
    resolve_realpath_or_null $runtime_link
}

def get_desktop_entry_exec [desktop_path: string] {
    if not ($desktop_path | path exists) {
        return null
    }

    let entry = (open $desktop_path --raw)
    let marker = (
        $entry
        | lines
        | where {|line| $line | str starts-with "Exec="}
        | get -o 0
    )

    if $marker == null {
        null
    } else {
        (
            $marker
            | str replace 'Exec=' ""
            | str trim
        )
    }
}

export def check_desktop_entry_freshness [] {
    let desktop_path = (get_desktop_entry_path)

    if not ($desktop_path | path exists) {
        return {
            status: "info"
            message: "Yazelix desktop entry not installed"
            details: "Run `yzx desktop install` if you want application-launcher integration."
            fix_available: false
        }
    }

    let desktop_exec = (get_desktop_entry_exec $desktop_path)
    let expected_yzx_path = (get_user_yzx_cli_path)
    let expected_exec = $"\"($expected_yzx_path)\" desktop launch"

    if $desktop_exec == null {
        return {
            status: "warning"
            message: "Yazelix desktop entry is invalid"
            details: "The installed desktop entry has no Exec line. Repair with `yzx desktop install`."
            fix_available: false
        }
    }

    if $desktop_exec != $expected_exec {
        return {
            status: "warning"
            message: "Yazelix desktop entry does not use the stable launcher path"
            details: $"Desktop entry Exec: ($desktop_exec)\nExpected Exec: ($expected_exec)\nRepair with `yzx desktop install`."
            fix_available: false
        }
    }

    {
        status: "ok"
        message: "Yazelix desktop entry uses the stable launcher path"
        details: $desktop_path
        fix_available: false
    }
}

def get_user_yzx_cli_path [] {
    ($env.HOME | path join ".local" "bin" "yzx")
}

def get_runtime_variants [current_runtime_target?: string] {
    let runtime_ref = (get_yazelix_runtime_reference_dir)
    if $current_runtime_target == null {
        [$runtime_ref] | uniq
    } else {
        [$runtime_ref, $current_runtime_target] | uniq
    }
}

def build_install_repair_hint [] {
    "Repair with `nix run github:luccahuguet/yazelix#install`."
}

def get_required_shell_hook_checks [current_runtime_target?: string] {
    let runtime_variants = (get_runtime_variants $current_runtime_target)
    let yzx_cli_path = (get_user_yzx_cli_path)
    [
        {
            shell: "bash"
            file: ($env.HOME | path join ".bashrc")
            acceptable_groups: (
                $runtime_variants
                | each {|runtime|
                    [
                        $"source \"($runtime | path join 'shells' 'bash' 'yazelix_bash_config.sh')\""
                        $"    \"($yzx_cli_path)\" \"$@\""
                    ]
                }
            )
        }
        {
            shell: "nushell"
            file: ($env.HOME | path join ".config" "nushell" "config.nu")
            acceptable_groups: (
                $runtime_variants
                | each {|runtime|
                    [
                        $"source \"($runtime | path join 'nushell' 'config' 'config.nu')\""
                        $"use ($runtime | path join 'nushell' 'scripts' 'core' 'yazelix.nu') *"
                    ]
                }
            )
        }
    ]
}

def evaluate_required_shell_hook [hook: record] {
    if not ($hook.file | path exists) {
        return {
            shell: $hook.shell
            status: "missing"
            file: $hook.file
        }
    }

    let content = (open $hook.file --raw)
    let matches_any_group = (
        $hook.acceptable_groups
        | any {|group|
            $group | all {|line| $content | str contains $line }
        }
    )
    if $matches_any_group {
        {
            shell: $hook.shell
            status: "current"
            file: $hook.file
        }
    } else {
        {
            shell: $hook.shell
            status: "outdated"
            file: $hook.file
        }
    }
}

export def check_install_artifact_staleness [] {
    mut results = []

    let runtime_link = (get_yazelix_state_dir | path join "runtime" "current")
    let current_runtime_target = (get_current_installed_runtime_target)

    if not ($runtime_link | path exists) and (not (path_is_symlink $runtime_link)) {
        $results = ($results | append {
            status: "warning"
            message: "Installed Yazelix runtime link is missing"
            details: $"Expected runtime link: ($runtime_link)\n(build_install_repair_hint)"
            fix_available: false
        })
    } else if $current_runtime_target == null {
        $results = ($results | append {
            status: "warning"
            message: "Installed Yazelix runtime link is broken"
            details: $"Runtime link exists but does not resolve to a valid runtime: ($runtime_link)\n(build_install_repair_hint)"
            fix_available: false
        })
    } else {
        $results = ($results | append {
            status: "ok"
            message: "Installed Yazelix runtime link is healthy"
            details: $"($runtime_link) -> ($current_runtime_target)"
            fix_available: false
        })
    }

    let yzx_cli_path = (get_user_yzx_cli_path)
    let yzx_cli_target = (resolve_realpath_or_null $yzx_cli_path)
    let expected_yzx_targets = (
        get_runtime_variants $current_runtime_target
        | each {|runtime| $runtime | path join "shells" "posix" "yzx_cli.sh" }
    )
    let expected_yzx_targets_resolved = (
        $expected_yzx_targets
        | each {|target| resolve_realpath_or_null $target }
        | compact
    )
    let all_expected_yzx_targets = ($expected_yzx_targets | append $expected_yzx_targets_resolved | uniq)

    if not ($yzx_cli_path | path exists) and (not (path_is_symlink $yzx_cli_path)) {
        $results = ($results | append {
            status: "warning"
            message: "Installed yzx CLI shim is missing"
            details: $"Expected CLI path: ($yzx_cli_path)\n(build_install_repair_hint)"
            fix_available: false
        })
    } else if $yzx_cli_target == null {
        $results = ($results | append {
            status: "warning"
            message: "Installed yzx CLI shim is broken"
            details: $"The yzx shim exists but does not resolve cleanly: ($yzx_cli_path)\n(build_install_repair_hint)"
            fix_available: false
        })
    } else if not ($all_expected_yzx_targets | any {|target| $yzx_cli_target == $target }) {
        $results = ($results | append {
            status: "warning"
            message: "Installed yzx CLI shim is stale"
            details: $"yzx target: ($yzx_cli_target)\nExpected one of: ($all_expected_yzx_targets | str join ', ')\n(build_install_repair_hint)"
            fix_available: false
        })
    } else {
        $results = ($results | append {
            status: "ok"
            message: "Installed yzx CLI shim matches the current runtime"
            details: $"($yzx_cli_path) -> ($yzx_cli_target)"
            fix_available: false
        })
    }

    let shell_hook_results = (
        get_required_shell_hook_checks $current_runtime_target
        | each {|hook| evaluate_required_shell_hook $hook }
    )
    for hook in $shell_hook_results {
        if $hook.status == "current" {
            $results = ($results | append {
                status: "ok"
                message: $"Required ($hook.shell) Yazelix hook is current"
                details: $hook.file
                fix_available: false
            })
        } else if $hook.status == "outdated" {
            $results = ($results | append {
                status: "warning"
                message: $"Required ($hook.shell) Yazelix hook is stale"
                details: $"Config file: ($hook.file)\n(build_install_repair_hint)"
                fix_available: false
            })
        } else if $hook.status == "missing" {
            $results = ($results | append {
                status: "warning"
                message: $"Required ($hook.shell) Yazelix hook is missing"
                details: $"Config file: ($hook.file)\n(build_install_repair_hint)"
                fix_available: false
            })
        }
    }

    $results
}

# Check log files
export def check_log_files [] {
    let logs_dir = ((get_yazelix_dir) | path join "logs")
    let logs_path = ($logs_dir | path expand)

    if not ($logs_path | path exists) {
        return {
            status: "info"
            message: "No logs directory found"
            details: "Logs will be created when needed"
            fix_available: false
        }
    }

    let large_logs = try {
        (ls $logs_path | where type == file and size > 10MB)
    } catch {
        []
    }

    if not ($large_logs | is-empty) {
        let large_files = ($large_logs | get name | path basename | str join ", ")
        {
            status: "warning"
            message: $"Large log files found: ($large_files)"
            details: "Consider cleaning up logs to improve performance"
            fix_available: true
        }
    } else {
        {
            status: "ok"
            message: "Log files are reasonable size"
            details: $"Logs directory: ($logs_path)"
            fix_available: false
        }
    }
}

def is_devenv_installed [] {
    is_preferred_devenv_available
}

# Check devenv availability inside the installed Yazelix runtime contract
export def check_devenv_installation [] {
    if (is_devenv_installed) {
        let version = try { (get_preferred_devenv_version_line | str trim) } catch { "unknown" }
        let path = try { resolve_preferred_devenv_path } catch { "unknown" }
        {
            status: "ok"
            message: $"devenv available: ($version)"
            details: $"Selected CLI: ($path)"
            fix_available: false
        }
    } else {
        {
            status: "warning"
            message: "devenv missing from the installed Yazelix runtime"
            details: "Repair with `yzx update runtime`, then rerun the affected launch or refresh command."
            fix_available: true
        }
    }
}

export def check_zellij_plugin_health [] {
    if ($env.ZELLIJ? | is-empty) {
        return [{
            status: "info"
            message: "Zellij plugin health check skipped (not inside Zellij)"
            details: "Run `yzx doctor` from inside the affected Yazelix session to verify Yazelix orchestrator permissions and managed pane detection."
            fix_available: false
        }]
    }

    let plugin_state = try {
        debug_editor_state
    } catch {|err|
        return [{
            status: "warning"
            message: "Could not contact the Yazelix pane-orchestrator plugin"
            details: $"Run this from inside the affected Yazelix session after fully restarting it. Underlying error: ($err.msg)"
            fix_available: false
        }]
    }

    if ($plugin_state.raw? | is-not-empty) {
        return [{
            status: "warning"
            message: "Yazelix pane-orchestrator returned an unexpected response"
            details: $"Unexpected payload: ($plugin_state.raw)"
            fix_available: false
        }]
    }

    let config = parse_yazelix_config
    let sidebar_enabled = ($config.enable_sidebar? | default true)
    build_zellij_plugin_health_results $plugin_state $sidebar_enabled
}

export def build_zellij_plugin_health_results [plugin_state: record, sidebar_enabled: bool] {
    mut results = []

    if not ($plugin_state.permissions_granted? | default false) {
        $results = ($results | append {
            status: "error"
            message: "Yazelix pane-orchestrator plugin permissions not granted"
            details: "Grant the required Yazelix Zellij plugin permissions: focus the top zjstatus bar and press `y` if it prompts, and also answer yes to the Yazelix orchestrator permission popup. If permission state gets out of sync after an update, run `yzx repair zellij-permissions` and restart Yazelix. Yazelix workspace bindings like `Alt+m`, `Alt+y`, `Ctrl+y`, `Alt+r`, `Alt+[`, and `Alt+]` depend on the orchestrator."
            fix_available: false
        })
    } else {
        $results = ($results | append {
            status: "ok"
            message: "Yazelix pane-orchestrator permissions granted"
            details: "The orchestrator plugin can handle Yazelix tab and pane actions in this Zellij session."
            fix_available: false
        })
    }

    if ($plugin_state.active_tab_position? | default null) == null {
        $results = ($results | append {
            status: "warning"
            message: "Yazelix pane-orchestrator does not see an active tab yet"
            details: "The plugin may still be initializing. Wait a moment and rerun `yzx doctor` inside this Yazelix session."
            fix_available: false
        })
        return $results
    }

    if $sidebar_enabled {
        if ($plugin_state.sidebar_pane_id? | is-empty) {
            $results = ($results | append {
                status: "warning"
                message: "Managed sidebar pane not detected in the current tab"
                details: "If sidebar mode is enabled, `Alt+y` and `Ctrl+y` may not work until the current tab uses a Yazelix sidebar layout."
                fix_available: false
            })
        } else {
            $results = ($results | append {
                status: "ok"
                message: $"Managed sidebar pane detected: ($plugin_state.sidebar_pane_id)"
                details: $"Layout state: ($plugin_state.active_swap_layout_name? | default 'unknown')"
                fix_available: false
            })
        }
    }

    if ($plugin_state.editor_pane_id? | is-empty) {
        $results = ($results | append {
            status: "info"
            message: "Managed editor pane not detected in the current tab"
            details: "This is normal until you open a managed Helix or Neovim editor pane in the current tab. An editor started manually from an ordinary shell pane does not count as the managed editor pane."
            fix_available: false
        })
    } else {
        $results = ($results | append {
            status: "ok"
            message: $"Managed editor pane detected: ($plugin_state.editor_pane_id)"
            details: null
            fix_available: false
        })
    }

    $results
}

# Fix conflicting Helix runtime
export def fix_helix_runtime_conflicts [conflicts: list] {
    mut success = true
    
    for $conflict in $conflicts {
        if $conflict.severity == "error" {
            let backup_path = $"($conflict.path).backup"
            
            let move_result = try {
                mv $conflict.path $backup_path
                print $"✅ Moved ($conflict.name) from ($conflict.path) to ($backup_path)"
                true
            } catch {
                print $"❌ Failed to move ($conflict.name) from ($conflict.path)"
                false
            }
            
            if not $move_result {
                $success = false
            }
        }
    }
    
    $success
}

# Clean large log files
export def fix_large_logs [] {
    let logs_dir = ((get_yazelix_dir) | path join "logs")
    let logs_path = ($logs_dir | path expand)
    
    if not ($logs_path | path exists) {
        return true
    }
    
    try {
        let large_logs = (ls $logs_path | where type == file and size > 10MB)
        
        for $log in $large_logs {
            rm $log.name
            print $"✅ Removed large log file: ($log.name | path basename)"
        }
        
        return true
    } catch {
        print "❌ Failed to clean log files"
        return false
    }
}

# Create yazelix.toml from default
export def fix_create_config [] {
    use ./config_surfaces.nu [copy_default_config_surfaces]
    let yazelix_config_dir = (get_yazelix_config_dir)
    let yazelix_runtime_dir = (get_yazelix_runtime_dir)
    let yazelix_config = (get_main_user_config_path $yazelix_config_dir)
    let yazelix_default = ($yazelix_runtime_dir | path join "yazelix_default.toml")

    try {
        copy_default_config_surfaces ($yazelix_default | path expand) ($yazelix_config | path expand) | ignore
        print $"✅ Created yazelix.toml from template"
        return true
    } catch {
        print "❌ Failed to create yazelix.toml"
        return false
    }
}


# Main doctor function
export def run_doctor_checks [verbose: bool = false, fix: bool = false] {
    print "🔍 Running Yazelix Health Checks...\n"
    
    # Collect all checks
    mut all_results = []

    # Runtime conflicts check
    $all_results = ($all_results | append (check_helix_runtime_conflicts))

    # Environment variables
    $all_results = ($all_results | append (check_environment_variables))

    # Managed Helix contract
    $all_results = ($all_results | append (check_managed_helix_integration))

    # Configuration
    $all_results = ($all_results | append (check_configuration --recover-interrupted-transactions=$fix))

    # Shared runtime preflight overlap with launch-facing checks
    $all_results = ($all_results | append (check_shared_runtime_preflight))

    # Shell integration
    $all_results = ($all_results | append (check_shell_integration))

    # Desktop entry freshness
    $all_results = ($all_results | append (check_desktop_entry_freshness))

    # Other repairable install artifacts
    $all_results = ($all_results | append (check_install_artifact_staleness))

    # Log files
    $all_results = ($all_results | append (check_log_files))

    # devenv installation (performance optimization)
    $all_results = ($all_results | append (check_devenv_installation))

    # Runtime drift against Yazelix pinned expectations
    $all_results = ($all_results | append (get_version_drift_results))

    # Zellij session-local plugin health
    $all_results = ($all_results | append (check_zellij_plugin_health))

    # Display results
    let errors = ($all_results | where status == "error")
    let warnings = ($all_results | where status == "warning") 
    
    # Show results
    for $result in $all_results {
        match $result.status {
            "ok" => { print $"✅ ($result.message)" }
            "info" => { print $"ℹ️  ($result.message)" }
            "warning" => { print $"⚠️  ($result.message)" }
            "error" => { print $"❌ ($result.message)" }
        }
        
        if $verbose and ($result.details | is-not-empty) {
            print $"   ($result.details)"
        }
    }
    
    print ""
    
    # Summary
    if not ($errors | is-empty) {
        print $"❌ Found ($errors | length) errors"
    }
    
    if not ($warnings | is-empty) {
        print $"⚠️  Found ($warnings | length) warnings"
    }
    
    if ($errors | is-empty) and ($warnings | is-empty) {
        print "🎉 All checks passed! Yazelix is healthy."
        return
    }
    
    # Show manual fix commands for critical issues
    let runtime_conflicts = ($all_results | where status == "error" and message =~ "runtime")
    if not ($runtime_conflicts | is-empty) {
        for $conflict in $runtime_conflicts {
            if ($conflict.fix_commands? | is-not-empty) {
                print "\n🔧 To fix runtime conflicts, run these commands:"
                for $cmd in $conflict.fix_commands {
                    print $"  ($cmd)"
                }
            }
        }
    }
    
    # Auto-fix if requested
    if $fix {
        print "\n🔧 Attempting to auto-fix issues...\n"
        
        # Fix runtime conflicts (with backup)
        let runtime_conflicts = ($all_results | where status in ["error", "warning"] and message =~ "runtime")
        for $conflict in $runtime_conflicts {
            if $conflict.fix_available and ($conflict.conflicts? | is-not-empty) {
                fix_helix_runtime_conflicts $conflict.conflicts
            }
        }
        
        # Fix large logs
        let log_issues = ($all_results | where status == "warning" and message =~ "log")
        if not ($log_issues | is-empty) {
            fix_large_logs
        }
        
        # Fix missing config
        let config_issues = ($all_results | where status == "info" and message =~ "default")
        if not ($config_issues | is-empty) {
            fix_create_config
        }

        let migration_issues = ($all_results | where config_diagnostic_report? != null)
        for $issue in $migration_issues {
            let report = $issue.config_diagnostic_report
            if $report.has_fixable_migrations {
                let apply_result = (apply_doctor_config_fixes $report)
                if $apply_result.status == "applied" {
                    print $"✅ Applied ($apply_result.applied_count) config migration fix\(es\) with backup: ($apply_result.backup_path)"
                    if ($apply_result.pack_backup_path? | is-not-empty) {
                        print $"✅ Backed up previous pack config to: ($apply_result.pack_backup_path)"
                    }
                    if ($apply_result.pack_config_path? | is-not-empty) and ($apply_result.pack_backup_path? | is-empty) and (($apply_result.pack_config_path | path exists)) {
                        print $"✅ Wrote pack config to: ($apply_result.pack_config_path)"
                    }
                }
            }
        }

        print "\n✅ Auto-fix completed. Run 'yzx doctor' again to verify."
    } else if (($all_results | where {|result| $result.fix_available } | is-not-empty)) {
        print "\n💡 Some issues can be auto-fixed. Run 'yzx doctor --fix' to resolve them."
    }
}
