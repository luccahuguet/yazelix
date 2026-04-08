#!/usr/bin/env nu
# Yazelix startup profiling harness

use common.nu [
    get_yazelix_state_dir
    resolve_yazelix_nu_bin
]
use repo_checkout.nu [require_yazelix_repo_root]
use runtime_project.nu [get_existing_yazelix_runtime_project_dir]
use startup_profile.nu [
    create_startup_profile_run
    load_startup_profile_report
    render_startup_profile_summary
]

def clear_startup_profile_caches [] {
    let runtime_project_dir = (get_existing_yazelix_runtime_project_dir)
    let devenv_cache = if $runtime_project_dir == null {
        null
    } else {
        ($runtime_project_dir | path join ".devenv")
    }

    if ($devenv_cache != null) and ($devenv_cache | path exists) {
        rm -rf $devenv_cache
    }

    let state_root = (get_yazelix_state_dir | path join "state")
    let cache_paths = [
        ($state_root | path join "rebuild_hash")
        ($state_root | path join "launch_state.json")
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

export def run_profiled_startup_harness [
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

export def profile_cold_launch [
    --clear-cache  # Clear runtime project and recorded state first to exercise the rebuild-heavy cold path
] {
    if (($env.IN_YAZELIX_SHELL? | default "") == "true") {
        print "❌ Error: Cold launch profiling must be run from a vanilla terminal"
        print ""
        print "To profile a cold startup path:"
        print "  1. Open a new terminal outside Yazelix"
        print "  2. Run: yzx dev profile --cold"
        return
    }

    print "🚀 Profiling cold Yazelix startup..."
    run_profiled_startup_harness "enter_cold" [] --clear-cache=$clear_cache | ignore
}

export def profile_launch [] {
    let in_yazelix_shell = (($env.IN_YAZELIX_SHELL? | default "") == "true")
    if $in_yazelix_shell {
        print "🚀 Profiling warm Yazelix startup from the current shell..."
        run_profiled_startup_harness "enter_warm" ["--skip-refresh"] | ignore
    } else {
        print "⚠️  Not currently inside a Yazelix shell."
        print "   Profiling the default current-terminal startup path instead."
        print ""
        run_profiled_startup_harness "enter_default" [] | ignore
    }
}
