#!/usr/bin/env nu
# yzx gc - Garbage collection for Nix store

# Format bytes to human readable
def format_size [bytes: int] {
    if $bytes < 1024 {
        $"($bytes) B"
    } else if $bytes < (1024 * 1024) {
        $"(($bytes / 1024) | math round --precision 1) KiB"
    } else if $bytes < (1024 * 1024 * 1024) {
        $"(($bytes / 1024 / 1024) | math round --precision 1) MiB"
    } else {
        $"(($bytes / 1024 / 1024 / 1024) | math round --precision 2) GiB"
    }
}

# Get /nix/store size in bytes
def get_store_size [] {
    let result = (do { ^du -sb /nix/store } | complete)
    if $result.exit_code == 0 {
        $result.stdout | split row "\t" | first | into int
    } else {
        0
    }
}

# Run devenv gc with quiet mode, filtering remaining noise
def run_devenv_gc [] {
    # Use --quiet to suppress devenv's verbose output, but nix errors still come through
    let result = (do { ^devenv gc --quiet } | complete)
    
    # Filter remaining noise from nix stderr
    let lines = $result.stdout + $result.stderr | lines
    let cleaned = $lines | where { |line|
        let l = $line | str trim
        not (
            ($l | str starts-with "error:") or
            ($l | str starts-with "warning:") or
            ($l | str contains "Cannot delete path") or
            ($l | str contains "referenced by the GC root") or
            ($l | str contains "unknown setting") or
            ($l | str starts-with "finding garbage") or
            ($l == "")
        )
    }
    
    for line in $cleaned {
        print $"  ($line)"
    }
}

# Run nix-collect-garbage and filter noisy output
def run_nix_gc [args: list<string>] {
    let result = (do { ^nix-collect-garbage ...$args } | complete)
    
    # Filter the output
    let lines = $result.stdout + $result.stderr | lines
    let cleaned = $lines | where { |line|
        let l = $line | str trim
        not (
            ($l | str starts-with "warning:") or
            ($l | str contains "unknown setting") or
            ($l | str starts-with "finding garbage") or
            ($l | str starts-with "deleting garbage") or
            ($l | str starts-with "deleting unused") or
            ($l | str starts-with "note: hard linking") or
            ($l == "")
        )
    }
    
    # Print cleaned output
    for line in $cleaned {
        print $"  ($line)"
    }
}

# Garbage collection for Nix store
#
# Modes:
#   yzx gc           - Clean devenv + remove unreferenced paths
#   yzx gc deep      - Also delete generations older than 30d
#   yzx gc deep 7d   - Also delete generations older than 7d
#   yzx gc deeper    - Delete ALL old generations
export def "yzx gc" [
    mode?: string      # "deep" or "deeper"
    period?: string    # e.g. "7d", "30d" (only for deep, default: 30d)
] {
    let before_size = get_store_size
    
    # Validate mode
    if ($mode | is-not-empty) and $mode != "deep" and $mode != "deeper" {
        print $"(ansi red)Unknown mode: ($mode)(ansi reset)"
        print "Usage: yzx gc [deep [period] | deeper]"
        return
    }
    
    # Run devenv gc
    print $"(ansi cyan)Cleaning devenv generations...(ansi reset)"
    run_devenv_gc
    
    # Run nix-collect-garbage with appropriate flags
    print $"(ansi cyan)Collecting garbage...(ansi reset)"
    
    if $mode == "deeper" {
        run_nix_gc ["-d"]
    } else if $mode == "deep" {
        let p = $period | default "30d"
        run_nix_gc ["--delete-older-than" $p]
    } else {
        run_nix_gc []
    }
    
    let after_size = get_store_size
    let freed = $before_size - $after_size
    
    print ""
    print $"(ansi green_bold)Nix Store(ansi reset)"
    print $"  Before: (ansi cyan)(format_size $before_size)(ansi reset)"
    print $"  After:  (ansi cyan)(format_size $after_size)(ansi reset)"
    if $freed > 0 {
        print $"  Freed:  (ansi green)(format_size $freed)(ansi reset)"
    } else {
        print $"  Freed:  (ansi yellow)0 B(ansi reset)"
    }
}
