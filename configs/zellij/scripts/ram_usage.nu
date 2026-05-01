#!/usr/bin/env nu

# Normalized RAM usage (0-100%) for the Zellij status bar, backed by Nushell's cross-platform sys mem data.

def clamp_round [value: number] {
    let clamped = if $value < 0 { 0 } else if $value > 100 { 100 } else { $value }
    $clamped | math round --precision 0 | into int
}

def ram_usage [] {
    let mem = (try { sys mem } catch { null })
    if $mem == null { return null }

    let total = (try { $mem.total } catch { null })
    let used = (try { $mem.used } catch { null })
    if $total == null or $used == null or $total <= 0B { return null }

    ($used / $total) * 100
}

def main [] {
    let usage = ram_usage
    if $usage == null {
        print "??%"
    } else {
        print $"(clamp_round $usage)%"
    }
}
