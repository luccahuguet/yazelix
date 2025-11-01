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

# Profile Nix environment evaluation
export def profile_nix_eval [] {
    print "📊 Profiling Nix evaluation..."

    let yazelix_dir = "~/.config/yazelix" | path expand

    # Profile nix develop evaluation
    let nix_profile = (profile_step "Nix flake evaluation" {
        bash -c $"cd ($yazelix_dir) && time nix develop --impure --command echo 'ready' 2>&1"
        | complete
    })

    # Parse the time output
    let output = $nix_profile.duration_ms

    {
        step: "Nix develop --impure"
        duration_ms: $output
        note: "First evaluation or config changed"
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

# Profile full launch sequence
export def profile_launch [
    --detailed(-d)  # Show detailed breakdown
] {
    print "🚀 Profiling Yazelix launch sequence...\n"

    mut all_results = []

    # Check if we're in a Yazelix shell already
    if ($env.IN_YAZELIX_SHELL? | is-not-empty) {
        print "⚠️  Already in Yazelix shell - measurements may not reflect cold start\n"
    }

    # Profile environment setup
    print "📋 Measuring environment setup components..."
    let env_results = (profile_environment_setup)
    $all_results = ($all_results | append $env_results)

    # Profile Nix evaluation (expensive, optional)
    if $detailed {
        print "\n⏱️  Measuring Nix evaluation (this will take ~4s)..."
        let nix_result = (profile_nix_eval)
        $all_results = ($all_results | append $nix_result)
    }

    # Display results
    print "\n📊 Profile Results:\n"
    print "╭─────────────────────────────────────────────┬──────────────╮"
    print "│ Step                                        │ Duration (ms)│"
    print "├─────────────────────────────────────────────┼──────────────┤"

    let total_ms = ($all_results | get duration_ms | math sum)

    for $result in $all_results {
        let step_padded = ($result.step | fill -a right -w 43)
        let duration = ($result.duration_ms | into string | fill -a left -w 12)
        print $"│ ($step_padded) │ ($duration) │"

        if ($result.note? | is-not-empty) {
            let note = ($result.note | fill -a right -w 43)
            print $"│   ↳ ($note) │              │"
        }
    }

    print "├─────────────────────────────────────────────┼──────────────┤"
    let total_padded = ("TOTAL" | fill -a right -w 43)
    let total_duration = ($total_ms | into string | fill -a left -w 12)
    print $"│ ($total_padded) │ ($total_duration) │"
    print "╰─────────────────────────────────────────────┴──────────────╯"

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

# Profile a specific component
export def profile_component [
    component: string  # Component to profile: nix, env, init, hooks
] {
    match $component {
        "nix" => { profile_nix_eval }
        "env" => { profile_environment_setup }
        _ => {
            error make {
                msg: $"Unknown component: ($component)"
                label: {
                    text: "Valid components: nix, env"
                }
            }
        }
    }
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
