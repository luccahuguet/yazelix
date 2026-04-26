use crate::{ScreenCell, ScreenFrame};
use std::collections::{HashMap, HashSet};

const ANSI_GREEN: &str = "\u{1b}[32m";
const ANSI_BLUE: &str = "\u{1b}[34m";
const ANSI_PURPLE: &str = "\u{1b}[35m";
const ANSI_CYAN: &str = "\u{1b}[36m";
const ANSI_RESET: &str = "\u{1b}[0m";

const RIGHT_GLIDER: &[(i32, i32)] = &[(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
const BLINKER: &[(i32, i32)] = &[(0, 0), (1, 0), (2, 0)];
const TOAD: &[(i32, i32)] = &[(1, 0), (2, 0), (3, 0), (0, 1), (1, 1), (2, 1)];
const BEACON: &[(i32, i32)] = &[
    (0, 0),
    (1, 0),
    (0, 1),
    (1, 1),
    (2, 2),
    (3, 2),
    (2, 3),
    (3, 3),
];
const R_PENTOMINO: &[(i32, i32)] = &[(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)];
const ACORN: &[(i32, i32)] = &[(1, 0), (3, 1), (0, 2), (1, 2), (4, 2), (5, 2), (6, 2)];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOfLifeCellStyle {
    FullBlock,
    Dotted,
}

impl GameOfLifeCellStyle {
    pub fn parse(raw: &str) -> Result<Self, GameOfLifeCellStyleParseError> {
        let normalized = raw.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "full_block" => Ok(Self::FullBlock),
            "dotted" => Ok(Self::Dotted),
            _ => Err(GameOfLifeCellStyleParseError { normalized }),
        }
    }

    pub(crate) fn glyph(self) -> char {
        match self {
            Self::FullBlock => '█',
            // Hardcoded scale 4: one life cell occupies exactly two 2x4 braille cells.
            Self::Dotted => '⣿',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameOfLifeCellStyleParseError {
    normalized: String,
}

impl GameOfLifeCellStyleParseError {
    pub fn normalized(&self) -> &str {
        &self.normalized
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameOfLifeVariant {
    Gliders,
    Oscillators,
    Bloom,
}

impl GameOfLifeVariant {
    fn from_style_name(style: &str) -> Option<Self> {
        match style {
            "game_of_life_gliders" => Some(Self::Gliders),
            "game_of_life_oscillators" => Some(Self::Oscillators),
            "game_of_life_bloom" => Some(Self::Bloom),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameOfLifeSpec {
    pub minimum_inner_width: usize,
    pub welcome_minimum_body_height: usize,
    pub screen_minimum_body_height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenAnimationContext {
    pub resolved_width: usize,
    pub resolved_height: usize,
    pub inner_width: usize,
    pub size_class: &'static str,
}

pub trait ScreenFrameProducer {
    fn render_frame(&self) -> Vec<String>;
    fn advance_frame(&mut self);
    fn resize(&mut self, context: ScreenAnimationContext);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameOfLifeScreenState {
    resolved_width: usize,
    resolved_height: usize,
    inner_width: usize,
    body_height: usize,
    cell_style: GameOfLifeCellStyle,
    cells: HashSet<(i32, i32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameOfLifeAnimation {
    style_name: String,
    cell_style: GameOfLifeCellStyle,
    state: GameOfLifeScreenState,
}

impl GameOfLifeAnimation {
    pub fn new(
        style_name: &str,
        context: ScreenAnimationContext,
        cell_style: GameOfLifeCellStyle,
    ) -> Self {
        Self {
            style_name: style_name.to_string(),
            cell_style,
            state: build_game_of_life_screen_state(
                style_name,
                context.size_class,
                context.resolved_width,
                context.resolved_height,
                context.inner_width,
                cell_style,
            ),
        }
    }
}

impl ScreenFrameProducer for GameOfLifeAnimation {
    fn render_frame(&self) -> Vec<String> {
        render_game_of_life_screen_state(&self.state)
    }

    fn advance_frame(&mut self) {
        step_game_of_life_screen_state(&mut self.state);
    }

    fn resize(&mut self, context: ScreenAnimationContext) {
        self.state = build_game_of_life_screen_state(
            &self.style_name,
            context.size_class,
            context.resolved_width,
            context.resolved_height,
            context.inner_width,
            self.cell_style,
        );
    }
}

pub fn is_game_of_life_style(style: &str) -> bool {
    GameOfLifeVariant::from_style_name(style).is_some()
}

pub fn game_of_life_spec(variant: &str) -> GameOfLifeSpec {
    match variant {
        "narrow" => GameOfLifeSpec {
            minimum_inner_width: 22,
            welcome_minimum_body_height: 8,
            screen_minimum_body_height: 8,
        },
        "medium" => GameOfLifeSpec {
            minimum_inner_width: 34,
            welcome_minimum_body_height: 12,
            screen_minimum_body_height: 12,
        },
        "wide" => GameOfLifeSpec {
            minimum_inner_width: 58,
            welcome_minimum_body_height: 14,
            screen_minimum_body_height: 14,
        },
        "hero" => GameOfLifeSpec {
            minimum_inner_width: 58,
            welcome_minimum_body_height: 16,
            screen_minimum_body_height: 16,
        },
        other => panic!("missing game_of_life spec: {other}"),
    }
}

pub fn resolve_game_of_life_body_height(minimum_height: usize, resolved_height: usize) -> usize {
    resolved_height.saturating_sub(6).max(minimum_height)
}

pub fn resolve_game_of_life_screen_body_height(
    minimum_height: usize,
    resolved_height: usize,
) -> usize {
    resolved_height.max(minimum_height)
}

pub fn game_of_life_grid_width(inner_width: usize) -> usize {
    (inner_width / 2).max(1)
}

pub fn game_of_life_grid_height(body_height: usize) -> usize {
    body_height.max(3)
}

fn shape_size(shape: &[(i32, i32)]) -> (i32, i32) {
    let max_x = shape.iter().map(|(x, _)| *x).max().unwrap_or(0);
    let max_y = shape.iter().map(|(_, y)| *y).max().unwrap_or(0);
    (max_x + 1, max_y + 1)
}

fn place_shape(
    shape: &[(i32, i32)],
    width: usize,
    height: usize,
    origin_x: i32,
    origin_y: i32,
) -> Vec<(i32, i32)> {
    let (shape_width, shape_height) = shape_size(shape);
    let max_x = (width as i32 - shape_width).max(0);
    let max_y = (height as i32 - shape_height).max(0);
    let origin_x = origin_x.clamp(0, max_x);
    let origin_y = origin_y.clamp(0, max_y);

    shape
        .iter()
        .map(|(x, y)| (x + origin_x, y + origin_y))
        .collect()
}

fn build_game_of_life_gliders_seed(width: usize, height: usize) -> HashSet<(i32, i32)> {
    let glider_count = if width >= 36 {
        6
    } else if width >= 22 {
        4
    } else {
        2
    };
    let right_edge_x = (width as i32 - 5).max(0);
    let inner_right_x = (width as i32 - 9).max(0);
    let middle_upper_y = (height as i32 / 2) - 3;
    let middle_lower_y = (height as i32 / 2) + 1;

    let placements = if glider_count == 2 {
        vec![(1, 1), (right_edge_x, height as i32 - 4)]
    } else if glider_count == 4 {
        vec![
            (1, 1),
            (right_edge_x, 2),
            (4, height as i32 - 7),
            (inner_right_x, height as i32 - 4),
        ]
    } else {
        vec![
            (1, 1),
            (right_edge_x, 2),
            (3, middle_upper_y),
            (inner_right_x, middle_lower_y),
            (5, height as i32 - 7),
            (right_edge_x, height as i32 - 4),
        ]
    };

    placements
        .into_iter()
        .flat_map(|(x, y)| place_shape(RIGHT_GLIDER, width, height, x, y))
        .collect()
}

fn build_game_of_life_oscillators_seed(width: usize, height: usize) -> HashSet<(i32, i32)> {
    let placements = vec![
        (BEACON, 1, 1),
        (BLINKER, (width as i32 / 2) - 1, 1),
        (TOAD, (width as i32 / 2) - 2, (height as i32 / 2) - 1),
        (BLINKER, 2, height as i32 - 2),
        (BEACON, width as i32 - 5, height as i32 - 5),
    ];
    placements
        .into_iter()
        .flat_map(|(shape, x, y)| place_shape(shape, width, height, x, y))
        .collect()
}

fn build_game_of_life_bloom_seed(width: usize, height: usize) -> HashSet<(i32, i32)> {
    vec![
        (R_PENTOMINO, 1, 1),
        (ACORN, (width as i32 / 2) - 3, (height as i32 / 3) - 1),
        (R_PENTOMINO, width as i32 - 4, height as i32 - 4),
        (
            R_PENTOMINO,
            (width as i32 / 2) - 1,
            ((height as i32 * 2) / 3) - 1,
        ),
    ]
    .into_iter()
    .flat_map(|(shape, x, y)| place_shape(shape, width, height, x, y))
    .collect()
}

pub fn build_live_game_of_life_seed(
    inner_width: usize,
    body_height: usize,
    style: &str,
) -> HashSet<(i32, i32)> {
    let width = game_of_life_grid_width(inner_width);
    let height = game_of_life_grid_height(body_height);
    match GameOfLifeVariant::from_style_name(style).unwrap_or(GameOfLifeVariant::Bloom) {
        GameOfLifeVariant::Gliders => build_game_of_life_gliders_seed(width, height),
        GameOfLifeVariant::Oscillators => build_game_of_life_oscillators_seed(width, height),
        GameOfLifeVariant::Bloom => build_game_of_life_bloom_seed(width, height),
    }
}

pub fn step_game_of_life_cells(
    cells: &HashSet<(i32, i32)>,
    width: usize,
    height: usize,
) -> HashSet<(i32, i32)> {
    let mut neighbor_counts: HashMap<(i32, i32), usize> = HashMap::new();
    for &(x, y) in cells {
        for ny in [y - 1, y, y + 1] {
            for nx in [x - 1, x, x + 1] {
                if nx == x && ny == y {
                    continue;
                }
                let wrapped_x = ((nx + width as i32) % width as i32).rem_euclid(width as i32);
                let wrapped_y = ((ny + height as i32) % height as i32).rem_euclid(height as i32);
                *neighbor_counts.entry((wrapped_x, wrapped_y)).or_insert(0) += 1;
            }
        }
    }

    neighbor_counts
        .into_iter()
        .filter_map(|(cell, neighbors)| {
            let alive = cells.contains(&cell);
            if neighbors == 3 || (alive && neighbors == 2) {
                Some(cell)
            } else {
                None
            }
        })
        .collect()
}

fn colorize_game_of_life_glyph(x: usize, y: usize, glyph: char) -> String {
    let palette = [ANSI_GREEN, ANSI_CYAN, ANSI_BLUE, ANSI_PURPLE];
    format!(
        "{}{}{}",
        palette[(x + y) % palette.len()],
        glyph,
        ANSI_RESET
    )
}

pub fn build_game_of_life_screen_lines(
    inner_width: usize,
    body_height: usize,
    resolved_width: usize,
    cell_style: GameOfLifeCellStyle,
    cells: &HashSet<(i32, i32)>,
) -> Vec<String> {
    let grid_width = game_of_life_grid_width(inner_width);
    let grid_height = game_of_life_grid_height(body_height);
    let mut frame = ScreenFrame::new(inner_width, body_height);
    for y in 0..grid_height {
        for x in 0..grid_width {
            if !cells.contains(&(x as i32, y as i32)) {
                continue;
            }

            let cell = ScreenCell {
                glyph: cell_style.glyph(),
                color_x: x,
                color_y: y,
            };
            let origin_x = x * 2;
            for dx in 0..2 {
                frame.set(origin_x + dx, y, cell);
            }
        }
    }
    frame.render_lines(resolved_width, |cell| {
        colorize_game_of_life_glyph(cell.color_x, cell.color_y, cell.glyph)
    })
}

pub fn build_game_of_life_screen_state(
    style: &str,
    layout_variant: &str,
    width: usize,
    height: usize,
    inner_width: usize,
    cell_style: GameOfLifeCellStyle,
) -> GameOfLifeScreenState {
    let spec = game_of_life_spec(layout_variant);
    let body_height =
        resolve_game_of_life_screen_body_height(spec.screen_minimum_body_height, height);
    GameOfLifeScreenState {
        resolved_width: width,
        resolved_height: height,
        inner_width,
        body_height,
        cell_style,
        cells: build_live_game_of_life_seed(inner_width, body_height, style),
    }
}

pub fn render_game_of_life_screen_state(state: &GameOfLifeScreenState) -> Vec<String> {
    build_game_of_life_screen_lines(
        state.inner_width,
        state.body_height,
        state.resolved_width,
        state.cell_style,
        &state.cells,
    )
}

pub fn step_game_of_life_screen_state(state: &mut GameOfLifeScreenState) {
    let width = game_of_life_grid_width(state.inner_width);
    let height = game_of_life_grid_height(state.body_height);
    state.cells = step_game_of_life_cells(&state.cells, width, height);
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test lane: default

    fn render_test_game_of_life_lines(
        inner_width: usize,
        body_height: usize,
        cells: &[(i32, i32)],
    ) -> Vec<String> {
        render_test_game_of_life_lines_with_style(
            inner_width,
            body_height,
            GameOfLifeCellStyle::FullBlock,
            cells,
        )
    }

    fn render_test_game_of_life_lines_with_style(
        inner_width: usize,
        body_height: usize,
        cell_style: GameOfLifeCellStyle,
        cells: &[(i32, i32)],
    ) -> Vec<String> {
        let cells = cells.iter().copied().collect::<HashSet<_>>();
        build_game_of_life_screen_lines(inner_width, body_height, inner_width, cell_style, &cells)
    }

    fn strip_ansi_codes(line: &str) -> String {
        let mut visible = String::new();
        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\u{1b}' && chars.peek() == Some(&'[') {
                chars.next();
                for code_ch in chars.by_ref() {
                    if code_ch.is_ascii_alphabetic() {
                        break;
                    }
                }
                continue;
            }
            visible.push(ch);
        }
        visible
    }

    // Regression: Game of Life gliders must keep the old Nushell full-block silhouette instead of shrinking into tiny braille dots.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn game_of_life_screen_lines_preserve_glider_silhouette_as_full_blocks() {
        let lines = render_test_game_of_life_lines(8, 3, &[(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]);
        let visible_lines = lines
            .iter()
            .map(|line| strip_ansi_codes(line))
            .collect::<Vec<_>>();
        assert_eq!(visible_lines, vec!["  ██    ", "    ██  ", "██████  "]);
    }

    // Defends: the dotted option stays at hardcoded scale 4, matching the full-block footprint without shrinking gliders.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn game_of_life_screen_lines_preserve_glider_footprint_as_dotted_scale_4() {
        let lines = render_test_game_of_life_lines_with_style(
            8,
            3,
            GameOfLifeCellStyle::Dotted,
            &[(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)],
        );
        let visible_lines = lines
            .iter()
            .map(|line| strip_ansi_codes(line))
            .collect::<Vec<_>>();
        assert_eq!(visible_lines, vec!["  ⣿⣿    ", "    ⣿⣿  ", "⣿⣿⣿⣿⣿⣿  "]);
    }

    // Regression: the Rust renderer must match the old Nushell Game of Life row contract: one life row per terminal row.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn game_of_life_screen_lines_preserve_life_rows_without_inserted_gaps() {
        let lines = render_test_game_of_life_lines(8, 4, &[(0, 0), (0, 1), (0, 2)]);
        let visible_lines = lines
            .iter()
            .map(|line| strip_ansi_codes(line))
            .collect::<Vec<_>>();
        assert_eq!(
            visible_lines,
            vec!["██      ", "██      ", "██      ", "        "]
        );
    }

    // Defends: retained Game of Life screen states still roll forward instead of collapsing into a fixed logo frame.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn game_of_life_screen_state_rolls_forward() {
        let mut state = build_game_of_life_screen_state(
            "game_of_life_gliders",
            "medium",
            80,
            24,
            34,
            GameOfLifeCellStyle::FullBlock,
        );
        let before = render_game_of_life_screen_state(&state).join("\n");
        step_game_of_life_screen_state(&mut state);
        let after = render_game_of_life_screen_state(&state).join("\n");
        assert_ne!(before, after);
        assert!(!after.contains("welcome to yazelix"));
    }

    // Defends: future screen animations can use a frame-producer contract with deterministic resize and advance behavior.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn game_of_life_animation_uses_frame_producer_resize_contract() {
        let mut animation = GameOfLifeAnimation::new(
            "game_of_life_gliders",
            ScreenAnimationContext {
                resolved_width: 80,
                resolved_height: 24,
                inner_width: 34,
                size_class: "medium",
            },
            GameOfLifeCellStyle::FullBlock,
        );
        let before = animation.render_frame();
        animation.advance_frame();
        let advanced = animation.render_frame();
        animation.resize(ScreenAnimationContext {
            resolved_width: 120,
            resolved_height: 32,
            inner_width: 58,
            size_class: "hero",
        });
        let resized = animation.render_frame();

        assert_ne!(before, advanced);
        assert_ne!(advanced, resized);
        assert_eq!(resized.len(), 32);
    }
}
