use crossterm::event::{self, Event};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use yazelix_screen::{
    BoidsAnimation, BoidsVariant, GameOfLifeAnimation, GameOfLifeCellStyle, MandelbrotAnimation,
    RawModeGuard, ScreenAnimationContext, ScreenFrameProducer, enter_screen_mode,
    game_of_life_spec, is_game_of_life_style, leave_screen_mode, render_screen_frame,
    terminal_height, terminal_width,
};

const GAME_OF_LIFE_STYLES: &[&str] = &[
    "game_of_life_gliders",
    "game_of_life_oscillators",
    "game_of_life_bloom",
];
const STANDALONE_RANDOM_POOL: &[&str] = &[
    "boids_predator",
    "boids_schools",
    "mandelbrot",
    "game_of_life_gliders",
    "game_of_life_oscillators",
    "game_of_life_bloom",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StandaloneStyle {
    Boids(BoidsVariant),
    GameOfLife(&'static str),
    Mandelbrot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    style: String,
    cell_style: GameOfLifeCellStyle,
    help: bool,
}

struct ScreenModeGuard;

impl ScreenModeGuard {
    fn new() -> std::io::Result<Self> {
        enter_screen_mode()?;
        Ok(Self)
    }
}

impl Drop for ScreenModeGuard {
    fn drop(&mut self) {
        let _ = leave_screen_mode();
    }
}

fn main() {
    match run(std::env::args().skip(1)) {
        Ok(()) => {}
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    }
}

fn run(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    let parsed = parse_args(args)?;
    if parsed.help {
        print_help();
        return Ok(());
    }

    run_screen(parsed.style.as_str(), parsed.cell_style)
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut help = false;
    let mut style = None;
    let mut cell_style = GameOfLifeCellStyle::FullBlock;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" | "help" => help = true,
            "--cell-style" => {
                let Some(raw) = iter.next() else {
                    return Err("Missing value after --cell-style".to_string());
                };
                cell_style = GameOfLifeCellStyle::parse(&raw).map_err(|error| {
                    format!(
                        "Invalid --cell-style value `{}`. Expected full_block or dotted",
                        error.normalized()
                    )
                })?;
            }
            other if style.is_none() => style = Some(other.to_string()),
            other => {
                return Err(format!("Unexpected argument `{other}`. Try `yzs --help`"));
            }
        }
    }

    Ok(Args {
        style: style.unwrap_or_else(|| "random".to_string()),
        cell_style,
        help,
    })
}

fn print_help() {
    println!("Show standalone Yazelix terminal screen animations");
    println!();
    println!("Usage:");
    println!("  yzs [STYLE] [--cell-style full_block|dotted]");
    println!();
    println!("Styles:");
    println!("  boids");
    println!("  boids_predator");
    println!("  boids_schools");
    println!("  mandelbrot");
    println!("  game_of_life_gliders");
    println!("  game_of_life_oscillators");
    println!("  game_of_life_bloom");
    println!("  random");
    println!();
    println!("Notes:");
    println!("  Runs outside Zellij and outside a Yazelix session");
    println!("  Press any key to exit");
}

fn run_screen(style: &str, cell_style: GameOfLifeCellStyle) -> Result<(), String> {
    let resolved_style = resolve_style(style, None)?;
    let _raw = RawModeGuard::new().map_err(|error| format!("Could not enter raw mode: {error}"))?;
    let _screen = ScreenModeGuard::new()
        .map_err(|error| format!("Could not enter alternate screen mode: {error}"))?;
    let mut width = terminal_width();
    let mut height = terminal_height();
    let mut animation = build_animation(resolved_style, width, height, cell_style);
    let frame_delay = frame_delay(resolved_style);

    loop {
        render_screen_frame(&animation.render_frame())
            .map_err(|error| format!("Could not render screen frame: {error}"))?;
        if poll_for_keypress(frame_delay)? {
            break;
        }

        let current_width = terminal_width();
        let current_height = terminal_height();
        if current_width != width || current_height != height {
            width = current_width;
            height = current_height;
            animation.resize(context_for_style(resolved_style, width, height));
            continue;
        }

        animation.advance_frame();
    }

    Ok(())
}

fn resolve_style(raw: &str, random_index: Option<usize>) -> Result<StandaloneStyle, String> {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized == "random" {
        let index =
            random_index.unwrap_or_else(|| system_random_index(STANDALONE_RANDOM_POOL.len()));
        return resolve_style(
            STANDALONE_RANDOM_POOL[index % STANDALONE_RANDOM_POOL.len()],
            None,
        );
    }

    if let Some(variant) = BoidsVariant::from_style_name(&normalized) {
        return Ok(StandaloneStyle::Boids(variant));
    }
    if normalized == "mandelbrot" {
        return Ok(StandaloneStyle::Mandelbrot);
    }
    if is_game_of_life_style(&normalized) {
        let style = GAME_OF_LIFE_STYLES
            .iter()
            .find(|candidate| **candidate == normalized)
            .copied()
            .expect("is_game_of_life_style matched known standalone style");
        return Ok(StandaloneStyle::GameOfLife(style));
    }

    Err(format!(
        "Unsupported standalone yzs style `{normalized}`. Try `yzs --help`"
    ))
}

fn system_random_index(max_len: usize) -> usize {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as usize;
    nanos % max_len.max(1)
}

fn build_animation(
    style: StandaloneStyle,
    width: usize,
    height: usize,
    cell_style: GameOfLifeCellStyle,
) -> Box<dyn ScreenFrameProducer> {
    let context = context_for_style(style, width, height);
    match style {
        StandaloneStyle::Boids(variant) => {
            Box::new(BoidsAnimation::with_variant(context, cell_style, variant))
        }
        StandaloneStyle::GameOfLife(style_name) => {
            Box::new(GameOfLifeAnimation::new(style_name, context, cell_style))
        }
        StandaloneStyle::Mandelbrot => Box::new(MandelbrotAnimation::new(context)),
    }
}

fn context_for_style(
    style: StandaloneStyle,
    width: usize,
    height: usize,
) -> ScreenAnimationContext {
    match style {
        StandaloneStyle::GameOfLife(_) => game_of_life_context(width, height),
        StandaloneStyle::Boids(_) | StandaloneStyle::Mandelbrot => {
            full_screen_context(width, height)
        }
    }
}

fn game_of_life_context(width: usize, height: usize) -> ScreenAnimationContext {
    let size_class = size_class(width);
    let spec = game_of_life_spec(size_class);
    ScreenAnimationContext {
        resolved_width: width,
        resolved_height: height,
        inner_width: fit_inner_width(width, spec.minimum_inner_width),
        size_class,
    }
}

fn full_screen_context(width: usize, height: usize) -> ScreenAnimationContext {
    ScreenAnimationContext {
        resolved_width: width,
        resolved_height: height,
        inner_width: width,
        size_class: size_class(width),
    }
}

fn size_class(width: usize) -> &'static str {
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

fn fit_inner_width(resolved_width: usize, minimum_width: usize) -> usize {
    resolved_width.saturating_sub(6).max(minimum_width)
}

fn frame_delay(style: StandaloneStyle) -> Duration {
    match style {
        StandaloneStyle::Boids(_) => Duration::from_millis(70),
        StandaloneStyle::Mandelbrot => Duration::from_millis(110),
        StandaloneStyle::GameOfLife(_) => Duration::from_millis(160),
    }
}

fn poll_for_keypress(timeout: Duration) -> Result<bool, String> {
    if !event::poll(timeout).map_err(|error| format!("Could not poll for keypress: {error}"))? {
        return Ok(false);
    }

    loop {
        match event::read().map_err(|error| format!("Could not read terminal event: {error}"))? {
            Event::Key(_) => return Ok(true),
            _ => {
                if !event::poll(Duration::from_millis(0))
                    .map_err(|error| format!("Could not poll terminal event queue: {error}"))?
                {
                    return Ok(false);
                }
            }
        }
    }
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: the standalone binary owns a small no-session style surface instead of borrowing yzx screen's config/session-only styles.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn resolve_style_accepts_only_standalone_animation_styles() {
        assert_eq!(
            resolve_style("boids", None).unwrap(),
            StandaloneStyle::Boids(BoidsVariant::Predator)
        );
        assert_eq!(
            resolve_style("game_of_life_bloom", None).unwrap(),
            StandaloneStyle::GameOfLife("game_of_life_bloom")
        );
        assert_eq!(
            resolve_style("mandelbrot", None).unwrap(),
            StandaloneStyle::Mandelbrot
        );
        assert!(resolve_style("static", None).is_err());
        assert!(resolve_style("logo", None).is_err());
    }

    // Defends: random standalone playback always resolves into an actual animation engine available without Yazelix config.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn random_style_resolves_inside_standalone_pool() {
        for index in 0..STANDALONE_RANDOM_POOL.len() * 2 {
            let resolved = resolve_style("random", Some(index)).unwrap();
            assert!(matches!(
                resolved,
                StandaloneStyle::Boids(_)
                    | StandaloneStyle::GameOfLife(_)
                    | StandaloneStyle::Mandelbrot
            ));
        }
    }

    // Defends: standalone Game of Life keeps the same minimum-width sizing contract as the integrated screen renderer.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn game_of_life_context_preserves_inner_width_floor() {
        let context = game_of_life_context(20, 10);

        assert_eq!(context.size_class, "narrow");
        assert_eq!(
            context.inner_width,
            game_of_life_spec("narrow").minimum_inner_width
        );
        assert_eq!(context.resolved_height, 10);
    }

    // Defends: CLI parsing keeps the package preview simple while still exposing dotted Game of Life cells for parity with Yazelix.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn parse_args_accepts_style_and_cell_style_without_session_config() {
        let parsed = parse_args([
            "game_of_life_gliders".to_string(),
            "--cell-style".to_string(),
            "dotted".to_string(),
        ])
        .unwrap();

        assert_eq!(parsed.style, "game_of_life_gliders");
        assert_eq!(parsed.cell_style, GameOfLifeCellStyle::Dotted);
        assert!(!parsed.help);
    }
}
