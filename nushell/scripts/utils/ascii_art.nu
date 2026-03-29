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
    } else if $resolved_width < 120 {
        "wide"
    } else {
        "hero"
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

def fit_inner_width [resolved_width: int, minimum_width: int] {
    let proposed_width = ($resolved_width - 4)
    if $proposed_width < $minimum_width {
        $minimum_width
    } else {
        $proposed_width
    }
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

def colorize_boid_char [char: string, index: int] {
    let colors = get_yazelix_colors
    let palette = [$colors.cyan, $colors.blue, $colors.purple]
    let color = ($palette | get ($index mod ($palette | length)))
    $"($color)($char)($colors.reset)"
}

def colorize_life_char [x: int, y: int] {
    let colors = get_yazelix_colors
    let palette = [$colors.green, $colors.cyan, $colors.blue, $colors.purple]
    let color = ($palette | get (($x + $y) mod ($palette | length)))
    $"($color)■($colors.reset)"
}

def make_border [inner_width: int, character: string] {
    repeat_char $character $inner_width
}

def build_logo_card_frame [spec: record, shown_body_count: int, accent: string = "full"] {
    let colors = get_yazelix_colors
    let inner_width = ($spec.inner_width | into int)
    let title_text = if $accent == "hint" {
        ($spec.title_hint_text? | default "YZX")
    } else {
        ($spec.title_text? | default "YAZELIX")
    }
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
            let aligned_line = if (($spec.body_alignment? | default "left") == "center") {
                center_text $item.item $inner_width
            } else {
                pad_text_right $item.item $inner_width
            }

            if $item.index < $shown_body_count {
                colorize_body_line $aligned_line $item.index
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

def get_logo_welcome_spec [variant: string, resolved_width: int] {
    match $variant {
        "narrow" => {
            {
                inner_width: (fit_inner_width $resolved_width 22)
                title_text: "YAZELIX"
                title_hint_text: "YZX"
                body_alignment: "left"
                body_lines: [
                    "yazi zellij helix"
                    "one shell. one flow."
                ]
                footer: "welcome to yazelix"
            }
        }
        "medium" => {
            {
                inner_width: (fit_inner_width $resolved_width 34)
                title_text: "YAZELIX"
                title_hint_text: "YZX"
                body_alignment: "left"
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
                inner_width: (fit_inner_width $resolved_width 58)
                title_text: "YAZELIX"
                title_hint_text: "YZX"
                body_alignment: "center"
                body_lines: [
                    "yazi + zellij + helix, wired together and ready"
                    "one shell, one workspace, one real flow"
                    "alt+shift+m menu · ctrl+y sidebar jump"
                    "packs, sessions, terminals, all under one roof"
                ]
                footer: "welcome to yazelix"
            }
        }
        "hero" => {
            {
                inner_width: (fit_inner_width $resolved_width 76)
                title_text: "Y A Z E L I X"
                title_hint_text: "Y Z X"
                body_alignment: "center"
                body_lines: [
                    "yazi + zellij + helix, wired together and ready"
                    "one shell, one workspace, one real flow"
                    "sidebar, editor, sessions, packs, and terminals already aligned"
                    "alt+shift+m menu · ctrl+y sidebar jump · alt+[ / alt+] layout family"
                    "launch once, then stay in flow"
                ]
                footer: "welcome to yazelix"
            }
        }
        _ => {
            error make {msg: $"Unsupported logo welcome variant: ($variant)"}
        }
    }
}

def get_boids_welcome_spec [variant: string, resolved_width: int] {
    match $variant {
        "narrow" => {
            {
                inner_width: (fit_inner_width $resolved_width 22)
                body_height: 4
                caption: "flocking..."
            }
        }
        "medium" => {
            {
                inner_width: (fit_inner_width $resolved_width 34)
                body_height: 5
                caption: "flocking..."
            }
        }
        "wide" => {
            {
                inner_width: (fit_inner_width $resolved_width 58)
                body_height: 5
                caption: "flocking..."
            }
        }
        "hero" => {
            {
                inner_width: (fit_inner_width $resolved_width 76)
                body_height: 7
                caption: "flocking..."
            }
        }
        _ => {
            error make {msg: $"Unsupported boids welcome variant: ($variant)"}
        }
    }
}

def get_life_welcome_spec [variant: string, resolved_width: int] {
    match $variant {
        "narrow" => {
            {
                inner_width: (fit_inner_width $resolved_width 22)
                body_height: 4
            }
        }
        "medium" => {
            {
                inner_width: (fit_inner_width $resolved_width 34)
                body_height: 5
            }
        }
        "wide" => {
            {
                inner_width: (fit_inner_width $resolved_width 58)
                body_height: 5
            }
        }
        "hero" => {
            {
                inner_width: (fit_inner_width $resolved_width 76)
                body_height: 7
            }
        }
        _ => {
            error make {msg: $"Unsupported life welcome variant: ($variant)"}
        }
    }
}

def make_boid_point [x: int, y: int, char: string, index: int] {
    { x: $x, y: $y, char: $char, index: $index }
}

def get_boid_positions [spec: record, phase: string] {
    let width = ($spec.inner_width | into int)
    let height = ($spec.body_height | into int)
    let mid_x = ($width / 2 | math floor)
    let low_y = if $height > 2 { $height - 2 } else { 1 }
    let mid_y = ($height / 2 | math floor)

    match $phase {
        "scatter" => [
            (make_boid_point 1 0 ">" 0)
            (make_boid_point ($width - 2) 0 "<" 1)
            (make_boid_point 3 $low_y "^" 2)
            (make_boid_point ($width - 4) $low_y "v" 3)
            (make_boid_point ($mid_x - 6) $mid_y "*" 4)
            (make_boid_point ($mid_x + 5) $mid_y "*" 5)
        ]
        "drift" => [
            (make_boid_point ($mid_x - 8) 1 ">" 0)
            (make_boid_point ($mid_x + 7) 1 "<" 1)
            (make_boid_point ($mid_x - 5) $mid_y "^" 2)
            (make_boid_point ($mid_x + 4) $mid_y "v" 3)
            (make_boid_point ($mid_x - 2) ($low_y - 1) "*" 4)
            (make_boid_point ($mid_x + 1) ($low_y - 1) "*" 5)
        ]
        "cluster" => [
            (make_boid_point ($mid_x - 4) 1 ">" 0)
            (make_boid_point ($mid_x + 3) 1 "<" 1)
            (make_boid_point ($mid_x - 2) $mid_y "^" 2)
            (make_boid_point ($mid_x + 1) $mid_y "v" 3)
            (make_boid_point ($mid_x - 6) ($low_y - 1) "*" 4)
            (make_boid_point ($mid_x + 5) ($low_y - 1) "*" 5)
        ]
        _ => {
            error make {msg: $"Unsupported boids phase: ($phase)"}
        }
    }
}

def render_boid_row [width: int, row_index: int, points: list<record>, caption?: string] {
    0..($width - 1)
    | each {|x|
        let point = ($points | where x == $x and y == $row_index | get -o 0)
        if $point == null {
            " "
        } else {
            colorize_boid_char $point.char $point.index
        }
    }
    | str join ""
}

def build_boids_frame [spec: record, phase: string] {
    let colors = get_yazelix_colors
    let width = ($spec.inner_width | into int)
    let height = ($spec.body_height | into int)
    let points = (get_boid_positions $spec $phase)
    let caption_row = if $phase == "cluster" {
        ($height - 1)
    } else {
        -1
    }
    let caption = if $phase == "cluster" {
        $"($colors.faint)($colors.purple)(center_text $spec.caption $width)($colors.reset)"
    } else {
        null
    }

    let body = (
        0..($height - 1)
        | each {|row_index|
            let row = if ($caption != null) and ($row_index == $caption_row) {
                $caption
            } else {
                render_boid_row $width $row_index $points
            }
            $"($colors.purple)│($colors.reset)($row)($colors.purple)│($colors.reset)"
        }
    )

    [
        $"($colors.purple)╭(make_border $width "─")╮($colors.reset)"
        ...$body
        $"($colors.purple)╰(make_border $width "─")╯($colors.reset)"
    ]
}

def make_life_cell [x: int, y: int] {
    { x: $x, y: $y }
}

def life_cell_key [cell: record] {
    $"($cell.x),($cell.y)"
}

def unique_life_cells [cells: list<record>] {
    mut keys = []
    mut unique = []

    for cell in $cells {
        let key = (life_cell_key $cell)
        if not ($key in $keys) {
            $keys = ($keys | append $key)
            $unique = ($unique | append $cell)
        }
    }

    $unique
}

def has_life_cell [cells: list<record>, x: int, y: int] {
    $cells | any {|cell| ($cell.x == $x) and ($cell.y == $y) }
}

def get_life_seed [spec: record] {
    let width = ($spec.inner_width | into int)
    let height = ($spec.body_height | into int)
    let mid_x = ($width / 2 | math floor)
    let mid_y = ($height / 2 | math floor)

    unique_life_cells [
        (make_life_cell 3 1)
        (make_life_cell 4 1)
        (make_life_cell 5 1)
        (make_life_cell ($width - 6) ($height - 2))
        (make_life_cell ($width - 5) ($height - 2))
        (make_life_cell ($width - 4) ($height - 2))
        (make_life_cell ($mid_x - 1) ($mid_y - 1))
        (make_life_cell $mid_x ($mid_y - 1))
        (make_life_cell ($mid_x + 1) ($mid_y - 1))
        (make_life_cell ($mid_x - 1) $mid_y)
        (make_life_cell $mid_x ($mid_y + 1))
    ]
}

def count_live_neighbors [cells: list<record>, x: int, y: int] {
    mut count = 0

    for ny in [($y - 1), $y, ($y + 1)] {
        for nx in [($x - 1), $x, ($x + 1)] {
            if ($nx == $x) and ($ny == $y) {
                continue
            }

            if (has_life_cell $cells $nx $ny) {
                $count += 1
            }
        }
    }

    $count
}

def step_life_cells [cells: list<record>, width: int, height: int] {
    mut candidates = []

    for cell in $cells {
        for ny in [($cell.y - 1), $cell.y, ($cell.y + 1)] {
            if ($ny < 0) or ($ny >= $height) {
                continue
            }

            for nx in [($cell.x - 1), $cell.x, ($cell.x + 1)] {
                if ($nx < 0) or ($nx >= $width) {
                    continue
                }

                $candidates = ($candidates | append [(make_life_cell $nx $ny)])
            }
        }
    }

    let unique_candidates = (unique_life_cells $candidates)
    mut next_cells = []

    for candidate in $unique_candidates {
        let neighbors = (count_live_neighbors $cells $candidate.x $candidate.y)
        let alive = (has_life_cell $cells $candidate.x $candidate.y)

        if ($neighbors == 3) or ($alive and ($neighbors == 2)) {
            $next_cells = ($next_cells | append [$candidate])
        }
    }

    unique_life_cells $next_cells
}

def render_life_row [width: int, row_index: int, cells: list<record>] {
    0..($width - 1)
    | each {|x|
        if (has_life_cell $cells $x $row_index) {
            colorize_life_char $x $row_index
        } else {
            " "
        }
    }
    | str join ""
}

def build_life_frame [spec: record, cells: list<record>] {
    let colors = get_yazelix_colors
    let width = ($spec.inner_width | into int)
    let height = ($spec.body_height | into int)
    let body = (
        0..($height - 1)
        | each {|row_index|
            let row = (render_life_row $width $row_index $cells)
            $"($colors.purple)│($colors.reset)($row)($colors.purple)│($colors.reset)"
        }
    )

    [
        $"($colors.purple)╭(make_border $width "─")╮($colors.reset)"
        ...$body
        $"($colors.purple)╰(make_border $width "─")╯($colors.reset)"
    ]
}

export def get_logo_welcome_frame [width?: int] {
    let resolved_width = ($width | default (get_terminal_width) | into int)
    let variant = (get_logo_welcome_variant $resolved_width)
    let spec = (get_logo_welcome_spec $variant $resolved_width)
    build_logo_card_frame $spec ($spec.body_lines | length)
}

export def get_logo_animation_frames [width?: int] {
    let resolved_width = ($width | default (get_terminal_width) | into int)
    let variant = (get_logo_welcome_variant $resolved_width)
    let spec = (get_logo_welcome_spec $variant $resolved_width)
    let final_count = ($spec.body_lines | length)

    [
        (build_logo_card_frame $spec 0 "hint")
        (build_logo_card_frame $spec 0 "full")
        (build_logo_card_frame $spec 1 "full")
        (build_logo_card_frame $spec $final_count "full")
    ]
}

export def get_boids_animation_frames [width?: int] {
    let resolved_width = ($width | default (get_terminal_width) | into int)
    let variant = (get_logo_welcome_variant $resolved_width)
    let spec = (get_boids_welcome_spec $variant $resolved_width)

    [
        (build_boids_frame $spec "scatter")
        (build_boids_frame $spec "drift")
        (build_boids_frame $spec "cluster")
        (get_logo_welcome_frame $width)
    ]
}

export def get_life_animation_frames [width?: int] {
    let resolved_width = ($width | default (get_terminal_width) | into int)
    let variant = (get_logo_welcome_variant $resolved_width)
    let spec = (get_life_welcome_spec $variant $resolved_width)
    let width_limit = ($spec.inner_width | into int)
    let height_limit = ($spec.body_height | into int)
    let frame0_cells = (get_life_seed $spec)
    let frame1_cells = (step_life_cells $frame0_cells $width_limit $height_limit)
    let frame2_cells = (step_life_cells $frame1_cells $width_limit $height_limit)

    [
        (build_life_frame $spec $frame0_cells)
        (build_life_frame $spec $frame1_cells)
        (build_life_frame $spec $frame2_cells)
        (get_logo_welcome_frame $width)
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
        for line in $frame {
            print $"\r\u{1b}[2K($line)"
        }
        sleep $frame_delay
        print ("\u{1b}[" + (($art_height + 1) | into string) + "A")
    }

    print ((0..($art_height - 1) | each { "" } | str join "\n"))
}

export def play_animation [duration: duration, width?: int] {
    let frames = (get_animated_ascii_art $width)
    play_frames $frames $duration
}

def get_welcome_playback_duration [welcome_style: string, duration: duration] {
    if $welcome_style == "life" {
        2sec
    } else {
        $duration
    }
}

export def render_welcome_style [welcome_style: string, duration: duration = 0.5sec, width?: int] {
    let resolved_style = (resolve_welcome_style $welcome_style)
    let playback_duration = (get_welcome_playback_duration $resolved_style $duration)

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
        play_animation $playback_duration $width
        return
    }

    if $resolved_style == "boids" {
        print ""
        play_frames (get_boids_animation_frames $width) $playback_duration
        return
    }

    if $resolved_style == "life" {
        print ""
        play_frames (get_life_animation_frames $width) $playback_duration
        return
    }

    if $resolved_style == "mandelbrot" {
        print ""
        # Dedicated renderers land in their own welcome-style beads.
        # Until then, animated styles share the logo-forward reveal contract.
        play_animation $playback_duration $width
        return
    }

    error make {msg: $"Unsupported welcome_style: ($resolved_style)"}
}
