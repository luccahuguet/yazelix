use std::{io, thread, time::Duration};

use yazelix_screen::{
    BoidsAnimation, BoidsVariant, GameOfLifeAnimation, GameOfLifeCellStyle, MandelbrotAnimation,
    ScreenAnimationContext, ScreenFrameProducer, enter_screen_mode, game_of_life_spec,
    is_game_of_life_style, leave_screen_mode, render_screen_frame, terminal_height, terminal_width,
};

const GAME_OF_LIFE_STYLES: &[&str] = &[
    "game_of_life_gliders",
    "game_of_life_oscillators",
    "game_of_life_bloom",
];

#[derive(Debug, Clone, Copy)]
enum ExampleStyle {
    Boids(BoidsVariant),
    GameOfLife(&'static str),
    Mandelbrot,
}

struct ScreenModeGuard;

impl ScreenModeGuard {
    fn new() -> io::Result<Self> {
        enter_screen_mode()?;
        Ok(Self)
    }
}

impl Drop for ScreenModeGuard {
    fn drop(&mut self) {
        let _ = leave_screen_mode();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let raw_style = args.next().unwrap_or_else(|| "mandelbrot".to_string());
    let frame_count = args
        .next()
        .map(|raw| raw.parse::<usize>())
        .transpose()?
        .unwrap_or(90);
    let style = resolve_style(&raw_style)?;
    let mut animation = build_animation(style);
    let frame_delay = frame_delay(style);

    let _screen = ScreenModeGuard::new()?;
    for _ in 0..frame_count {
        render_screen_frame(&animation.render_frame())?;
        animation.advance_frame();
        thread::sleep(frame_delay);
    }

    Ok(())
}

fn resolve_style(raw: &str) -> Result<ExampleStyle, io::Error> {
    let normalized = raw.trim().to_ascii_lowercase();
    if let Some(variant) = BoidsVariant::from_style_name(&normalized) {
        return Ok(ExampleStyle::Boids(variant));
    }
    if normalized == "mandelbrot" {
        return Ok(ExampleStyle::Mandelbrot);
    }
    if is_game_of_life_style(&normalized) {
        let style = GAME_OF_LIFE_STYLES
            .iter()
            .find(|candidate| **candidate == normalized)
            .copied()
            .expect("is_game_of_life_style matched a known example style");
        return Ok(ExampleStyle::GameOfLife(style));
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "unsupported style `{normalized}`; expected boids, boids_predator, boids_schools, mandelbrot, game_of_life_gliders, game_of_life_oscillators, or game_of_life_bloom"
        ),
    ))
}

fn build_animation(style: ExampleStyle) -> Box<dyn ScreenFrameProducer> {
    let context = context_for_style(style, terminal_width(), terminal_height());
    match style {
        ExampleStyle::Boids(variant) => Box::new(BoidsAnimation::with_variant(
            context,
            GameOfLifeCellStyle::FullBlock,
            variant,
        )),
        ExampleStyle::GameOfLife(style_name) => Box::new(GameOfLifeAnimation::new(
            style_name,
            context,
            GameOfLifeCellStyle::FullBlock,
        )),
        ExampleStyle::Mandelbrot => Box::new(MandelbrotAnimation::new(context)),
    }
}

fn context_for_style(style: ExampleStyle, width: usize, height: usize) -> ScreenAnimationContext {
    let size_class = size_class(width);
    let inner_width = match style {
        ExampleStyle::GameOfLife(_) => {
            let spec = game_of_life_spec(size_class);
            width.saturating_sub(6).max(spec.minimum_inner_width)
        }
        ExampleStyle::Boids(_) | ExampleStyle::Mandelbrot => width,
    };

    ScreenAnimationContext {
        resolved_width: width,
        resolved_height: height,
        inner_width,
        size_class,
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

fn frame_delay(style: ExampleStyle) -> Duration {
    match style {
        ExampleStyle::Boids(_) => Duration::from_millis(70),
        ExampleStyle::Mandelbrot => Duration::from_millis(110),
        ExampleStyle::GameOfLife(_) => Duration::from_millis(160),
    }
}
