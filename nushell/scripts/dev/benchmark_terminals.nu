#!/usr/bin/env nu
# Benchmark terminal launch performance
# Measures how fast each terminal starts and launches Yazelix

use ../utils/terminal_launcher.nu *
use ../utils/constants.nu [SUPPORTED_TERMINALS, TERMINAL_METADATA]
use ../utils/common.nu [get_yazelix_config_dir get_yazelix_runtime_dir]
use sweep/sweep_process_manager.nu [get_terminal_pids, cleanup_visual_test]

# Get terminals that are actually available in the current nix environment
def get_available_terminals []: nothing -> list<string> {
    $SUPPORTED_TERMINALS | where {|term|
        let meta = ($TERMINAL_METADATA | get -o $term | default {})
        let wrapper = $meta.wrapper
        let direct = $term
        (command_exists $wrapper) or (command_exists $direct)
    }
}

# Benchmark a single terminal launch
def benchmark_terminal [
    terminal: string
    iterations: int = 3
]: nothing -> record {
    print $"📊 Benchmarking ($terminal)..."

    let test_id = $"bench_(date now | format date '%Y%m%d_%H%M%S')_($terminal)"
    let session_name = $"sweep_test_($test_id)"

    mut times = []
    mut successes = 0
    mut failures = 0

    for i in 1..$iterations {
        print $"   Run ($i)/($iterations)..."

        # Get terminal process baseline before launch
        let before_pids = get_terminal_pids $terminal

        let start = (date now)

        # Launch terminal with Yazelix, wait briefly, then kill
        let result = try {
            # Set environment for test
            with-env {
                YAZELIX_SWEEP_TEST_ID: $test_id,
                YAZELIX_SHELLHOOK_SKIP_WELCOME: "true"
            } {
                # Launch via yzx launch command with --terminal flag to force specific terminal
                let runtime_dir = (get_yazelix_runtime_dir)
                ^nu -c $"use \"($runtime_dir)/nushell/scripts/core/yazelix.nu\" *; yzx launch --terminal ($terminal)"
            }

            # Give it time to fully start
            sleep 2sec

            let end = (date now)
            let duration = ($end - $start) | into int

            # Clean up terminal and session
            cleanup_visual_test $session_name $terminal $before_pids 0sec

            {success: true, duration: $duration}
        } catch {|err|
            let end = (date now)
            let duration = ($end - $start) | into int

            print $"   ⚠️  Launch failed: ($err.msg)"

            # Try cleanup even on failure
            try {
                cleanup_visual_test $session_name $terminal $before_pids 0sec
            }

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

    # Calculate statistics
    if ($times | is-empty) {
        {
            terminal: $terminal
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
    let ns_float = ($ns | into float)

    if $ns_float >= 1_000_000_000 {
        let seconds = ($ns_float / 1_000_000_000)
        let rounded = ($seconds | math round --precision 3)
        $"($rounded)s"
    } else if $ns_float >= 1_000_000 {
        let milliseconds = ($ns_float / 1_000_000)
        let rounded = ($milliseconds | math round --precision 1)
        $"($rounded)ms"
    } else if $ns_float >= 1_000 {
        let microseconds = ($ns_float / 1_000)
        let rounded = ($microseconds | math round --precision 1)
        $"($rounded)μs"
    } else {
        $"($ns)ns"
    }
}

# Main benchmark function
export def main [
    --iterations(-n): int = 1  # Number of iterations per terminal
    --terminal(-t): string     # Test only specific terminal
    --verbose(-v)              # Show detailed output
] {
    let requested_terminal = ($terminal | default "")
    print "========================================="
    print "Yazelix Terminal Launch Benchmark"
    print "========================================="
    print ""

    # Get available terminals from current environment
    let available_terminals = get_available_terminals
    let unavailable_terminals = ($SUPPORTED_TERMINALS | where {|term| $term not-in $available_terminals})

    if ($available_terminals | is-empty) {
        print "❌ No supported terminals found in your yazelix environment!"
        print ""
        print "💡 To add terminals, edit ~/.config/yazelix/user_configs/yazelix.toml:"
        let config_dir = (get_yazelix_config_dir)
        print $"   config path: ($config_dir | path join \"yazelix.toml\")"
        print "   terminals = [\"ghostty\" \"wezterm\" \"kitty\" \"alacritty\" \"foot\"];"
        print ""
        print "   Then reload: yzx enter"
        exit 1
    }

    # Show availability status
    if not ($unavailable_terminals | is-empty) {
        print $"📋 Available terminals: (($available_terminals | str join ', '))"
        print $"⚠️  Unavailable terminals: (($unavailable_terminals | str join ', '))"
        print ""
        print "💡 To benchmark more terminals, add them to ~/.config/yazelix/user_configs/yazelix.toml:"
        let config_dir = (get_yazelix_config_dir)
        print $"   config path: ($config_dir | path join \"yazelix.toml\")"
        let quoted_terminals = ($unavailable_terminals | each {|t| $'"($t)"'} | str join ' ')
        print $"   terminals = [($quoted_terminals)];"
        print "   Then reload: yzx enter"
        print ""
    }

    # Determine which terminals to test
    let terminals_to_test = if ($requested_terminal | is-not-empty) {
        if $requested_terminal in $available_terminals {
            [$requested_terminal]
        } else if $requested_terminal in $SUPPORTED_TERMINALS {
            print $"❌ Terminal '($requested_terminal)' is supported but not available in your environment"
            print ""
            print "💡 To add it, edit ~/.config/yazelix/user_configs/yazelix.toml:"
            let config_dir = (get_yazelix_config_dir)
            print $"   config path: ($config_dir | path join \"yazelix.toml\")"
            print $"   terminals = [\"($requested_terminal)\"];"
            print "   Then reload: yzx enter"
            exit 1
        } else {
            print $"❌ Terminal '($requested_terminal)' is not supported"
            print $"   Supported terminals: (($SUPPORTED_TERMINALS | str join ', '))"
            exit 1
        }
    } else {
        $available_terminals
    }

    print $"🔍 Benchmarking terminals: (($terminals_to_test | str join ', '))"
    print $"🔢 Iterations per terminal: ($iterations)"
    if $verbose {
        print $"🔎 Full available terminal set: (($available_terminals | str join ', '))"
    }
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

    # Print table with formatted durations
    let display_results = ($sorted_results | each {|row|
        {
            terminal: $row.terminal
            avg_time: (format_time $row.avg_time)
            min_time: (format_time $row.min_time)
            max_time: (format_time $row.max_time)
        }
    })
    print ($display_results | table)

    print ""
    print "🏆 Winner:"
    let fastest = ($sorted_results | first)
    print $"   ($fastest.terminal) - (format_time $fastest.avg_time) average"

    print ""
    print "📈 Rankings:"
    for i in 0..(($sorted_results | length) - 1) {
        let result = ($sorted_results | get -o $i)
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
