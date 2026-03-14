#!/usr/bin/env nu
# Cursor Trail Helper Functions
# Helper functions for Ghostty cursor trail management

use constants.nu [CURSOR_TRAIL_SHADERS, GHOSTTY_CURSOR_EFFECTS]

# Get the random cursor trail pool (derived from CURSOR_TRAIL_SHADERS)
# Excludes "none" and "party" from random selection
export def get_cursor_trail_random_pool [] {
    $CURSOR_TRAIL_SHADERS
        | columns
        | where $it != "none" and $it != "party"
}

export def get_ghostty_cursor_effect_random_pool [] {
    $GHOSTTY_CURSOR_EFFECTS | where $it != "none"
}

export def select_random_ghostty_cursor_effects [] {
    let pool = (get_ghostty_cursor_effect_random_pool)
    if ($pool | is-empty) {
        ["tail"]
    } else {
        mut selected = []
        for effect in $pool {
            if ((random int 0..1) == 1) {
                $selected = ($selected | append $effect)
            }
        }

        if ($selected | is-empty) {
            let max_index = (($pool | length) - 1)
            let index = (random int 0..$max_index)
            [$pool | get $index]
        } else {
            $selected
        }
    }
}

export def ghostty_cursor_effects_require_always_animation [effects: list<string>] {
    let needs_always = ["ripple", "sonic_boom", "rectangle_boom", "ripple_rectangle"]
    $effects | any {|effect| $effect in $needs_always }
}
