//! Front-door rendering and screen playback for Rust-owned welcome/tutor/report UX.

use crate::bridge::{CoreError, ErrorClass};
use crossterm::event::{self, Event};
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
pub use yazelix_screen::GameOfLifeCellStyle;
use yazelix_screen::{
    BoidsAnimation, GameOfLifeAnimation, RawModeGuard as ScreenRawModeGuard,
    ScreenAnimationContext, ScreenFrameProducer, build_game_of_life_screen_lines,
    build_live_game_of_life_seed, center_frame_lines, center_text, game_of_life_grid_height,
    game_of_life_grid_width, game_of_life_spec, is_game_of_life_style,
    resolve_game_of_life_body_height, step_game_of_life_cells, terminal_height, terminal_width,
    visible_line_width,
};

const ASCII_ART_DATA_JSON: &str = include_str!("../assets/ascii_art_data.json");

const ANSI_RED: &str = "\u{1b}[31m";
const ANSI_GREEN: &str = "\u{1b}[32m";
const ANSI_YELLOW: &str = "\u{1b}[33m";
const ANSI_BLUE: &str = "\u{1b}[34m";
const ANSI_PURPLE: &str = "\u{1b}[35m";
const ANSI_CYAN: &str = "\u{1b}[36m";
const ANSI_RESET: &str = "\u{1b}[0m";
const ANSI_FAINT: &str = "\u{1b}[2m";

const GAME_OF_LIFE_RANDOM_POOL: &[&str] = &[
    "game_of_life_gliders",
    "game_of_life_oscillators",
    "game_of_life_bloom",
];

#[derive(Debug, Clone, Deserialize)]
struct AsciiArtData {
    style_catalog: Vec<StyleCatalogEntry>,
    logo_welcome_specs: HashMap<String, LogoWelcomeSpec>,
    boids_welcome_specs: HashMap<String, BoidsWelcomeSpec>,
}

#[derive(Debug, Clone, Deserialize)]
struct StyleCatalogEntry {
    name: String,
    welcome: bool,
    screen: bool,
    random: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct LogoWelcomeSpec {
    minimum_inner_width: usize,
    title_text: String,
    title_hint_text: String,
    body_alignment: String,
    body_lines: Vec<String>,
    footer: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BoidsWelcomeSpec {
    minimum_inner_width: usize,
    body_height: usize,
    caption: String,
}

fn ascii_art_data() -> &'static AsciiArtData {
    static DATA: OnceLock<AsciiArtData> = OnceLock::new();
    DATA.get_or_init(|| serde_json::from_str(ASCII_ART_DATA_JSON).expect("valid ascii_art_data"))
}

fn system_random_index(max_len: usize) -> usize {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as usize;
    nanos % max_len
}

fn style_values_for_surface(surface: &str) -> Vec<&'static str> {
    ascii_art_data()
        .style_catalog
        .iter()
        .filter_map(|style| match surface {
            "welcome" if style.welcome => Some(style.name.as_str()),
            "screen" if style.screen => Some(style.name.as_str()),
            "random" if style.random => Some(style.name.as_str()),
            _ => None,
        })
        .collect()
}

pub fn resolve_welcome_style(
    requested: &str,
    random_index: Option<usize>,
) -> Result<String, CoreError> {
    let normalized = requested.trim().to_ascii_lowercase();
    let allowed = style_values_for_surface("welcome");
    if !allowed.iter().any(|candidate| *candidate == normalized) {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "invalid_welcome_style",
            format!(
                "Invalid welcome style `{normalized}`. Expected one of: {}",
                allowed.join(", ")
            ),
            "Pick one of the documented welcome styles from `yazelix.toml` or `yzx screen --help`.",
            serde_json::json!({ "style": normalized }),
        ));
    }

    if normalized != "random" {
        return Ok(normalized);
    }

    for candidate in GAME_OF_LIFE_RANDOM_POOL {
        if !allowed
            .iter()
            .any(|allowed_style| allowed_style == candidate)
        {
            panic!("missing retained random welcome style: {candidate}");
        }
    }
    let pool = GAME_OF_LIFE_RANDOM_POOL;
    let selected = random_index.unwrap_or_else(|| system_random_index(pool.len())) % pool.len();
    Ok(pool[selected].to_string())
}

pub fn resolve_screen_style(
    requested: Option<&str>,
    random_index: Option<usize>,
) -> Result<String, CoreError> {
    let normalized = requested.unwrap_or("random").trim().to_ascii_lowercase();
    let allowed = style_values_for_surface("screen");
    if !allowed.iter().any(|candidate| *candidate == normalized) {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "invalid_screen_style",
            format!(
                "Invalid screen style `{normalized}`. Expected one of: {}",
                allowed.join(", ")
            ),
            "Run `yzx screen --help` to see the retained animated screen styles.",
            serde_json::json!({ "style": normalized }),
        ));
    }

    if normalized == "random" {
        return resolve_welcome_style("random", random_index);
    }
    Ok(normalized)
}

fn screen_frame_delay(resolved_style: &str) -> Duration {
    match resolved_style {
        style if is_game_of_life_style(style) => Duration::from_millis(160),
        "boids" => Duration::from_millis(90),
        _ => Duration::from_millis(120),
    }
}

fn get_logo_welcome_variant(width: usize) -> &'static str {
    if width < 44 {
        "narrow"
    } else if width < 72 {
        "medium"
    } else if width < 120 {
        "wide"
    } else {
        "hero"
    }
}

fn pad_text_right(text: &str, width: usize) -> String {
    let visible_width = visible_line_width(text);
    if visible_width >= width {
        return text.to_string();
    }
    format!("{text}{}", " ".repeat(width - visible_width))
}

fn fit_inner_width(resolved_width: usize, minimum_width: usize) -> usize {
    let proposed = resolved_width.saturating_sub(6);
    proposed.max(minimum_width)
}

// Welcome specs already encode the designed card width for each variant.
// Stretching them toward the terminal width creates near-edge-to-edge cards
// with large dead-space gutters inside the frame.
fn welcome_inner_width(designed_width: usize) -> usize {
    designed_width
}

fn colorize_logo_text(text: &str) -> String {
    let palette = [ANSI_RED, ANSI_GREEN, ANSI_YELLOW, ANSI_BLUE, ANSI_PURPLE];
    text.chars()
        .enumerate()
        .map(|(index, ch)| {
            if ch == ' ' {
                " ".to_string()
            } else {
                format!("{}{}{}", palette[index % palette.len()], ch, ANSI_RESET)
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn colorize_body_line(text: &str) -> String {
    let base_color = ANSI_GREEN;
    let accent_color = ANSI_BLUE;
    let base = format!("{base_color}{text}{ANSI_RESET}");
    [
        "reproducible",
        "declarative",
        "helix",
        "zellij",
        "terminals",
        "shells",
        "packs",
        "SSH",
    ]
    .into_iter()
    .fold(base, |acc, needle| {
        acc.replace(needle, &format!("{accent_color}{needle}{base_color}"))
    })
}

fn colorize_footer_text(text: &str) -> String {
    format!("{ANSI_YELLOW}{text}{ANSI_RESET}")
}

fn colorize_boid_char(ch: char, index: usize) -> String {
    let palette = [ANSI_CYAN, ANSI_BLUE, ANSI_PURPLE];
    format!("{}{}{}", palette[index % palette.len()], ch, ANSI_RESET)
}

fn make_border(inner_width: usize) -> String {
    "─".repeat(inner_width)
}

fn frame_content_width(inner_width: usize) -> usize {
    inner_width + 2
}

fn logo_spec(variant: &str, width: usize) -> &LogoWelcomeSpec {
    let spec = ascii_art_data()
        .logo_welcome_specs
        .get(variant)
        .unwrap_or_else(|| panic!("missing logo spec: {variant}"));
    let _ = width;
    spec
}

fn boids_spec(variant: &str) -> &BoidsWelcomeSpec {
    ascii_art_data()
        .boids_welcome_specs
        .get(variant)
        .unwrap_or_else(|| panic!("missing boids spec: {variant}"))
}

fn build_logo_card_frame(
    spec: &LogoWelcomeSpec,
    inner_width: usize,
    shown_body_count: usize,
    accent: &str,
) -> Vec<String> {
    let content_width = frame_content_width(inner_width);
    let title_text = if accent == "hint" {
        spec.title_hint_text.as_str()
    } else {
        spec.title_text.as_str()
    };
    let title_plain = center_text(title_text, content_width);
    let title_colored = if accent == "hint" {
        format!("{ANSI_FAINT}{ANSI_PURPLE}{title_plain}{ANSI_RESET}")
    } else {
        colorize_logo_text(&title_plain)
    };

    let body_lines = spec
        .body_lines
        .iter()
        .enumerate()
        .map(|(index, body)| {
            let aligned = if spec.body_alignment == "center" {
                center_text(body, content_width)
            } else {
                pad_text_right(body, content_width)
            };

            if index < shown_body_count {
                colorize_body_line(&aligned)
            } else {
                format!(
                    "{ANSI_FAINT}{}{ANSI_RESET}",
                    pad_text_right("", content_width)
                )
            }
        })
        .collect::<Vec<_>>();

    let footer = colorize_footer_text(&center_text(&spec.footer, content_width));
    let mut out = Vec::new();
    out.push(format!(
        "{ANSI_PURPLE}╭{}╮{ANSI_RESET}",
        make_border(inner_width)
    ));
    out.push(title_colored);
    for line in body_lines {
        out.push(line);
    }
    out.push(footer);
    out.push(format!(
        "{ANSI_PURPLE}╰{}╯{ANSI_RESET}",
        make_border(inner_width)
    ));
    out
}

fn get_logo_welcome_frame(width: usize) -> Vec<String> {
    let variant = get_logo_welcome_variant(width);
    let spec = logo_spec(variant, width);
    let inner_width = welcome_inner_width(spec.minimum_inner_width);
    center_frame_lines(
        build_logo_card_frame(spec, inner_width, spec.body_lines.len(), "full"),
        width,
    )
}

fn get_logo_animation_frames(width: usize) -> Vec<Vec<String>> {
    let variant = get_logo_welcome_variant(width);
    let spec = logo_spec(variant, width);
    let inner_width = welcome_inner_width(spec.minimum_inner_width);
    vec![
        center_frame_lines(build_logo_card_frame(spec, inner_width, 0, "hint"), width),
        center_frame_lines(build_logo_card_frame(spec, inner_width, 0, "full"), width),
        center_frame_lines(build_logo_card_frame(spec, inner_width, 1, "full"), width),
        center_frame_lines(
            build_logo_card_frame(spec, inner_width, spec.body_lines.len(), "full"),
            width,
        ),
    ]
}

fn boid_points(
    inner_width: usize,
    body_height: usize,
    phase: &str,
) -> Vec<(usize, usize, char, usize)> {
    let mid_x = inner_width / 2;
    let low_y = if body_height > 2 { body_height - 2 } else { 1 };
    let mid_y = body_height / 2;
    match phase {
        "scatter" => vec![
            (1, 0, '>', 0),
            (inner_width.saturating_sub(2), 0, '<', 1),
            (3, low_y, '^', 2),
            (inner_width.saturating_sub(4), low_y, 'v', 3),
            (mid_x.saturating_sub(6), mid_y, '*', 4),
            (mid_x + 5, mid_y, '*', 5),
        ],
        "drift" => vec![
            (mid_x.saturating_sub(8), 1, '>', 0),
            (mid_x + 7, 1, '<', 1),
            (mid_x.saturating_sub(5), mid_y, '^', 2),
            (mid_x + 4, mid_y, 'v', 3),
            (mid_x.saturating_sub(2), low_y.saturating_sub(1), '*', 4),
            (mid_x + 1, low_y.saturating_sub(1), '*', 5),
        ],
        _ => vec![
            (mid_x.saturating_sub(4), 1, '>', 0),
            (mid_x + 3, 1, '<', 1),
            (mid_x.saturating_sub(2), mid_y, '^', 2),
            (mid_x + 1, mid_y, 'v', 3),
            (mid_x.saturating_sub(6), low_y.saturating_sub(1), '*', 4),
            (mid_x + 5, low_y.saturating_sub(1), '*', 5),
        ],
    }
}

fn build_boids_frame(width: usize) -> Vec<Vec<String>> {
    let variant = get_logo_welcome_variant(width);
    let spec = boids_spec(variant);
    let inner_width = welcome_inner_width(spec.minimum_inner_width);
    let content_width = frame_content_width(inner_width);
    ["scatter", "drift", "cluster"]
        .into_iter()
        .map(|phase| {
            let points = boid_points(inner_width, spec.body_height, phase);
            let caption_row = if phase == "cluster" {
                Some(spec.body_height.saturating_sub(1))
            } else {
                None
            };
            let mut lines = vec![format!(
                "{ANSI_PURPLE}╭{}╮{ANSI_RESET}",
                make_border(inner_width)
            )];
            for row_index in 0..spec.body_height {
                let row = if caption_row == Some(row_index) {
                    format!(
                        "{ANSI_FAINT}{ANSI_PURPLE}{}{ANSI_RESET}",
                        center_text(&spec.caption, content_width)
                    )
                } else {
                    let mut row = String::new();
                    for x in 0..inner_width {
                        let point = points
                            .iter()
                            .find(|(px, py, _, _)| *px == x && *py == row_index);
                        if let Some((_, _, ch, index)) = point {
                            row.push_str(&colorize_boid_char(*ch, *index));
                        } else {
                            row.push(' ');
                        }
                    }
                    center_text(&row, content_width)
                };
                lines.push(row);
            }
            lines.push(format!(
                "{ANSI_PURPLE}╰{}╯{ANSI_RESET}",
                make_border(inner_width)
            ));
            center_frame_lines(lines, width)
        })
        .chain(std::iter::once(get_logo_welcome_frame(width)))
        .collect()
}

fn welcome_sequence(
    resolved_style: &str,
    width: usize,
    height: usize,
    duration: Duration,
    cell_style: GameOfLifeCellStyle,
) -> Vec<Vec<String>> {
    match resolved_style {
        "static" => vec![get_logo_welcome_frame(width)],
        "logo" => get_logo_animation_frames(width),
        "boids" => build_boids_frame(width),
        style if is_game_of_life_style(style) => {
            let variant = get_logo_welcome_variant(width);
            let spec = game_of_life_spec(variant);
            let inner_width = welcome_inner_width(spec.minimum_inner_width);
            let body_height =
                resolve_game_of_life_body_height(spec.welcome_minimum_body_height, height);
            let width_limit = game_of_life_grid_width(inner_width);
            let height_limit = game_of_life_grid_height(body_height);
            let frame_delay = Duration::from_millis(220);
            let frame_count =
                ((duration.as_secs_f64() / frame_delay.as_secs_f64()).ceil() as usize).max(2);
            let mut cells = build_live_game_of_life_seed(inner_width, body_height, style);
            let mut frames = vec![build_game_of_life_screen_lines(
                inner_width,
                body_height,
                width,
                cell_style,
                &cells,
            )];
            for _ in 1..frame_count {
                cells = step_game_of_life_cells(&cells, width_limit, height_limit);
                frames.push(build_game_of_life_screen_lines(
                    inner_width,
                    body_height,
                    width,
                    cell_style,
                    &cells,
                ));
            }
            frames.push(get_logo_welcome_frame(width));
            frames
        }
        _ => vec![get_logo_welcome_frame(width)],
    }
}

fn trim_resting_frame(mut frames: Vec<Vec<String>>) -> Vec<Vec<String>> {
    if frames.len() <= 1 {
        frames
    } else {
        frames.pop();
        frames
    }
}

fn screen_cycle_frames_non_game_of_life(
    resolved_style: &str,
    width: usize,
) -> Result<Vec<Vec<String>>, CoreError> {
    match resolved_style {
        "logo" => Ok(trim_resting_frame(get_logo_animation_frames(width))),
        other => Err(CoreError::classified(
            ErrorClass::Internal,
            "unsupported_screen_style",
            format!("Unsupported screen style: {other}"),
            "Report this as a Yazelix front-door rendering bug.",
            serde_json::json!({ "style": other }),
        )),
    }
}

fn poll_for_keypress(timeout: Duration) -> Result<bool, CoreError> {
    if !event::poll(timeout).map_err(|source| {
        CoreError::io(
            "front_door_keypress_poll",
            "Failed to read front-door terminal input.",
            "Run Yazelix in an interactive terminal that supports keypress polling.",
            ".",
            source,
        )
    })? {
        return Ok(false);
    }

    match event::read().map_err(|source| {
        CoreError::io(
            "front_door_keypress_read",
            "Failed to read front-door terminal input.",
            "Run Yazelix in an interactive terminal that supports keypress polling.",
            ".",
            source,
        )
    })? {
        Event::Key(_) => Ok(true),
        _ => Ok(false),
    }
}

fn map_front_door_flush_error(source: io::Error) -> CoreError {
    CoreError::io(
        "front_door_flush",
        "Failed to flush front-door terminal output.",
        "Retry in a writable interactive terminal.",
        ".",
        source,
    )
}

fn flush_stdout() -> Result<(), CoreError> {
    yazelix_screen::flush_stdout().map_err(map_front_door_flush_error)
}

fn raw_mode_guard() -> Result<ScreenRawModeGuard, CoreError> {
    ScreenRawModeGuard::new().map_err(|source| {
        CoreError::io(
            "front_door_raw_mode_enable",
            "Failed to enable raw terminal mode for the front-door surface.",
            "Run Yazelix in a terminal that supports raw input mode.",
            ".",
            source,
        )
    })
}

fn play_inline_frames(
    frames: &[Vec<String>],
    frame_delay: Duration,
    width: usize,
) -> Result<(), CoreError> {
    if frames.is_empty() {
        return Ok(());
    }

    println!();
    let max_frame_height = frames.iter().map(Vec::len).max().unwrap_or(0);
    let last_index = frames.len().saturating_sub(1);
    let resting_logo = get_logo_welcome_frame(width);

    for (index, frame) in frames.iter().enumerate() {
        let mut padded = frame.clone();
        while padded.len() < max_frame_height {
            padded.push(String::new());
        }

        for line in &padded {
            print!("\r\u{1b}[2K{line}\n");
        }
        flush_stdout()?;

        if index < last_index {
            if poll_for_keypress(frame_delay)? {
                print!("\u{1b}[H\u{1b}[2J\n");
                for line in &resting_logo {
                    println!("{line}");
                }
                flush_stdout()?;
                return Ok(());
            }
            print!("\u{1b}[{}A", max_frame_height + 1);
        } else {
            print!("\u{1b}[{}A", max_frame_height.saturating_sub(frame.len()));
        }
        flush_stdout()?;
    }

    Ok(())
}

fn enter_screen_mode() -> Result<(), CoreError> {
    yazelix_screen::enter_screen_mode().map_err(map_front_door_flush_error)
}

fn leave_screen_mode() -> Result<(), CoreError> {
    yazelix_screen::leave_screen_mode().map_err(map_front_door_flush_error)
}

fn render_screen_frame(frame: &[String]) -> Result<(), CoreError> {
    yazelix_screen::render_screen_frame(frame).map_err(map_front_door_flush_error)
}

fn game_of_life_screen_context(width: usize, height: usize) -> ScreenAnimationContext {
    let size_class = get_logo_welcome_variant(width);
    let spec = game_of_life_spec(size_class);
    ScreenAnimationContext {
        resolved_width: width,
        resolved_height: height,
        inner_width: fit_inner_width(width, spec.minimum_inner_width),
        size_class,
    }
}

fn boids_screen_context(width: usize, height: usize) -> ScreenAnimationContext {
    ScreenAnimationContext {
        resolved_width: width,
        resolved_height: height,
        inner_width: width,
        size_class: get_logo_welcome_variant(width),
    }
}

pub fn play_welcome_style(style: &str, duration: Duration) -> Result<(), CoreError> {
    play_welcome_style_with_cell_style(style, duration, GameOfLifeCellStyle::FullBlock)
}

pub fn play_welcome_style_with_cell_style(
    style: &str,
    duration: Duration,
    cell_style: GameOfLifeCellStyle,
) -> Result<(), CoreError> {
    let _raw = raw_mode_guard()?;
    let width = terminal_width();
    let height = terminal_height();
    let resolved_style = resolve_welcome_style(style, None)?;
    let playback_duration = if resolved_style == "logo" {
        Duration::from_millis(500)
    } else {
        duration
    };

    if resolved_style == "static" {
        for line in get_logo_welcome_frame(width) {
            println!("{line}");
        }
        println!();
        return Ok(());
    }

    let frames = welcome_sequence(
        &resolved_style,
        width,
        height,
        playback_duration,
        cell_style,
    );
    let frame_delay = if resolved_style.starts_with("game_of_life_") {
        Duration::from_millis(220)
    } else {
        let divisor = frames.len().max(1) as u32;
        playback_duration
            .checked_div(divisor)
            .unwrap_or_else(|| Duration::from_millis(120))
    };
    play_inline_frames(&frames, frame_delay, width)
}

pub fn run_screen_surface(style: Option<&str>) -> Result<i32, CoreError> {
    run_screen_surface_with_cell_style(style, GameOfLifeCellStyle::FullBlock)
}

pub fn run_screen_surface_with_cell_style(
    style: Option<&str>,
    cell_style: GameOfLifeCellStyle,
) -> Result<i32, CoreError> {
    let _raw = raw_mode_guard()?;
    let resolved_style = resolve_screen_style(style, None)?;
    let frame_delay = screen_frame_delay(&resolved_style);
    let is_game_of_life = is_game_of_life_style(&resolved_style);
    let is_boids = resolved_style == "boids";
    let mut width = terminal_width();
    let mut height = terminal_height();
    let mut frames = if is_game_of_life || is_boids {
        Vec::new()
    } else {
        screen_cycle_frames_non_game_of_life(&resolved_style, width)?
    };
    let mut frame_index = 0usize;
    let mut game_of_life_state = if is_game_of_life {
        Some(GameOfLifeAnimation::new(
            &resolved_style,
            game_of_life_screen_context(width, height),
            cell_style,
        ))
    } else {
        None
    };
    let mut boids_state = if is_boids {
        Some(BoidsAnimation::new(
            boids_screen_context(width, height),
            cell_style,
        ))
    } else {
        None
    };

    enter_screen_mode()?;
    let result = (|| -> Result<(), CoreError> {
        loop {
            if let Some(state) = game_of_life_state.as_ref() {
                render_screen_frame(&state.render_frame())?;
            } else if let Some(state) = boids_state.as_ref() {
                render_screen_frame(&state.render_frame())?;
            } else {
                if frames.is_empty() {
                    return Err(CoreError::classified(
                        ErrorClass::Internal,
                        "missing_screen_frames",
                        format!("No frames available for yzx screen style: {resolved_style}"),
                        "Report this as a Yazelix front-door rendering bug.",
                        serde_json::json!({ "style": resolved_style }),
                    ));
                }
                render_screen_frame(&frames[frame_index % frames.len()])?;
            }

            if poll_for_keypress(frame_delay)? {
                break;
            }

            let current_width = terminal_width();
            let current_height = terminal_height();
            if current_width != width || current_height != height {
                width = current_width;
                height = current_height;
                if is_game_of_life {
                    if let Some(state) = game_of_life_state.as_mut() {
                        state.resize(game_of_life_screen_context(width, height));
                    }
                } else if is_boids {
                    if let Some(state) = boids_state.as_mut() {
                        state.resize(boids_screen_context(width, height));
                    }
                } else {
                    frames = screen_cycle_frames_non_game_of_life(&resolved_style, width)?;
                    frame_index = 0;
                }
                continue;
            }

            if let Some(state) = game_of_life_state.as_mut() {
                state.advance_frame();
            } else if let Some(state) = boids_state.as_mut() {
                state.advance_frame();
            } else {
                frame_index += 1;
            }
        }
        Ok(())
    })();
    let leave_result = leave_screen_mode();
    result?;
    leave_result?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trimmed_frame_line_width(line: &str) -> usize {
        visible_line_width(line.trim())
    }

    fn contains_vertical_border(line: &str) -> bool {
        line.contains('│')
    }

    // Test lane: default
    // Defends: `random` still resolves only to the retained Game of Life screen styles instead of drifting back to logo, boids, or static.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn random_screen_style_stays_within_retained_game_of_life_pool() {
        for index in 0..8 {
            let resolved = resolve_screen_style(Some("random"), Some(index)).unwrap();
            assert!(GAME_OF_LIFE_RANDOM_POOL.contains(&resolved.as_str()));
        }
    }

    // Defends: `yzx screen` continues to reject the startup-only `static` style instead of quietly rendering a non-animated frame.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn screen_style_rejects_static() {
        let err = resolve_screen_style(Some("static"), None).unwrap_err();
        assert_eq!(err.code(), "invalid_screen_style");
    }

    // Regression: wide terminals must not let the logo welcome card stretch to a near-full-width frame.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn logo_welcome_frame_keeps_wide_variant_at_designed_width() {
        let frame = get_logo_welcome_frame(110);
        assert_eq!(trimmed_frame_line_width(&frame[0]), 60);
    }

    // Regression: crossing the wide-to-hero breakpoint must not reintroduce a sudden width jump.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn hero_breakpoint_keeps_logo_welcome_width_stable() {
        let wide = get_logo_welcome_frame(119);
        let hero = get_logo_welcome_frame(120);
        assert_eq!(trimmed_frame_line_width(&wide[0]), 60);
        assert_eq!(trimmed_frame_line_width(&hero[0]), 60);
        assert!(
            wide[1..wide.len() - 1]
                .iter()
                .all(|line| !contains_vertical_border(line))
        );
        assert!(
            hero[1..hero.len() - 1]
                .iter()
                .all(|line| !contains_vertical_border(line))
        );
    }

    // Regression: the bordered animated boids welcome follows the same stable hero-width contract.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn boids_welcome_frame_keeps_hero_variant_width_stable() {
        let boids = build_boids_frame(120);
        assert_eq!(trimmed_frame_line_width(&boids[0][0]), 60);
        assert!(
            boids[0][1..boids[0].len() - 1]
                .iter()
                .all(|line| !contains_vertical_border(line))
        );
    }
}
