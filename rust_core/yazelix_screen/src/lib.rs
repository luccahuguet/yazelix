//! Terminal screen primitives shared by Yazelix front-door animation surfaces.

mod boids;
mod game_of_life;

use crossterm::terminal;
use std::io::{self, Write};

pub use boids::BoidsAnimation;
pub use game_of_life::{
    GameOfLifeAnimation, GameOfLifeCellStyle, GameOfLifeCellStyleParseError, GameOfLifeScreenState,
    GameOfLifeSpec, ScreenAnimationContext, ScreenFrameProducer, build_game_of_life_screen_lines,
    build_game_of_life_screen_state, build_live_game_of_life_seed, game_of_life_grid_height,
    game_of_life_grid_width, game_of_life_spec, is_game_of_life_style,
    render_game_of_life_screen_state, resolve_game_of_life_body_height,
    resolve_game_of_life_screen_body_height, step_game_of_life_cells,
    step_game_of_life_screen_state,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenCell {
    pub glyph: char,
    pub color_x: usize,
    pub color_y: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenFrame {
    width: usize,
    height: usize,
    cells: Vec<Option<ScreenCell>>,
}

impl ScreenFrame {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![None; width.saturating_mul(height)],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, cell: ScreenCell) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.cells[y * self.width + x] = Some(cell);
    }

    pub fn render_lines<F>(&self, resolved_width: usize, render_cell: F) -> Vec<String>
    where
        F: Fn(ScreenCell) -> String,
    {
        let lines = (0..self.height)
            .map(|y| {
                let mut line = String::new();
                for x in 0..self.width {
                    match self.cells[y * self.width + x] {
                        Some(cell) => line.push_str(&render_cell(cell)),
                        None => line.push(' '),
                    }
                }
                line
            })
            .collect();
        center_frame_lines(lines, resolved_width)
    }
}

pub fn terminal_width() -> usize {
    std::env::var("YAZELIX_WELCOME_WIDTH")
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|width| *width > 0)
        .or_else(|| terminal::size().ok().map(|(width, _)| width as usize))
        .unwrap_or(80)
}

pub fn terminal_height() -> usize {
    std::env::var("YAZELIX_WELCOME_HEIGHT")
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|height| *height > 0)
        .or_else(|| terminal::size().ok().map(|(_, height)| height as usize))
        .unwrap_or(24)
}

pub fn visible_line_width(line: &str) -> usize {
    let mut count = 0;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for inner in chars.by_ref() {
                if inner.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        count += 1;
    }
    count
}

pub fn center_text(text: &str, width: usize) -> String {
    let visible_width = visible_line_width(text);
    if visible_width >= width {
        return text.to_string();
    }

    let left = (width - visible_width) / 2;
    let right = width - visible_width - left;
    format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
}

pub fn center_frame_lines(lines: Vec<String>, width: usize) -> Vec<String> {
    lines
        .into_iter()
        .map(|line| center_text(&line, width))
        .collect()
}

pub fn screen_frame_output(frame: &[String]) -> String {
    let mut out = String::from("\u{1b}[H\u{1b}[2J");
    for (row_index, line) in frame.iter().enumerate() {
        out.push_str(&format!("\u{1b}[{};1H\u{1b}[2K{line}", row_index + 1));
    }
    out
}

pub fn flush_stdout() -> io::Result<()> {
    io::stdout().flush()
}

pub fn render_screen_frame(frame: &[String]) -> io::Result<()> {
    print!("{}", screen_frame_output(frame));
    flush_stdout()
}

pub fn enter_screen_mode() -> io::Result<()> {
    print!("\u{1b}[?1049h\u{1b}[?25l\u{1b}[?7l\u{1b}[2J\u{1b}[H");
    flush_stdout()
}

pub fn leave_screen_mode() -> io::Result<()> {
    print!("\u{1b}[?7h\u{1b}[?25h\u{1b}[?1049l");
    flush_stdout()
}

pub struct RawModeGuard;

impl RawModeGuard {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test lane: default

    // Regression: raw alternate-screen rendering must not rely on newlines after full-width lines, which can wrap into every other row.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn screen_frame_output_addresses_rows_without_newlines() {
        let output = screen_frame_output(&["aaaaaaaa".to_string(), "bbbbbbbb".to_string()]);
        assert!(!output.contains('\n'));
        assert!(output.contains("\u{1b}[1;1H\u{1b}[2Kaaaaaaaa"));
        assert!(output.contains("\u{1b}[2;1H\u{1b}[2Kbbbbbbbb"));
    }
}
