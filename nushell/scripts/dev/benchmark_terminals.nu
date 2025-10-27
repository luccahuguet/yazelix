#!/usr/bin/env nu
# Benchmark terminal launch performance
# Measures how fast each terminal starts and launches Yazelix

use ../utils/terminal_launcher.nu *
use ../utils/constants.nu [SUPPORTED_TERMINALS, TERMINAL_METADATA]
use ../utils/config_parser.nu parse_yazelix_config
use sweep/sweep_config_generator.nu [generate_sweep_config, cleanup_test_config]

# Get all supported terminals (like yzx sweep terminals does)
def get_all_terminals []: nothing -> list<string> {
    $SUPPORTED_TERMINALS
}

# Benchmark a single terminal launch
def benchmark_terminal [
    terminal: string
    iterations: int = 3
]: nothing -> record {
    print $"📊 Benchmarking ($terminal)..."

    let yazelix_dir = $"($env.HOME)/.config/yazelix"
    let test_id = $"bench_(date now | format date '%Y%m%d_%H%M%S')_($terminal)"

    # Create temporary config with this terminal (like sweep does)
    let config_path = generate_sweep_config "nu" $terminal {
        helix_mode: "release",
        enable_sidebar: false,
        persistent_sessions: false,
        recommended_deps: true,
        yazi_extensions: true
    } $test_id

    mut times = []
    mut successes = 0
    mut failures = 0

    for i in 1..$iterations {
        print $"   Run ($i)/($iterations)..."

        let start = (date now)

        # Launch terminal with Yazelix using config override, wait briefly, then kill
        let result = try {
            # Set environment for test (using config override like sweep does)
            with-env {
                YAZELIX_CONFIG_OVERRIDE: $config_path,
                YAZELIX_SWEEP_TEST_ID: $test_id,
                YAZELIX_SKIP_WELCOME: "true"
            } {
                # Launch via yzx launch command
                ^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx launch"
            }

            # Give it time to fully start
            sleep 2sec

            # Try to kill any zellij sessions from this test
            ^zellij kill-session $"sweep_test_($test_id)" o+e>| ignore

            let end = (date now)
            let duration = ($end - $start) | into int

            {success: true, duration: $duration}
        } catch {|err|
            let end = (date now)
            let duration = ($end - $start) | into int

            print $"   ⚠️  Launch failed: ($err.msg)"
            {success: false, duration: $duration}
        }

        # Update counters outside try-catch block
        if $result.success {
            $successes = $successes + 1
            $times = ($times | append $result.duration)
            print $"   ✅ Completed in (format_time $result.duration)"
        } else {
            $failures = $failures + 1
        }

        # Cool down between runs
        if $i < $iterations {
            sleep 1sec
        }
    }

    # Clean up temporary config
    cleanup_test_config $config_path

    # Calculate statistics
    if ($times | is-empty) {
        {
            terminal: $terminal
            success_rate: 0.0
            avg_time: null
            min_time: null
            max_time: null
            iterations: $iterations
            successes: $successes
            failures: $failures
        }
    } else {
        {
            terminal: $terminal
            success_rate: ($successes / $iterations)
            avg_time: ($times | math avg | math round)
            min_time: ($times | math min)
            max_time: ($times | math max)
            iterations: $iterations
            successes: $successes
            failures: $failures
        }
    }
}

# Format time in nanoseconds to human-readable
def format_time [ns: int] {
    let ms = ($ns / 1_000_000)
    let s = ($ms / 1000)

    if $s >= 1 {
        $"($s)s"
    } else if $ms >= 1 {
        $"($ms)ms"
    } else {
        let us = ($ns / 1_000)
        $"($us)μs"
    }
}

# Main benchmark function
export def main [
    --iterations(-n): int = 3  # Number of iterations per terminal
    --terminal(-t): string     # Test only specific terminal
    --verbose(-v)              # Show detailed output
] {
    print "========================================="
    print "Yazelix Terminal Launch Benchmark"
    print "========================================="
    print ""

    # Determine which terminals to test (test ALL supported terminals like sweep does)
    let terminals_to_test = if ($terminal | is-not-empty) {
        if $terminal in $SUPPORTED_TERMINALS {
            [$terminal]
        } else {
            print $"❌ Terminal '($terminal)' is not supported"
            print $"   Supported terminals: (($SUPPORTED_TERMINALS | str join ', '))"
            exit 1
        }
    } else {
        get_all_terminals
    }

    print $"🔍 Testing terminals: (($terminals_to_test | str join ', '))"
    print $"🔢 Iterations per terminal: ($iterations)"
    print ""
    print "💡 Note: This benchmarks ALL supported terminals using temporary configs,"
    print "   similar to 'yzx sweep terminals'. Terminals will be launched even if"
    print "   not currently in your yazelix.nix config."
    print ""

    # Run benchmarks
    mut results = []
    for term in $terminals_to_test {
        let result = benchmark_terminal $term $iterations
        $results = ($results | append $result)
        print ""
    }

    # Display results
    print "========================================="
    print "📊 Benchmark Results"
    print "========================================="
    print ""

    # Sort by average time
    let sorted_results = ($results
        | where avg_time != null
        | sort-by avg_time)

    if ($sorted_results | is-empty) {
        print "❌ All terminal launches failed!"
        exit 1
    }

    # Print table
    print ($sorted_results | select terminal success_rate avg_time min_time max_time | table)

    print ""
    print "🏆 Winner:"
    let fastest = ($sorted_results | first)
    print $"   ($fastest.terminal) - (format_time $fastest.avg_time) average"

    print ""
    print "📈 Rankings:"
    for i in 0..(($sorted_results | length) - 1) {
        let result = ($sorted_results | get $i)
        let rank = $i + 1
        print $"   ($rank). ($result.terminal) - (format_time $result.avg_time) avg (format_time $result.min_time) min)"
    }

    # Show failures if any
    let failed = ($results | where avg_time == null)
    if not ($failed | is-empty) {
        print ""
        print "❌ Failed terminals:"
        for fail in $failed {
            print $"   ($fail.terminal) - ($fail.successes)/($fail.iterations) succeeded"
        }
    }

    print ""
}
