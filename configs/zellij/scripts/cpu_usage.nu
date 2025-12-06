#!/usr/bin/env nu

# Normalized CPU usage (0-100%) for the Zellij status bar, works on Linux/macOS.

def clamp_round [value: number] {
    let clamped = if $value < 0 { 0 } else if $value > 100 { 100 } else { $value }
    $clamped | math round --precision 0 | into int
}

def parse_proc_line [line: string] {
    let parts = ($line | split row " " | where {|x| $x != ""})
    if ($parts | length) < 8 { return null }
    {
        user: ($parts.1 | into float)
        nice: ($parts.2 | into float)
        system: ($parts.3 | into float)
        idle: ($parts.4 | into float)
        iowait: ($parts.5 | into float)
        irq: ($parts.6 | into float)
        softirq: ($parts.7 | into float)
        steal: (if ($parts | length) > 8 { $parts.8 | into float } else { 0.0 })
    }
}

def linux_proc_usage [] {
    if not ("/proc/stat" | path exists) { return null }

    let snap1 = parse_proc_line (open /proc/stat | lines | first)
    if $snap1 == null { return null }
    sleep 100ms
    let snap2 = parse_proc_line (open /proc/stat | lines | first)
    if $snap2 == null { return null }

    let idle1 = $snap1.idle + $snap1.iowait
    let idle2 = $snap2.idle + $snap2.iowait
    let total1 = [
        $snap1.user $snap1.nice $snap1.system $snap1.idle $snap1.iowait $snap1.irq $snap1.softirq $snap1.steal
    ] | math sum
    let total2 = [
        $snap2.user $snap2.nice $snap2.system $snap2.idle $snap2.iowait $snap2.irq $snap2.softirq $snap2.steal
    ] | math sum

    let total_delta = $total2 - $total1
    let idle_delta = $idle2 - $idle1
    if $total_delta <= 0 { return null }

    (1 - ($idle_delta / $total_delta)) * 100
}

def ps_normalized_usage [] {
    let cores = (sys cpu | length)
    let core_count = if $cores < 1 { 1 } else { $cores }

    let total = (
        try {
            ^ps -A -o %cpu
            | skip 1
            | where {|it| not ($it | str trim | is-empty)}
            | each {|it| try { $it | str trim | into float } catch { null }}
            | where {|it| $it != null}
            | math sum
        } catch { null }
    )

    if $total == null { null } else { $total / $core_count }
}

def main [] {
    let host = sys host
    let os_summary = (try { $host.long_os_version } catch { try { $host.name } catch { "" } }) | str downcase
    let from_proc = if ($os_summary | str contains "linux") { linux_proc_usage } else { null }
    let usage = if $from_proc == null { ps_normalized_usage } else { $from_proc }

    if $usage == null {
        print "??%"
    } else {
        print $"(clamp_round $usage)%"
    }
}
