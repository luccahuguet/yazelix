use crate::{
    GameOfLifeCellStyle, ScreenAnimationContext, ScreenCell, ScreenFrame, ScreenFrameProducer,
};

const ANSI_BLUE: &str = "\u{1b}[34m";
const ANSI_GREEN: &str = "\u{1b}[32m";
const ANSI_YELLOW: &str = "\u{1b}[33m";
const ANSI_PURPLE: &str = "\u{1b}[35m";
const ANSI_CYAN: &str = "\u{1b}[36m";
const ANSI_RESET: &str = "\u{1b}[0m";

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn limit(self, max: f64) -> Self {
        let length = self.length();
        if length <= max || length == 0.0 {
            self
        } else {
            self.scale(max / length)
        }
    }

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    fn scale(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Boid {
    position: Vec2,
    velocity: Vec2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoidsAnimation {
    context: ScreenAnimationContext,
    cell_style: GameOfLifeCellStyle,
    boids: Vec<Boid>,
}

impl BoidsAnimation {
    pub fn new(context: ScreenAnimationContext, cell_style: GameOfLifeCellStyle) -> Self {
        let boids = seed_boids(context);
        Self {
            context,
            cell_style,
            boids,
        }
    }

    fn grid_width(&self) -> usize {
        (self.context.inner_width / 2).max(1)
    }

    fn grid_height(&self) -> usize {
        self.context.resolved_height.max(1)
    }
}

impl ScreenFrameProducer for BoidsAnimation {
    fn render_frame(&self) -> Vec<String> {
        let grid_width = self.grid_width();
        let grid_height = self.grid_height();
        let mut frame = ScreenFrame::new(self.context.inner_width, grid_height);
        for (index, boid) in self.boids.iter().enumerate() {
            let x = wrapped_index(boid.position.x.round(), grid_width);
            let y = wrapped_index(boid.position.y.round(), grid_height);
            let glyphs = boid_glyph_pair(self.cell_style, boid.velocity);
            let origin_x = x * 2;
            for (dx, glyph) in glyphs.into_iter().enumerate() {
                frame.set(
                    origin_x + dx,
                    y,
                    ScreenCell {
                        glyph,
                        color_x: index,
                        color_y: 0,
                    },
                );
            }
        }
        frame.render_lines(self.context.resolved_width, |cell| {
            colorize_boid_cell(cell.color_x, cell.glyph)
        })
    }

    fn advance_frame(&mut self) {
        let grid_width = self.grid_width() as f64;
        let grid_height = self.grid_height() as f64;
        step_boids(&mut self.boids, grid_width, grid_height);
    }

    fn resize(&mut self, context: ScreenAnimationContext) {
        self.context = context;
        self.boids = seed_boids(context);
    }
}

fn seed_boids(context: ScreenAnimationContext) -> Vec<Boid> {
    let grid_width = (context.inner_width / 2).max(1);
    let grid_height = context.resolved_height.max(1);
    let area = grid_width.saturating_mul(grid_height);
    let count = (area / 90).clamp(8, 32);
    let mut seed = (grid_width as u64)
        .wrapping_mul(1_103_515_245)
        .wrapping_add((grid_height as u64).wrapping_mul(12_345))
        .wrapping_add(0x5EED);

    (0..count)
        .map(|index| {
            let px = unit_from_seed(&mut seed) * grid_width as f64;
            let py = unit_from_seed(&mut seed) * grid_height as f64;
            let angle = unit_from_seed(&mut seed) * std::f64::consts::TAU;
            let speed = 0.35 + (index % 5) as f64 * 0.04;
            Boid {
                position: Vec2::new(px, py),
                velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
            }
        })
        .collect()
}

fn unit_from_seed(seed: &mut u64) -> f64 {
    *seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    ((*seed >> 33) as f64) / ((1u64 << 31) as f64)
}

fn step_boids(boids: &mut [Boid], width: f64, height: f64) {
    let previous = boids.to_vec();
    let perception = 8.0;
    let separation_distance = 3.0;
    let max_speed = 0.85;

    for (index, boid) in boids.iter_mut().enumerate() {
        let mut separation = Vec2::zero();
        let mut alignment = Vec2::zero();
        let mut cohesion = Vec2::zero();
        let mut neighbors = 0.0;

        for (other_index, other) in previous.iter().enumerate() {
            if index == other_index {
                continue;
            }
            let offset = other.position.sub(previous[index].position);
            let distance = offset.length();
            if distance == 0.0 || distance > perception {
                continue;
            }

            if distance < separation_distance {
                separation = separation.sub(offset.scale(1.0 / distance.max(0.2)));
            }
            alignment = alignment.add(other.velocity);
            cohesion = cohesion.add(other.position);
            neighbors += 1.0;
        }

        let mut velocity = boid.velocity;
        if neighbors > 0.0 {
            alignment = alignment.scale(1.0 / neighbors).sub(boid.velocity);
            cohesion = cohesion.scale(1.0 / neighbors).sub(boid.position);
            velocity = velocity
                .add(separation.scale(0.10))
                .add(alignment.scale(0.05))
                .add(cohesion.scale(0.008));
        }

        boid.velocity = velocity.limit(max_speed);
        boid.position = wrap_position(boid.position.add(boid.velocity), width, height);
    }
}

fn wrap_position(position: Vec2, width: f64, height: f64) -> Vec2 {
    Vec2::new(position.x.rem_euclid(width), position.y.rem_euclid(height))
}

fn wrapped_index(value: f64, limit: usize) -> usize {
    value.rem_euclid(limit as f64) as usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoidDirection {
    East,
    West,
    North,
    South,
}

fn boid_direction(velocity: Vec2) -> BoidDirection {
    if velocity.x.abs() >= velocity.y.abs() {
        if velocity.x >= 0.0 {
            BoidDirection::East
        } else {
            BoidDirection::West
        }
    } else if velocity.y >= 0.0 {
        BoidDirection::South
    } else {
        BoidDirection::North
    }
}

fn boid_glyph_pair(cell_style: GameOfLifeCellStyle, velocity: Vec2) -> [char; 2] {
    match (cell_style, boid_direction(velocity)) {
        (GameOfLifeCellStyle::FullBlock, BoidDirection::East) => ['▐', '█'],
        (GameOfLifeCellStyle::FullBlock, BoidDirection::West) => ['█', '▌'],
        (GameOfLifeCellStyle::FullBlock, BoidDirection::North) => ['▀', '▀'],
        (GameOfLifeCellStyle::FullBlock, BoidDirection::South) => ['▄', '▄'],
        (GameOfLifeCellStyle::Dotted, BoidDirection::East) => ['⣶', '⣿'],
        (GameOfLifeCellStyle::Dotted, BoidDirection::West) => ['⣿', '⣶'],
        (GameOfLifeCellStyle::Dotted, BoidDirection::North) => ['⠛', '⠛'],
        (GameOfLifeCellStyle::Dotted, BoidDirection::South) => ['⣤', '⣤'],
    }
}

fn colorize_boid_cell(index: usize, glyph: char) -> String {
    let palette = [ANSI_CYAN, ANSI_BLUE, ANSI_PURPLE, ANSI_GREEN, ANSI_YELLOW];
    format!("{}{}{}", palette[index % palette.len()], glyph, ANSI_RESET)
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

    // Defends: boids use deterministic in-house state updates instead of host randomness.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn boids_animation_is_deterministic_and_advances() {
        let mut first = BoidsAnimation::new(context(80, 24), GameOfLifeCellStyle::FullBlock);
        let mut second = BoidsAnimation::new(context(80, 24), GameOfLifeCellStyle::FullBlock);
        assert_eq!(first, second);

        first.advance_frame();
        second.advance_frame();
        assert_eq!(first, second);
        assert_ne!(
            BoidsAnimation::new(context(80, 24), GameOfLifeCellStyle::FullBlock),
            first
        );
    }

    // Regression: boids must render as scaled 2-column cells, not tiny one-dot artifacts with skipped rows.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn boids_render_scaled_cells_without_inserted_rows() {
        let animation = BoidsAnimation::new(context(80, 24), GameOfLifeCellStyle::FullBlock);
        let visible = animation
            .render_frame()
            .into_iter()
            .map(|line| strip_ansi_codes(&line))
            .collect::<Vec<_>>();

        assert_eq!(visible.len(), 24);
        assert!(visible.iter().all(|line| line.chars().count() == 80));
        assert!(visible.iter().any(|line| {
            ["▐█", "█▌", "▀▀", "▄▄"]
                .into_iter()
                .any(|signature| line.contains(signature))
        }));
    }

    // Defends: boids resize through the same frame-producer contract future animations will use.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn boids_resize_preserves_frame_dimensions() {
        let mut animation = BoidsAnimation::new(context(80, 24), GameOfLifeCellStyle::Dotted);
        animation.resize(context(60, 12));
        let visible = animation
            .render_frame()
            .into_iter()
            .map(|line| strip_ansi_codes(&line))
            .collect::<Vec<_>>();

        assert_eq!(visible.len(), 12);
        assert!(visible.iter().all(|line| line.chars().count() == 60));
        assert!(visible.iter().any(|line| {
            ["⣶⣿", "⣿⣶", "⠛⠛", "⣤⣤"]
                .into_iter()
                .any(|signature| line.contains(signature))
        }));
    }

    // Regression: boid units must not collapse into identical pulsing blocks; shape encodes direction and color is stable per boid.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn boid_visual_identity_is_stable_and_directional() {
        assert_eq!(
            boid_glyph_pair(GameOfLifeCellStyle::FullBlock, Vec2::new(1.0, 0.1)),
            ['▐', '█']
        );
        assert_eq!(
            boid_glyph_pair(GameOfLifeCellStyle::FullBlock, Vec2::new(-1.0, 0.1)),
            ['█', '▌']
        );
        assert_eq!(
            boid_glyph_pair(GameOfLifeCellStyle::FullBlock, Vec2::new(0.1, -1.0)),
            ['▀', '▀']
        );
        assert_eq!(
            boid_glyph_pair(GameOfLifeCellStyle::FullBlock, Vec2::new(0.1, 1.0)),
            ['▄', '▄']
        );
        assert_eq!(colorize_boid_cell(3, '█'), colorize_boid_cell(3, '█'));
        assert_ne!(colorize_boid_cell(0, '█'), colorize_boid_cell(1, '█'));
    }
}
