#!/usr/bin/env nu

def normalize_failure_class [failure_class: string] {
    let normalized = ($failure_class | str downcase | str trim)
    if $normalized == "config" {
        "config problem"
    } else if $normalized == "generated-state" {
        "generated-state problem"
    } else if $normalized == "host-dependency" {
        "host-dependency problem"
    } else {
        error make {msg: $"Unsupported failure class: ($failure_class)"}
    }
}

export def format_failure_classification [failure_class: string, recovery_hint: string] {
    let label = (normalize_failure_class $failure_class)
    [
        $"Failure class: ($label)."
        $"Recovery: ($recovery_hint)"
    ] | str join "\n"
}
