//! Front-door rendering and screen playback for Rust-owned welcome/tutor/report UX.

use crate::bridge::{CoreError, ErrorClass};
use crossterm::event::{self, Event};
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
pub use yazelix_screen::GameOfLifeCellStyle;
use yazelix_screen::{
    BoidsAnimation, BoidsVariant, GameOfLifeAnimation, MandelbrotAnimation,
    RawModeGuard as ScreenRawModeGuard, ScreenAnimationContext, ScreenFrameProducer,
    build_game_of_life_screen_lines, build_live_game_of_life_seed, center_frame_lines, center_text,
    game_of_life_grid_height, game_of_life_grid_width, game_of_life_spec, is_boids_style,
    is_game_of_life_style, mandelbrot_frame_delay, resolve_game_of_life_body_height,
    step_game_of_life_cells, terminal_height, terminal_width, visible_line_width,
};

const ASCII_ART_DATA_JSON: &str = include_str!("../assets/ascii_art_data.json");

const ANSI_RED: &str = "\u{1b}[31m";
const ANSI_GREEN: &str = "\u{1b}[32m";
const ANSI_YELLOW: &str = "\u{1b}[33m";
const ANSI_BLUE: &str = "\u{1b}[34m";
const ANSI_PURPLE: &str = "\u{1b}[35m";
const ANSI_RESET: &str = "\u{1b}[0m";
const ANSI_FAINT: &str = "\u{1b}[2m";
const ASCII_MAGICIAN_ATTRIBUTION: &str = "ascii magician GIF by 1mposter";
const ASCII_MAGICIAN_FRAME_COUNT: usize = 198;
const ASCII_MAGICIAN_FRAME_DELAY_MS: u64 = 90;
const ASCII_MAGICIAN_FRAME_DIR: &str = "assets/third_party/ascii_magician_1mposter_frames";
const ASCII_MAGICIAN_EDGE_INSET_COLUMNS: usize = 4;
const ASCII_MAGICIAN_EDGE_INSET_ROWS: usize = 4;
const KITTY_MAGICIAN_IMAGE_ID_BASE: u32 = 7_930_000;

const GAME_OF_LIFE_RANDOM_POOL: &[&str] = &[
    "game_of_life_gliders",
    "game_of_life_oscillators",
    "game_of_life_bloom",
];
const BOIDS_RANDOM_POOL: &[&str] = &["boids_predator", "boids_schools"];
const RANDOM_ANIMATION_FAMILY_COUNT: usize = 3;

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
    body_height: usize,
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

fn assert_random_animation_pool_is_allowed(allowed: &[&str]) {
    for candidate in GAME_OF_LIFE_RANDOM_POOL
        .iter()
        .chain(BOIDS_RANDOM_POOL.iter())
        .chain(["mandelbrot"].iter())
    {
        if !allowed
            .iter()
            .any(|allowed_style| allowed_style == candidate)
        {
            panic!("missing retained random animation style: {candidate}");
        }
    }
}

fn resolve_random_animation_style(allowed: &[&str], random_index: Option<usize>) -> String {
    assert_random_animation_pool_is_allowed(allowed);
    let subpool_width = lcm(
        GAME_OF_LIFE_RANDOM_POOL.len(),
        BOIDS_RANDOM_POOL.len().max(1),
    );
    let slot_count = RANDOM_ANIMATION_FAMILY_COUNT * subpool_width;
    let selected = random_index.unwrap_or_else(|| system_random_index(slot_count)) % slot_count;
    let family = selected % RANDOM_ANIMATION_FAMILY_COUNT;
    let family_index = (selected / RANDOM_ANIMATION_FAMILY_COUNT) % subpool_width;

    match family {
        0 => GAME_OF_LIFE_RANDOM_POOL[family_index % GAME_OF_LIFE_RANDOM_POOL.len()].to_string(),
        1 => BOIDS_RANDOM_POOL[family_index % BOIDS_RANDOM_POOL.len()].to_string(),
        _ => "mandelbrot".to_string(),
    }
}

fn lcm(left: usize, right: usize) -> usize {
    if left == 0 || right == 0 {
        return 0;
    }
    left / gcd(left, right) * right
}

fn gcd(mut left: usize, mut right: usize) -> usize {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
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
            "Pick one of the documented welcome styles from `settings.jsonc` or `yzx screen --help`.",
            serde_json::json!({ "style": normalized }),
        ));
    }

    if normalized != "random" {
        return Ok(normalized);
    }

    Ok(resolve_random_animation_style(&allowed, random_index))
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
        return Ok(resolve_random_animation_style(&allowed, random_index));
    }
    Ok(normalized)
}

fn screen_frame_delay(resolved_style: &str) -> Duration {
    match resolved_style {
        style if is_game_of_life_style(style) => Duration::from_millis(160),
        style if is_boids_style(style) => Duration::from_millis(70),
        "mandelbrot" => mandelbrot_frame_delay(),
        "magician" => ascii_magician_frame_delay(),
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

fn build_boids_frame(
    width: usize,
    height: usize,
    duration: Duration,
    cell_style: GameOfLifeCellStyle,
    variant: BoidsVariant,
) -> Vec<Vec<String>> {
    let size_class = get_logo_welcome_variant(width);
    let spec = boids_spec(size_class);
    let inner_width = width.saturating_sub(1).max(1);
    let body_height = boids_welcome_body_height(spec, height);
    let frame_delay = Duration::from_millis(70);
    let frame_count = ((duration.as_secs_f64() / frame_delay.as_secs_f64()).ceil() as usize).max(3);
    let mut animation = BoidsAnimation::with_variant(
        ScreenAnimationContext {
            resolved_width: inner_width,
            resolved_height: body_height,
            inner_width,
            size_class,
        },
        cell_style,
        variant,
    );
    let mut frames = Vec::new();

    for _ in 0..frame_count {
        let rows = animation.render_frame();
        frames.push(rows);
        animation.advance_frame();
    }

    frames.push(get_logo_welcome_frame(width));
    frames
}

fn boids_welcome_body_height(spec: &BoidsWelcomeSpec, terminal_height: usize) -> usize {
    terminal_height
        .saturating_sub(2)
        .max(spec.body_height)
        .max(1)
}

fn ascii_magician_frame_delay() -> Duration {
    Duration::from_millis(ASCII_MAGICIAN_FRAME_DELAY_MS)
}

fn ascii_magician_image_id() -> u32 {
    KITTY_MAGICIAN_IMAGE_ID_BASE + process::id() % 10_000
}

fn ascii_magician_frame_path(runtime_dir: &Path, frame_index: usize) -> PathBuf {
    runtime_dir.join(ASCII_MAGICIAN_FRAME_DIR).join(format!(
        "frame_{:03}.png",
        frame_index % ASCII_MAGICIAN_FRAME_COUNT
    ))
}

fn require_ascii_magician_assets(runtime_dir: &Path) -> Result<(), CoreError> {
    for frame_index in [0, ASCII_MAGICIAN_FRAME_COUNT - 1] {
        let frame_path = ascii_magician_frame_path(runtime_dir, frame_index);
        if !frame_path.is_file() {
            return Err(CoreError::classified(
                ErrorClass::Runtime,
                "missing_magician_frame_asset",
                format!("Missing magician GIF frame asset: {}", frame_path.display()),
                "Reinstall Yazelix so the runtime includes assets/third_party/ascii_magician_1mposter_frames.",
                serde_json::json!({ "path": frame_path }),
            ));
        }
    }
    Ok(())
}

fn kitty_graphics_supported() -> bool {
    let zellij_passthrough = std::env::var("YAZELIX_ZELLIJ_KITTY_PASSTHROUGH")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let in_zellij =
        std::env::var_os("ZELLIJ").is_some() || std::env::var_os("ZELLIJ_SESSION_NAME").is_some();

    if in_zellij {
        return zellij_passthrough;
    }

    zellij_passthrough
        || std::env::var_os("KITTY_WINDOW_ID").is_some()
        || std::env::var("TERM")
            .map(|value| value.contains("kitty"))
            .unwrap_or(false)
        || std::env::var("TERM_PROGRAM")
            .map(|value| {
                value.eq_ignore_ascii_case("ghostty") || value.eq_ignore_ascii_case("ratty")
            })
            .unwrap_or(false)
}

fn require_kitty_graphics_for_magician() -> Result<(), CoreError> {
    if kitty_graphics_supported() {
        return Ok(());
    }

    Err(CoreError::classified(
        ErrorClass::Runtime,
        "magician_requires_kitty_graphics",
        "The magician style requires Kitty graphics protocol support.",
        "Run Yazelix in the packaged Ghostty/Ratty runtime with Zellij Kitty passthrough, or choose a non-image welcome style.",
        serde_json::json!({}),
    ))
}

fn base64_encode_bytes(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);
        let packed = ((first as u32) << 16) | ((second as u32) << 8) | third as u32;

        encoded.push(TABLE[((packed >> 18) & 0x3f) as usize] as char);
        encoded.push(TABLE[((packed >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(TABLE[((packed >> 6) & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() > 2 {
            encoded.push(TABLE[(packed & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }
    }
    encoded
}

fn kitty_png_file_command(image_id: u32, columns: usize, rows: usize, path: &Path) -> String {
    let payload = base64_encode_bytes(path.to_string_lossy().as_bytes());
    format!(
        "\u{1b}_Ga=T,f=100,t=f,i={image_id},p=1,c={columns},r={rows},C=1,z=-1,q=2;{payload}\u{1b}\\"
    )
}

fn kitty_delete_image_command(image_id: u32) -> String {
    format!(
        "\u{1b}_Ga=d,d=i,i={image_id},p=1,q=2;\u{1b}\\\
         \u{1b}_Ga=d,d=I,i={image_id},q=2;\u{1b}\\"
    )
}

fn ascii_magician_graphics_layout(width: usize, height: usize) -> (usize, usize, usize, usize) {
    let available_rows = height
        .saturating_sub(ASCII_MAGICIAN_EDGE_INSET_ROWS.saturating_mul(2) + 1)
        .max(1);
    let available_columns = width
        .saturating_sub(ASCII_MAGICIAN_EDGE_INSET_COLUMNS.saturating_mul(2))
        .max(1);
    let rows = available_rows.min(available_columns.max(2) / 2).max(1);
    let columns = rows.saturating_mul(2).min(available_columns);
    let top_padding = height.saturating_sub(rows + 1) / 2;
    let left_padding = width.saturating_sub(columns) / 2;
    (columns, rows, top_padding, left_padding)
}

fn draw_ascii_magician_graphics_frame(
    runtime_dir: &Path,
    width: usize,
    height: usize,
    frame_index: usize,
    image_id: u32,
) -> Result<(), CoreError> {
    let (columns, rows, top_padding, left_padding) = ascii_magician_graphics_layout(width, height);
    let frame_path = ascii_magician_frame_path(runtime_dir, frame_index);

    print!("\u{1b}[H\u{1b}[2J");
    for _ in 0..top_padding {
        println!();
    }
    if left_padding > 0 {
        print!("{}", " ".repeat(left_padding));
    }
    print!(
        "{}",
        kitty_png_file_command(image_id, columns, rows, &frame_path)
    );
    for _ in 0..rows {
        println!();
    }
    println!(
        "{}",
        center_text(
            &format!("{ANSI_FAINT}{ANSI_PURPLE}{ASCII_MAGICIAN_ATTRIBUTION}{ANSI_RESET}"),
            width
        )
    );
    flush_stdout()
}

fn cleanup_ascii_magician_graphics(image_id: u32) -> Result<(), CoreError> {
    print!("{}\u{1b}[H\u{1b}[2J", kitty_delete_image_command(image_id));
    flush_stdout()
}

fn play_ascii_magician_graphics_welcome(
    runtime_dir: &Path,
    duration: Duration,
) -> Result<(), CoreError> {
    require_kitty_graphics_for_magician()?;
    require_ascii_magician_assets(runtime_dir)?;

    let image_id = ascii_magician_image_id();
    let frame_delay = ascii_magician_frame_delay();
    let started_at = Instant::now();
    let mut frame_index = 0usize;

    while started_at.elapsed() < duration.max(frame_delay) {
        draw_ascii_magician_graphics_frame(
            runtime_dir,
            terminal_width(),
            terminal_height(),
            frame_index,
            image_id,
        )?;
        if poll_for_keypress(frame_delay)? {
            break;
        }
        frame_index += 1;
    }

    cleanup_ascii_magician_graphics(image_id)?;
    for line in get_logo_welcome_frame(terminal_width()) {
        println!("{line}");
    }
    flush_stdout()
}

fn run_ascii_magician_graphics_screen(runtime_dir: &Path) -> Result<i32, CoreError> {
    require_kitty_graphics_for_magician()?;
    require_ascii_magician_assets(runtime_dir)?;

    let image_id = ascii_magician_image_id();
    enter_screen_mode()?;
    let result = (|| -> Result<(), CoreError> {
        let mut frame_index = 0usize;
        loop {
            draw_ascii_magician_graphics_frame(
                runtime_dir,
                terminal_width(),
                terminal_height(),
                frame_index,
                image_id,
            )?;
            if poll_for_keypress(ascii_magician_frame_delay())? {
                break;
            }
            frame_index += 1;
        }
        Ok(())
    })();
    let cleanup_before_leave = cleanup_ascii_magician_graphics(image_id);
    let leave_result = leave_screen_mode();
    let cleanup_after_leave = cleanup_ascii_magician_graphics(image_id);
    result?;
    cleanup_before_leave?;
    leave_result?;
    cleanup_after_leave?;
    Ok(0)
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
        style if is_boids_style(style) => build_boids_frame(
            width,
            height,
            duration,
            cell_style,
            BoidsVariant::from_style_name(style).expect("validated boids style"),
        ),
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
    _height: usize,
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
            print!("\r\u{1b}[2K{}\n", inline_printable_line(line));
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

fn inline_printable_line(line: &str) -> &str {
    line.trim_end_matches(' ')
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

fn play_mandelbrot_welcome_screen(duration: Duration) -> Result<(), CoreError> {
    let frame_delay = mandelbrot_frame_delay();
    let mut width = terminal_width();
    let mut height = terminal_height();
    let mut state = MandelbrotAnimation::new(mandelbrot_screen_context(width, height));
    let started = Instant::now();

    enter_screen_mode()?;
    let result = (|| -> Result<(), CoreError> {
        while started.elapsed() < duration {
            render_screen_frame(&state.render_frame())?;

            let remaining = duration.saturating_sub(started.elapsed());
            if poll_for_keypress(frame_delay.min(remaining))? {
                break;
            }

            let current_width = terminal_width();
            let current_height = terminal_height();
            if current_width != width || current_height != height {
                width = current_width;
                height = current_height;
                state.resize(mandelbrot_screen_context(width, height));
            } else {
                state.advance_frame();
            }
        }
        Ok(())
    })();
    let leave_result = leave_screen_mode();
    result?;
    leave_result?;
    Ok(())
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

fn mandelbrot_screen_context(width: usize, height: usize) -> ScreenAnimationContext {
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
    play_welcome_style_inner(style, duration, cell_style, None)
}

pub fn play_welcome_style_with_runtime_dir(
    style: &str,
    duration: Duration,
    cell_style: GameOfLifeCellStyle,
    runtime_dir: &Path,
) -> Result<(), CoreError> {
    play_welcome_style_inner(style, duration, cell_style, Some(runtime_dir))
}

fn missing_magician_runtime_dir() -> CoreError {
    CoreError::classified(
        ErrorClass::Runtime,
        "missing_magician_runtime_dir",
        "The magician style requires the Yazelix runtime asset directory.",
        "Run this command through the packaged `yzx` launcher so YAZELIX_RUNTIME_DIR is available.",
        serde_json::json!({}),
    )
}

fn play_welcome_style_inner(
    style: &str,
    duration: Duration,
    cell_style: GameOfLifeCellStyle,
    runtime_dir: Option<&Path>,
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
    if resolved_style == "mandelbrot" {
        return play_mandelbrot_welcome_screen(playback_duration);
    }
    if resolved_style == "magician" {
        let runtime_dir = runtime_dir.ok_or_else(missing_magician_runtime_dir)?;
        return play_ascii_magician_graphics_welcome(runtime_dir, playback_duration);
    }

    let frames = welcome_sequence(
        &resolved_style,
        width,
        height,
        playback_duration,
        cell_style,
    );
    let frame_delay = match resolved_style.as_str() {
        style if is_game_of_life_style(style) => Duration::from_millis(220),
        style if is_boids_style(style) => Duration::from_millis(70),
        "mandelbrot" => mandelbrot_frame_delay(),
        _ => {
            let divisor = frames.len().max(1) as u32;
            playback_duration
                .checked_div(divisor)
                .unwrap_or_else(|| Duration::from_millis(120))
        }
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
    run_screen_surface_inner(style, cell_style, None)
}

pub fn run_screen_surface_with_runtime_dir(
    style: Option<&str>,
    cell_style: GameOfLifeCellStyle,
    runtime_dir: &Path,
) -> Result<i32, CoreError> {
    run_screen_surface_inner(style, cell_style, Some(runtime_dir))
}

fn run_screen_surface_inner(
    style: Option<&str>,
    cell_style: GameOfLifeCellStyle,
    runtime_dir: Option<&Path>,
) -> Result<i32, CoreError> {
    let _raw = raw_mode_guard()?;
    let resolved_style = resolve_screen_style(style, None)?;
    if resolved_style == "magician" {
        let runtime_dir = runtime_dir.ok_or_else(missing_magician_runtime_dir)?;
        return run_ascii_magician_graphics_screen(runtime_dir);
    }

    let frame_delay = screen_frame_delay(&resolved_style);
    let is_game_of_life = is_game_of_life_style(&resolved_style);
    let boids_variant = BoidsVariant::from_style_name(&resolved_style);
    let is_boids = boids_variant.is_some();
    let is_mandelbrot = resolved_style == "mandelbrot";
    let mut width = terminal_width();
    let mut height = terminal_height();
    let mut frames = if is_game_of_life || is_boids || is_mandelbrot {
        Vec::new()
    } else {
        screen_cycle_frames_non_game_of_life(&resolved_style, width, height)?
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
    let mut mandelbrot_state = if is_mandelbrot {
        Some(MandelbrotAnimation::new(mandelbrot_screen_context(
            width, height,
        )))
    } else {
        None
    };
    let mut boids_state = if is_boids {
        Some(BoidsAnimation::with_variant(
            boids_screen_context(width, height),
            cell_style,
            boids_variant.expect("validated boids style"),
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
            } else if let Some(state) = mandelbrot_state.as_ref() {
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
                } else if is_mandelbrot {
                    if let Some(state) = mandelbrot_state.as_mut() {
                        state.resize(mandelbrot_screen_context(width, height));
                    }
                } else {
                    frames = screen_cycle_frames_non_game_of_life(&resolved_style, width, height)?;
                    frame_index = 0;
                }
                continue;
            }

            if let Some(state) = game_of_life_state.as_mut() {
                state.advance_frame();
            } else if let Some(state) = boids_state.as_mut() {
                state.advance_frame();
            } else if let Some(state) = mandelbrot_state.as_mut() {
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
    // Defends: `yzx screen random` rotates across retained animation families instead of getting stuck in Game of Life variants.
    #[test]
    fn random_screen_style_rotates_across_animation_families() {
        let mut game_of_life_count = 0;
        let mut boids_count = 0;
        let mut mandelbrot_count = 0;

        for index in 0..18 {
            let resolved = resolve_screen_style(Some("random"), Some(index)).unwrap();
            assert_ne!(resolved, "static");
            assert_ne!(resolved, "logo");
            assert_ne!(resolved, "magician");
            if is_game_of_life_style(&resolved) {
                game_of_life_count += 1;
            } else if is_boids_style(&resolved) {
                boids_count += 1;
            } else if resolved == "mandelbrot" {
                mandelbrot_count += 1;
            } else {
                panic!("unexpected random screen style: {resolved}");
            }
        }

        assert_eq!(game_of_life_count, 6);
        assert_eq!(boids_count, 6);
        assert_eq!(mandelbrot_count, 6);
    }

    // Defends: welcome random splits selection evenly across Game of Life, boids, and Mandelbrot families while excluding explicit non-random styles.
    #[test]
    fn random_welcome_style_rotates_evenly_across_animation_families() {
        let mut game_of_life_count = 0;
        let mut boids_count = 0;
        let mut mandelbrot_count = 0;
        let mut boids_styles = Vec::new();

        for index in 0..18 {
            let resolved = resolve_welcome_style("random", Some(index)).unwrap();
            assert_ne!(resolved, "static");
            assert_ne!(resolved, "logo");
            assert_ne!(resolved, "magician");
            if is_game_of_life_style(&resolved) {
                game_of_life_count += 1;
            } else if is_boids_style(&resolved) {
                boids_count += 1;
                boids_styles.push(resolved);
            } else if resolved == "mandelbrot" {
                mandelbrot_count += 1;
            } else {
                panic!("unexpected random welcome style: {resolved}");
            }
        }

        assert_eq!(game_of_life_count, 6);
        assert_eq!(boids_count, 6);
        assert_eq!(mandelbrot_count, 6);
        assert_eq!(
            boids_styles,
            BOIDS_RANDOM_POOL
                .iter()
                .cycle()
                .take(6)
                .map(|style| style.to_string())
                .collect::<Vec<_>>()
        );
    }

    // Defends: `yzx screen` continues to reject the startup-only `static` style instead of quietly rendering a non-animated frame.
    #[test]
    fn screen_style_rejects_static() {
        let err = resolve_screen_style(Some("static"), None).unwrap_err();
        assert_eq!(err.code(), "invalid_screen_style");
    }

    // Defends: Mandelbrot is accepted for welcome playback without changing the startup-only rejection of static in `yzx screen`.
    #[test]
    fn mandelbrot_is_available_to_welcome_and_screen() {
        assert_eq!(
            resolve_screen_style(Some("mandelbrot"), None).unwrap(),
            "mandelbrot"
        );
        assert_eq!(
            resolve_welcome_style("mandelbrot", None).unwrap(),
            "mandelbrot"
        );
        for index in 0..8 {
            assert_ne!(
                resolve_screen_style(Some("random"), Some(index)).unwrap(),
                "static"
            );
            assert_ne!(
                resolve_welcome_style("random", Some(index)).unwrap(),
                "static"
            );
        }
    }

    // Defends: the attributed GIF-derived magician style is an explicit welcome and screen style, not an implicit random pick.
    #[test]
    fn magician_is_available_to_welcome_and_screen() {
        assert_eq!(
            resolve_screen_style(Some("magician"), None).unwrap(),
            "magician"
        );
        assert_eq!(resolve_welcome_style("magician", None).unwrap(), "magician");
        for index in 0..18 {
            assert_ne!(
                resolve_screen_style(Some("random"), Some(index)).unwrap(),
                "magician"
            );
            assert_ne!(
                resolve_welcome_style("random", Some(index)).unwrap(),
                "magician"
            );
        }
    }

    // Regression: wide terminals must not let the logo welcome card stretch to a near-full-width frame.
    #[test]
    fn logo_welcome_frame_keeps_wide_variant_at_designed_width() {
        let frame = get_logo_welcome_frame(110);
        assert_eq!(trimmed_frame_line_width(&frame[0]), 60);
    }

    // Regression: crossing the wide-to-hero breakpoint must not reintroduce a sudden width jump.
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

    // Regression: animated boids welcome renders as an unframed broad flock instead of being trapped in the logo card border.
    #[test]
    fn boids_welcome_frame_uses_unframed_broad_surface() {
        let boids = build_boids_frame(
            120,
            40,
            Duration::from_millis(360),
            GameOfLifeCellStyle::FullBlock,
            BoidsVariant::Predator,
        );
        assert_eq!(boids[0].len(), 38);
        assert!(boids[0].iter().all(|line| visible_line_width(line) == 119));
        assert!(boids[0].iter().all(|line| !contains_vertical_border(line)));
        assert!(boids[0].iter().all(|line| !line.contains('╭')));
        assert!(boids[0].iter().all(|line| !line.contains('╯')));
    }

    // Regression: welcome boids use the actual pane height instead of staying pinned to the tiny catalog minimum at the top of the terminal.
    #[test]
    fn boids_welcome_body_height_tracks_pane_height() {
        let spec = boids_spec("hero");

        assert_eq!(boids_welcome_body_height(spec, 60), 58);
        assert_eq!(boids_welcome_body_height(spec, 4), spec.body_height);
    }

    // Defends: the magician renderer addresses runtime GIF-derived PNG frames through Kitty file payloads instead of lossy ANSI pixel blocks.
    #[test]
    fn magician_graphics_uses_runtime_assets_and_kitty_file_payloads() {
        let runtime_dir = Path::new("/runtime");
        assert_eq!(
            ascii_magician_frame_path(runtime_dir, 0),
            PathBuf::from(
                "/runtime/assets/third_party/ascii_magician_1mposter_frames/frame_000.png"
            )
        );
        assert_eq!(
            ascii_magician_frame_path(runtime_dir, ASCII_MAGICIAN_FRAME_COUNT),
            ascii_magician_frame_path(runtime_dir, 0)
        );
        assert_eq!(
            ascii_magician_frame_path(runtime_dir, ASCII_MAGICIAN_FRAME_COUNT - 1),
            PathBuf::from(
                "/runtime/assets/third_party/ascii_magician_1mposter_frames/frame_197.png"
            )
        );
        assert_eq!(base64_encode_bytes(b"hello"), "aGVsbG8=");

        let command = kitty_png_file_command(123, 80, 40, Path::new("/tmp/frame.png"));
        assert!(command.starts_with("\u{1b}_Ga=T,f=100,t=f,i=123,p=1,c=80,r=40,C=1,z=-1,q=2;"));
        assert!(command.contains("L3RtcC9mcmFtZS5wbmc="));
        assert!(command.ends_with("\u{1b}\\"));

        let delete_command = kitty_delete_image_command(123);
        assert!(delete_command.contains("\u{1b}_Ga=d,d=i,i=123,p=1,q=2;\u{1b}\\"));
        assert!(delete_command.contains("\u{1b}_Ga=d,d=I,i=123,q=2;\u{1b}\\"));
    }

    // Regression: the checked-in magician derivative must keep the full coalesced GIF motion instead of collapsing to a handful of repeated frames.
    #[test]
    fn magician_derivative_keeps_all_unique_gif_frames() {
        let frame_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(ASCII_MAGICIAN_FRAME_DIR);
        let mut unique_frames = std::collections::HashSet::new();

        for frame_index in 0..ASCII_MAGICIAN_FRAME_COUNT {
            let frame_path = frame_dir.join(format!("frame_{frame_index:03}.png"));
            assert!(frame_path.is_file(), "missing {}", frame_path.display());
            unique_frames
                .insert(std::fs::read(&frame_path).unwrap_or_else(|err| {
                    panic!("failed to read {}: {err}", frame_path.display())
                }));
        }

        assert_eq!(unique_frames.len(), ASCII_MAGICIAN_FRAME_COUNT);
        assert!(
            !frame_dir
                .join(format!("frame_{ASCII_MAGICIAN_FRAME_COUNT:03}.png"))
                .exists()
        );
    }

    // Regression: the magician GIF must stay close to the original square bitmap proportions in a full pane.
    #[test]
    fn magician_graphics_layout_preserves_square_image_shape() {
        assert_eq!(ascii_magician_graphics_layout(120, 40), (62, 31, 4, 29));
        assert_eq!(ascii_magician_graphics_layout(190, 60), (102, 51, 4, 44));
    }

    // Regression: inline welcome playback trims trailing padding so centered frames do not trigger terminal autowrap artifacts.
    #[test]
    fn inline_printable_line_trims_right_padding() {
        assert_eq!(inline_printable_line("hello   "), "hello");
        assert_eq!(
            visible_line_width(inline_printable_line(&center_text("YAZELIX", 120))),
            63
        );
    }
}
