#!/usr/bin/env nu
# Yazelix Performance Profiler
# Profiles launch sequence and environment setup to identify bottlenecks

use logging.nu log_to_file
use common.nu [ensure_yazelix_runtime_project_dir get_max_cores get_max_jobs get_yazelix_dir get_yazelix_nix_config]
use config_parser.nu [parse_yazelix_config]
use config_surfaces.nu get_main_user_config_path
use devenv_cli.nu resolve_preferred_devenv_path

# Profile a single step with timing
def profile_step [name: string, code: closure] {
    let start = (date now)
    do $code
    let end = (date now)
    let duration = ($end - $start)

    # Convert nanoseconds to milliseconds
    let duration_ms = (($duration | into int) / 1000000)

    {
        step: $name
        duration_ms: $duration_ms
    }
}

# Profile environment setup components
export def profile_environment_setup [] {
    print "📊 Profiling environment setup components..."

    let yazelix_dir = (get_yazelix_dir)
    mut results = []

    # Profile initializers generation
    let init_result = (profile_step "Shell initializers generation" {
        nu $"($yazelix_dir)/nushell/scripts/setup/initializers.nu" $yazelix_dir true false "bash,nu"
        | complete
    })
    $results = ($results | append $init_result)

    # Profile config detection
    let config_result = (profile_step "Config hash computation" {
        let primary_config = (get_main_user_config_path $yazelix_dir)
        if ($primary_config | path exists) {
            open --raw $primary_config | hash sha256
        }
    })
    $results = ($results | append $config_result)

    # Profile directory creation
    let dir_result = (profile_step "XDG directory setup" {
        let state_dir = "~/.local/share/yazelix/state" | path expand
        mkdir $state_dir
    })
    $results = ($results | append $dir_result)

    $results
}

# Profile cold launch from vanilla terminal (emulates desktop entry or fresh terminal launch)
export def profile_cold_launch [
    --clear-cache  # Toggle yazelix.toml option and clear cache to force full Nix re-evaluation (simulates config change)
] {
    # Check if we're in a Yazelix shell
    if ($env.IN_YAZELIX_SHELL? | is-not-empty) {
        print "❌ Error: Cold launch profiling must be run from a vanilla terminal\n"
        print "To profile cold launch (emulates desktop entry or fresh terminal):"
        print "  1. Open a new terminal (NOT from Yazelix)"
        let core_script = ((get_yazelix_dir) | path join "nushell" "scripts" "core" "yazelix.nu")
        print $"  2. Run: nu -c 'use \"($core_script)\" *; yzx dev profile --cold'\n"
        return
    }

    print "🚀 Profiling cold Yazelix launch (desktop entry / vanilla terminal)...\n"

    let yazelix_dir = (get_yazelix_dir)

    # Clear cache if requested
    if $clear_cache {
        print "🗑️  Clearing .devenv cache...\n"
        let devenv_cache = $"((ensure_yazelix_runtime_project_dir))/.devenv"
        if ($devenv_cache | path exists) {
            rm -rf $devenv_cache
        }
    }

    mut results = []

    print "⏱️  Measuring cold launch (this will take a few seconds)...\n"

    # Profile the actual launch command that works
    let launch_start = (date now)
    # Simulate what happens during yzx launch - run devenv shell with immediate exit
    # This is the only command that actually does the full 6s build
    let config = parse_yazelix_config
    let max_jobs = get_max_jobs ($config.max_jobs? | default "half" | into string)
    let max_cores = get_max_cores ($config.build_cores? | default "2" | into string)
    let nix_config = get_yazelix_nix_config
    let devenv_path = (resolve_preferred_devenv_path)
    let shell_result = with-env {NIX_CONFIG: $nix_config} {
        do {
            cd $yazelix_dir
            if (which timeout | is-not-empty) {
                print --raw "exit\n" | ^timeout 15 $devenv_path --max-jobs ($max_jobs | into string) --cores ($max_cores | into string) shell | complete
            } else {
                print --raw "exit\n" | ^$devenv_path --max-jobs ($max_jobs | into string) --cores ($max_cores | into string) shell | complete
            }
        }
    }
    let launch_end = (date now)
    let launch_ms = ((($launch_end - $launch_start) | into int) / 1000000)

    if $shell_result.exit_code != 0 {
        error make {
            msg: "Cold launch profiling failed"
            label: {
                text: ($shell_result.stderr | default $shell_result.stdout | default "devenv shell exited unsuccessfully")
                span: (metadata $yazelix_dir).span
            }
        }
    }

    $results = ($results | append {
        step: "Full devenv shell build"
        duration_ms: $launch_ms
    })

    # Calculate total
    let total_ms = ($results | get duration_ms | math sum)

    # Display breakdown
    print "\n📊 Cold Launch Breakdown:\n"
    let display_results = ($results | each {|result|
        let duration = ($result.duration_ms | math round --precision 2)
        {
            Step: $result.step
            "Duration (ms)": $duration
        }
    })
    print ($display_results | table)
    print $"\nTotal: ($total_ms | math round --precision 2)ms\n"

    # Performance assessment
    if $clear_cache {
        print "💡 Performance Assessment (config change simulation):\n"
        if $total_ms < 3000 {
            print "🚀 Excellent! Even after config changes, Nix re-evaluation is very fast."
        } else if $total_ms < 8000 {
            print "✅ Good. This is expected after config changes (Nix re-evaluation)."
        } else if $total_ms < 15000 {
            print "⚠️  Slower than expected. Check for:"
            print "   - Slow disk I/O"
            print "   - Large number of packages in yazelix.toml"
        } else {
            print "❌ Very slow. This may indicate a problem."
        }
    } else {
        print "💡 Performance Assessment (cached launch):\n"
        if $total_ms < 500 {
            print "🚀 Excellent! devenv SQLite cache is working perfectly."
        } else if $total_ms < 1500 {
            print "✅ Good. Cached launch is efficient."
        } else if $total_ms < 3000 {
            print "⚠️  Slower than expected for cached launch."
        } else {
            print "❌ Cache may not be working. Check .devenv/ directory."
        }
    }

}

# Profile full launch sequence
export def profile_launch [] {
    print "🚀 Profiling Yazelix launch sequence...\n"

    mut all_results = []

    # Check if we're in a Yazelix shell already
    if ($env.IN_YAZELIX_SHELL? | is-not-empty) {
        print "⚠️  Already in Yazelix shell - measurements reflect warm start\n"
        print "💡 For cold start profiling, use: yzx dev profile --cold (from vanilla terminal)\n"
    }

    # Profile environment setup
    print "📋 Measuring environment setup components..."
    let env_results = (profile_environment_setup)
    $all_results = ($all_results | append $env_results)

    # Display results
    print "\n📊 Profile Results:\n"

    let total_ms = ($all_results | get duration_ms | math sum)

    # Format and display results
    let display_results = ($all_results | each {|result|
        let duration = ($result.duration_ms | math round --precision 2)
        {
            Step: $result.step
            "Duration (ms)": $duration
        }
    })

    print ($display_results | table)

    # Show total
    let total_rounded = ($total_ms | math round --precision 2)
    print $"\nTotal: ($total_rounded)ms"

    # Performance assessment
    print "\n💡 Performance Assessment:\n"

    if $total_ms < 200 {
        print "\n🚀 Excellent! Your setup is well-optimized."
    } else if $total_ms < 500 {
        print "\n✅ Good performance. Environment setup is efficient."
    } else if $total_ms < 2000 {
        print "\n⚠️  Moderate performance. Consider checking for slow operations."
    } else {
        print "\n❌ Slow performance detected. This may indicate:"
        print "   - First launch after config change (expected)"
        print "   - Slow disk I/O"
        print "   - Many extra packages in yazelix.toml"
    }
}
