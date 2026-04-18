#!/usr/bin/env nu
# Development helper commands for maintainers

use ../utils/common.nu [
    get_yazelix_state_dir
    resolve_yazelix_nu_bin
]
use ../maintainer/repo_checkout.nu [require_yazelix_repo_root]
use ../maintainer/issue_sync.nu run_dev_issue_sync
use ../maintainer/plugin_build.nu build_pane_orchestrator_wasm
use ../maintainer/version_bump.nu perform_version_bump
use ../maintainer/update_workflow.nu run_dev_update_workflow
use ../utils/startup_profile.nu [
    create_startup_profile_run
    load_startup_profile_report
    render_startup_profile_summary
    wait_for_startup_profile_step
]

# Development and maintainer commands
export def "yzx dev" [] {
    print "Run 'yzx dev --help' to see available maintainer subcommands"
}

# Refresh maintainer flake inputs and run update canaries
export def "yzx dev update" [
    --yes      # Skip confirmation prompt
    --no-canary  # Skip canary refresh/build checks after updating flake.lock
    --activate: string = ""  # Required unless --canary-only: installer, home_manager, or none
    --home-manager-dir: string = "~/.config/home-manager"  # Home Manager flake directory used when --activate home_manager
    --home-manager-input: string = "yazelix-hm"  # Home Manager flake input name to refresh before switch
    --home-manager-attr: string = ""  # Optional Home Manager flake output attribute appended as #attr during switch
    --canary-only  # Run canary checks without updating flake.lock or syncing pins
    --canaries: list<string> = []  # Canary subset: default, shell_layout
] {
    run_dev_update_workflow $yes $no_canary $activate $home_manager_dir $home_manager_input $home_manager_attr $canary_only $canaries
}

# Bump the tracked Yazelix version and create release metadata
export def "yzx dev bump" [
    version: string  # Version tag to release, for example v14
] {
    let result = (perform_version_bump (require_yazelix_repo_root) $version)
    print $"✅ Bumped Yazelix from ($result.previous_version) to ($result.target_version)"
    print $"   commit: ($result.commit_sha)"
    print $"   tag: ($result.tag)"
}

# Sync GitHub issue lifecycle into Beads locally
export def "yzx dev sync_issues" [
    --dry-run  # Show the local GitHub→Beads reconciliation plan without mutating Beads
] {
    run_dev_issue_sync $dry_run
}

# Build the Zellij pane-orchestrator wasm
export def "yzx dev build_pane_orchestrator" [
    --sync  # Sync the built wasm into the repo/runtime paths after a successful build
] {
    build_pane_orchestrator_wasm $sync
}

def clear_startup_profile_caches [] {
    let state_root = (get_yazelix_state_dir | path join "state")
    let cache_paths = [
        ($state_root | path join "rebuild_hash")
    ]
    for cache_path in $cache_paths {
        if ($cache_path | path exists) {
            rm -f $cache_path
        }
    }
}

def build_profile_metadata [scenario: string, clear_cache: bool, repo_root: string] {
    {
        scenario: $scenario
        clear_cache: $clear_cache
        repo_root: $repo_root
        cwd: (pwd)
        in_yazelix_shell: (($env.IN_YAZELIX_SHELL? | default "") == "true")
        host: (sys host)
    }
}

def print_startup_profile_summary [summary: record] {
    print ""
    print "📊 Startup Profile Report"
    print $"   scenario: ($summary.run.scenario)"
    print $"   report: ($summary.report_path)"
    print $"   total: ($summary.total_duration_ms)ms"
    print ""
    print (render_startup_profile_summary $summary)
}

def build_desktop_profile_command [desktop_module: string] {
    let module_literal = ($desktop_module | to nuon)
    $"use ($module_literal) *; yzx desktop launch"
}

def build_launch_profile_command [
    launch_module: string
    --terminal(-t): string = ""
    --verbose
] {
    let module_literal = ($launch_module | to nuon)
    mut command = $"use ($module_literal) *; yzx launch"
    if ($terminal | is-not-empty) {
        let terminal_literal = ($terminal | to nuon)
        $command = $"($command) --terminal ($terminal_literal)"
    }
    if $verbose {
        $command = $"($command) --verbose"
    }

    $command
}

def wait_for_profile_handoff [profile_run: record, scenario_label: string] {
    let completed = (
        wait_for_startup_profile_step
            $profile_run.report_path
            "inner"
            "zellij_handoff_ready"
            --timeout-ms=15000
    )

    if not $completed {
        error make {
            msg: $"($scenario_label) profiling timed out waiting for inner.zellij_handoff_ready. Report: ($profile_run.report_path)"
        }
    }
}

def run_dev_profile_harness [
    scenario: string
    startup_args: list<string>
    --clear-cache
] {
    let clear_cache_enabled = $clear_cache
    let repo_root = (require_yazelix_repo_root)
    let nu_bin = (resolve_yazelix_nu_bin)
    let start_script = ($repo_root | path join "nushell" "scripts" "core" "start_yazelix.nu")
    let profile_run = (create_startup_profile_run $scenario (build_profile_metadata $scenario $clear_cache_enabled $repo_root))

    if $clear_cache_enabled {
        clear_startup_profile_caches
    }

    let profile_env = (
        $profile_run.env
        | upsert YAZELIX_STARTUP_PROFILE_SKIP_WELCOME "true"
        | upsert YAZELIX_STARTUP_PROFILE_EXIT_BEFORE_ZELLIJ "true"
        | upsert YAZELIX_SHELLHOOK_SKIP_WELCOME "true"
    )
    let result = (with-env $profile_env {
        do { ^$nu_bin $start_script ...$startup_args } | complete
    })

    if $result.exit_code != 0 {
        error make {
            msg: "Startup profiling failed"
            label: {
                text: ($result.stderr | default $result.stdout | default "profiled startup exited unsuccessfully")
                span: (metadata $start_script).span
            }
        }
    }

    let summary = (load_startup_profile_report $profile_run.report_path)
    print_startup_profile_summary $summary
    $summary
}

def run_cold_profile_command [clear_cache: bool] {
    if (($env.IN_YAZELIX_SHELL? | default "") == "true") {
        print "❌ Error: Cold launch profiling must be run from a vanilla terminal"
        print ""
        print "To profile a cold startup path:"
        print "  1. Open a new terminal outside Yazelix"
        print "  2. Run: yzx dev profile --cold"
        return
    }

    print "🚀 Profiling cold Yazelix startup..."
    run_dev_profile_harness "enter_cold" [] --clear-cache=$clear_cache | ignore
}

def run_default_profile_command [] {
    let in_yazelix_shell = (($env.IN_YAZELIX_SHELL? | default "") == "true")
    if $in_yazelix_shell {
        print "🚀 Profiling warm Yazelix startup from the current shell..."
        run_dev_profile_harness "enter_warm" [] | ignore
    } else {
        print "⚠️  Not currently inside a Yazelix shell."
        print "   Profiling the default current-terminal startup path instead."
        print ""
        run_dev_profile_harness "enter_default" [] | ignore
    }
}

# Run Yazelix test suite
export def "yzx dev test" [
    --verbose(-v)  # Show detailed test output
    --new-window(-n)  # Run tests in a new Yazelix window
    --lint-only  # Run only syntax validation
    --profile  # Print timing summaries for the default suite
    --sweep  # Run the non-visual configuration sweep only
    --visual  # Run the visual terminal sweep only
    --all(-a)  # Run the default suite plus sweep + visual lanes
    --delay: int = 3  # Delay between visual terminal launches in seconds
] {
    use ../maintainer/test_runner.nu run_all_tests
    run_all_tests --verbose=$verbose --new-window=$new_window --lint-only=$lint_only --profile=$profile --sweep=$sweep --visual=$visual --all=$all --delay $delay
}

def run_desktop_profile_command [] {
    if (($env.IN_YAZELIX_SHELL? | default "") == "true") {
        print "❌ Error: Desktop launch profiling must be run from outside a Yazelix shell"
        print ""
        print "Open a new terminal outside Yazelix and run: yzx dev profile --desktop"
        return
    }

    print "🚀 Profiling desktop launch startup..."
    let repo_root = (require_yazelix_repo_root)
    let nu_bin = (resolve_yazelix_nu_bin)
    let desktop_module = ($repo_root | path join "nushell" "scripts" "yzx" "desktop.nu")
    let profile_run = (create_startup_profile_run "desktop_launch" (build_profile_metadata "desktop_launch" false $repo_root))

    let profile_env = (
        $profile_run.env
        | upsert YAZELIX_STARTUP_PROFILE_SKIP_WELCOME "true"
        | upsert YAZELIX_STARTUP_PROFILE_EXIT_BEFORE_ZELLIJ "true"
        | upsert YAZELIX_SHELLHOOK_SKIP_WELCOME "true"
    )
    let result = (with-env $profile_env {
        do { ^$nu_bin -c (build_desktop_profile_command $desktop_module) } | complete
    })

    if $result.exit_code != 0 {
        let error_output = ($result.stderr | default $result.stdout | default "desktop launch profiling exited unsuccessfully")
        error make {
            msg: $"Desktop launch profiling failed: ($error_output)"
        }
    }

    wait_for_profile_handoff $profile_run "Desktop launch"

    let summary = (load_startup_profile_report $profile_run.report_path)
    print_startup_profile_summary $summary
    $summary
}

def run_launch_profile_command [
    --clear-cache
    --terminal(-t): string = ""
    --verbose
] {
    if (($env.IN_YAZELIX_SHELL? | default "") == "true") {
        print "❌ Error: Managed launch profiling must be run from outside a Yazelix shell"
        print ""
        print "Open a new terminal outside Yazelix and run: yzx dev profile --launch"
        return
    }

    print "🚀 Profiling managed new-window launch..."
    let repo_root = (require_yazelix_repo_root)
    let nu_bin = (resolve_yazelix_nu_bin)
    let launch_module = ($repo_root | path join "nushell" "scripts" "yzx" "launch.nu")
    let profile_run = (create_startup_profile_run "managed_launch" (build_profile_metadata "managed_launch" ($clear_cache | default false) $repo_root))

    if $clear_cache {
        clear_startup_profile_caches
    }

    let profile_env = (
        $profile_run.env
        | upsert YAZELIX_STARTUP_PROFILE_SKIP_WELCOME "true"
        | upsert YAZELIX_STARTUP_PROFILE_EXIT_BEFORE_ZELLIJ "true"
        | upsert YAZELIX_SHELLHOOK_SKIP_WELCOME "true"
    )
    let launch_command = (build_launch_profile_command $launch_module --terminal=$terminal --verbose=$verbose)

    let result = (with-env $profile_env {
        do { ^$nu_bin -c $launch_command } | complete
    })

    if $result.exit_code != 0 {
        let error_output = ($result.stderr | default $result.stdout | default "managed launch profiling exited unsuccessfully")
        error make {
            msg: $"Managed launch profiling failed: ($error_output)"
        }
    }

    wait_for_profile_handoff $profile_run "Managed launch"

    let summary = (load_startup_profile_report $profile_run.report_path)
    print_startup_profile_summary $summary
    $summary
}

# Profile launch sequence and identify bottlenecks
export def "yzx dev profile" [
    --cold(-c)        # Profile cold launch from vanilla terminal (emulates desktop entry or fresh terminal launch)
    --desktop         # Profile desktop entry launch path
    --launch          # Profile managed new-window launch path
    --clear-cache     # Clear recorded runtime/project cache state first so the profiled run exercises the rebuild-heavy path
    --terminal(-t): string = ""  # Override terminal selection for launch profiling
    --verbose         # Enable verbose logging for launch profiling
] {
    if $desktop {
        run_desktop_profile_command
    } else if $launch {
        run_launch_profile_command --clear-cache=$clear_cache --terminal=$terminal --verbose=$verbose
    } else if $cold {
        run_cold_profile_command $clear_cache
    } else {
        run_default_profile_command
    }
}

# Lint Nushell scripts with repo-tuned nu-lint config from the maintainer tool surface
export def "yzx dev lint_nu" [
    --format(-f): string = "pretty"  # Output format: pretty or compact
    ...paths: string                 # Specific files or directories (default: nushell/)
] {
    let yazelix_dir = require_yazelix_repo_root
    let config_path = ($yazelix_dir | path join ".nu-lint.toml")

    if not ($config_path | path exists) {
        print $"Error: .nu-lint.toml not found at ($config_path)"
        exit 1
    }

    let targets = if ($paths | is-empty) {
        [($yazelix_dir | path join "nushell")]
    } else {
        $paths
    }
    if (which nu-lint | is-empty) {
        print "Error: nu-lint not found in PATH."
        print "Install nu-lint in your maintainer environment, then rerun this command."
        exit 1
    }

    ^nu-lint --config $config_path --format $format ...$targets
}
