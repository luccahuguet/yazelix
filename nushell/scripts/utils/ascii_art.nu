#!/usr/bin/env nu
# Width-aware welcome art for Yazelix.

export const WELCOME_STYLE_VALUES = ["static", "logo", "boids", "game_of_life", "mandelbrot", "random"]
export const ANIMATED_WELCOME_STYLE_VALUES = ["game_of_life"]

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

def colorize_game_of_life_char [x: int, y: int] {
    let colors = get_yazelix_colors
    let palette = [$colors.green, $colors.cyan, $colors.blue, $colors.purple]
    let color = ($palette | get (($x + $y) mod ($palette | length)))
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
                    "alt+shift+m menu | ctrl+y sidebar jump"
                    "packs, sessions, terminals, all under one roof"
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
                    "yazi + zellij + helix, wired together and ready"
                    "one shell, one workspace, one real flow"
                    "sidebar, editor, sessions, packs, and terminals already aligned"
                    "alt+shift+m menu | ctrl+y sidebar jump | alt+[ / alt+] layout family"
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

def resolve_game_of_life_body_height [minimum_height: int, resolved_height: int] {
    let fullscreen_height = ($resolved_height - 6)
    if $fullscreen_height > $minimum_height {
        $fullscreen_height
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
    $shape | each {|pair| make_game_of_life_cell ($pair.0 + $offset_x) ($pair.1 + $offset_y) }
}

def get_right_glider_shape [] {
    [[1 0] [2 1] [0 2] [1 2] [2 2]]
}

def get_left_glider_shape [] {
    [[1 0] [0 1] [2 2] [1 2] [0 2]]
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
    { x: ($parts | get 0 | into int), y: ($parts | get 1 | into int) }
}

def count_game_of_life_neighbors_record [cells: list<record>, width: int, height: int] {
    mut counts = {}

    for cell in $cells {
        for ny in [($cell.y - 1), $cell.y, ($cell.y + 1)] {
            if ($ny < 0) or ($ny >= $height) {
                continue
            }

            for nx in [($cell.x - 1), $cell.x, ($cell.x + 1)] {
                if ($nx < 0) or ($nx >= $width) {
                    continue
                }

                if ($nx == $cell.x) and ($ny == $cell.y) {
                    continue
                }

                $counts = (add_neighbor_count $counts $nx $ny)
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

def step_game_of_life_cells_n_fast [cells: list<record>, width: int, height: int, generations: int] {
    mut current = $cells

    for _ in 1..$generations {
        $current = (step_game_of_life_cells_fast $current $width $height)
    }

    $current
}

def build_live_game_of_life_seed [spec: record] {
    let width = (get_game_of_life_grid_width ($spec.inner_width | into int))
    let height = ($spec.body_height | into int)
    let glider_pairs = if $width >= 36 {
        3
    } else if $width >= 22 {
        2
    } else {
        1
    }
    let max_start_y = if ($height - 3) < 0 { 0 } else { $height - 3 }
    let left_rows = if $glider_pairs == 1 {
        [($height / 2 | math floor)]
    } else if $glider_pairs == 2 {
        [1, ($height - 4)]
    } else {
        [1, ($height / 2 | math floor), ($height - 4)]
    }
    let right_rows = if $glider_pairs == 1 {
        [($height / 2 | math floor)]
    } else if $glider_pairs == 2 {
        [($height - 4), 1]
    } else {
        [($height - 4), ($height / 2 | math floor), 1]
    }
    let left_offsets = if $glider_pairs == 1 {
        [1]
    } else if $glider_pairs == 2 {
        [1, 9]
    } else {
        [1, 9, 17]
    }
    let right_offsets = if $glider_pairs == 1 {
        [($width - 4)]
    } else if $glider_pairs == 2 {
        [($width - 4), ($width - 12)]
    } else {
        [($width - 4), ($width - 12), ($width - 20)]
    }
    let right_glider = (get_right_glider_shape)
    let left_glider = (get_left_glider_shape)

    let left_glider_cells = (
        $left_offsets
        | enumerate
        | each {|item|
            let row = (($left_rows | get $item.index) | into int)
            let clamped_row = if $row > $max_start_y { $max_start_y } else { $row }
            offset_game_of_life_shape $right_glider $item.item $clamped_row
        }
        | flatten
    )

    let right_glider_cells = (
        $right_offsets
        | enumerate
        | each {|item|
            let row = (($right_rows | get $item.index) | into int)
            let clamped_row = if $row > $max_start_y { $max_start_y } else { $row }
            offset_game_of_life_shape $left_glider $item.item $clamped_row
        }
        | flatten
    )

    unique_game_of_life_cells ($left_glider_cells | append $right_glider_cells)
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

def build_game_of_life_frame [spec: record, cells: list<record>] {
    let colors = get_yazelix_colors
    let inner_width = ($spec.inner_width | into int)
    let height = ($spec.body_height | into int)
    let grid_width = (get_game_of_life_grid_width $inner_width)
    let cell_keys = (build_game_of_life_cell_keys $cells)
    let body = (
        0..($height - 1)
        | each {|row_index|
            let row = (render_game_of_life_row $grid_width $inner_width $row_index $cell_keys)
            $"($colors.purple)│($colors.reset)($row)($colors.purple)│($colors.reset)"
        }
    )

    [
        $"($colors.purple)╭(make_border $inner_width "─")╮($colors.reset)"
        ...$body
        $"($colors.purple)╰(make_border $inner_width "─")╯($colors.reset)"
    ]
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

export def get_game_of_life_animation_frames [width?: int] {
    let resolved_width = ($width | default (get_terminal_width) | into int)
    let resolved_height = (get_terminal_height)
    let variant = (get_logo_welcome_variant $resolved_width)
    let spec = (get_game_of_life_welcome_spec $variant $resolved_width $resolved_height)
    let width_limit = (get_game_of_life_grid_width ($spec.inner_width | into int))
    let height_limit = ($spec.body_height | into int)
    mut current_cells = (build_live_game_of_life_seed $spec)
    mut simulation_frames = [(build_game_of_life_frame $spec $current_cells)]

    for _ in 1..7 {
        $current_cells = (step_game_of_life_cells_fast $current_cells $width_limit $height_limit)
        $simulation_frames = ($simulation_frames | append [(build_game_of_life_frame $spec $current_cells)])
    }

    [
        ...($simulation_frames | each {|frame| center_frame_lines $frame $resolved_width })
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

def get_welcome_playback_duration [welcome_style: string, duration: duration] {
    if $welcome_style == "game_of_life" {
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

    if $resolved_style == "game_of_life" {
        print ""
        play_frames (get_game_of_life_animation_frames $width) $playback_duration
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
