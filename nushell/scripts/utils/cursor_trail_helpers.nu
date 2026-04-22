#!/usr/bin/env nu
# Cursor Trail Helper Functions
# Helper functions for Ghostty cursor trail management

use constants.nu [get_cursor_trail_shaders get_ghostty_trail_effects get_ghostty_mode_effects]

# Get the random cursor trail pool (derived from CURSOR_TRAIL_SHADERS)
# Excludes "none" and "party" from random selection
export def get_cursor_trail_random_pool [] {
    (get_cursor_trail_shaders)
        | columns
        | where $it != "none" and $it != "party"
}

export def select_random_ghostty_trail_effect [] {
    let pool = (get_ghostty_trail_effects)
    if ($pool | is-empty) {
        null
    } else {
        let max_index = (($pool | length) - 1)
        let index = (random int 0..$max_index)
        $pool | get -o $index
    }
}

export def select_random_ghostty_mode_effect [] {
    let pool = (get_ghostty_mode_effects)
    if ($pool | is-empty) {
        null
    } else {
        let max_index = (($pool | length) - 1)
        let index = (random int 0..$max_index)
        $pool | get -o $index
    }
}

export def ghostty_effect_requires_always_animation [effect: string] {
    let needs_always = ["ripple", "sonic_boom", "rectangle_boom", "ripple_rectangle"]
    $effect in $needs_always
}
