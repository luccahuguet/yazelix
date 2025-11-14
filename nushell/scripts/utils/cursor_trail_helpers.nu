#!/usr/bin/env nu
# Cursor Trail Helper Functions
# Helper functions for Ghostty cursor trail management

use constants.nu CURSOR_TRAIL_SHADERS

# Get the random cursor trail pool (derived from CURSOR_TRAIL_SHADERS)
# Excludes "none" and "party" from random selection
export def get_cursor_trail_random_pool [] {
    $CURSOR_TRAIL_SHADERS
        | columns
        | where $it != "none" and $it != "party"
}
