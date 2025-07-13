#!/usr/bin/env nu
# Yazelix ASCII Art for Welcome Screen
# Diagonal magic rays with YAZELIX sandwiched in the middle of each row

export def get_welcome_ascii_art [] {
    let purple = (ansi purple)
    let cyan = (ansi cyan)
    let blue = (ansi blue)
    let reset = (ansi reset)

    # Generate diagonal magic rays with YAZELIX in the middle
    let stream_width = 150
    let stream_height = 12
    let magic_stream = (
        0..($stream_height - 1) | each { |row|
            # Create diagonal rays
            let diagonal_offset = ($row * 5)
            let ray_start = (5 + $diagonal_offset)
            let ray_length = 80

            # Calculate the middle position for YAZELIX
            let total_width = ($ray_start + $ray_length)
            let yazelix_start = ($total_width / 2 - 3)  # YAZELIX is 7 chars, so center at -3
            let yazelix_pos = ($yazelix_start - $ray_start)

            # Create padding for the ray position
            let padding = ('' | fill -c ' ' -w $ray_start)

            # Generate the ray with YAZELIX in the middle
            let before_yazelix = (
                0..($yazelix_pos - 1) | each { |i|
                    let color_idx = ($i + $row) mod 3
                    let color = if $color_idx == 0 { $purple } else if $color_idx == 1 { $cyan } else { $blue }
                    let symbol = if ($i mod 2) == 0 { "█" } else { "▓" }
                    $color + $symbol
                } | str join ""
            )

            let yazelix_text = "   \u{1b}[35mY\u{1b}[36mA\u{1b}[34mZ\u{1b}[35mE\u{1b}[36mL\u{1b}[34mI\u{1b}[35mX   "

            let after_yazelix = (
                ($yazelix_pos + 7)..($ray_length - 1) | each { |i|
                    let color_idx = ($i + $row) mod 3
                    let color = if $color_idx == 0 { $purple } else if $color_idx == 1 { $cyan } else { $blue }
                    let symbol = if ($i mod 2) == 0 { "█" } else { "▓" }
                    $color + $symbol
                } | str join ""
            )

            let ray_content = $before_yazelix + $yazelix_text + $after_yazelix

            $padding + $ray_content + $reset
        }
    )

    # Return just the magic stream (no separate YAZELIX line needed)
    $magic_stream
}