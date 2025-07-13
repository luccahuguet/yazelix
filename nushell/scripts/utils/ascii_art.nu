#!/usr/bin/env nu
# Yazelix ASCII Art for Welcome Screen
# Hybrid: Purple ball of magic + colorful YAZELIX text

export def get_welcome_ascii_art [] {
    let pink = (ansi magenta)
    let purple = (ansi purple)
    let accent = (ansi blue)
    let yellow = (ansi yellow)
    let cyan = (ansi cyan)
    let red = (ansi red)
    let green = (ansi green)
    let reset = (ansi reset)

    # Generate magic stream layers using a wide, wavy pattern
    let stream_width = 120
    let stream_height = 20
    let magic_stream = (
        0..($stream_height - 1) | each { |row|
            # Calculate wave offset step by step to avoid linter errors
            let rowf = ($row | into float)
            let half = ($rowf / 2)
            let wave = ($half | math sin)
            let wave10 = ($wave * 10)
            let wave_offset = ($wave10 | math round | into int)
            let padding = (20 + $wave_offset)
            let pad_str = ('' | fill -c ' ' -w $padding)
            let colors = [$pink, $purple, $yellow, $cyan, $red, $green, $accent]
            let symbols = ["✦", "✧", "✩", "✪", "✫", "✬", "✭", "✮", "✯"]
            let line_content = (
                0..($stream_width - 1) | each { |i|
                    let color_idx = ($i + $row) mod ($colors | length)
                    let symbol_idx = ($i + $row * 2) mod ($symbols | length)
                    ($colors | get $color_idx) + ($symbols | get $symbol_idx)
                } | str join ""
            )
            $pad_str + $line_content + $reset
        }
    )

    # Add YAZELIX text with proper color rendering using raw ANSI codes
    let yazelix_padding = ('' | fill -c ' ' -w 20)
    let yazelix_line = (
        $yazelix_padding +
        "\u{1b}[35mY" +
        "\u{1b}[31mA" +
        "\u{1b}[33mZ" +
        "\u{1b}[36mE" +
        "\u{1b}[32mL" +
        "\u{1b}[95mI" +
        "\u{1b}[34mX" +
        "\u{1b}[0m"
    )

    # Combine all lines
    ($magic_stream | append "" | append $yazelix_line)
}