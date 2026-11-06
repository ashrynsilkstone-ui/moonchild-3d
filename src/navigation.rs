//! Small, deliberately scoped walking controller for the first club section.
use bevy::prelude::*;

pub const EYE: f32 = 1.78;
const FLOOR: f32 = 3.0;
const X_MIN: f32 = -9.8;
const X_MAX: f32 = -8.2;
const Z_START: f32 = 1.2;
const Z_END: f32 = 7.8;
const STEPS: usize = 20;

fn gallery_front(x: f32) -> f32 { 5.4 + 0.025 * x * x }
fn on_stairs(x: f32, z: f32) -> bool { (X_MIN..=X_MAX).contains(&x) && (Z_START..=Z_END).contains(&z) }
fn on_gallery(x: f32, z: f32) -> bool { x.abs() <= 10.25 && z >= gallery_front(x) && z <= 9.38 }

fn floor_at(x: f32, z: f32, old: f32) -> f32 {
    if on_stairs(x, z) {
        let t = ((z - Z_START) / (Z_END - Z_START)).clamp(0.0, 1.0);
        (((t * STEPS as f32).floor() + 1.0).min(STEPS as f32) / STEPS as f32) * FLOOR
    } else if on_gallery(x, z) && old > FLOOR - 0.3 { FLOOR } else { 0.0 }
}

fn blocked(x: f32, z: f32, floor: f32) -> bool {
    // Existing stage and outer walls; below-gallery bar, upper-floor sofas.
    if !(-11.4..=11.4).contains(&x) || !(-5.0..=9.4).contains(&z) { return true; }
    if floor < 1.0 && (6.0..=10.7).contains(&x) && (0.2..=4.8).contains(&z) { return true; }
    if floor > 2.7 && (x.abs() - 4.2).abs() < 1.45 && (7.9..=9.3).contains(&z) { return true; }
    false
}

/// Sweep in small increments, rejecting jumps and drops; no physics engine implied.
pub fn walk_position(current: Vec3, movement: Vec3) -> Vec3 {
    let mut pos = current;
    let count = (movement.length() / 0.06).ceil().max(1.0) as usize;
    let step = movement / count as f32;
    for _ in 0..count {
        // Axis separation makes walls slide instead of stopping diagonally.
        for delta in [Vec3::new(step.x,0.0,0.0), Vec3::new(0.0,0.0,step.z)] {
            let next = pos + delta;
            let old_floor = pos.y - EYE;
            let next_floor = floor_at(next.x, next.z, old_floor);
            if (next_floor - old_floor).abs() <= 0.23 && !blocked(next.x, next.z, next_floor) {
                pos = Vec3::new(next.x, next_floor + EYE, next.z);
            }
        }
    }
    pos
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stair_reaches_gallery_and_descends() {
        let mut p = Vec3::new(-9.0, EYE, 1.0);
        for _ in 0..150 { p = walk_position(p, Vec3::new(0.0,0.0,0.05)); }
        assert!((p.y - (EYE + FLOOR)).abs() < 0.01, "{p:?}");
        for _ in 0..150 { p = walk_position(p, Vec3::new(0.0,0.0,-0.05)); }
        assert!((p.y - EYE).abs() < 0.01, "{p:?}");
    }
    #[test]
    fn upper_floor_edge_blocks_fall() {
        let p = Vec3::new(0.0,EYE+FLOOR,5.5);
        assert!(walk_position(p,Vec3::new(0.0,0.0,-2.0)).z >= 5.4);
    }
    #[test]
    fn bar_and_stage_are_not_walkthrough() {
        let p = walk_position(Vec3::new(5.5,EYE,2.0),Vec3::new(3.0,0.0,0.0));
        assert!(p.x <= 6.0);
        let p = walk_position(Vec3::new(0.0,EYE,-4.8),Vec3::new(0.0,0.0,-2.0));
        assert!(p.z >= -5.0);
    }
}
