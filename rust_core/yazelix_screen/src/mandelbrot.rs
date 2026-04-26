use crate::{ScreenAnimationContext, ScreenCell, ScreenFrame, ScreenFrameProducer};

const ANSI_RED: &str = "\u{1b}[31m";
const ANSI_GREEN: &str = "\u{1b}[32m";
const ANSI_YELLOW: &str = "\u{1b}[33m";
const ANSI_BLUE: &str = "\u{1b}[34m";
const ANSI_PURPLE: &str = "\u{1b}[35m";
const ANSI_CYAN: &str = "\u{1b}[36m";
const ANSI_RESET: &str = "\u{1b}[0m";

const MANDELBROT_LOOP_FRAMES: usize = 960;
const MANDELBROT_MIN_ZOOM: f64 = 0.82;
const MANDELBROT_MAX_ZOOM: f64 = 56.0;
const MANDELBROT_START_CENTER_X: f64 = -0.56;
const MANDELBROT_START_CENTER_Y: f64 = 0.0;
const MANDELBROT_MINIBROT_CENTER_X: f64 = -1.754_877_666_246_692_7;

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

fn mandelbrot_max_iterations_for_zoom(width: usize, height: usize, zoom: f64) -> usize {
    let base_iterations = mandelbrot_max_iterations(width, height);
    let zoom_iterations = zoom.max(1.0).log2().mul_add(14.0, 0.0).round() as usize;
    base_iterations
        .saturating_add(zoom_iterations)
        .clamp(48, 220)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MandelbrotEscape {
    iterations: usize,
    normalized_depth: usize,
}

pub fn mandelbrot_escape_iterations(cx: f64, cy: f64, max_iterations: usize) -> usize {
    mandelbrot_escape(cx, cy, max_iterations).iterations
}

fn mandelbrot_escape(cx: f64, cy: f64, max_iterations: usize) -> MandelbrotEscape {
    let mut zx = 0.0;
    let mut zy = 0.0;

    for iteration in 0..max_iterations {
        let magnitude_squared = zx * zx + zy * zy;
        if magnitude_squared > 4.0 {
            return MandelbrotEscape {
                iterations: iteration,
                normalized_depth: continuous_escape_depth(iteration, magnitude_squared),
            };
        }
        let next_x = zx * zx - zy * zy + cx;
        zy = 2.0 * zx * zy + cy;
        zx = next_x;
    }

    MandelbrotEscape {
        iterations: max_iterations,
        normalized_depth: max_iterations.saturating_mul(24),
    }
}

fn continuous_escape_depth(iteration: usize, magnitude_squared: f64) -> usize {
    let magnitude = magnitude_squared.sqrt().max(2.0);
    let smooth_iteration = iteration as f64 + 1.0 - magnitude.ln().ln() / std::f64::consts::LN_2;
    (smooth_iteration.max(0.0) * 24.0).round() as usize
}

fn render_mandelbrot_frame(context: ScreenAnimationContext, frame_index: usize) -> Vec<String> {
    let width = context.inner_width.max(1);
    let height = context.resolved_height.max(1);
    let view = mandelbrot_view(frame_index);
    let max_iterations = mandelbrot_max_iterations_for_zoom(width, height, view.zoom);
    let mut frame = ScreenFrame::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let (cx, cy) = mandelbrot_point(x, y, width, height, view);
            let escape = mandelbrot_escape(cx, cy, max_iterations);
            if let Some(cell) = mandelbrot_cell(escape, max_iterations) {
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
    zoom: f64,
}

fn mandelbrot_view(frame_index: usize) -> MandelbrotView {
    let progress = mandelbrot_minibrot_loop_progress(frame_index);
    let zoom = MANDELBROT_MIN_ZOOM * (MANDELBROT_MAX_ZOOM / MANDELBROT_MIN_ZOOM).powf(progress);
    let scale_x = 3.12 / zoom;
    let (center_x, center_y) = mandelbrot_camera_center(progress);

    MandelbrotView {
        center_x,
        center_y,
        scale_x,
        scale_y: scale_x * 0.64,
        zoom,
    }
}

fn mandelbrot_minibrot_loop_progress(frame_index: usize) -> f64 {
    let cycle_frame = frame_index % MANDELBROT_LOOP_FRAMES;

    if MANDELBROT_LOOP_FRAMES <= 1 {
        0.0
    } else {
        cycle_frame as f64 / (MANDELBROT_LOOP_FRAMES - 1) as f64
    }
}

fn mandelbrot_camera_center(progress: f64) -> (f64, f64) {
    // True terminal-scale Mandelbrot loop: zoom monotonically toward the large
    // real-axis period-3 baby Mandelbrot. The cycle reset happens only after
    // the mini copy fills the viewport, giving an endless zoom-in illusion
    // instead of the previous fake zoom-in/zoom-out palindrome.
    let center_progress = ease_out_cubic(progress);
    (
        lerp(
            MANDELBROT_START_CENTER_X,
            MANDELBROT_MINIBROT_CENTER_X,
            center_progress,
        ),
        lerp(MANDELBROT_START_CENTER_Y, 0.0, center_progress),
    )
}

fn lerp(start: f64, end: f64, progress: f64) -> f64 {
    let progress = progress.clamp(0.0, 1.0);
    start + (end - start) * progress
}

fn ease_out_cubic(value: f64) -> f64 {
    let value = value.clamp(0.0, 1.0);
    1.0 - (1.0 - value).powi(3)
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

fn mandelbrot_cell(escape: MandelbrotEscape, max_iterations: usize) -> Option<ScreenCell> {
    let iterations = escape.iterations;
    if iterations <= 1 {
        return None;
    }
    let glyph = if iterations >= max_iterations {
        '█'
    } else {
        let gradient = ['.', '·', ':', '░', '▒', '▓'];
        let bucket = (escape.normalized_depth * gradient.len() / (max_iterations * 24))
            .min(gradient.len() - 1);
        gradient[bucket]
    };

    Some(ScreenCell {
        glyph,
        color_x: escape.normalized_depth,
        color_y: iterations,
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
    let color = palette[(cell.color_x / 11 + cell.color_y / 7) % palette.len()];
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

    fn strip_ansi_from_frame(frame: Vec<String>) -> Vec<String> {
        frame
            .into_iter()
            .map(|line| strip_ansi_codes(&line))
            .collect()
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

    // Regression: Mandelbrot must visibly zoom through fractal structure, not merely pulse color.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn mandelbrot_loop_changes_visible_fractal_structure() {
        let initial = strip_ansi_from_frame(render_mandelbrot_frame(context(64, 20), 0));
        let deep_zoom = strip_ansi_from_frame(render_mandelbrot_frame(
            context(64, 20),
            MANDELBROT_LOOP_FRAMES * 3 / 8,
        ));

        assert_ne!(initial, deep_zoom);
    }

    // Defends: the finite animation cycle can repeat forever without a visible loop-boundary snap.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn mandelbrot_frame_loop_boundary_is_exactly_repeatable() {
        assert_eq!(
            render_mandelbrot_frame(context(48, 16), 0),
            render_mandelbrot_frame(context(48, 16), MANDELBROT_LOOP_FRAMES)
        );
    }

    // Defends: the Mandelbrot view zooms monotonically into a self-similar minibrot instead of returning along the same path.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn mandelbrot_view_reaches_minibrot_without_zooming_out() {
        let home = mandelbrot_view(0);
        let quarter = mandelbrot_view(MANDELBROT_LOOP_FRAMES / 4);
        let half = mandelbrot_view(MANDELBROT_LOOP_FRAMES / 2);
        let late = mandelbrot_view(MANDELBROT_LOOP_FRAMES * 3 / 4);
        let minibrot = mandelbrot_view(MANDELBROT_LOOP_FRAMES - 1);
        let loop_home = mandelbrot_view(MANDELBROT_LOOP_FRAMES);

        assert!(quarter.zoom > home.zoom);
        assert!(half.zoom > quarter.zoom);
        assert!(late.zoom > half.zoom);
        assert!(minibrot.zoom > late.zoom);
        assert!(minibrot.scale_x < home.scale_x / 40.0);
        assert!((minibrot.center_x - MANDELBROT_MINIBROT_CENTER_X).abs() < f64::EPSILON);
        assert!(minibrot.center_y.abs() < f64::EPSILON);
        assert_eq!(home, loop_home);
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
        assert_eq!(mandelbrot_max_iterations_for_zoom(300, 120, 56.0), 177);
    }
}
