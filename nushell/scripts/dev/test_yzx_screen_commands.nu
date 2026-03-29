#!/usr/bin/env nu

use ../yzx/screen.nu [get_yzx_screen_cycle_frames resolve_yzx_screen_style]
use ../utils/ascii_art.nu [get_logo_welcome_frame get_max_visible_width]
use ../utils/ascii_art.nu [get_game_of_life_screen_state render_game_of_life_screen_state step_game_of_life_screen_state]

def test_screen_style_rejects_static [] {
    print "🧪 Testing yzx screen rejects the non-animated static style..."

    try {
        resolve_yzx_screen_style "static" | ignore
        print "  ❌ yzx screen unexpectedly accepted the static style"
        false
    } catch {|err|
        if ($err.msg | str contains "Invalid screen style 'static'") {
            print "  ✅ yzx screen only accepts animated screen styles"
            true
        } else {
            print $"  ❌ Unexpected error: ($err.msg)"
            false
        }
    }
}

def test_game_of_life_screen_cycle_stays_bounded_and_omits_resting_logo [] {
    print "🧪 Testing yzx screen uses an animated game_of_life cycle instead of the resting welcome frame..."

    try {
        let frames = (get_yzx_screen_cycle_frames "game_of_life" 100)
        let static_logo = (get_logo_welcome_frame 100)
        let final_frame = ($frames | last)
        let max_width = ($frames | each {|frame| get_max_visible_width $frame } | math max)

        if (
            (($frames | length) >= 2)
            and ($max_width <= 100)
            and ($final_frame != $static_logo)
        ) {
            print "  ✅ yzx screen keeps game_of_life animated and width-aware"
            true
        } else {
            print $"  ❌ Unexpected screen cycle result: frames=(($frames | length)) max_width=($max_width) final_is_logo=(($final_frame == $static_logo))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_game_of_life_screen_state_rolls_forward [] {
    print "🧪 Testing yzx screen keeps a live rolling game_of_life state instead of replaying a short canned cycle..."

    try {
        let initial_state = (get_game_of_life_screen_state 100 24)
        let next_state = (step_game_of_life_screen_state $initial_state)
        let initial_frame = (render_game_of_life_screen_state $initial_state)
        let next_frame = (render_game_of_life_screen_state $next_state)

        if ($initial_frame != $next_frame) {
            print "  ✅ yzx screen advances the live game_of_life state each frame"
            true
        } else {
            print "  ❌ yzx screen game_of_life state did not advance"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_game_of_life_screen_uses_full_height_budget [] {
    print "🧪 Testing yzx screen uses the full pane height instead of the shorter welcome reservation..."

    try {
        let frame = (get_yzx_screen_cycle_frames "game_of_life" 100 | get 0)
        let state = (get_game_of_life_screen_state 100 24)
        let rendered_state = (render_game_of_life_screen_state $state)

        if (
            (($frame | length) == 24)
            and (($rendered_state | length) == 24)
        ) {
            print "  ✅ yzx screen game_of_life fills the full pane height"
            true
        } else {
            print $"  ❌ Unexpected yzx screen heights: cycle=(($frame | length)) live=(($rendered_state | length))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

export def run_screen_canonical_tests [] {
    [
        (test_screen_style_rejects_static)
        (test_game_of_life_screen_cycle_stays_bounded_and_omits_resting_logo)
        (test_game_of_life_screen_state_rolls_forward)
        (test_game_of_life_screen_uses_full_height_budget)
    ]
}

def main [] {
    let results = (run_screen_canonical_tests)
    let passed = ($results | where $it == true | length)
    let total = ($results | length)

    if $passed == $total {
        print $"✅ All yzx screen tests passed \(($passed)/($total)\)"
    } else {
        print $"❌ Some yzx screen tests failed \(($passed)/($total)\)"
        error make { msg: "yzx screen tests failed" }
    }
}
