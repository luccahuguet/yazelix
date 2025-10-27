#!/usr/bin/env nu
# Benchmark different approaches for detecting yazelix.nix changes

def main [] {
    print "========================================="
    print "Yazelix Config Detection Benchmark"
    print "========================================="
    print ""

    let yazelix_dir = $"($env.HOME)/.config/yazelix"
    let config_file = $"($yazelix_dir)/yazelix.nix"

    # Check if config file exists
    if not ($config_file | path exists) {
        print $"❌ Error: Config file not found at ($config_file)"
        exit 1
    }

    print $"📁 Config file: ($config_file)"
    print $"📏 File size: (ls $config_file | get 0.size | into string)"
    print ""

    let iterations = 10

    # Benchmark 1: Nushell hash
    print "1️⃣  Benchmarking: Nushell built-in hash"
    print "   Command: open yazelix.nix | hash sha256"
    let nushell_times = (seq 1 $iterations | each {|i|
        let start = (date now)
        let _ = (open $config_file | hash sha256)
        let end = (date now)
        ($end - $start) | into int
    })

    let nushell_min = ($nushell_times | math min)
    let nushell_max = ($nushell_times | math max)
    let nushell_avg = ($nushell_times | math avg | math round)

    print $"   Min: ($nushell_min)ns = (format_time $nushell_min)"
    print $"   Max: ($nushell_max)ns = (format_time $nushell_max)"
    print $"   Avg: ($nushell_avg)ns = (format_time $nushell_avg)"
    print ""

    # Benchmark 2: Nix hash command
    print "2️⃣  Benchmarking: Nix hash command"
    print "   Command: nix hash file yazelix.nix"

    # Check if nix is available
    mut nix_times = []
    if (which nix | length) == 0 {
        print "   ⚠️  Nix not found in PATH, skipping..."
        print ""
    } else {
        $nix_times = (seq 1 $iterations | each {|i|
            let start = (date now)
            let _ = (^nix hash file $config_file)
            let end = (date now)
            ($end - $start) | into int
        })

        let nix_min = ($nix_times | math min)
        let nix_max = ($nix_times | math max)
        let nix_avg = ($nix_times | math avg | math round)

        print $"   Min: ($nix_min)ns = (format_time $nix_min)"
        print $"   Max: ($nix_max)ns = (format_time $nix_max)"
        print $"   Avg: ($nix_avg)ns = (format_time $nix_avg)"
        print ""
    }

    # Benchmark 3: Cached nix develop
    print "3️⃣  Benchmarking: Cached nix develop"
    print "   Command: nix develop --impure ~/.config/yazelix --command true"

    mut nix_develop_times = []
    if (which nix | length) == 0 {
        print "   ⚠️  Nix not found in PATH, skipping..."
        print ""
    } else {
        # First run to ensure cache is warm
        print "   Warming cache..."
        ^nix develop --impure $yazelix_dir --command true

        print "   Running benchmark..."
        $nix_develop_times = (seq 1 $iterations | each {|i|
            let start = (date now)
            let _ = (^nix develop --impure $yazelix_dir --command true)
            let end = (date now)
            ($end - $start) | into int
        })

        let dev_min = ($nix_develop_times | math min)
        let dev_max = ($nix_develop_times | math max)
        let dev_avg = ($nix_develop_times | math avg | math round)

        print $"   Min: ($dev_min)ns = (format_time $dev_min)"
        print $"   Max: ($dev_max)ns = (format_time $dev_max)"
        print $"   Avg: ($dev_avg)ns = (format_time $dev_avg)"
        print ""
    }

    # Summary
    print "========================================="
    print "📊 Summary"
    print "========================================="
    print $"Nushell hash:     (format_time $nushell_avg) avg"

    if (($nix_times | length) > 0) and (($nix_develop_times | length) > 0) {
        let nix_avg = ($nix_times | math avg | math round)
        let dev_avg = ($nix_develop_times | math avg | math round)

        print $"Nix hash command: (format_time $nix_avg) avg"
        print $"Cached nix develop: (format_time $dev_avg) avg"

        print ""
        print "🏆 Winner:"
        let fastest = ([$nushell_avg, $nix_avg] | math min)

        if $fastest == $nushell_avg {
            print "   Nushell built-in hash is fastest!"
            print $"   Overhead: (format_time $nushell_avg) per launch"
        } else {
            print "   Nix hash command is fastest!"
            print $"   Overhead: (format_time $nix_avg) per launch"
        }

        print ""
        print "💡 Recommendation:"
        if $dev_avg < 500_000_000 {  # 500ms
            print $"   Cached nix develop is fast enough \((format_time $dev_avg)\)!"
            print "   Consider skipping hash checking entirely and always reloading."
            print "   This is simpler and nix's cache handles it efficiently."
        } else {
            print $"   Cached nix develop is slow \((format_time $dev_avg)\)."
            print $"   Use hash checking with the fastest method: (if $fastest == $nushell_avg { 'Nushell hash' } else { 'Nix hash' })"
        }
    } else {
        print "⚠️  Nix benchmarks skipped (nix not in PATH)"
    }

    print ""
}

# Format time in nanoseconds to human-readable
def format_time [ns: int] {
    let ms = ($ns / 1_000_000)
    let us = ($ns / 1_000)

    if $ms >= 1000 {
        let s = ($ms / 1000)
        $"($s)s"
    } else if $ms >= 1 {
        $"($ms)ms"
    } else if $us >= 1 {
        $"($us)μs"
    } else {
        $"($ns)ns"
    }
}
