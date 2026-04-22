#!/usr/bin/env nu

use ../utils/ascii_art.nu [
    get_screen_cycle_frames
    get_screen_frame_delay
    get_terminal_height
    get_terminal_width
    get_game_of_life_screen_state
    render_game_of_life_screen_state
    resolve_screen_style
    step_game_of_life_screen_state
]
use ../utils/keypress_polling.nu poll_for_keypress_status

def enter_screen_mode [] {
    print -n "\u{1b}[?1049h\u{1b}[?25l\u{1b}[2J\u{1b}[H"
}

def leave_screen_mode [] {
    print -n "\u{1b}[?25h\u{1b}[?1049l"
}

def render_screen_frame [frame: list<string>] {
    print -n "\u{1b}[H\u{1b}[2J"
    for line in $frame {
        print $line
    }
}

def poll_for_screen_keypress [timeout: duration] {
    let result = (poll_for_keypress_status $timeout)

    if $result.status == "error" {
        error make {msg: $"yzx screen requires an interactive terminal that supports timed keypress reads: ($result.message)"}
    }

    ($result.status == "key")
}

export def resolve_yzx_screen_style [requested_style?: string] {
    resolve_screen_style $requested_style
}

export def get_yzx_screen_cycle_frames [screen_style?: string, width?: int] {
    let resolved_style = (resolve_screen_style $screen_style)
    get_screen_cycle_frames $resolved_style $width
}

# Show an animated Yazelix full-terminal screen
export def "yzx screen" [
    style?: string  # Animated screen style: logo, boids, one of the game_of_life variants, or random
] {
    let resolved_style = (resolve_screen_style $style)
    let frame_delay = (get_screen_frame_delay $resolved_style)
    let is_game_of_life = ($resolved_style | str starts-with "game_of_life_")
    mut width = (get_terminal_width)
    mut height = (get_terminal_height)
    mut frames = if $is_game_of_life { [] } else { get_screen_cycle_frames $resolved_style $width }
    mut frame_index = 0
    mut game_of_life_state = if $is_game_of_life {
        (get_game_of_life_screen_state $resolved_style $width $height)
    } else {
        null
    }

    enter_screen_mode

    let screen_error = (try {
        loop {
            if $is_game_of_life {
                render_screen_frame (render_game_of_life_screen_state $game_of_life_state)
            } else {
                if ($frames | is-empty) {
                    error make {msg: $"No frames available for yzx screen style: ($resolved_style)"}
                }

                render_screen_frame ($frames | get -o ($frame_index mod ($frames | length)))
            }

            if (poll_for_screen_keypress $frame_delay) {
                break
            }

            let current_width = (get_terminal_width)
            let current_height = (get_terminal_height)
            if ($current_width != $width) or ($current_height != $height) {
                $width = $current_width
                $height = $current_height

                if $is_game_of_life {
                    $game_of_life_state = (get_game_of_life_screen_state $resolved_style $width $height)
                } else {
                    $frames = (get_screen_cycle_frames $resolved_style $width)
                    $frame_index = 0
                }

                continue
            }

            if $is_game_of_life {
                $game_of_life_state = (step_game_of_life_screen_state $game_of_life_state)
            } else {
                $frame_index = ($frame_index + 1)
            }
        }

        null
    } catch {|err|
        $err
    })

    leave_screen_mode

    if $screen_error != null {
        error make {msg: $screen_error.msg}
    }
}
