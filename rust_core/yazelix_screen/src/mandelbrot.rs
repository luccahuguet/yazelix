use crate::{ScreenAnimationContext, ScreenCell, ScreenFrame, ScreenFrameProducer};

const ANSI_RED: &str = "\u{1b}[31m";
const ANSI_GREEN: &str = "\u{1b}[32m";
const ANSI_YELLOW: &str = "\u{1b}[33m";
const ANSI_BLUE: &str = "\u{1b}[34m";
const ANSI_PURPLE: &str = "\u{1b}[35m";
const ANSI_CYAN: &str = "\u{1b}[36m";
const ANSI_RESET: &str = "\u{1b}[0m";

#[derive(Debug, Clone, PartialEq)]
pub struct MandelbrotAnimation {
    context: ScreenAnimationContext,
    frame_index: usize,
}

impl MandelbrotAnimation {
    pub fn new(context: ScreenAnimationContext) -> Self {
        Self {
            context,
            frame_index: 0,
        }
    }
}

impl ScreenFrameProducer for MandelbrotAnimation {
    fn render_frame(&self) -> Vec<String> {
        render_mandelbrot_frame(self.context, self.frame_index)
    }

    fn advance_frame(&mut self) {
        self.frame_index = self.frame_index.wrapping_add(1);
    }

    fn resize(&mut self, context: ScreenAnimationContext) {
        self.context = context;
        self.frame_index = 0;
    }
}

pub fn mandelbrot_max_iterations(width: usize, height: usize) -> usize {
    (42 + width.saturating_mul(height) / 320).clamp(48, 96)
}

pub fn mandelbrot_escape_iterations(cx: f64, cy: f64, max_iterations: usize) -> usize {
    let mut x = 0.0;
    let mut y = 0.0;

    for iteration in 0..max_iterations {
        if x * x + y * y > 4.0 {
            return iteration;
        }
        let next_x = x * x - y * y + cx;
        y = 2.0 * x * y + cy;
        x = next_x;
    }

    max_iterations
}

fn render_mandelbrot_frame(context: ScreenAnimationContext, frame_index: usize) -> Vec<String> {
    let width = context.inner_width.max(1);
    let height = context.resolved_height.max(1);
    let max_iterations = mandelbrot_max_iterations(width, height);
    let view = mandelbrot_view(frame_index);
    let mut frame = ScreenFrame::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let (cx, cy) = mandelbrot_point(x, y, width, height, view);
            let iterations = mandelbrot_escape_iterations(cx, cy, max_iterations);
            if let Some(cell) = mandelbrot_cell(iterations, max_iterations, frame_index) {
                frame.set(x, y, cell);
            }
        }
    }

    frame.render_lines(context.resolved_width, colorize_mandelbrot_cell)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MandelbrotView {
    center_x: f64,
    center_y: f64,
    scale_x: f64,
    scale_y: f64,
}

fn mandelbrot_view(frame_index: usize) -> MandelbrotView {
    let phase = (frame_index % 180) as f64 / 180.0;
    let wave = (phase * std::f64::consts::TAU).sin();
    let zoom = 0.88 + phase * 2.4;

    MandelbrotView {
        center_x: -0.62 + wave * 0.055,
        center_y: 0.02 + (phase * std::f64::consts::TAU).cos() * 0.035,
        scale_x: 3.1 / zoom,
        scale_y: 1.95 / zoom,
    }
}

fn mandelbrot_point(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    view: MandelbrotView,
) -> (f64, f64) {
    let nx = if width <= 1 {
        0.5
    } else {
        x as f64 / (width - 1) as f64
    };
    let ny = if height <= 1 {
        0.5
    } else {
        y as f64 / (height - 1) as f64
    };

    (
        view.center_x + (nx - 0.5) * view.scale_x,
        view.center_y + (ny - 0.5) * view.scale_y,
    )
}

fn mandelbrot_cell(
    iterations: usize,
    max_iterations: usize,
    frame_index: usize,
) -> Option<ScreenCell> {
    if iterations <= 1 {
        return None;
    }
    let glyph = if iterations >= max_iterations {
        '█'
    } else {
        let gradient = ['·', ':', '░', '▒', '▓'];
        let bucket = (iterations * gradient.len() / max_iterations).min(gradient.len() - 1);
        gradient[bucket]
    };

    Some(ScreenCell {
        glyph,
        color_x: iterations,
        color_y: frame_index,
    })
}

fn colorize_mandelbrot_cell(cell: ScreenCell) -> String {
    let palette = [
        ANSI_BLUE,
        ANSI_CYAN,
        ANSI_GREEN,
        ANSI_YELLOW,
        ANSI_PURPLE,
        ANSI_RED,
    ];
    let color = palette[(cell.color_x + cell.color_y / 5) % palette.len()];
    format!("{color}{}{}", cell.glyph, ANSI_RESET)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test lane: default

    fn context(width: usize, height: usize) -> ScreenAnimationContext {
        ScreenAnimationContext {
            resolved_width: width,
            resolved_height: height,
            inner_width: width,
            size_class: "test",
        }
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

    // Defends: Mandelbrot uses deterministic in-house CPU frames without host randomness or external engines.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn mandelbrot_animation_is_deterministic_and_advances() {
        let mut first = MandelbrotAnimation::new(context(48, 16));
        let mut second = MandelbrotAnimation::new(context(48, 16));
        assert_eq!(first.render_frame(), second.render_frame());

        let initial = first.render_frame();
        first.advance_frame();
        second.advance_frame();

        assert_eq!(first.render_frame(), second.render_frame());
        assert_ne!(initial, first.render_frame());
    }

    // Defends: narrow terminals still get a complete frame with no skipped or over-wide rows.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn mandelbrot_renders_narrow_frames_at_exact_dimensions() {
        let visible = MandelbrotAnimation::new(context(16, 8))
            .render_frame()
            .into_iter()
            .map(|line| strip_ansi_codes(&line))
            .collect::<Vec<_>>();

        assert_eq!(visible.len(), 8);
        assert!(visible.iter().all(|line| line.chars().count() == 16));
        assert!(visible.iter().any(|line| {
            line.chars()
                .any(|ch| matches!(ch, '·' | ':' | '░' | '▒' | '▓' | '█'))
        }));
    }

    // Defends: core Mandelbrot math keeps known outside and inside points stable.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn mandelbrot_escape_iterations_classify_known_points() {
        let max_iterations = 64;

        assert_eq!(
            mandelbrot_escape_iterations(0.0, 0.0, max_iterations),
            max_iterations
        );
        assert!(mandelbrot_escape_iterations(2.0, 2.0, max_iterations) < 3);
    }

    // Defends: CPU work is bounded by a small deterministic iteration budget across practical terminal sizes.
    // Strength: defect=1 behavior=2 resilience=2 cost=1 uniqueness=2 total=8/10
    #[test]
    fn mandelbrot_iteration_budget_is_bounded() {
        assert_eq!(mandelbrot_max_iterations(1, 1), 48);
        assert_eq!(mandelbrot_max_iterations(120, 40), 57);
        assert_eq!(mandelbrot_max_iterations(300, 120), 96);
    }
}
