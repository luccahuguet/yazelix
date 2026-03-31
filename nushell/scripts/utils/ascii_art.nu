#!/usr/bin/env nu
# Width-aware welcome art for Yazelix.

export const WELCOME_STYLE_VALUES = ["static", "logo", "boids", "game_of_life", "mandelbrot", "random"]
export const ANIMATED_WELCOME_STYLE_VALUES = ["game_of_life"]
export const SCREEN_STYLE_VALUES = ["logo", "boids", "game_of_life", "random"]

# Export the color scheme used in the welcome art for consistent styling.
export def get_yazelix_colors [] {
    {
        red: (ansi red)
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

export def get_screen_style_random_pool [] {
    get_welcome_style_random_pool
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

    $pool | get -o $selected_index
}

def trim_resting_frame [frames: list<list<string>>] {
    if (($frames | length) <= 1) {
        $frames
    } else {
        $frames | first (($frames | length) - 1)
    }
}

export def resolve_screen_style [screen_style?: string, random_index?: int] {
    let requested = ($screen_style | default "random" | into string | str downcase)

    if not ($requested in $SCREEN_STYLE_VALUES) {
        let allowed_text = ($SCREEN_STYLE_VALUES | str join ", ")
        error make {msg: $"Invalid screen style '($requested)'. Expected one of: ($allowed_text)"}
    }

    if $requested == "random" {
        return (resolve_welcome_style "random" $random_index)
    }

    $requested
}

export def get_screen_cycle_frames [screen_style: string, width?: int] {
    let resolved_style = (resolve_screen_style $screen_style)

    match $resolved_style {
        "logo" => (trim_resting_frame (get_logo_animation_frames $width))
        "boids" => (trim_resting_frame (get_boids_animation_frames $width))
        "game_of_life" => (get_game_of_life_screen_cycle_frames $width)
        _ => {
            error make {msg: $"Unsupported screen style: ($resolved_style)"}
        }
    }
}

export def get_screen_frame_delay [screen_style: string] {
    let resolved_style = (resolve_screen_style $screen_style)

    match $resolved_style {
        "game_of_life" => 160ms
        "logo" => 120ms
        "boids" => 120ms
        _ => 140ms
    }
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

export def get_terminal_height [] {
    let explicit = ($env.YAZELIX_WELCOME_HEIGHT? | default "")
    if ($explicit | is-not-empty) {
        return ($explicit | into int)
    }

    try {
        let size = (term size)
        let height = ($size.rows? | default 24)
        if ($height | into int) > 0 {
            $height | into int
        } else {
            24
        }
    } catch {
        24
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
    let visible_width = (get_visible_line_width $text)
    if $visible_width >= $width {
        return $text
    }

    let left_padding = ((($width - $visible_width) / 2) | math floor)
    let right_padding = ($width - $visible_width - $left_padding)
    $"((' ' | fill -w $left_padding))($text)((' ' | fill -w $right_padding))"
}

def pad_text_right [text: string, width: int] {
    let visible_width = (get_visible_line_width $text)
    if $visible_width >= $width {
        return $text
    }

    $"($text)((' ' | fill -w ($width - $visible_width)))"
}

def fit_inner_width [resolved_width: int, minimum_width: int] {
    let proposed_width = ($resolved_width - 6)
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
    let palette = [$colors.red, $colors.green, $colors.yellow, $colors.blue, $colors.purple]
    let reset = $colors.reset
    let chars = ($text | split chars)

    $chars
    | enumerate
    | each {|item|
        if $item.item == " " {
            " "
        } else {
            let color = ($palette | get -o ($item.index mod ($palette | length)) | default $colors.red)
            $"($color)($item.item)($reset)"
        }
    }
    | str join ""
}

def colorize_body_line [text: string] {
    let colors = get_yazelix_colors
    let base_color = $colors.green
    let accent_color = $colors.blue
    let base = $"($base_color)($text)($colors.reset)"

    (
        $base
        | str replace -a "reproducible" $"($accent_color)reproducible($base_color)"
        | str replace -a "declarative" $"($accent_color)declarative($base_color)"
        | str replace -a "helix" $"($accent_color)helix($base_color)"
        | str replace -a "zellij" $"($accent_color)zellij($base_color)"
        | str replace -a "terminals" $"($accent_color)terminals($base_color)"
        | str replace -a "shells" $"($accent_color)shells($base_color)"
        | str replace -a "packs" $"($accent_color)packs($base_color)"
        | str replace -a "SSH" $"($accent_color)SSH($base_color)"
    )
}

def colorize_footer_text [text: string] {
    let colors = get_yazelix_colors
    $"($colors.yellow)($text)($colors.reset)"
}

def colorize_boid_char [char: string, index: int] {
    let colors = get_yazelix_colors
    let palette = [$colors.cyan, $colors.blue, $colors.purple]
    let color = ($palette | get -o ($index mod ($palette | length)) | default $colors.cyan)
    $"($color)($char)($colors.reset)"
}

def colorize_game_of_life_char [x: int, y: int] {
    let colors = get_yazelix_colors
    let palette = [$colors.green, $colors.cyan, $colors.blue, $colors.purple]
    let color = ($palette | get -o (($x + $y) mod ($palette | length)) | default $colors.green)
    $"($color)█($colors.reset)"
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
                colorize_body_line $aligned_line
            } else {
                $"($colors.faint)(pad_text_right "" $inner_width)($colors.reset)"
            }
        }
    )

    let footer_plain = (center_text $spec.footer $inner_width)
    let footer_colored = (colorize_footer_text $footer_plain)

    [
        $"($colors.purple)╭(make_border $inner_width "─")╮($colors.reset)"
        $"($colors.purple)│($colors.reset)($title_colored)($colors.purple)│($colors.reset)"
        ...($body_lines | each {|line| $"($colors.purple)│($colors.reset)($line)($colors.purple)│($colors.reset)" })
        $"($colors.purple)│($colors.reset)($footer_colored)($colors.purple)│($colors.reset)"
        $"($colors.purple)╰(make_border $inner_width "─")╯($colors.reset)"
    ]
}

def center_frame_lines [lines: list<string>, target_width: int] {
    $lines | each {|line| center_text $line $target_width }
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
                    "your reproducible terminal IDE"
                    "zero-conflict helix/zellij keys"
                    "top terminals, shells, and packs"
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
                    "your reproducible, declarative terminal IDE"
                    "zero-conflict keybindings between helix and zellij"
                    "supports all top terminals and shells"
                    "curated program packs \(all configurable\)"
                ]
                footer: "welcome to yazelix"
            }
        }
        "hero" => {
            {
                inner_width: (fit_inner_width $resolved_width 76)
                title_text: "YAZELIX"
                title_hint_text: "YZX"
                body_alignment: "center"
                body_lines: [
                    "your reproducible, declarative terminal IDE"
                    "zero-conflict keybindings between helix and zellij"
                    "supports all top terminals and shells"
                    "curated program packs \(all configurable\)"
                    "shines over SSH"
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

def resolve_game_of_life_body_height [minimum_height: int, resolved_height: int] {
    let fullscreen_height = ($resolved_height - 6)
    if $fullscreen_height > $minimum_height {
        $fullscreen_height
    } else {
        $minimum_height
    }
}

def resolve_game_of_life_screen_body_height [minimum_height: int, resolved_height: int] {
    if $resolved_height > $minimum_height {
        $resolved_height
    } else {
        $minimum_height
    }
}

def get_game_of_life_welcome_spec [variant: string, resolved_width: int, resolved_height: int] {
    match $variant {
        "narrow" => {
            {
                inner_width: (fit_inner_width $resolved_width 22)
                body_height: (resolve_game_of_life_body_height 8 $resolved_height)
            }
        }
        "medium" => {
            {
                inner_width: (fit_inner_width $resolved_width 34)
                body_height: (resolve_game_of_life_body_height 12 $resolved_height)
            }
        }
        "wide" => {
            {
                inner_width: (fit_inner_width $resolved_width 58)
                body_height: (resolve_game_of_life_body_height 14 $resolved_height)
            }
        }
        "hero" => {
            {
                inner_width: (fit_inner_width $resolved_width 76)
                body_height: (resolve_game_of_life_body_height 16 $resolved_height)
            }
        }
        _ => {
            error make {msg: $"Unsupported game_of_life welcome variant: ($variant)"}
        }
    }
}

def get_game_of_life_screen_spec [variant: string, resolved_width: int, resolved_height: int] {
    match $variant {
        "narrow" => {
            {
                inner_width: (fit_inner_width $resolved_width 22)
                body_height: (resolve_game_of_life_screen_body_height 8 $resolved_height)
            }
        }
        "medium" => {
            {
                inner_width: (fit_inner_width $resolved_width 34)
                body_height: (resolve_game_of_life_screen_body_height 12 $resolved_height)
            }
        }
        "wide" => {
            {
                inner_width: (fit_inner_width $resolved_width 58)
                body_height: (resolve_game_of_life_screen_body_height 14 $resolved_height)
            }
        }
        "hero" => {
            {
                inner_width: (fit_inner_width $resolved_width 76)
                body_height: (resolve_game_of_life_screen_body_height 16 $resolved_height)
            }
        }
        _ => {
            error make {msg: $"Unsupported game_of_life screen variant: ($variant)"}
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

def render_boid_row [width: int, row_index: int, points: list<record>] {
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

def make_game_of_life_cell [x: int, y: int] {
    { x: $x, y: $y }
}

def game_of_life_cell_key [cell: record] {
    $"($cell.x),($cell.y)"
}

def unique_game_of_life_cells [cells: list<record>] {
    mut keys = []
    mut unique = []

    for cell in $cells {
        let key = (game_of_life_cell_key $cell)
        if not ($key in $keys) {
            $keys = ($keys | append $key)
            $unique = ($unique | append $cell)
        }
    }

    $unique
}

def offset_game_of_life_shape [shape: list<list<int>>, offset_x: int, offset_y: int] {
    $shape | each {|pair|
        let x = ($pair | get -o 0)
        let y = ($pair | get -o 1)
        if ($x == null) or ($y == null) {
            error make {msg: $"Invalid game-of-life shape pair: ($pair | to json -r)"}
        }

        make_game_of_life_cell ($x + $offset_x) ($y + $offset_y)
    }
}

def get_right_glider_shape [] {
    [[1 0] [2 1] [0 2] [1 2] [2 2]]
}

def build_record_cell_map [cells: list<record>] {
    mut cell_map = {}

    for cell in $cells {
        $cell_map = ($cell_map | upsert (game_of_life_cell_key $cell) true)
    }

    $cell_map
}

def add_neighbor_count [counts: record, x: int, y: int] {
    let key = $"($x),($y)"
    let current = ($counts | get -o $key | default 0)
    $counts | upsert $key ($current + 1)
}

def split_game_of_life_key [key: string] {
    let parts = ($key | split row ",")
    let x_part = ($parts | get -o 0)
    let y_part = ($parts | get -o 1)
    if ($x_part == null) or ($y_part == null) {
        error make {msg: $"Invalid game-of-life cell key: ($key)"}
    }

    { x: ($x_part | into int), y: ($y_part | into int) }
}

def count_game_of_life_neighbors_record [cells: list<record>, width: int, height: int] {
    mut counts = {}

    for cell in $cells {
        for ny in [($cell.y - 1), $cell.y, ($cell.y + 1)] {
            for nx in [($cell.x - 1), $cell.x, ($cell.x + 1)] {
                if ($nx == $cell.x) and ($ny == $cell.y) {
                    continue
                }

                let wrapped_x = (($nx + $width) mod $width)
                let wrapped_y = (($ny + $height) mod $height)
                $counts = (add_neighbor_count $counts $wrapped_x $wrapped_y)
            }
        }
    }

    $counts
}

def step_game_of_life_cells_fast [cells: list<record>, width: int, height: int] {
    let alive_map = (build_record_cell_map $cells)
    let neighbor_counts = (count_game_of_life_neighbors_record $cells $width $height)
    mut next_cells = []

    for column in ($neighbor_counts | transpose key value) {
        let point = (split_game_of_life_key $column.key)
        let alive = ($alive_map | get -o $column.key | default false)
        let neighbors = ($column.value | into int)

        if ($neighbors == 3) or ($alive and ($neighbors == 2)) {
            $next_cells = ($next_cells | append [(make_game_of_life_cell $point.x $point.y)])
        }
    }

    unique_game_of_life_cells $next_cells
}

def build_live_game_of_life_seed [spec: record] {
    let width = (get_game_of_life_grid_width ($spec.inner_width | into int))
    let height = ($spec.body_height | into int)
    let glider_count = if $width >= 36 {
        6
    } else if $width >= 22 {
        4
    } else {
        2
    }
    let max_start_y = if ($height - 3) < 0 { 0 } else { $height - 3 }
    let right_glider = (get_right_glider_shape)
    let right_edge_x = if ($width - 5) < 0 { 0 } else { $width - 5 }
    let inner_right_x = if ($width - 9) < 0 { 0 } else { $width - 9 }
    let middle_upper_y = (($height / 2) | math floor) - 3
    let middle_lower_y = (($height / 2) | math floor) + 1
    let raw_placements = if $glider_count == 2 {
        [
            { x: 1, y: 1 }
            { x: $right_edge_x, y: ($height - 4) }
        ]
    } else if $glider_count == 4 {
        [
            { x: 1, y: 1 }
            { x: $right_edge_x, y: 2 }
            { x: 4, y: ($height - 7) }
            { x: $inner_right_x, y: ($height - 4) }
        ]
    } else {
        [
            { x: 1, y: 1 }
            { x: $right_edge_x, y: 2 }
            { x: 3, y: $middle_upper_y }
            { x: $inner_right_x, y: $middle_lower_y }
            { x: 5, y: ($height - 7) }
            { x: $right_edge_x, y: ($height - 4) }
        ]
    }
    let placements = (
        $raw_placements
        | each {|placement|
            let row_int = ($placement.y | into int)
            let clamped_y = if $row_int > $max_start_y { $max_start_y } else if $row_int < 0 { 0 } else { $row_int }
            let col_int = ($placement.x | into int)
            let clamped_x = if $col_int < 0 { 0 } else if $col_int > ($width - 3) { ($width - 3) } else { $col_int }
            { x: $clamped_x, y: $clamped_y }
        }
    )

    let glider_cells = (
        $placements
        | each {|placement|
            offset_game_of_life_shape $right_glider $placement.x $placement.y
        }
        | flatten
    )

    unique_game_of_life_cells $glider_cells
}

def build_game_of_life_cell_keys [cells: list<record>] {
    $cells | each {|cell| game_of_life_cell_key $cell }
}

def get_game_of_life_grid_width [inner_width: int] {
    let grid_width = (($inner_width / 2) | math floor)
    if $grid_width < 1 { 1 } else { $grid_width }
}

def render_game_of_life_row [grid_width: int, inner_width: int, row_index: int, cell_keys: list<string>] {
    let row = (
    0..($grid_width - 1)
    | each {|x|
        if ($"($x),($row_index)" in $cell_keys) {
            $"(colorize_game_of_life_char $x $row_index)(colorize_game_of_life_char $x $row_index)"
        } else {
            "  "
        }
    }
    | str join ""
    )

    pad_text_right $row $inner_width
}

def build_game_of_life_screen_lines [spec: record, cells: list<record>, resolved_width: int] {
    let inner_width = ($spec.inner_width | into int)
    let height = ($spec.body_height | into int)
    let grid_width = (get_game_of_life_grid_width $inner_width)
    let cell_keys = (build_game_of_life_cell_keys $cells)

    let body = (
        0..($height - 1)
        | each {|row_index|
            render_game_of_life_row $grid_width $inner_width $row_index $cell_keys
        }
    )

    center_frame_lines $body $resolved_width
}

export def get_logo_welcome_frame [width?: int] {
    let resolved_width = ($width | default (get_terminal_width) | into int)
    let variant = (get_logo_welcome_variant $resolved_width)
    let spec = (get_logo_welcome_spec $variant $resolved_width)
    center_frame_lines (build_logo_card_frame $spec ($spec.body_lines | length)) $resolved_width
}

export def get_logo_animation_frames [width?: int] {
    let resolved_width = ($width | default (get_terminal_width) | into int)
    let variant = (get_logo_welcome_variant $resolved_width)
    let spec = (get_logo_welcome_spec $variant $resolved_width)
    let final_count = ($spec.body_lines | length)

    [
        (center_frame_lines (build_logo_card_frame $spec 0 "hint") $resolved_width)
        (center_frame_lines (build_logo_card_frame $spec 0 "full") $resolved_width)
        (center_frame_lines (build_logo_card_frame $spec 1 "full") $resolved_width)
        (center_frame_lines (build_logo_card_frame $spec $final_count "full") $resolved_width)
    ]
}

export def get_boids_animation_frames [width?: int] {
    let resolved_width = ($width | default (get_terminal_width) | into int)
    let variant = (get_logo_welcome_variant $resolved_width)
    let spec = (get_boids_welcome_spec $variant $resolved_width)

    [
        (center_frame_lines (build_boids_frame $spec "scatter") $resolved_width)
        (center_frame_lines (build_boids_frame $spec "drift") $resolved_width)
        (center_frame_lines (build_boids_frame $spec "cluster") $resolved_width)
        (get_logo_welcome_frame $width)
    ]
}

export def get_game_of_life_welcome_frame_delay [] {
    220ms
}

export def get_game_of_life_animation_frames [width?: int, duration_seconds: float = 2.0] {
    let resolved_width = ($width | default (get_terminal_width) | into int)
    let resolved_height = (get_terminal_height)
    let variant = (get_logo_welcome_variant $resolved_width)
    let spec = (get_game_of_life_welcome_spec $variant $resolved_width $resolved_height)
    let width_limit = (get_game_of_life_grid_width ($spec.inner_width | into int))
    let height_limit = ($spec.body_height | into int)
    let fixed_frame_delay = (get_game_of_life_welcome_frame_delay)
    let computed_frame_count = ((($duration_seconds * 1sec) / $fixed_frame_delay) | math ceil | into int)
    let simulation_frame_count = if $computed_frame_count < 2 { 2 } else { $computed_frame_count }
    mut current_cells = (build_live_game_of_life_seed $spec)
    mut simulation_frames = [(build_game_of_life_screen_lines $spec $current_cells $resolved_width)]

    for _ in 1..($simulation_frame_count - 1) {
        $current_cells = (step_game_of_life_cells_fast $current_cells $width_limit $height_limit)
        $simulation_frames = ($simulation_frames | append [(build_game_of_life_screen_lines $spec $current_cells $resolved_width)])
    }

    [
        ...$simulation_frames
        (get_logo_welcome_frame $width)
    ]
}

export def get_game_of_life_screen_cycle_frames [width?: int, height?: int, duration_seconds: float = 2.0] {
    let resolved_width = ($width | default (get_terminal_width) | into int)
    let resolved_height = ($height | default (get_terminal_height) | into int)
    let variant = (get_logo_welcome_variant $resolved_width)
    let spec = (get_game_of_life_screen_spec $variant $resolved_width $resolved_height)
    let width_limit = (get_game_of_life_grid_width ($spec.inner_width | into int))
    let height_limit = ($spec.body_height | into int)
    let fixed_frame_delay = (get_game_of_life_welcome_frame_delay)
    let computed_frame_count = ((($duration_seconds * 1sec) / $fixed_frame_delay) | math ceil | into int)
    let simulation_frame_count = if $computed_frame_count < 2 { 2 } else { $computed_frame_count }
    mut current_cells = (build_live_game_of_life_seed $spec)
    mut simulation_frames = [(build_game_of_life_screen_lines $spec $current_cells $resolved_width)]

    for _ in 1..($simulation_frame_count - 1) {
        $current_cells = (step_game_of_life_cells_fast $current_cells $width_limit $height_limit)
        $simulation_frames = ($simulation_frames | append [(build_game_of_life_screen_lines $spec $current_cells $resolved_width)])
    }

    $simulation_frames
}

export def get_game_of_life_screen_state [width?: int, height?: int] {
    let resolved_width = ($width | default (get_terminal_width) | into int)
    let resolved_height = ($height | default (get_terminal_height) | into int)
    let variant = (get_logo_welcome_variant $resolved_width)
    let spec = (get_game_of_life_screen_spec $variant $resolved_width $resolved_height)

    {
        resolved_width: $resolved_width
        resolved_height: $resolved_height
        spec: $spec
        cells: (build_live_game_of_life_seed $spec)
    }
}

export def step_game_of_life_screen_state [state: record] {
    let spec = $state.spec
    let width_limit = (get_game_of_life_grid_width ($spec.inner_width | into int))
    let height_limit = ($spec.body_height | into int)

    $state | upsert cells (step_game_of_life_cells_fast $state.cells $width_limit $height_limit)
}

export def render_game_of_life_screen_state [state: record] {
    build_game_of_life_screen_lines $state.spec $state.cells ($state.resolved_width | into int)
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
    let max_frame_height = ($frames | each {|frame| $frame | length } | math max)
    let last_index = (($frames | length) - 1)

    for item in ($frames | enumerate) {
        let frame = $item.item
        let padded_frame = if (($frame | length) < $max_frame_height) {
            let filler = (0..(($max_frame_height - ($frame | length)) - 1) | each { "" })
            ($frame | append $filler)
        } else {
            $frame
        }

        for line in $padded_frame {
            print $"\r\u{1b}[2K($line)"
        }

        if $item.index < $last_index {
            sleep $frame_delay
            print ("\u{1b}[" + (($max_frame_height + 1) | into string) + "A")
        } else {
            print ("\u{1b}[" + (($max_frame_height - ($frame | length)) | into string) + "A")
        }
    }
}

export def play_frames_interruptibly [frames: list<list<string>>, frame_delay: duration, poller?: closure, on_skip?: closure] {
    if ($frames | is-empty) {
        return false
    }

    let max_frame_height = ($frames | each {|frame| $frame | length } | math max)
    let last_index = (($frames | length) - 1)

    for item in ($frames | enumerate) {
        let frame = $item.item
        let padded_frame = if (($frame | length) < $max_frame_height) {
            let filler = (0..(($max_frame_height - ($frame | length)) - 1) | each { "" })
            ($frame | append $filler)
        } else {
            $frame
        }

        for line in $padded_frame {
            print $"\r\u{1b}[2K($line)"
        }

        if $item.index < $last_index {
            let should_skip = if $poller == null {
                false
            } else {
                do $poller $frame_delay
            }

            if $should_skip {
                if $on_skip != null {
                    do $on_skip $frame_delay
                }
                print ("\u{1b}[" + (($max_frame_height - ($frame | length)) | into string) + "A")
                return true
            }

            sleep $frame_delay
            print ("\u{1b}[" + (($max_frame_height + 1) | into string) + "A")
        } else {
            print ("\u{1b}[" + (($max_frame_height - ($frame | length)) | into string) + "A")
        }
    }

    false
}

def repaint_resting_logo_after_skip [width?] {
    let logo_frame = (get_logo_welcome_frame $width)

    print -n "\u{1b}[H\u{1b}[2J"
    print ""
    for line in $logo_frame {
        print $line
    }
}

export def play_frames_with_delay [frames: list<list<string>>, frame_delay: duration] {
    if ($frames | is-empty) {
        return
    }

    let max_frame_height = ($frames | each {|frame| $frame | length } | math max)
    let last_index = (($frames | length) - 1)

    for item in ($frames | enumerate) {
        let frame = $item.item
        let padded_frame = if (($frame | length) < $max_frame_height) {
            let filler = (0..(($max_frame_height - ($frame | length)) - 1) | each { "" })
            ($frame | append $filler)
        } else {
            $frame
        }

        for line in $padded_frame {
            print $"\r\u{1b}[2K($line)"
        }

        if $item.index < $last_index {
            sleep $frame_delay
            print ("\u{1b}[" + (($max_frame_height + 1) | into string) + "A")
        } else {
            print ("\u{1b}[" + (($max_frame_height - ($frame | length)) | into string) + "A")
        }
    }
}

export def play_animation [duration: duration, width?: int] {
    let frames = (get_animated_ascii_art $width)
    play_frames $frames $duration
}

export def get_welcome_playback_duration [welcome_style: string, duration_seconds: float] {
    if $welcome_style == "logo" {
        0.5sec
    } else {
        ($duration_seconds * 1sec)
    }
}

export def render_welcome_style [welcome_style: string, duration_seconds: float = 2.0, width?: int] {
    render_welcome_style_interruptibly $welcome_style $duration_seconds $width | ignore
}

export def render_welcome_style_interruptibly [welcome_style: string, duration_seconds: float = 2.0, width?, poller?: closure] {
    let resolved_style = (resolve_welcome_style $welcome_style)
    let playback_duration = (get_welcome_playback_duration $resolved_style $duration_seconds)
    let skip_to_resting_logo = {|_frame_delay| repaint_resting_logo_after_skip $width }

    if $resolved_style == "static" {
        let ascii_art = (get_welcome_ascii_art $width)
        for line in $ascii_art {
            print $line
        }
        print ""
        return false
    }

    if $resolved_style == "logo" {
        print ""
        let frames = (get_animated_ascii_art $width)
        return (play_frames_interruptibly $frames ($playback_duration / ($frames | length)) $poller $skip_to_resting_logo)
    }

    if $resolved_style == "boids" {
        print ""
        let frames = (get_boids_animation_frames $width)
        return (play_frames_interruptibly $frames ($playback_duration / ($frames | length)) $poller $skip_to_resting_logo)
    }

    if $resolved_style == "game_of_life" {
        print ""
        let frames = (get_game_of_life_animation_frames $width $duration_seconds)
        return (play_frames_interruptibly $frames (get_game_of_life_welcome_frame_delay) $poller $skip_to_resting_logo)
    }

    if $resolved_style == "mandelbrot" {
        print ""
        # Dedicated renderers land in their own welcome-style beads.
        # Until then, animated styles share the logo-forward reveal contract.
        play_animation $playback_duration $width
        return false
    }

    error make {msg: $"Unsupported welcome_style: ($resolved_style)"}
}
