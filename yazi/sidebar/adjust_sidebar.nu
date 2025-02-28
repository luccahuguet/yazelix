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

    if not ($env.YAZI_ID? | is-empty) {
        # Quit the current instance
        ya emit-to $env.YAZI_ID quit
        sleep 100ms
        # Relaunch with the new size
        with-env { YAZI_SIDEBAR_SIZE: $sidebar_size } {
            yazi
        }
    } else {
        print "No YAZI_ID, launching standalone"
        with-env { YAZI_SIDEBAR_SIZE: $sidebar_size } { yazi }
    }
}
