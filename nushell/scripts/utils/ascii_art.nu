#!/usr/bin/env nu
# Yazelix ASCII Art for Welcome Screen
# Magic sphere with YAZELIX in the center

# Export the color scheme used in the ASCII art for consistent styling
export def get_yazelix_colors [] {
    {
        purple: (ansi purple)
        cyan: (ansi cyan)
        blue: (ansi blue)
        reset: (ansi reset)
    }
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
            # Calculate sphere shape - wider in the middle, narrower at top/bottom
            let max_width = 40
            let center_row = ($sphere_height / 2)
            let distance_from_center = (($row - $center_row) | math abs)
            let width = if $row < $center_row {
                $max_width - ($distance_from_center * 3)
            } else {
                $max_width - ($distance_from_center * 3)
            }

            # Ensure minimum width
            let width = if $width < 10 { 10 } else { $width }

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
    $magic_sphere
}