use crate::{ScreenAnimationContext, ScreenCell, ScreenFrame, ScreenFrameProducer};

const ANSI_NEON_LIME: &str = "\u{1b}[38;5;46m";
const ANSI_NEON_CYAN: &str = "\u{1b}[38;5;51m";
const ANSI_ELECTRIC_BLUE: &str = "\u{1b}[38;5;33m";
const ANSI_VIOLET: &str = "\u{1b}[38;5;129m";
const ANSI_HOT_PINK: &str = "\u{1b}[38;5;201m";
const ANSI_ORANGE: &str = "\u{1b}[38;5;208m";
const ANSI_SUN_YELLOW: &str = "\u{1b}[38;5;226m";
const ANSI_RESET: &str = "\u{1b}[0m";

const MANDELBROT_LOOP_FRAMES: usize = 960;
const MANDELBROT_SEAHORSE_CENTER: Complex64 = Complex64 {
    re: -0.775_683_77,
    im: 0.136_467_37,
};
const MANDELBROT_SEAHORSE_BASE_SCALE_X: f64 = 0.000_5;
const MANDELBROT_SEAHORSE_LOOP_POWERS: f64 = 121.0;
const MANDELBROT_SEAHORSE_MIN_ITERATIONS: usize = 800;
const MANDELBROT_SEAHORSE_MAX_ITERATIONS: usize = 2_500;
const MANDELBROT_SEAHORSE_PERIOD_MULTIPLIER: Complex64 = Complex64 {
    re: 1.042_778_623_972_814,
    im: -0.276_965_371_839_069_33,
};
const MANDELBROT_SEAHORSE_TOUR_WAYPOINTS: [MandelbrotTourWaypoint; 9] = [
    MandelbrotTourWaypoint::new(0.00, 0.00, 0.00), // loop seam and valley entrance
    MandelbrotTourWaypoint::new(0.10, 0.13, -0.16), // valley entrance
    MandelbrotTourWaypoint::new(0.22, -0.36, -0.08), // side tendrils
    MandelbrotTourWaypoint::new(0.36, -0.20, 0.30), // bulb boundary
    MandelbrotTourWaypoint::new(0.50, 0.26, 0.20), // spiral arm
    MandelbrotTourWaypoint::new(0.66, 0.42, -0.14), // dense filament field
    MandelbrotTourWaypoint::new(0.80, 0.08, -0.34), // lower tendrils
    MandelbrotTourWaypoint::new(0.92, -0.10, -0.12), // return valley
    MandelbrotTourWaypoint::new(1.00, 0.00, 0.00), // exact loop seam
];

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
    let zoom_iterations = zoom.max(1.0).log2().mul_add(96.0, 0.0).round() as usize;
    base_iterations.saturating_add(zoom_iterations).clamp(
        MANDELBROT_SEAHORSE_MIN_ITERATIONS,
        MANDELBROT_SEAHORSE_MAX_ITERATIONS,
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Complex64 {
    re: f64,
    im: f64,
}

impl Complex64 {
    fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }

    fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    fn powf(self, exponent: f64) -> Self {
        let magnitude = self.abs().powf(exponent);
        let angle = self.arg() * exponent;
        Self {
            re: magnitude * angle.cos(),
            im: magnitude * angle.sin(),
        }
    }

    fn scale(self, scalar: f64) -> Self {
        Self {
            re: self.re * scalar,
            im: self.im * scalar,
        }
    }

    #[cfg(test)]
    fn distance(self, other: Self) -> f64 {
        (self.re - other.re).hypot(self.im - other.im)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MandelbrotTourWaypoint {
    progress: f64,
    offset_units: Complex64,
}

impl MandelbrotTourWaypoint {
    const fn new(progress: f64, re_units: f64, im_units: f64) -> Self {
        Self {
            progress,
            offset_units: Complex64 {
                re: re_units,
                im: im_units,
            },
        }
    }

    fn offset(self) -> Complex64 {
        self.offset_units.scale(MANDELBROT_SEAHORSE_BASE_SCALE_X)
    }
}

impl std::ops::Add for Complex64 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl std::ops::Mul for Complex64 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MandelbrotEscape {
    iterations: usize,
    normalized_depth: usize,
    distance_estimate: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MandelbrotSample {
    escape: MandelbrotEscape,
    score: f64,
}

pub fn mandelbrot_escape_iterations(cx: f64, cy: f64, max_iterations: usize) -> usize {
    mandelbrot_escape(cx, cy, max_iterations).iterations
}

fn mandelbrot_escape(cx: f64, cy: f64, max_iterations: usize) -> MandelbrotEscape {
    let mut zx = 0.0;
    let mut zy = 0.0;
    let mut derivative_x = 0.0;
    let mut derivative_y = 0.0;

    for iteration in 0..max_iterations {
        let magnitude_squared = zx * zx + zy * zy;
        if magnitude_squared > 4.0 {
            return MandelbrotEscape {
                iterations: iteration,
                normalized_depth: continuous_escape_depth(iteration, magnitude_squared),
                distance_estimate: mandelbrot_distance_estimate(
                    magnitude_squared,
                    derivative_x,
                    derivative_y,
                ),
            };
        }
        let next_derivative_x = 2.0 * (zx * derivative_x - zy * derivative_y) + 1.0;
        let next_derivative_y = 2.0 * (zx * derivative_y + zy * derivative_x);
        let next_x = zx * zx - zy * zy + cx;
        zy = 2.0 * zx * zy + cy;
        zx = next_x;
        derivative_x = next_derivative_x;
        derivative_y = next_derivative_y;
    }

    MandelbrotEscape {
        iterations: max_iterations,
        normalized_depth: max_iterations.saturating_mul(24),
        distance_estimate: None,
    }
}

fn mandelbrot_distance_estimate(
    magnitude_squared: f64,
    derivative_x: f64,
    derivative_y: f64,
) -> Option<f64> {
    let derivative_magnitude = derivative_x.hypot(derivative_y);
    if derivative_magnitude <= f64::EPSILON {
        return None;
    }

    let magnitude = magnitude_squared.sqrt();
    Some(0.5 * magnitude * magnitude.ln() / derivative_magnitude)
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
    let mut cells = render_mandelbrot_cells(width, height, view, max_iterations);

    let seam_blend = mandelbrot_seam_blend(mandelbrot_loop_progress(frame_index));
    if seam_blend > 0.0 {
        let home_view = mandelbrot_view(0);
        let home_max_iterations = mandelbrot_max_iterations_for_zoom(width, height, home_view.zoom);
        let home_cells = render_mandelbrot_cells(width, height, home_view, home_max_iterations);
        blend_mandelbrot_cells_toward_home(&mut cells, &home_cells, width, height, seam_blend);
    }

    let mut frame = ScreenFrame::new(width, height);
    for y in 0..height {
        for x in 0..width {
            if let Some(cell) = cells[y * width + x] {
                frame.set(x, y, cell);
            }
        }
    }

    frame.render_lines(context.resolved_width, colorize_mandelbrot_cell)
}

fn render_mandelbrot_cells(
    width: usize,
    height: usize,
    view: MandelbrotView,
    max_iterations: usize,
) -> Vec<Option<ScreenCell>> {
    let mut samples = vec![None; width.saturating_mul(height)];
    let mut scores = Vec::with_capacity(width.saturating_mul(height));

    for y in 0..height {
        for x in 0..width {
            let (cx, cy) = mandelbrot_point(x, y, width, height, view);
            let escape = mandelbrot_escape(cx, cy, max_iterations);
            if let Some(sample) = mandelbrot_sample(escape, max_iterations, view, width) {
                scores.push(sample.score);
                samples[y * width + x] = Some(sample);
            }
        }
    }

    scores.sort_by(f64::total_cmp);
    let mut cells = vec![None; width.saturating_mul(height)];
    for (index, sample) in samples.into_iter().enumerate() {
        let Some(sample) = sample else {
            continue;
        };
        let rank = scores.partition_point(|score| *score < sample.score);
        let percentile = if scores.len() <= 1 {
            1.0
        } else {
            rank as f64 / (scores.len() - 1) as f64
        };
        cells[index] = mandelbrot_cell(sample, percentile, view);
    }

    cells
}

fn blend_mandelbrot_cells_toward_home(
    cells: &mut [Option<ScreenCell>],
    home_cells: &[Option<ScreenCell>],
    width: usize,
    height: usize,
    seam_blend: f64,
) {
    const BAYER_4X4: [[usize; 4]; 4] =
        [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

    let threshold = (seam_blend.clamp(0.0, 1.0) * 16.0).round() as usize;
    let threshold = threshold.clamp(1, 15);
    for y in 0..height {
        for x in 0..width {
            if BAYER_4X4[y % 4][x % 4] < threshold {
                cells[y * width + x] = home_cells[y * width + x];
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MandelbrotView {
    center: Complex64,
    multiplier: Complex64,
    tour_offset: Complex64,
    base_scale_x: f64,
    scale_x: f64,
    zoom: f64,
}

fn mandelbrot_view(frame_index: usize) -> MandelbrotView {
    let progress = mandelbrot_loop_progress(frame_index);
    let zoom_progress = mandelbrot_zoom_progress(progress);
    let multiplier = MANDELBROT_SEAHORSE_PERIOD_MULTIPLIER
        .powf(-MANDELBROT_SEAHORSE_LOOP_POWERS * zoom_progress);
    let tour_offset = mandelbrot_tour_offset(progress);
    let scale_x = MANDELBROT_SEAHORSE_BASE_SCALE_X * multiplier.abs();

    MandelbrotView {
        center: MANDELBROT_SEAHORSE_CENTER + multiplier * tour_offset,
        multiplier,
        tour_offset,
        base_scale_x: MANDELBROT_SEAHORSE_BASE_SCALE_X,
        scale_x,
        zoom: 1.0 / multiplier.abs(),
    }
}

fn mandelbrot_zoom_progress(progress: f64) -> f64 {
    let t = progress.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mandelbrot_tour_offset(progress: f64) -> Complex64 {
    if !(0.0..1.0).contains(&progress) {
        return MANDELBROT_SEAHORSE_TOUR_WAYPOINTS[0].offset();
    }

    let segment_index = MANDELBROT_SEAHORSE_TOUR_WAYPOINTS
        .windows(2)
        .position(|waypoints| progress >= waypoints[0].progress && progress < waypoints[1].progress)
        .unwrap_or(MANDELBROT_SEAHORSE_TOUR_WAYPOINTS.len() - 2);
    let segment_start = MANDELBROT_SEAHORSE_TOUR_WAYPOINTS[segment_index];
    let segment_end = MANDELBROT_SEAHORSE_TOUR_WAYPOINTS[segment_index + 1];
    let previous = MANDELBROT_SEAHORSE_TOUR_WAYPOINTS
        .get(segment_index.wrapping_sub(1))
        .copied()
        .unwrap_or(segment_start);
    let next = MANDELBROT_SEAHORSE_TOUR_WAYPOINTS
        .get(segment_index + 2)
        .copied()
        .unwrap_or(segment_end);
    let segment_progress = ((progress - segment_start.progress)
        / (segment_end.progress - segment_start.progress))
        .clamp(0.0, 1.0);

    catmull_rom_complex(
        previous.offset(),
        segment_start.offset(),
        segment_end.offset(),
        next.offset(),
        segment_progress,
    )
}

fn catmull_rom_complex(
    previous: Complex64,
    start: Complex64,
    end: Complex64,
    next: Complex64,
    progress: f64,
) -> Complex64 {
    Complex64 {
        re: catmull_rom_coordinate(previous.re, start.re, end.re, next.re, progress),
        im: catmull_rom_coordinate(previous.im, start.im, end.im, next.im, progress),
    }
}

fn catmull_rom_coordinate(previous: f64, start: f64, end: f64, next: f64, progress: f64) -> f64 {
    let t = progress.clamp(0.0, 1.0);
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * (2.0 * start
        + (end - previous) * t
        + (2.0 * previous - 5.0 * start + 4.0 * end - next) * t2
        + (-previous + 3.0 * start - 3.0 * end + next) * t3)
}

fn mandelbrot_loop_progress(frame_index: usize) -> f64 {
    if MANDELBROT_LOOP_FRAMES <= 1 {
        0.0
    } else {
        (frame_index % MANDELBROT_LOOP_FRAMES) as f64 / MANDELBROT_LOOP_FRAMES as f64
    }
}

fn mandelbrot_seam_blend(progress: f64) -> f64 {
    const BLEND_START: f64 = 0.985;
    if progress <= BLEND_START {
        return 0.0;
    }

    let t = ((progress - BLEND_START) / (1.0 - BLEND_START)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
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

    let base = Complex64 {
        re: (nx - 0.5) * view.base_scale_x,
        im: (ny - 0.5) * view.base_scale_x * 0.64,
    };
    let point = view.center + view.multiplier * base;
    (point.re, point.im)
}

fn mandelbrot_sample(
    escape: MandelbrotEscape,
    max_iterations: usize,
    view: MandelbrotView,
    width: usize,
) -> Option<MandelbrotSample> {
    let iterations = escape.iterations;
    if iterations <= 1 {
        return None;
    }

    let pixel_scale = view.scale_x / width.max(1) as f64;
    let distance_pixels = escape
        .distance_estimate
        .map(|distance| distance / pixel_scale.max(f64::MIN_POSITIVE))
        .unwrap_or(0.0);
    let boundary_weight = (1.0 / (1.0 + distance_pixels.max(0.0).powf(0.7))).clamp(0.0, 1.0);
    let dwell_weight = (iterations as f64 / max_iterations as f64).powf(0.35);
    let score = boundary_weight * 0.65 + dwell_weight * 0.35;

    Some(MandelbrotSample { escape, score })
}

fn mandelbrot_cell(
    sample: MandelbrotSample,
    percentile: f64,
    view: MandelbrotView,
) -> Option<ScreenCell> {
    let normalized_percentile = percentile.clamp(0.0, 1.0);
    let (glyph, intensity_bucket) = if normalized_percentile < 0.14 {
        return None;
    } else if normalized_percentile < 0.34 {
        ('░', 1)
    } else if normalized_percentile < 0.58 {
        ('▒', 2)
    } else if normalized_percentile < 0.80 {
        ('▓', 4)
    } else {
        ('█', 6)
    };

    let rank_band = (normalized_percentile * 6.0).floor() as usize;
    let zoom_band = (view.zoom.max(1.0).log2() / 4.0).round().max(0.0) as usize;
    let iteration_band = sample.escape.iterations / 160;
    Some(ScreenCell {
        glyph,
        color_x: rank_band + zoom_band + iteration_band,
        color_y: intensity_bucket,
    })
}

fn colorize_mandelbrot_cell(cell: ScreenCell) -> String {
    let palette = [
        ANSI_NEON_CYAN,
        ANSI_ELECTRIC_BLUE,
        ANSI_VIOLET,
        ANSI_HOT_PINK,
        ANSI_ORANGE,
        ANSI_SUN_YELLOW,
        ANSI_NEON_LIME,
    ];
    let color = palette[(cell.color_x + cell.color_y / 2) % palette.len()];
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

    fn render_test_frame(context: ScreenAnimationContext, frame_index: usize) -> Vec<String> {
        render_mandelbrot_frame(context, frame_index)
    }

    fn visible_frame_similarity(first: &[String], second: &[String]) -> f64 {
        let mut matching_cells = 0;
        let mut total_cells = 0;

        for (first_line, second_line) in first.iter().zip(second.iter()) {
            for (first_cell, second_cell) in first_line.chars().zip(second_line.chars()) {
                if first_cell == second_cell {
                    matching_cells += 1;
                }
                total_cells += 1;
            }
        }

        matching_cells as f64 / total_cells as f64
    }

    fn dominant_visible_glyph_fraction(frame: &[String]) -> f64 {
        let mut counts = std::collections::BTreeMap::new();
        let mut total_cells = 0;

        for line in frame {
            for cell in line.chars() {
                *counts.entry(cell).or_insert(0) += 1;
                total_cells += 1;
            }
        }

        counts.values().copied().max().unwrap_or(0) as f64 / total_cells as f64
    }

    fn visible_glyph_fraction(frame: &[String], predicate: impl Fn(char) -> bool) -> f64 {
        let mut matching_cells = 0;
        let mut total_cells = 0;

        for line in frame {
            for cell in line.chars() {
                if predicate(cell) {
                    matching_cells += 1;
                }
                total_cells += 1;
            }
        }

        matching_cells as f64 / total_cells as f64
    }

    fn visible_mass_center(frame: &[String]) -> Option<(f64, f64)> {
        let mut weighted_x = 0.0;
        let mut weighted_y = 0.0;
        let mut total_weight = 0.0;

        for (y, line) in frame.iter().enumerate() {
            for (x, cell) in line.chars().enumerate() {
                let weight = match cell {
                    '█' => 4.0,
                    '▓' => 3.0,
                    '▒' => 2.0,
                    '░' => 1.0,
                    _ => 0.0,
                };
                weighted_x += x as f64 * weight;
                weighted_y += y as f64 * weight;
                total_weight += weight;
            }
        }

        (total_weight > 0.0).then_some((weighted_x / total_weight, weighted_y / total_weight))
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
        let initial = strip_ansi_from_frame(render_test_frame(context(64, 20), 0));
        let deep_zoom = strip_ansi_from_frame(render_test_frame(
            context(64, 20),
            MANDELBROT_LOOP_FRAMES * 3 / 8,
        ));

        assert_ne!(initial, deep_zoom);
        assert!(visible_frame_similarity(&initial, &deep_zoom) <= 0.65);
    }

    // Regression: Mandelbrot must not spend sampled loop points as uniform full-block or low-detail screens.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn mandelbrot_sampled_frames_keep_visible_variation() {
        for frame_index in [
            0,
            MANDELBROT_LOOP_FRAMES / 8,
            MANDELBROT_LOOP_FRAMES / 4,
            MANDELBROT_LOOP_FRAMES / 2,
            MANDELBROT_LOOP_FRAMES * 3 / 4,
            MANDELBROT_LOOP_FRAMES - 1,
        ] {
            let visible = strip_ansi_from_frame(render_test_frame(context(64, 20), frame_index));

            assert!(
                dominant_visible_glyph_fraction(&visible) <= 0.86,
                "frame {frame_index} collapsed to one visible glyph"
            );
        }
    }

    // Regression: Mandelbrot should render as a filled fractal surface, not a sparse dotted contour wallpaper.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn mandelbrot_frames_use_filled_density_not_sparse_dot_texture() {
        for frame_index in [
            0,
            MANDELBROT_LOOP_FRAMES / 4,
            MANDELBROT_LOOP_FRAMES / 2,
            MANDELBROT_LOOP_FRAMES * 3 / 4,
            MANDELBROT_LOOP_FRAMES - 1,
        ] {
            let visible = strip_ansi_from_frame(render_test_frame(context(64, 20), frame_index));
            let dotted_fraction =
                visible_glyph_fraction(&visible, |cell| matches!(cell, '.' | '·' | ':'));
            let filled_fraction =
                visible_glyph_fraction(&visible, |cell| matches!(cell, '░' | '▒' | '▓' | '█'));

            assert_eq!(dotted_fraction, 0.0);
            assert!(
                filled_fraction >= 0.70,
                "frame {frame_index} is too sparse: filled fraction {filled_fraction}"
            );
        }
    }

    // Defends: the finite animation cycle repeats at the real cycle boundary, not by resetting the penultimate frame.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn mandelbrot_loop_boundary_repeats_first_frame_exactly() {
        assert_eq!(
            render_test_frame(context(48, 16), 0),
            render_test_frame(context(48, 16), MANDELBROT_LOOP_FRAMES)
        );
    }

    // Regression: the last visible frame before wrap must already resemble the first frame.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn mandelbrot_loop_seam_is_structurally_continuous() {
        let first = strip_ansi_from_frame(render_test_frame(context(64, 20), 0));
        let penultimate = strip_ansi_from_frame(render_test_frame(
            context(64, 20),
            MANDELBROT_LOOP_FRAMES - 1,
        ));

        assert_ne!(first, penultimate);
        let similarity = visible_frame_similarity(&first, &penultimate);
        assert!(
            similarity >= 0.70,
            "penultimate frame similarity was {similarity}"
        );
    }

    // Defends: the Mandelbrot view uses Seahorse Valley Misiurewicz scaling instead of a wide, boring minibrot path.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn mandelbrot_view_zooms_into_seahorse_spiral_without_zooming_out() {
        let home = mandelbrot_view(0);
        let quarter = mandelbrot_view(MANDELBROT_LOOP_FRAMES / 4);
        let half = mandelbrot_view(MANDELBROT_LOOP_FRAMES / 2);
        let late = mandelbrot_view(MANDELBROT_LOOP_FRAMES * 3 / 4);
        let seam = mandelbrot_view(MANDELBROT_LOOP_FRAMES - 1);
        let loop_home = mandelbrot_view(MANDELBROT_LOOP_FRAMES);

        assert_eq!(home.center, MANDELBROT_SEAHORSE_CENTER);
        assert!(quarter.zoom > home.zoom);
        assert!(half.zoom > quarter.zoom);
        assert!(late.zoom > half.zoom);
        assert!(seam.zoom > late.zoom);
        assert!(seam.scale_x < home.scale_x / 2.0);
        assert!((home.scale_x - MANDELBROT_SEAHORSE_BASE_SCALE_X).abs() < f64::EPSILON);
        assert!((home.multiplier.re - 1.0).abs() < f64::EPSILON);
        assert!(home.multiplier.im.abs() < f64::EPSILON);
        assert_eq!(home, loop_home);
    }

    // Regression: Mandelbrot must tour through Seahorse structures instead of pinning one spiral to pane center.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn mandelbrot_camera_tour_moves_view_center_through_local_offsets() {
        let home = mandelbrot_view(0);
        let zero = Complex64 { re: 0.0, im: 0.0 };

        assert_eq!(mandelbrot_tour_offset(0.0), zero);
        assert_eq!(mandelbrot_tour_offset(1.0), zero);

        for waypoint in MANDELBROT_SEAHORSE_TOUR_WAYPOINTS
            .iter()
            .filter(|waypoint| waypoint.progress > 0.0 && waypoint.progress < 1.0)
        {
            let offset = mandelbrot_tour_offset(waypoint.progress);
            assert_eq!(offset, waypoint.offset());
            assert!(
                offset.distance(zero) >= MANDELBROT_SEAHORSE_BASE_SCALE_X * 0.14,
                "tour waypoint at progress {} is too close to the static center",
                waypoint.progress
            );
        }

        for frame_index in [
            MANDELBROT_LOOP_FRAMES / 10,
            MANDELBROT_LOOP_FRAMES / 4,
            MANDELBROT_LOOP_FRAMES / 2,
            MANDELBROT_LOOP_FRAMES * 2 / 3,
            MANDELBROT_LOOP_FRAMES * 4 / 5,
        ] {
            let view = mandelbrot_view(frame_index);
            assert_ne!(view.tour_offset, zero);
            assert_ne!(
                view.center, home.center,
                "frame {frame_index} stayed pinned to the static Seahorse center"
            );
        }
    }

    // Regression: the camera tour should move visible fractal mass across the pane, not only rotate in place.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn mandelbrot_camera_tour_moves_visible_mass_across_pane() {
        let centers = [
            0,
            MANDELBROT_LOOP_FRAMES / 10,
            MANDELBROT_LOOP_FRAMES / 4,
            MANDELBROT_LOOP_FRAMES / 2,
            MANDELBROT_LOOP_FRAMES * 2 / 3,
            MANDELBROT_LOOP_FRAMES * 4 / 5,
        ]
        .map(|frame_index| {
            let visible = strip_ansi_from_frame(render_test_frame(context(72, 22), frame_index));
            visible_mass_center(&visible).expect("frame should contain visible Mandelbrot mass")
        });
        let min_x = centers
            .iter()
            .map(|(x, _)| *x)
            .fold(f64::INFINITY, f64::min);
        let max_x = centers
            .iter()
            .map(|(x, _)| *x)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_y = centers
            .iter()
            .map(|(_, y)| *y)
            .fold(f64::INFINITY, f64::min);
        let max_y = centers
            .iter()
            .map(|(_, y)| *y)
            .fold(f64::NEG_INFINITY, f64::max);

        assert!(
            max_x - min_x >= 4.0,
            "visible mass did not travel enough horizontally: {centers:?}"
        );
        assert!(
            max_y - min_y >= 0.75,
            "visible mass did not travel enough vertically: {centers:?}"
        );
    }

    // Defends: the loop depth is chosen to return near the same orientation after a materially deep zoom.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn mandelbrot_loop_power_aligns_seahorse_rotation() {
        let total_turns = MANDELBROT_SEAHORSE_PERIOD_MULTIPLIER.arg().abs()
            * MANDELBROT_SEAHORSE_LOOP_POWERS
            / (std::f64::consts::PI * 2.0);
        let turn_error = (total_turns - total_turns.round()).abs();
        let total_zoom = MANDELBROT_SEAHORSE_PERIOD_MULTIPLIER
            .abs()
            .powf(MANDELBROT_SEAHORSE_LOOP_POWERS);

        assert!(turn_error < 0.001);
        assert!(total_zoom > 9_000.0);
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
        assert!(
            visible
                .iter()
                .any(|line| { line.chars().any(|ch| matches!(ch, '░' | '▒' | '▓' | '█')) })
        );
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
        assert_eq!(
            mandelbrot_max_iterations_for_zoom(64, 20, 1.0),
            MANDELBROT_SEAHORSE_MIN_ITERATIONS
        );
        assert_eq!(mandelbrot_max_iterations_for_zoom(300, 120, 1_450.0), 1_104);
        assert_eq!(
            mandelbrot_max_iterations_for_zoom(300, 120, 1_000_000_000.0),
            MANDELBROT_SEAHORSE_MAX_ITERATIONS
        );
    }
}
