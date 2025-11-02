#!/usr/bin/env nu
# Yazelix Performance Profiler
# Profiles launch sequence and environment setup to identify bottlenecks

use logging.nu log_to_file

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

    let yazelix_dir = "~/.config/yazelix" | path expand
    let log_file = "~/.local/share/yazelix/logs/profile.log" | path expand

    mut results = []

    # Profile initializers generation
    let init_result = (profile_step "Shell initializers generation" {
        nu $"($yazelix_dir)/nushell/scripts/setup/initializers.nu" $yazelix_dir true false "bash,nu"
        | complete
    })
    $results = ($results | append $init_result)

    # Profile config detection
    let config_result = (profile_step "Config hash computation" {
        let config_file = $"($yazelix_dir)/yazelix.nix"
        if ($config_file | path exists) {
            open $config_file | hash sha256
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

# Profile full cold launch (must be run from outside Yazelix)
export def profile_cold_launch [
    --clear_cache  # Clear devenv cache to simulate config change
] {
    # Check if we're in a Yazelix shell
    if ($env.IN_YAZELIX_SHELL? | is-not-empty) {
        print "❌ Error: Cold launch profiling must be run from a vanilla terminal\n"
        print "To profile cold launch:"
        print "  1. Open a new terminal (NOT from Yazelix)"
        print "  2. Run: nu -c 'use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx profile --cold'\n"
        return
    }

    print "🚀 Profiling cold Yazelix launch...\n"

    let yazelix_dir = "~/.config/yazelix" | path expand

    # Clear cache if requested
    if $clear_cache {
        print "🗑️  Clearing devenv cache to simulate config change..."
        rm -rf $"($yazelix_dir)/.devenv"
        print "✅ Cache cleared\n"
    }

    # Profile full devenv shell launch
    print "⏱️  Measuring devenv shell startup (this will take a few seconds)...\n"

    let start = (date now)

    # Launch devenv shell with a simple command
    bash -c $"cd ($yazelix_dir) && devenv shell -- bash -c 'echo \"Yazelix shell ready\"'" | complete

    let end = (date now)
    let duration_ms = ((($end - $start) | into int) / 1000000)

    print $"\n📊 Results:"
    print $"  Cold launch time: ($duration_ms)ms\n"

    # Performance assessment
    if $clear_cache {
        print "💡 Performance Assessment (with cache cleared):\n"
        if $duration_ms < 2000 {
            print "🚀 Excellent! Even with cache invalidation, launch is very fast."
        } else if $duration_ms < 5000 {
            print "✅ Good. This is expected after config changes (Nix re-evaluation)."
        } else if $duration_ms < 10000 {
            print "⚠️  Slower than expected. Check for:"
            print "   - Slow disk I/O"
            print "   - Large number of packages in yazelix.toml"
        } else {
            print "❌ Very slow. This may indicate a problem."
        }
    } else {
        print "💡 Performance Assessment (with cache):\n"
        if $duration_ms < 500 {
            print "🚀 Excellent! SQLite cache is working perfectly."
        } else if $duration_ms < 1500 {
            print "✅ Good. Cached launch is efficient."
        } else if $duration_ms < 3000 {
            print "⚠️  Slower than expected for cached launch."
        } else {
            print "❌ Cache may not be working. Check .devenv/ directory."
        }
    }

    # Save results
    let log_file = "~/.local/share/yazelix/logs/profile.log" | path expand
    let timestamp = (date now | format date "%Y-%m-%d %H:%M:%S")
    let cache_status = if $clear_cache { "no-cache" } else { "cached" }

    $"($timestamp) - Cold launch \(($cache_status)\): ($duration_ms)ms\n" | save --append $log_file
    print $"\n📝 Results saved to: ($log_file)"
}

# Profile full launch sequence
export def profile_launch [] {
    print "🚀 Profiling Yazelix launch sequence...\n"

    mut all_results = []

    # Check if we're in a Yazelix shell already
    if ($env.IN_YAZELIX_SHELL? | is-not-empty) {
        print "⚠️  Already in Yazelix shell - measurements reflect warm start\n"
        print "💡 For cold start profiling, use: yzx profile --cold (from vanilla terminal)\n"
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
        print "   - Many extra packages in yazelix.nix"
    }

    # Save results
    let log_file = "~/.local/share/yazelix/logs/profile.log" | path expand
    let timestamp = (date now | format date "%Y-%m-%d %H:%M:%S")

    $"($timestamp) - Total: ($total_ms)ms\n" | save --append $log_file

    print $"\n📝 Results saved to: ($log_file)"
}

# Show historical profile data
export def profile_history [] {
    let log_file = "~/.local/share/yazelix/logs/profile.log" | path expand

    if not ($log_file | path exists) {
        print "No profile history found. Run 'yzx profile' first."
        return
    }

    print "📊 Profile History:\n"
    open $log_file | lines | last 10
}
