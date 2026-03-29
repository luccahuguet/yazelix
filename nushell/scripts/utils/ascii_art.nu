#!/usr/bin/env nu
# Width-aware welcome art for Yazelix.

export const WELCOME_STYLE_VALUES = ["static", "logo", "boids", "life", "mandelbrot", "random"]
export const ANIMATED_WELCOME_STYLE_VALUES = ["logo", "boids", "life", "mandelbrot"]

# Export the color scheme used in the welcome art for consistent styling.
export def get_yazelix_colors [] {
    {
        purple: (ansi purple)
        cyan: (ansi cyan)
        blue: (ansi blue)
        green: (ansi green)
        yellow: (ansi yellow)
        reset: (ansi reset)
        faint: "\u{1b}[2m"
        bold: "\u{1b}[1m"
    }
}

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

def strip_ansi_codes [text: string] {
    $text | str replace -ar "\\x1b\\[[0-9;]*m" ""
}

export def get_visible_line_width [line: string] {
    strip_ansi_codes $line | split chars | length
}

export def get_max_visible_width [lines: list<string>] {
    if ($lines | is-empty) {
        0
    } else {
        $lines | each {|line| get_visible_line_width $line } | math max
    }
}

export def get_terminal_width [] {
    let explicit = ($env.YAZELIX_WELCOME_WIDTH? | default "")
    if ($explicit | is-not-empty) {
        return ($explicit | into int)
    }

    try {
        let size = (term size)
        let width = ($size.columns? | default 80)
        if ($width | into int) > 0 {
            $width | into int
        } else {
            80
        }
    } catch {
        80
    }
}

export def get_logo_welcome_variant [width?: int] {
    let resolved_width = ($width | default (get_terminal_width) | into int)

    if $resolved_width < 44 {
        "narrow"
    } else if $resolved_width < 72 {
        "medium"
    } else {
        "wide"
    }
}

def center_text [text: string, width: int] {
    let visible_width = ($text | str length)
    if $visible_width >= $width {
        return $text
    }

    let left_padding = ((($width - $visible_width) / 2) | math floor)
    let right_padding = ($width - $visible_width - $left_padding)
    $"((' ' | fill -w $left_padding))($text)((' ' | fill -w $right_padding))"
}

def pad_text_right [text: string, width: int] {
    let visible_width = ($text | str length)
    if $visible_width >= $width {
        return $text
    }

    $"($text)((' ' | fill -w ($width - $visible_width)))"
}

def repeat_char [character: string, count: int] {
    if $count <= 0 {
        ""
    } else {
        0..($count - 1) | each { $character } | str join ""
    }
}

def colorize_logo_text [text: string] {
    let colors = get_yazelix_colors
    let palette = [$colors.purple, $colors.cyan, $colors.blue]
    let reset = $colors.reset
    let chars = ($text | split chars)

    $chars
    | enumerate
    | each {|item|
        if $item.item == " " {
            " "
        } else {
            let color = ($palette | get ($item.index mod ($palette | length)))
            $"($color)($item.item)($reset)"
        }
    }
    | str join ""
}

def colorize_body_line [text: string, index: int] {
    let colors = get_yazelix_colors
    let color = if ($index mod 2) == 0 { $colors.cyan } else { $colors.blue }
    $"($color)($text)($colors.reset)"
}

def make_border [inner_width: int, character: string] {
    repeat_char $character ($inner_width + 2)
}

def build_logo_card_frame [spec: record, shown_body_count: int, accent: string = "full"] {
    let colors = get_yazelix_colors
    let inner_width = ($spec.inner_width | into int)
    let title_text = if $accent == "hint" { "YZX" } else { "YAZELIX" }
    let title_plain = (center_text $title_text $inner_width)
    let title_colored = if $accent == "hint" {
        $"($colors.faint)($colors.purple)($title_plain)($colors.reset)"
    } else {
        colorize_logo_text $title_plain
    }

    let body_lines = (
        $spec.body_lines
        | enumerate
        | each {|item|
            if $item.index < $shown_body_count {
                colorize_body_line (pad_text_right $item.item $inner_width) $item.index
            } else {
                $"($colors.faint)(pad_text_right "" $inner_width)($colors.reset)"
            }
        }
    )

    let footer_plain = (center_text $spec.footer $inner_width)
    let footer_colored = $"($colors.faint)($colors.purple)($footer_plain)($colors.reset)"

    [
        $"($colors.purple)╭(make_border $inner_width "─")╮($colors.reset)"
        $"($colors.purple)│($colors.reset)($title_colored)($colors.purple)│($colors.reset)"
        ...($body_lines | each {|line| $"($colors.purple)│($colors.reset)($line)($colors.purple)│($colors.reset)" })
        $"($colors.purple)│($colors.reset)($footer_colored)($colors.purple)│($colors.reset)"
        $"($colors.purple)╰(make_border $inner_width "─")╯($colors.reset)"
    ]
}

def get_logo_welcome_spec [variant: string] {
    match $variant {
        "narrow" => {
            {
                inner_width: 22
                body_lines: [
                    "yazi zellij helix"
                    "one shell. one flow."
                ]
                footer: "welcome to yazelix"
            }
        }
        "medium" => {
            {
                inner_width: 34
                body_lines: [
                    "yazi + zellij + helix"
                    "one shell, one workspace"
                    "alt+shift+m opens yzx menu"
                ]
                footer: "welcome to yazelix"
            }
        }
        "wide" => {
            {
                inner_width: 46
                body_lines: [
                    "yazi + zellij + helix, wired together"
                    "one shell, one workspace, one real flow"
                    "alt+shift+m menu · ctrl+y sidebar"
                ]
                footer: "welcome to yazelix"
            }
        }
        _ => {
            error make {msg: $"Unsupported logo welcome variant: ($variant)"}
        }
    }
}

export def get_logo_welcome_frame [width?: int] {
    let variant = (get_logo_welcome_variant $width)
    let spec = (get_logo_welcome_spec $variant)
    build_logo_card_frame $spec ($spec.body_lines | length)
}

export def get_logo_animation_frames [width?: int] {
    let variant = (get_logo_welcome_variant $width)
    let spec = (get_logo_welcome_spec $variant)
    let final_count = ($spec.body_lines | length)

    [
        (build_logo_card_frame $spec 0 "hint")
        (build_logo_card_frame $spec 0 "full")
        (build_logo_card_frame $spec 1 "full")
        (build_logo_card_frame $spec $final_count "full")
    ]
}

export def get_welcome_ascii_art [width?: int] {
    get_logo_welcome_frame $width
}

export def get_animated_ascii_art [width?: int] {
    get_logo_animation_frames $width
}

export def play_frames [frames: list<list<string>>, duration: duration] {
    if ($frames | is-empty) {
        return
    }

    let frame_delay = ($duration / ($frames | length))
    let art_height = (($frames | first) | length)

    for frame in $frames {
        print ($frame | str join "\n")
        sleep $frame_delay
        print ("\u{1b}[" + (($art_height + 1) | into string) + "A")
    }

    print ((0..($art_height - 1) | each { "" } | str join "\n"))
}

export def play_animation [duration: duration, width?: int] {
    let frames = (get_animated_ascii_art $width)
    play_frames $frames $duration
}

export def render_welcome_style [welcome_style: string, duration: duration = 0.5sec, width?: int] {
    let resolved_style = (resolve_welcome_style $welcome_style)

    if $resolved_style == "static" {
        let ascii_art = (get_welcome_ascii_art $width)
        for line in $ascii_art {
            print $line
        }
        print ""
        return
    }

    if $resolved_style == "logo" {
        print ""
        play_animation $duration $width
        return
    }

    if $resolved_style in ["boids", "life", "mandelbrot"] {
        print ""
        # Dedicated renderers land in their own welcome-style beads.
        # Until then, animated styles share the logo-forward reveal contract.
        play_animation $duration $width
        return
    }

    error make {msg: $"Unsupported welcome_style: ($resolved_style)"}
}
