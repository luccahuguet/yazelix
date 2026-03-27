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

def format_duration_ms [duration_ms: number] {
    if $duration_ms < 1000 {
        $"(($duration_ms | into int))ms"
    } else {
        $"(($duration_ms / 1000) | math round --precision 1)s"
    }
}

# Get /nix/store size in bytes
def get_store_size [] {
    let result = (do { ^du -sb /nix/store } | complete)
    let size_line = (
        $result.stdout
        | default ""
        | lines
        | where {|line| $line | str contains "/nix/store" }
        | last
    )

    if ($size_line | is-not-empty) {
        $size_line | split row "\t" | first | into int
    } else if $result.exit_code == 0 {
        0
    } else {
        error make {msg: $"Failed to measure /nix/store size.\n($result.stderr | default $result.stdout | str trim)"}
    }
}

# Filter GC output down to high-signal lines
def filter_gc_lines [stdout: string, stderr: string, ignored_prefixes: list<string>, ignored_contains: list<string>] {
    let lines = ($stdout + $stderr) | lines
    $lines | where { |line|
        let l = $line | str trim
        let has_ignored_prefix = ($ignored_prefixes | any {|prefix| $l | str starts-with $prefix })
        let has_ignored_substring = ($ignored_contains | any {|needle| $l | str contains $needle })
        not ($has_ignored_prefix or $has_ignored_substring or ($l == ""))
    }
}

# Run devenv gc with quiet mode, filtering remaining noise
def run_devenv_gc [] {
    let start = (date now)
    let result = (do { ^devenv gc --quiet } | complete)

    {
        exit_code: $result.exit_code
        duration_ms: (((((date now) - $start) | into int) / 1000000) | into int)
        lines: (filter_gc_lines $result.stdout $result.stderr
            ["warning:" "finding garbage"]
            ["Cannot delete path" "referenced by the GC root" "unknown setting"])
        raw_stdout: ($result.stdout | default "")
        raw_stderr: ($result.stderr | default "")
    }
}

# Run nix-collect-garbage and filter noisy output
def run_nix_gc [args: list<string>] {
    let start = (date now)
    let result = (do { ^nix-collect-garbage ...$args } | complete)

    {
        exit_code: $result.exit_code
        duration_ms: (((((date now) - $start) | into int) / 1000000) | into int)
        lines: (filter_gc_lines $result.stdout $result.stderr
            ["warning:" "finding garbage" "deleting garbage" "deleting unused" "note: hard linking"]
            ["unknown setting"])
        raw_stdout: ($result.stdout | default "")
        raw_stderr: ($result.stderr | default "")
    }
}

def print_gc_phase_output [phase_name: string, result: record, empty_message: string] {
    let duration = (format_duration_ms $result.duration_ms)
    if $result.exit_code == 0 {
        print $"  Done in (ansi green)($duration)(ansi reset)"
        if ($result.lines | is-empty) {
            print $"  (ansi dark_gray)($empty_message)(ansi reset)"
        } else {
            for line in $result.lines {
                print $"  ($line)"
            }
        }
        return
    }

    print $"  (ansi red)Failed after ($duration)(ansi reset)"
    let failure_lines = if ($result.lines | is-empty) {
        (($result.raw_stderr + "\n" + $result.raw_stdout) | lines | where {|line| ($line | str trim) != "" })
    } else {
        $result.lines
    }

    for line in $failure_lines {
        print $"  ($line)"
    }

    exit 1
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
    # Validate mode
    if ($mode | is-not-empty) and $mode != "deep" and $mode != "deeper" {
        print $"(ansi red)Unknown mode: ($mode)(ansi reset)"
        print "Usage: yzx gc [deep [period] | deeper]"
        return
    }

    print $"(ansi cyan)Measuring current Nix store size...(ansi reset)"
    let before_size = get_store_size
    print $"  Current size: (ansi cyan)(format_size $before_size)(ansi reset)"
    
    # Run devenv gc
    print $"(ansi cyan)Cleaning devenv generations... this can take a while(ansi reset)"
    let devenv_result = (run_devenv_gc)
    print_gc_phase_output "devenv gc" $devenv_result "No additional devenv GC output."

    # Run nix-collect-garbage with appropriate flags
    print $"(ansi cyan)Collecting garbage... this can take a while(ansi reset)"
    
    if $mode == "deeper" {
        let nix_result = (run_nix_gc ["-d"])
        print_gc_phase_output "nix-collect-garbage" $nix_result "No additional Nix GC output."
    } else if $mode == "deep" {
        let p = $period | default "30d"
        let nix_result = (run_nix_gc ["--delete-older-than" $p])
        print_gc_phase_output "nix-collect-garbage" $nix_result "No additional Nix GC output."
    } else {
        let nix_result = (run_nix_gc [])
        print_gc_phase_output "nix-collect-garbage" $nix_result "No additional Nix GC output."
    }
    
    print $"(ansi cyan)Re-measuring Nix store size...(ansi reset)"
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
