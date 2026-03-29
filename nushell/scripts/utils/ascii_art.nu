#!/usr/bin/env nu
# Yazelix ASCII Art for Welcome Screen
# Magic sphere with YAZELIX in the center

# Export the color scheme used in the ASCII art for consistent styling
export def get_yazelix_colors [] {
    {
        purple: (ansi purple)
        cyan: (ansi cyan)
        blue: (ansi blue)
        green: (ansi green)
        yellow: (ansi yellow)
        reset: (ansi reset)
    }
}

export const WELCOME_STYLE_VALUES = ["static", "logo", "boids", "life", "mandelbrot", "random"]
export const ANIMATED_WELCOME_STYLE_VALUES = ["logo", "boids", "life", "mandelbrot"]

export def get_welcome_style_random_pool [] {
    $ANIMATED_WELCOME_STYLE_VALUES
}

export def resolve_welcome_style [welcome_style: string, random_index?: int] {
    let normalized = ($welcome_style | into string | str downcase)

    if $normalized != "random" {
        return $normalized
    }

    let pool = (get_welcome_style_random_pool)
    if ($pool | is-empty) {
        error make {msg: "Welcome style random pool is empty."}
    }

    let max_index = (($pool | length) - 1)
    let selected_index = if $random_index == null {
        random int 0..$max_index
    } else {
        let provided = ($random_index | into int)
        $provided mod ($pool | length)
    }

    $pool | get $selected_index
}

export def get_welcome_ascii_art [] {
    let colors = get_yazelix_colors
    let purple = $colors.purple
    let cyan = $colors.cyan
    let blue = $colors.blue
    let reset = $colors.reset

    # Generate magic sphere with YAZELIX in the center
    let sphere_height = 15
    let magic_sphere = (
        0..($sphere_height - 1) | each { |row|
            # Calculate sphere shape - start narrow and rapidly expand
            let max_width = 60  # Reduced from 80 to a more reasonable size
            let center_row = ($sphere_height / 2)
            let distance_from_center = (($row - $center_row) | math abs)

            # More aggressive growth pattern - start smaller, grow faster
            let width = if $row < $center_row {
                # Top half: start very narrow, grow rapidly
                let progress = ($row / $center_row)
                let base_width = 2  # Start with just 2 stars
                let growth_factor = ($progress * $progress * $progress * $max_width * 1.2)  # Reduced multiplier from 1.5 to 1.2
                ($base_width + $growth_factor) | math round
            } else {
                # Bottom half: mirror the top half
                let progress = (($sphere_height - $row - 1) / $center_row)
                let base_width = 2
                let growth_factor = ($progress * $progress * $progress * $max_width * 1.2)
                ($base_width + $growth_factor) | math round
            }

            # Ensure minimum width
            let width = if $width < 2 { 2 } else { $width }

            # No padding - left align the sphere
            let pad_str = ""

            let before_count = (($width - 7) / 2 | math floor)
            let after_count = $width - $before_count - 7 + 2

            let before_yazelix = (
                0..($before_count - 1) | each { |i|
                    let color_idx = ($i + $row) mod 3
                    let color = if $color_idx == 0 { $purple } else if $color_idx == 1 { $cyan } else { $blue }
                    let symbol = "★ "
                    $color + $symbol
                } | str join ""
            )

            let yazelix_text = "\u{1b}[35mY\u{1b}[36mA\u{1b}[34mZ\u{1b}[35mE\u{1b}[36mL\u{1b}[34mI\u{1b}[35mX"

            let after_yazelix = (
                0..($after_count - 1) | each { |i|
                    let color_idx = ($before_count + 7 + $i + $row) mod 3
                    let color = if $color_idx == 0 { $purple } else if $color_idx == 1 { $cyan } else { $blue }
                    let symbol = "★ "
                    $color + $symbol
                } | str join ""
            )

            let sphere_content = if ($row == 7) {
                $before_yazelix + $yazelix_text + $after_yazelix
            } else {
                (0..($width - 1) | each { |i|
                    let color_idx = ($i + $row) mod 3
                    let color = if $color_idx == 0 { $purple } else if $color_idx == 1 { $cyan } else { $blue }
                    let symbol = "★ "
                    $color + $symbol
                } | str join "")
            }

            $pad_str + $sphere_content + $reset
        }
    )

    # Return the magic sphere
    $magic_sphere | where { |line| $line != "" }
}

export def get_animated_ascii_art [frame_count: int = 10] {
    let colors = get_yazelix_colors
    let purple = $colors.purple
    let cyan = $colors.cyan
    let blue = $colors.blue
    let reset = $colors.reset

    # Generate animated magic sphere with YAZELIX in the center
    let sphere_height = 15
    let max_width = 60
    let center_row = ($sphere_height / 2)

    # Create frames for animation
    0..($frame_count - 1) | each { |frame|
        0..($sphere_height - 1) | each { |row|
            # Calculate sphere shape
            let width = if $row < $center_row {
                let progress = ($row / $center_row)
                let base_width = 2
                let growth_factor = ($progress * $progress * $progress * $max_width * 1.2)
                ($base_width + $growth_factor) | math round
            } else {
                let progress = (($sphere_height - $row - 1) / $center_row)
                let base_width = 2
                let growth_factor = ($progress * $progress * $progress * $max_width * 1.2)
                ($base_width + $growth_factor) | math round
            }

            let width = if $width < 2 { 2 } else { $width }
            let before_count = (($width - 7) / 2 | math floor)
            let after_count = $width - $before_count - 7 + 2

            # Animate colors by adding frame offset
            let before_yazelix = (
                0..($before_count - 1) | each { |i|
                    let color_idx = ($i + $row + $frame) mod 3
                    let color = if $color_idx == 0 { $purple } else if $color_idx == 1 { $cyan } else { $blue }
                    let symbol = "★ "
                    $color + $symbol
                } | str join ""
            )

            # Animate YAZELIX text colors
            let yazelix_chars = ["Y", "A", "Z", "E", "L", "I", "X"]
            let yazelix_text = (
                0..6 | each { |i|
                    let color_idx = ($i + $frame) mod 3
                    let color = if $color_idx == 0 { $purple } else if $color_idx == 1 { $cyan } else { $blue }
                    $color + ($yazelix_chars | get $i)
                } | str join ""
            )

            let after_yazelix = (
                0..($after_count - 1) | each { |i|
                    let color_idx = ($before_count + 7 + $i + $row + $frame) mod 3
                    let color = if $color_idx == 0 { $purple } else if $color_idx == 1 { $cyan } else { $blue }
                    let symbol = "★ "
                    $color + $symbol
                } | str join ""
            )

            let sphere_content = if ($row == 7) {
                $before_yazelix + $yazelix_text + $after_yazelix
            } else {
                (0..($width - 1) | each { |i|
                    let color_idx = ($i + $row + $frame) mod 3
                    let color = if $color_idx == 0 { $purple } else if $color_idx == 1 { $cyan } else { $blue }
                    let symbol = "★ "
                    $color + $symbol
                } | str join "")
            }

            $sphere_content + $reset
        }
    }
}

export def play_animation [duration: duration ] {
    let frames = get_animated_ascii_art 12
    let frame_delay = ($duration / ($frames | length))
    let art_height = 15

    # Play animation once (draw at current cursor position)
    for frame in $frames {
        print ($frame | str join "\n")
        sleep $frame_delay
        # Move cursor up by art_height + 1 lines to redraw in place (accounting for newline)
        print ("\u{1b}[" + (($art_height + 1) | into string) + "A")
    }
    # After animation, move cursor just below the art
    print ((0..($art_height - 1) | each { "" } | str join "\n"))
}

export def render_welcome_style [welcome_style: string, duration: duration = 0.5sec] {
    let resolved_style = (resolve_welcome_style $welcome_style)

    if $resolved_style == "static" {
        let ascii_art = get_welcome_ascii_art
        for line in $ascii_art {
            print $line
        }
        print ""
        return
    }

    if $resolved_style in ["logo", "boids", "life", "mandelbrot"] {
        print ""
        # The dedicated boids/life/mandelbrot renderers land in their own beads.
        # Until then, animated styles share the current Yazelix logo animation.
        play_animation $duration
        return
    }

    error make {msg: $"Unsupported welcome_style: ($resolved_style)"}
}
