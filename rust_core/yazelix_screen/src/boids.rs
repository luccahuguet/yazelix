use crate::{
    GameOfLifeCellStyle, ScreenAnimationContext, ScreenCell, ScreenFrame, ScreenFrameProducer,
};

const ANSI_BLUE: &str = "\u{1b}[34m";
const ANSI_GREEN: &str = "\u{1b}[32m";
const ANSI_RED: &str = "\u{1b}[31m";
const ANSI_YELLOW: &str = "\u{1b}[33m";
const ANSI_PURPLE: &str = "\u{1b}[35m";
const ANSI_CYAN: &str = "\u{1b}[36m";
const ANSI_RESET: &str = "\u{1b}[0m";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoidsVariant {
    Predator,
    Schools,
}

impl BoidsVariant {
    pub fn from_style_name(style: &str) -> Option<Self> {
        match style {
            "boids" | "boids_predator" => Some(Self::Predator),
            "boids_schools" => Some(Self::Schools),
            _ => None,
        }
    }

    pub fn canonical_style_name(self) -> &'static str {
        match self {
            Self::Predator => "boids_predator",
            Self::Schools => "boids_schools",
        }
    }
}

pub fn is_boids_style(style: &str) -> bool {
    BoidsVariant::from_style_name(style).is_some()
}

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

    fn normalized(self) -> Self {
        let length = self.length();
        if length == 0.0 {
            Self::zero()
        } else {
            self.scale(1.0 / length)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoidRole {
    Flock,
    Predator,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Boid {
    position: Vec2,
    velocity: Vec2,
    species: usize,
    role: BoidRole,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoidsAnimation {
    context: ScreenAnimationContext,
    cell_style: GameOfLifeCellStyle,
    variant: BoidsVariant,
    boids: Vec<Boid>,
}

impl BoidsAnimation {
    pub fn new(context: ScreenAnimationContext, cell_style: GameOfLifeCellStyle) -> Self {
        Self::with_variant(context, cell_style, BoidsVariant::Predator)
    }

    pub fn with_variant(
        context: ScreenAnimationContext,
        cell_style: GameOfLifeCellStyle,
        variant: BoidsVariant,
    ) -> Self {
        let boids = seed_boids(context, variant);
        Self {
            context,
            cell_style,
            variant,
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
        for role_pass in [BoidRole::Flock, BoidRole::Predator] {
            for (index, boid) in self.boids.iter().enumerate() {
                if boid.role != role_pass {
                    continue;
                }
                let x = wrapped_index(boid.position.x.round(), grid_width);
                let y = wrapped_index(boid.position.y.round(), grid_height);
                let glyphs = boid_glyph_cells(self.cell_style, boid.role, boid.velocity);
                let color_x = boid_color_index(index, boid, self.variant);
                let origin_x = x * 2;
                for cell in glyphs {
                    frame.set(
                        origin_x + cell.dx,
                        y + cell.dy,
                        ScreenCell {
                            glyph: cell.glyph,
                            color_x,
                            color_y: 0,
                        },
                    );
                }
            }
        }
        frame.render_lines(self.context.resolved_width, |cell| {
            colorize_boid_cell(cell.color_x, cell.glyph)
        })
    }

    fn advance_frame(&mut self) {
        let grid_width = self.grid_width() as f64;
        let grid_height = self.grid_height() as f64;
        step_boids(&mut self.boids, grid_width, grid_height, self.variant);
    }

    fn resize(&mut self, context: ScreenAnimationContext) {
        self.context = context;
        self.boids = seed_boids(context, self.variant);
    }
}

fn seed_boids(context: ScreenAnimationContext, variant: BoidsVariant) -> Vec<Boid> {
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
            let role = if variant == BoidsVariant::Predator && index == 0 {
                BoidRole::Predator
            } else {
                BoidRole::Flock
            };
            let (px, py) = if role == BoidRole::Predator {
                (grid_width as f64 * 0.5, grid_height as f64 * 0.5)
            } else {
                (
                    unit_from_seed(&mut seed) * grid_width as f64,
                    unit_from_seed(&mut seed) * grid_height as f64,
                )
            };
            let angle = unit_from_seed(&mut seed) * std::f64::consts::TAU;
            let speed = if role == BoidRole::Predator {
                1.08
            } else {
                0.62 + (index % 5) as f64 * 0.07
            };
            Boid {
                position: Vec2::new(px, py),
                velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                species: match variant {
                    BoidsVariant::Schools => index % 3,
                    _ => 0,
                },
                role,
            }
        })
        .collect()
}

fn unit_from_seed(seed: &mut u64) -> f64 {
    *seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    ((*seed >> 33) as f64) / ((1u64 << 31) as f64)
}

fn step_boids(boids: &mut [Boid], width: f64, height: f64, variant: BoidsVariant) {
    let previous = boids.to_vec();

    for (index, boid) in boids.iter_mut().enumerate() {
        let mut velocity = match (variant, boid.role) {
            (BoidsVariant::Predator, BoidRole::Predator) => predator_velocity(index, &previous),
            _ => flock_velocity(index, &previous, variant),
        };
        let max_speed = match (variant, boid.role) {
            (BoidsVariant::Predator, BoidRole::Predator) => 1.70,
            (BoidsVariant::Schools, BoidRole::Flock) => 1.28,
            _ => 1.35,
        };
        velocity = velocity.limit(max_speed);
        boid.velocity = velocity;
        boid.position = wrap_position(boid.position.add(boid.velocity), width, height);
    }
}

fn flock_velocity(index: usize, previous: &[Boid], variant: BoidsVariant) -> Vec2 {
    let boid = previous[index];
    let perception = match variant {
        BoidsVariant::Schools => 10.0,
        _ => 8.0,
    };
    let separation_distance = 3.0;
    let mut predator_flee = Vec2::zero();
    let mut separation = Vec2::zero();
    let mut alignment = Vec2::zero();
    let mut cohesion = Vec2::zero();
    let mut neighbors = 0.0;

    for (other_index, other) in previous.iter().enumerate() {
        if index == other_index {
            continue;
        }
        let offset = other.position.sub(boid.position);
        let distance = offset.length();
        if distance == 0.0 || distance > perception {
            continue;
        }

        if variant == BoidsVariant::Predator && other.role == BoidRole::Predator {
            predator_flee = predator_flee.sub(offset.scale(1.4 / distance.max(0.4)));
            continue;
        }

        if variant == BoidsVariant::Schools
            && other.role == BoidRole::Flock
            && other.species != boid.species
        {
            separation = separation.sub(offset.scale(0.55 / distance.max(0.4)));
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
        let (separation_weight, alignment_weight, cohesion_weight) = match variant {
            BoidsVariant::Schools => (0.14, 0.07, 0.012),
            BoidsVariant::Predator => (0.12, 0.04, 0.006),
        };
        velocity = velocity
            .add(separation.scale(separation_weight))
            .add(alignment.scale(alignment_weight))
            .add(cohesion.scale(cohesion_weight));
    }

    velocity.add(predator_flee.scale(0.22))
}

fn predator_velocity(index: usize, previous: &[Boid]) -> Vec2 {
    let predator = previous[index];
    let mut nearest: Option<(f64, Vec2)> = None;
    for other in previous.iter().filter(|boid| boid.role == BoidRole::Flock) {
        let offset = other.position.sub(predator.position);
        let distance = offset.length();
        if distance == 0.0 {
            continue;
        }
        if nearest
            .as_ref()
            .map_or(true, |(nearest_distance, _)| distance < *nearest_distance)
        {
            nearest = Some((distance, offset));
        }
    }

    let Some((_, offset)) = nearest else {
        return predator.velocity;
    };
    let desired = offset.normalized().scale(1.70);
    predator
        .velocity
        .add(desired.sub(predator.velocity).limit(0.28))
}

fn boid_color_index(_index: usize, boid: &Boid, variant: BoidsVariant) -> usize {
    match (variant, boid.role) {
        (BoidsVariant::Predator, BoidRole::Predator) => 0,
        (BoidsVariant::Predator, BoidRole::Flock) => 1,
        (_, BoidRole::Predator) => 0,
        (BoidsVariant::Schools, BoidRole::Flock) => 2 + boid.species % 3,
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
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
    North,
    NorthEast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BoidGlyphCell {
    dx: usize,
    dy: usize,
    glyph: char,
}

const fn glyph_cell(dx: usize, dy: usize, glyph: char) -> BoidGlyphCell {
    BoidGlyphCell { dx, dy, glyph }
}

const PREDATOR_HORIZONTAL_BODY: char = '●';
const PREDATOR_VERTICAL_BODY: char = '⬤';

const PREY_EAST: [BoidGlyphCell; 1] = [glyph_cell(0, 0, '▶')];
const PREY_SOUTH_EAST: [BoidGlyphCell; 1] = [glyph_cell(0, 0, '▶')];
const PREY_SOUTH: [BoidGlyphCell; 1] = [glyph_cell(0, 0, '▼')];
const PREY_SOUTH_WEST: [BoidGlyphCell; 1] = [glyph_cell(0, 0, '◀')];
const PREY_WEST: [BoidGlyphCell; 1] = [glyph_cell(0, 0, '◀')];
const PREY_NORTH_WEST: [BoidGlyphCell; 1] = [glyph_cell(0, 0, '◀')];
const PREY_NORTH: [BoidGlyphCell; 1] = [glyph_cell(0, 0, '▲')];
const PREY_NORTH_EAST: [BoidGlyphCell; 1] = [glyph_cell(0, 0, '▶')];

const PREDATOR_EAST: [BoidGlyphCell; 2] = [
    glyph_cell(0, 0, PREDATOR_HORIZONTAL_BODY),
    glyph_cell(1, 0, '▶'),
];
const PREDATOR_SOUTH_EAST: [BoidGlyphCell; 2] = [
    glyph_cell(0, 0, PREDATOR_HORIZONTAL_BODY),
    glyph_cell(1, 0, '▶'),
];
const PREDATOR_SOUTH: [BoidGlyphCell; 2] = [
    glyph_cell(0, 0, PREDATOR_VERTICAL_BODY),
    glyph_cell(0, 1, '▼'),
];
const PREDATOR_SOUTH_WEST: [BoidGlyphCell; 2] = [
    glyph_cell(0, 0, '◀'),
    glyph_cell(1, 0, PREDATOR_HORIZONTAL_BODY),
];
const PREDATOR_WEST: [BoidGlyphCell; 2] = [
    glyph_cell(0, 0, '◀'),
    glyph_cell(1, 0, PREDATOR_HORIZONTAL_BODY),
];
const PREDATOR_NORTH_WEST: [BoidGlyphCell; 2] = [
    glyph_cell(0, 0, '◀'),
    glyph_cell(1, 0, PREDATOR_HORIZONTAL_BODY),
];
const PREDATOR_NORTH: [BoidGlyphCell; 2] = [
    glyph_cell(0, 0, '▲'),
    glyph_cell(0, 1, PREDATOR_VERTICAL_BODY),
];
const PREDATOR_NORTH_EAST: [BoidGlyphCell; 2] = [
    glyph_cell(0, 0, PREDATOR_HORIZONTAL_BODY),
    glyph_cell(1, 0, '▶'),
];

fn prey_glyph_cells(direction: BoidDirection) -> &'static [BoidGlyphCell] {
    match direction {
        BoidDirection::East => &PREY_EAST,
        BoidDirection::SouthEast => &PREY_SOUTH_EAST,
        BoidDirection::South => &PREY_SOUTH,
        BoidDirection::SouthWest => &PREY_SOUTH_WEST,
        BoidDirection::West => &PREY_WEST,
        BoidDirection::NorthWest => &PREY_NORTH_WEST,
        BoidDirection::North => &PREY_NORTH,
        BoidDirection::NorthEast => &PREY_NORTH_EAST,
    }
}

fn predator_glyph_cells(direction: BoidDirection) -> &'static [BoidGlyphCell] {
    match direction {
        BoidDirection::East => &PREDATOR_EAST,
        BoidDirection::SouthEast => &PREDATOR_SOUTH_EAST,
        BoidDirection::South => &PREDATOR_SOUTH,
        BoidDirection::SouthWest => &PREDATOR_SOUTH_WEST,
        BoidDirection::West => &PREDATOR_WEST,
        BoidDirection::NorthWest => &PREDATOR_NORTH_WEST,
        BoidDirection::North => &PREDATOR_NORTH,
        BoidDirection::NorthEast => &PREDATOR_NORTH_EAST,
    }
}

fn boid_direction(velocity: Vec2) -> BoidDirection {
    if velocity.length() == 0.0 {
        return BoidDirection::East;
    }

    let sector = (velocity.y.atan2(velocity.x) / std::f64::consts::FRAC_PI_4).round() as i32;
    match sector.rem_euclid(8) {
        0 => BoidDirection::East,
        1 => BoidDirection::SouthEast,
        2 => BoidDirection::South,
        3 => BoidDirection::SouthWest,
        4 => BoidDirection::West,
        5 => BoidDirection::NorthWest,
        6 => BoidDirection::North,
        _ => BoidDirection::NorthEast,
    }
}

fn boid_glyph_cells(
    _cell_style: GameOfLifeCellStyle,
    role: BoidRole,
    velocity: Vec2,
) -> &'static [BoidGlyphCell] {
    let direction = boid_direction(velocity);
    match role {
        BoidRole::Flock => prey_glyph_cells(direction),
        BoidRole::Predator => predator_glyph_cells(direction),
    }
}

fn colorize_boid_cell(index: usize, glyph: char) -> String {
    let palette = [
        ANSI_RED,
        ANSI_CYAN,
        ANSI_BLUE,
        ANSI_PURPLE,
        ANSI_GREEN,
        ANSI_YELLOW,
    ];
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

    fn glyph_text(role: BoidRole, velocity: Vec2) -> String {
        boid_glyph_cells(GameOfLifeCellStyle::FullBlock, role, velocity)
            .iter()
            .map(|cell| cell.glyph)
            .collect()
    }

    // Defends: public boids style names include the legacy alias and retained behavior-backed variants, with flow removed from the live surface.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn boids_style_names_resolve_to_variants() {
        assert_eq!(
            BoidsVariant::from_style_name("boids"),
            Some(BoidsVariant::Predator)
        );
        assert_eq!(
            BoidsVariant::from_style_name("boids_predator"),
            Some(BoidsVariant::Predator)
        );
        assert_eq!(
            BoidsVariant::from_style_name("boids_schools"),
            Some(BoidsVariant::Schools)
        );
        assert_eq!(BoidsVariant::from_style_name("boids_flow"), None);
        assert_eq!(BoidsVariant::from_style_name("game_of_life_bloom"), None);
    }

    // Defends: boids variants alter simulation behavior, not just labels or colors.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn boids_variants_apply_distinct_motion_rules() {
        let starting_boids = vec![
            Boid {
                position: Vec2::new(4.0, 5.0),
                velocity: Vec2::new(0.25, 0.0),
                species: 0,
                role: BoidRole::Predator,
            },
            Boid {
                position: Vec2::new(8.0, 5.0),
                velocity: Vec2::new(-0.1, 0.0),
                species: 0,
                role: BoidRole::Flock,
            },
            Boid {
                position: Vec2::new(9.0, 5.0),
                velocity: Vec2::new(-0.1, 0.1),
                species: 1,
                role: BoidRole::Flock,
            },
        ];
        let mut predator = starting_boids.clone();
        let mut schools = starting_boids.clone();

        step_boids(&mut predator, 40.0, 20.0, BoidsVariant::Predator);
        step_boids(&mut schools, 40.0, 20.0, BoidsVariant::Schools);

        assert!(predator[0].velocity.x > starting_boids[0].velocity.x);
        assert_ne!(schools[1].velocity, starting_boids[1].velocity);
        assert_ne!(predator, schools);
    }

    // Defends: boids colors map predator roles and school species to stable visual identities.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn boids_variant_colors_follow_roles_and_species() {
        let predator = Boid {
            position: Vec2::zero(),
            velocity: Vec2::zero(),
            species: 0,
            role: BoidRole::Predator,
        };
        let first_school = Boid {
            position: Vec2::zero(),
            velocity: Vec2::zero(),
            species: 0,
            role: BoidRole::Flock,
        };
        let second_school = Boid {
            species: 1,
            ..first_school
        };
        let other_prey = Boid {
            species: 3,
            ..first_school
        };

        assert_ne!(
            boid_color_index(0, &predator, BoidsVariant::Predator),
            boid_color_index(1, &first_school, BoidsVariant::Predator)
        );
        assert_eq!(
            boid_color_index(1, &first_school, BoidsVariant::Predator),
            boid_color_index(2, &other_prey, BoidsVariant::Predator)
        );
        assert_ne!(
            boid_color_index(1, &first_school, BoidsVariant::Schools),
            boid_color_index(2, &second_school, BoidsVariant::Schools)
        );
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

    // Regression: boids must render as cardinal filled directional triangles, not braille blobs, diagonal corner blocks, or plain cells with skipped rows.
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
        assert!(visible.iter().all(|line| {
            !["→", "←", "↑", "↓", "⠐", "⠶", "⠈", "⢆", "◤", "◥", "◢", "◣"]
                .into_iter()
                .any(|glyph| line.contains(glyph))
        }));
        assert!(visible.iter().any(|line| {
            ["▶", "◀", "▲", "▼"]
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
        assert!(visible.iter().all(|line| {
            !["→", "←", "↑", "↓", "⠐", "⠶", "⠈", "⢆", "◤", "◥", "◢", "◣"]
                .into_iter()
                .any(|glyph| line.contains(glyph))
        }));
        assert!(visible.iter().any(|line| {
            ["▶", "◀", "▲", "▼"]
                .into_iter()
                .any(|signature| line.contains(signature))
        }));
    }

    // Regression: boid units must not collapse into identical pulsing blocks; diagonal movement collapses to readable cardinal sprites.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn boid_visual_identity_is_stable_and_directional() {
        assert_eq!(glyph_text(BoidRole::Flock, Vec2::new(1.0, 0.1)), "▶");
        assert_eq!(glyph_text(BoidRole::Flock, Vec2::new(1.0, 1.0)), "▶");
        assert_eq!(glyph_text(BoidRole::Flock, Vec2::new(0.1, 1.0)), "▼");
        assert_eq!(glyph_text(BoidRole::Flock, Vec2::new(-1.0, 1.0)), "◀");
        assert_eq!(glyph_text(BoidRole::Flock, Vec2::new(-1.0, 0.1)), "◀");
        assert_eq!(glyph_text(BoidRole::Flock, Vec2::new(-1.0, -1.0)), "◀");
        assert_eq!(glyph_text(BoidRole::Flock, Vec2::new(0.1, -1.0)), "▲");
        assert_eq!(glyph_text(BoidRole::Flock, Vec2::new(1.0, -1.0)), "▶");
        assert_eq!(colorize_boid_cell(3, '▶'), colorize_boid_cell(3, '▶'));
        assert_ne!(colorize_boid_cell(0, '▶'), colorize_boid_cell(1, '▶'));
    }

    // Defends: predator sprites use small horizontal bodies and larger vertical bodies while preserving cardinal pointing.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn predator_sprite_is_larger_than_prey_sprite() {
        let prey = boid_glyph_cells(
            GameOfLifeCellStyle::FullBlock,
            BoidRole::Flock,
            Vec2::new(1.0, -1.0),
        );
        let predator = boid_glyph_cells(
            GameOfLifeCellStyle::FullBlock,
            BoidRole::Predator,
            Vec2::new(1.0, -1.0),
        );

        assert!(predator.len() > prey.len());
        assert!(
            predator
                .iter()
                .any(|cell| cell.glyph == PREDATOR_HORIZONTAL_BODY)
        );
        assert_eq!(glyph_text(BoidRole::Predator, Vec2::new(1.0, -1.0)), "●▶");
        assert_eq!(glyph_text(BoidRole::Predator, Vec2::new(-1.0, 0.1)), "◀●");
        assert_eq!(glyph_text(BoidRole::Predator, Vec2::new(0.1, -1.0)), "▲⬤");
        assert_eq!(glyph_text(BoidRole::Predator, Vec2::new(0.1, 1.0)), "⬤▼");
        assert_ne!(PREDATOR_HORIZONTAL_BODY, PREDATOR_VERTICAL_BODY);
    }

    // Defends: the faster boids tuning moves creatures far enough per frame to read as intentional animation.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn boids_motion_uses_faster_velocity_floor() {
        let animation = BoidsAnimation::with_variant(
            context(80, 24),
            GameOfLifeCellStyle::FullBlock,
            BoidsVariant::Predator,
        );
        let slowest_prey = animation
            .boids
            .iter()
            .filter(|boid| boid.role == BoidRole::Flock)
            .map(|boid| boid.velocity.length())
            .fold(f64::INFINITY, f64::min);

        assert!(slowest_prey >= 0.62);
    }
}
