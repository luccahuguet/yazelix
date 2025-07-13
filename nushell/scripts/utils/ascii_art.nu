#!/usr/bin/env nu
# Yazelix ASCII Art for Welcome Screen
# Magic sphere with YAZELIX in the center

export def get_welcome_ascii_art [] {
    let purple = (ansi purple)
    let cyan = (ansi cyan)
    let blue = (ansi blue)
    let reset = (ansi reset)

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

            # Generate the sphere line with YAZELIX in the center
            let before_yazelix = (
                0..(($width / 2 - 3) - 1) | each { |i|
                    let color_idx = ($i + $row) mod 3
                    let color = if $color_idx == 0 { $purple } else if $color_idx == 1 { $cyan } else { $blue }
                    let symbol = "★ "
                    $color + $symbol
                } | str join ""
            )

            let yazelix_text = "\u{1b}[35mY\u{1b}[36mA\u{1b}[34mZ\u{1b}[35mE\u{1b}[36mL\u{1b}[34mI\u{1b}[35mX"

            let after_yazelix = (
                ($width / 2 + 4)..($width - 1) | each { |i|
                    let color_idx = ($i + $row) mod 3
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