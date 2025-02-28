#!/usr/bin/env nu

def main [] {
    let term_width = (do -i { tput cols } | into int | default 80)
    let sidebar_size = if $term_width > 120 {
        "big"
    } else if $term_width > 80 {
        "medium"
    } else {
        "small"
    }
    # Quit current Yazi instance if running
    # if not ($env.YAZI_ID? | is-empty) {
    #     ya emit quit
    #     sleep 10ms  # Small delay to ensure quit completes
    # }
    # Launch new instance with env var
    with-env { YAZI_SIDEBAR_SIZE: $sidebar_size } {
        yazi &  # Background to avoid blocking
    }
}
