#!/usr/bin/env nu
# Cursor Trail Helper Functions
# Helper functions for Ghostty cursor trail management

use constants.nu [CURSOR_TRAIL_SHADERS, GHOSTTY_TRAIL_EFFECTS, GHOSTTY_MODE_EFFECTS]

# Get the random cursor trail pool (derived from CURSOR_TRAIL_SHADERS)
# Excludes "none" and "party" from random selection
export def get_cursor_trail_random_pool [] {
    $CURSOR_TRAIL_SHADERS
        | columns
        | where $it != "none" and $it != "party"
}

export def select_random_ghostty_trail_effect [] {
    let pool = $GHOSTTY_TRAIL_EFFECTS
    if ($pool | is-empty) {
        null
    } else {
        let max_index = (($pool | length) - 1)
        let index = (random int 0..$max_index)
        $pool | get -o $index
    }
}

export def select_random_ghostty_mode_effect [] {
    let pool = $GHOSTTY_MODE_EFFECTS
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
