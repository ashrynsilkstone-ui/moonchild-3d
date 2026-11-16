//! Mesh levels of detail via Bevy's `VisibilityRange` (dithered crossfade).
//!
//! Detailed, repeated props are generated twice: LOD0 with dense tessellation
//! for close-ups and LOD1 with far fewer segments. Both share the same origin;
//! between `switch` and `switch + FADE` metres they crossfade.
use std::ops::Range;
use bevy::camera::visibility::VisibilityRange;

pub const FADE: f32 = 1.5;
const FAR: Range<f32> = 1.0e5..1.0e5;

/// Switch distances in metres.
pub const SMALL: f32 = 7.0;   // bottles, lanterns
pub const MEDIUM: f32 = 11.0; // stools, tables, moving-head bodies
pub const LARGE: f32 = 16.0;  // lounge upholstery, truss

/// Range for the detailed level. `aabb` measures from the bounding-box centre
/// (needed for meshes built in world coordinates around the origin).
pub fn near(switch: f32, aabb: bool) -> VisibilityRange {
    VisibilityRange { start_margin: 0.0..0.0, end_margin: switch..switch + FADE, use_aabb: aabb }
}
/// Range for the reduced level.
pub fn far(switch: f32, aabb: bool) -> VisibilityRange {
    VisibilityRange { start_margin: switch..switch + FADE, end_margin: FAR, use_aabb: aabb }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lod_ranges_hand_over_without_gap() {
        for s in [SMALL, MEDIUM, LARGE] {
            let (a, b) = (near(s, false), far(s, false));
            assert_eq!(a.end_margin, b.start_margin);
            assert!(a.start_margin.end <= a.end_margin.start && b.start_margin.end <= b.end_margin.start);
        }
    }
}
