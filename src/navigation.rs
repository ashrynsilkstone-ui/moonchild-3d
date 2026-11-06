//! Multi-level first-person movement with simple volume collision.
//!
//! Every solid part of the club is a [`Volume`]: a 2D footprint with a bottom
//! and a (possibly stepped) top height. The same list serves three purposes:
//! * walkable floors: the highest top within reach of a step,
//! * walls/furniture: volumes that overlap the player's body,
//! * ceilings: volumes above the head (gallery underside).
//!
//! Dimensions are shared with the geometry through [`layout`].
use std::sync::LazyLock;
use bevy::prelude::*;

/// Shared club layout, in metres. Geometry and collision use the same values.
pub mod layout {
    pub const ROOM_X: f32 = 11.85;
    pub const ROOM_Z: f32 = 9.85;
    pub const CEILING: f32 = 7.0;
    pub const GALLERY_Y: f32 = 3.0;
    pub const GALLERY_UNDERSIDE: f32 = 2.66;
    /// Front edge of the curved gallery: a 20 m radius arc wrapping the dance floor.
    pub fn gallery_front(x: f32) -> f32 { 25.4 - (400.0 - x * x).max(0.0).sqrt() }
    pub const STAIR_X: (f32, f32) = (-9.85, -8.15);
    pub const STAIR_Z: (f32, f32) = (0.4, 7.0);
    pub const STAIR_STEPS: u32 = 20;
    pub const STAGE_X: f32 = 7.0;
    pub const STAGE_Z: (f32, f32) = (-9.75, -5.35);
    pub const STAGE_Y: f32 = 0.96;
    /// Side steps onto the stage, rising towards -x.
    pub const STAGE_STEPS_X: (f32, f32) = (7.0, 8.25);
    pub const STAGE_STEPS_Z: (f32, f32) = (-7.7, -6.3);
    pub const STAGE_STEP_COUNT: u32 = 5;
    pub const DANCE_CENTER: (f32, f32) = (0.0, 0.2);
    pub const DANCE_RADIUS: f32 = 5.0;
    pub const DANCE_TOP: f32 = 0.025;
    pub const BAR_CENTER: (f32, f32) = (8.35, 2.5);
    pub const BAR_RADIUS: f32 = 1.85;
    pub const BAR_TOP: f32 = 1.12;
    pub const BAR_WING_END: f32 = 11.6;
    pub const RING_CENTER: (f32, f32, f32) = (0.0, 6.0, 0.2);
    pub const RING_RADIUS: f32 = 4.2;
    pub const GALLERY_COLUMNS: [f32; 4] = [-6.4, -2.2, 2.2, 6.4];
    pub fn column_z(x: f32) -> f32 { gallery_front(x) + 0.25 }
    /// C-shaped lounge sofas: (centre x, floor y, centre z).
    pub const LOUNGES: [(f32, f32, f32); 4] = [(-4.8, GALLERY_Y, 8.15), (4.8, GALLERY_Y, 8.15), (-6.0, 0.0, 8.15), (6.0, 0.0, 8.15)];
    pub const LOUNGE_RADIUS: f32 = 1.2;
    pub const LOUNGE_ARC: (f32, f32) = (-0.35, std::f32::consts::PI + 0.35);
    pub const ROOM_COLUMNS: [(f32, f32); 6] = [(-10.6, -7.8), (-10.6, 0.0), (-10.6, 9.4), (10.6, -7.8), (10.6, 0.0), (10.6, 9.4)];
    /// The DJ booth follows an arc so that it curves towards the dance floor.
    pub const BOOTH_CENTER_Z: f32 = -11.3;
    pub const BOOTH_RADIUS: f32 = 3.5;
    pub const BOOTH_HALF_ANGLE: f32 = 0.575;
    pub const BOOTH_TOP: f32 = 2.08;
    pub const SPEAKERS_X: [f32; 4] = [-6.2, -5.25, 5.25, 6.2];
    pub const BAR_STOOL_ANGLES: [f32; 6] = [1.95, 2.35, 2.75, 3.55, 3.95, 4.35];
}
use layout::*;

pub const EYE: f32 = 1.78;
const HEAD: f32 = EYE + 0.12;
const GRAVITY: f32 = 16.0;
const JUMP_SPEED: f32 = 6.5;
const RADIUS: f32 = 0.25;
/// Feet within this distance of an edge still stand on it.
const SUPPORT_RADIUS: f32 = 0.12;
const MAX_STEP: f32 = 0.26;

#[derive(Clone, Copy)]
enum Shape {
    Rect { min: Vec2, max: Vec2 },
    Circle { c: Vec2, r: f32 },
    Capsule { a: Vec2, b: Vec2, r: f32 },
    Gallery,
}

#[derive(Clone, Copy)]
enum Top {
    Flat(f32),
    /// Steps along x (`along_x`) or z, from `start` (first step) to `end` (top).
    Steps { along_x: bool, start: f32, end: f32, count: u32, height: f32 },
}

#[derive(Clone, Copy)]
struct Volume { shape: Shape, bottom: f32, top: Top }

impl Shape {
    /// Approximate signed distance in the ground plane (negative inside).
    fn distance(&self, p: Vec2) -> f32 {
        match *self {
            Shape::Rect { min, max } => {
                let c = (min + max) * 0.5; let h = (max - min) * 0.5;
                let q = (p - c).abs() - h;
                q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0)
            }
            Shape::Circle { c, r } => p.distance(c) - r,
            Shape::Capsule { a, b, r } => {
                let ab = b - a;
                let t = ((p - a).dot(ab) / ab.length_squared().max(1e-6)).clamp(0.0, 1.0);
                p.distance(a + ab * t) - r
            }
            Shape::Gallery => {
                let front = gallery_front(p.x);
                // Vertical offset to the curve, corrected by its slope.
                let slope = p.x / (400.0 - p.x * p.x).max(1.0).sqrt();
                let d_front = (front - p.y) / (1.0 + slope * slope).sqrt();
                d_front.max(p.y - ROOM_Z).max(p.x.abs() - ROOM_X)
            }
        }
    }
}

impl Top {
    fn at(&self, p: Vec2) -> f32 {
        match *self {
            Top::Flat(h) => h,
            Top::Steps { along_x, start, end, count, height } => {
                let c = if along_x { p.x } else { p.y };
                let t = ((c - start) / (end - start)).clamp(0.0, 1.0);
                (((t * count as f32).floor() + 1.0).min(count as f32) / count as f32) * height
            }
        }
    }
}

fn rect(x0: f32, z0: f32, x1: f32, z1: f32) -> Shape { Shape::Rect { min: Vec2::new(x0.min(x1), z0.min(z1)), max: Vec2::new(x0.max(x1), z0.max(z1)) } }
fn solid(shape: Shape, bottom: f32, top: f32) -> Volume { Volume { shape, bottom, top: Top::Flat(top) } }

/// Polyline of capsules, for rails and curved furniture.
fn capsules(points: &[Vec2], r: f32, bottom: f32, top: f32, out: &mut Vec<Volume>) {
    for w in points.windows(2) { out.push(solid(Shape::Capsule { a: w[0], b: w[1], r }, bottom, top)); }
}
fn front_points(x0: f32, x1: f32) -> Vec<Vec2> {
    let n = ((x1 - x0) / 0.4).ceil().max(1.0) as usize;
    (0..=n).map(|i| { let x = x0 + (x1 - x0) * i as f32 / n as f32; Vec2::new(x, gallery_front(x)) }).collect()
}
fn arc_points(c: Vec2, r: f32, a0: f32, a1: f32, n: usize) -> Vec<Vec2> {
    (0..=n).map(|i| { let a = a0 + (a1 - a0) * i as f32 / n as f32; c + Vec2::new(a.cos(), a.sin()) * r }).collect()
}

static VOLUMES: LazyLock<Vec<Volume>> = LazyLock::new(|| {
    let mut v = vec![];
    // Room walls.
    v.push(solid(rect(-30.0, -30.0, -ROOM_X, 30.0), -10.0, 20.0));
    v.push(solid(rect(ROOM_X, -30.0, 30.0, 30.0), -10.0, 20.0));
    v.push(solid(rect(-30.0, -30.0, 30.0, -ROOM_Z), -10.0, 20.0));
    v.push(solid(rect(-30.0, ROOM_Z, 30.0, 30.0), -10.0, 20.0));
    for &(x, z) in &ROOM_COLUMNS { v.push(solid(rect(x - 0.21, z - 0.21, x + 0.21, z + 0.21), -1.0, CEILING)); }
    // Dance floor disc (a small step), stage and stage steps.
    v.push(solid(Shape::Circle { c: Vec2::new(DANCE_CENTER.0, DANCE_CENTER.1), r: DANCE_RADIUS + 0.06 }, 0.0, DANCE_TOP));
    v.push(solid(rect(-STAGE_X, STAGE_Z.0, STAGE_X, STAGE_Z.1), 0.0, STAGE_Y));
    v.push(Volume { shape: rect(STAGE_STEPS_X.0, STAGE_STEPS_Z.0, STAGE_STEPS_X.1, STAGE_STEPS_Z.1), bottom: 0.0,
        top: Top::Steps { along_x: true, start: STAGE_STEPS_X.1, end: STAGE_STEPS_X.0, count: STAGE_STEP_COUNT, height: STAGE_Y } });
    // DJ booth on its arc, and the speaker stacks.
    let booth: Vec<Vec2> = arc_points(Vec2::new(0.0, BOOTH_CENTER_Z), BOOTH_RADIUS - 0.05,
        std::f32::consts::FRAC_PI_2 - BOOTH_HALF_ANGLE, std::f32::consts::FRAC_PI_2 + BOOTH_HALF_ANGLE, 10);
    capsules(&booth, 0.42, STAGE_Y, BOOTH_TOP + 0.05, &mut v);
    for &x in &SPEAKERS_X { v.push(solid(rect(x - 0.42, -8.9, x + 0.42, -8.1), STAGE_Y, 3.4)); }
    // Gallery slab, stair landing and the stairs themselves.
    v.push(solid(Shape::Gallery, GALLERY_UNDERSIDE, GALLERY_Y));
    v.push(solid(rect(STAIR_X.0, STAIR_Z.1 - 0.05, STAIR_X.1, 8.1), GALLERY_UNDERSIDE, GALLERY_Y));
    v.push(Volume { shape: rect(STAIR_X.0, STAIR_Z.0, STAIR_X.1, STAIR_Z.1), bottom: 0.0,
        top: Top::Steps { along_x: false, start: STAIR_Z.0, end: STAIR_Z.1, count: STAIR_STEPS, height: GALLERY_Y } });
    // Gallery balustrade: an invisible extra height stops jumping over the rail.
    capsules(&front_points(-ROOM_X, STAIR_X.0), 0.04, GALLERY_Y, GALLERY_Y + 2.5, &mut v);
    capsules(&front_points(STAIR_X.1, ROOM_X), 0.04, GALLERY_Y, GALLERY_Y + 2.5, &mut v);
    for &x in &[STAIR_X.0, STAIR_X.1] {
        capsules(&[Vec2::new(x, STAIR_Z.0), Vec2::new(x, gallery_front(x))], 0.04, 0.0, GALLERY_Y + 2.5, &mut v);
    }
    for &x in &GALLERY_COLUMNS { v.push(solid(Shape::Circle { c: Vec2::new(x, column_z(x)), r: 0.16 }, 0.0, GALLERY_UNDERSIDE)); }
    // Semicircular bar with its two straight wings; the service area behind it is closed.
    let bc = Vec2::new(BAR_CENTER.0, BAR_CENTER.1);
    v.push(solid(Shape::Circle { c: bc, r: BAR_RADIUS + 0.12 }, 0.0, BAR_TOP));
    v.push(solid(rect(bc.x, bc.y - BAR_RADIUS - 0.12, ROOM_X, bc.y + BAR_RADIUS + 0.12), 0.0, BAR_TOP));
    for &a in &BAR_STOOL_ANGLES {
        let p = bc + Vec2::new(a.cos(), a.sin()) * (BAR_RADIUS + 0.48);
        v.push(solid(Shape::Circle { c: p, r: 0.2 }, 0.0, 0.78));
    }
    // Lounges: the C-shaped sofa and a round table.
    for &(x, y, z) in &LOUNGES {
        let pts = arc_points(Vec2::new(x, z), LOUNGE_RADIUS + 0.1, LOUNGE_ARC.0, LOUNGE_ARC.1, 12);
        capsules(&pts, 0.42, y, y + 0.95, &mut v);
        v.push(solid(Shape::Circle { c: Vec2::new(x, z), r: 0.42 }, y, y + 0.72));
    }
    v
});

fn body_blocked(p: Vec2, feet: f32, from: Vec2) -> bool {
    VOLUMES.iter().any(|v| {
        let d = v.shape.distance(p);
        if d >= RADIUS { return false; }
        let top = v.top.at(p);
        let overlaps = v.bottom < feet + HEAD && top > feet + MAX_STEP;
        // Moving out of a volume (e.g. after stepping off a platform edge) is always allowed.
        overlaps && d < v.shape.distance(from) - 1e-5
    })
}

/// Highest floor under the feet that can be reached from `feet` by stepping.
fn support(p: Vec2, feet: f32) -> f32 {
    VOLUMES.iter().filter(|v| v.shape.distance(p) < SUPPORT_RADIUS)
        .map(|v| v.top.at(p)).filter(|&t| t <= feet + MAX_STEP + 1e-4).fold(0.0, f32::max)
}

/// Lowest underside above the head.
fn ceiling(p: Vec2, feet: f32) -> f32 {
    VOLUMES.iter().filter(|v| v.shape.distance(p) < RADIUS && v.bottom > feet + MAX_STEP)
        .map(|v| v.bottom).fold(CEILING, f32::min)
}

#[derive(Default)]
pub struct Motion { vertical_speed: f32, grounded: bool }
impl Motion {
    pub fn standing() -> Self { Self { vertical_speed: 0.0, grounded: true } }
}

/// Advances in small sub-steps to prevent tunnelling on slow frames.
/// `position` is the eye position; collision uses the feet height.
pub fn advance(position: Vec3, displacement: Vec3, jump: bool, dt: f32, motion: &mut Motion) -> Vec3 {
    let dt = dt.clamp(0.0, 0.1);
    if jump && motion.grounded { motion.vertical_speed = JUMP_SPEED; motion.grounded = false; }
    let count = ((dt / (1.0 / 120.0)).ceil() as usize).max((displacement.length() / 0.04).ceil() as usize).max(1);
    let delta = displacement / count as f32;
    let subdt = dt / count as f32;
    let mut p = position;
    for _ in 0..count {
        let mut feet = p.y - EYE;
        for d in [Vec2::new(delta.x, 0.0), Vec2::new(0.0, delta.z)] {
            let from = Vec2::new(p.x, p.z);
            let next = from + d;
            if d == Vec2::ZERO || body_blocked(next, feet, from) { continue; }
            let floor = support(next, feet);
            if motion.grounded {
                if floor < feet - MAX_STEP { motion.grounded = false; } else { feet = floor; }
            }
            p.x = next.x; p.z = next.y;
        }
        let here = Vec2::new(p.x, p.z);
        if motion.grounded {
            // Keep standing on whatever is underneath (also handles a removed support).
            let floor = support(here, feet);
            if floor < feet - MAX_STEP { motion.grounded = false; } else { feet = floor; }
        }
        if !motion.grounded {
            let before = feet;
            motion.vertical_speed -= GRAVITY * subdt;
            feet += motion.vertical_speed * subdt;
            let max_feet = ceiling(here, before) - HEAD;
            if feet > max_feet { feet = max_feet.max(before); motion.vertical_speed = motion.vertical_speed.min(0.0); }
            let floor = support(here, before.max(feet));
            if motion.vertical_speed <= 0.0 && feet <= floor { feet = floor; motion.vertical_speed = 0.0; motion.grounded = true; }
        }
        p.y = feet + EYE;
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(p: &mut Vec3, m: &mut Motion, d: Vec3, jump: bool) { *p = advance(*p, d, jump, 1.0 / 60.0, m); }
    fn walk(p: &mut Vec3, m: &mut Motion, d: Vec3, frames: usize) { for _ in 0..frames { run(p, m, d, false); } }
    fn feet(p: Vec3) -> f32 { p.y - EYE }

    #[test]
    fn stage_front_blocks_walking_but_accepts_jump_and_landing() {
        let mut m = Motion::standing(); let mut p = Vec3::new(0.0, EYE, -4.6);
        walk(&mut p, &mut m, Vec3::new(0.0, 0.0, -0.04), 50);
        assert!(p.z >= STAGE_Z.1 + RADIUS - 0.02, "{p:?}");
        for i in 0..90 { run(&mut p, &mut m, Vec3::new(0.0, 0.0, if i < 30 { -0.04 } else { 0.0 }), i == 0); }
        assert!(p.z < STAGE_Z.1 && (feet(p) - STAGE_Y).abs() < 0.01 && m.grounded, "{p:?}");
    }
    #[test]
    fn walk_off_stage_lands_on_floor() {
        let mut m = Motion::standing(); let mut p = Vec3::new(-2.5, EYE + STAGE_Y, -5.7);
        walk(&mut p, &mut m, Vec3::new(0.0, 0.0, 0.025), 90);
        assert!(p.z > -5.0 && feet(p).abs() < 0.03 && m.grounded, "{p:?}");
    }
    #[test]
    fn stage_side_steps_are_walkable() {
        let mut m = Motion::standing(); let mut p = Vec3::new(9.2, EYE, -7.0);
        walk(&mut p, &mut m, Vec3::new(-0.04, 0.0, 0.0), 80);
        assert!(p.x < STAGE_X && (feet(p) - STAGE_Y).abs() < 0.01, "{p:?}");
    }
    #[test]
    fn stairs_reach_gallery_and_descend() {
        let mut m = Motion::standing(); let mut p = Vec3::new(-9.0, EYE, -0.2);
        walk(&mut p, &mut m, Vec3::new(0.0, 0.0, 0.05), 160);
        assert!((feet(p) - GALLERY_Y).abs() < 0.01 && p.z > STAIR_Z.1, "{p:?}");
        walk(&mut p, &mut m, Vec3::new(0.0, 0.0, -0.05), 170);
        assert!(feet(p).abs() < 0.03 && m.grounded, "{p:?}");
    }
    #[test]
    fn gallery_rail_holds_and_lounge_is_reachable() {
        let mut m = Motion::standing(); let mut p = Vec3::new(0.0, EYE + GALLERY_Y, 6.8);
        for i in 0..90 { run(&mut p, &mut m, Vec3::new(0.0, 0.0, -0.05), i == 10); }
        assert!(p.z > gallery_front(0.0) && (feet(p) - GALLERY_Y).abs() < 0.01, "{p:?}");
        // Walk sideways along the gallery towards a lounge.
        walk(&mut p, &mut m, Vec3::new(0.05, 0.0, 0.04), 60);
        assert!((feet(p) - GALLERY_Y).abs() < 0.01 && p.x > 2.0, "{p:?}");
    }
    #[test]
    fn under_gallery_is_walkable_and_jump_hits_slab() {
        let mut m = Motion::standing(); let mut p = Vec3::new(0.0, EYE, 4.0);
        walk(&mut p, &mut m, Vec3::new(0.0, 0.0, 0.05), 90);
        assert!(p.z > 8.5 && feet(p).abs() < 0.01, "{p:?}");
        let mut max = p.y;
        for i in 0..100 { run(&mut p, &mut m, Vec3::ZERO, i == 0); max = max.max(p.y); }
        assert!(max <= GALLERY_UNDERSIDE - 0.1 && feet(p).abs() < 0.01, "{max} {p:?}");
    }
    #[test]
    fn bar_counter_blocks() {
        let mut m = Motion::standing(); let mut p = Vec3::new(5.0, EYE, 2.5);
        walk(&mut p, &mut m, Vec3::new(0.04, 0.0, 0.0), 80);
        assert!(p.x < BAR_CENTER.0 - BAR_RADIUS, "{p:?}");
    }
    #[test]
    fn dance_floor_edge_is_a_small_step() {
        let mut m = Motion::standing(); let mut p = Vec3::new(0.0, EYE, 6.0);
        walk(&mut p, &mut m, Vec3::new(0.0, 0.0, -0.04), 40);
        assert!((feet(p) - DANCE_TOP).abs() < 0.001, "{p:?}");
    }
    #[test]
    fn no_double_jump() {
        let mut m = Motion::standing(); let mut p = Vec3::new(0.0, EYE + DANCE_TOP, 0.0); let mut max = p.y;
        for i in 0..120 { run(&mut p, &mut m, Vec3::ZERO, i == 0 || i == 15); max = max.max(p.y); }
        assert!(max - EYE < 1.4 && m.grounded, "{max} {p:?}");
    }
}
