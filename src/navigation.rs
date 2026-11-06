//! Simplified first-person collision, platform movement and jumping.
use bevy::prelude::*;

pub const EYE: f32 = 1.78;
const GALLERY: f32 = 3.0;
const STAGE: f32 = 0.96;
const GRAVITY: f32 = 16.0;
const JUMP_SPEED: f32 = 6.5;
const PLAYER_RADIUS: f32 = 0.22;
const MAX_STEP: f32 = 0.23;

#[derive(Default)]
pub struct Motion {
    vertical_speed: f32,
    grounded: bool,
}
impl Motion {
    pub fn standing() -> Self { Self { vertical_speed: 0.0, grounded: true } }
}

fn gallery_front(x: f32) -> f32 { 5.4 + 0.025 * x * x }
fn on_stairs(x: f32, z: f32) -> bool { (-9.8..=-8.2).contains(&x) && (1.2..=7.8).contains(&z) }
fn on_gallery(x: f32, z: f32) -> bool { x.abs() <= 10.25 && z >= gallery_front(x) && z <= 9.38 }
fn on_stage(x: f32, z: f32) -> bool { x.abs() <= 7.0 && (-9.75..=-5.35).contains(&z) }
fn near_stage(x: f32, z: f32) -> bool {
    x.abs() <= 7.0 + PLAYER_RADIUS && (-9.75 - PLAYER_RADIUS..=-5.35 + PLAYER_RADIUS).contains(&z)
}
fn base_height(x: f32, z: f32, previous: f32) -> f32 {
    if on_stairs(x,z) {
        let t=((z-1.2)/6.6).clamp(0.0,1.0);
        (((t*20.0).floor()+1.0).min(20.0)/20.0)*GALLERY
    } else if on_gallery(x,z) && previous >= GALLERY - MAX_STEP { GALLERY }
    else if on_stage(x,z) && previous >= STAGE - 0.02 { STAGE }
    else { 0.0 }
}
fn stage_distance(x: f32, z: f32) -> f32 {
    let dx=(x.abs()-7.0).max(0.0);
    let dz=if z > -5.35 { z+5.35 } else if z < -9.75 { -9.75-z } else { 0.0 };
    dx*dx+dz*dz
}
fn obstacle(x:f32, z:f32, feet:f32) -> bool {
    if !(-11.4..=11.4).contains(&x) || !(-9.4..=9.4).contains(&z) { return true; }
    if feet < 1.0 && (6.0..=10.7).contains(&x) && (0.2..=4.8).contains(&z) { return true; }
    if feet > 2.7 && (x.abs()-4.2).abs() < 1.45 && (7.9..=9.3).contains(&z) { return true; }
    // A conservative boundary around the DJ console and speaker stacks on stage.
    if feet >= STAGE - 0.02 && feet < 2.4 && (-8.75..=-7.10).contains(&z) && x.abs() < 3.65 { return true; }
    if feet >= STAGE - 0.02 && feet < 3.0 && (-9.15..=-7.86).contains(&z) && (4.55..=6.9).contains(&x.abs()) { return true; }
    false
}

/// Advances in fixed small increments to prevent tunnelling on slow frames.
/// The eye position is stored in `position`; all collision calculations use feet height.
pub fn advance(position: Vec3, displacement: Vec3, jump: bool, dt: f32, motion: &mut Motion) -> Vec3 {
    let dt=dt.clamp(0.0,0.1);
    if jump && motion.grounded { motion.vertical_speed=JUMP_SPEED; motion.grounded=false; }
    let count=((dt / (1.0/120.0)).ceil() as usize).max((displacement.length()/0.04).ceil() as usize).max(1);
    let delta=displacement/count as f32;
    let subdt=dt/count as f32;
    let mut p=position;
    for _ in 0..count {
        let mut feet=p.y-EYE;
        for d in [Vec3::new(delta.x,0.0,0.0),Vec3::new(0.0,0.0,delta.z)] {
            let next=p+d;
            if obstacle(next.x,next.z,feet) { continue; }
            // Once stepping off a platform, allow outward travel while falling
            // inside its expanded collision boundary; never allow entry through it.
            if near_stage(next.x,next.z) && feet < STAGE-0.001 {
                let escaping=near_stage(p.x,p.z) && stage_distance(next.x,next.z)>stage_distance(p.x,p.z);
                if !escaping { continue; }
            }
            let next_floor=base_height(next.x,next.z,feet);
            if motion.grounded {
                let difference=next_floor-feet;
                if difference > MAX_STEP { continue; }
                if difference < -MAX_STEP {
                    // Gallery edges have a rail; the stage can be stepped off.
                    if feet > 2.7 && on_gallery(p.x,p.z) { continue; }
                    motion.grounded=false;
                } else { feet=next_floor; }
            } else if next_floor > feet+0.001 { continue; }
            p.x=next.x; p.z=next.z; p.y=feet+EYE;
        }
        if !motion.grounded {
            let before=feet;
            motion.vertical_speed -= GRAVITY*subdt;
            feet += motion.vertical_speed*subdt;
            // Do not jump into the underside of the gallery or beyond the ceiling.
            let ceiling = if on_gallery(p.x,p.z) && before < GALLERY-0.3 { 2.72 } else { 7.0 };
            let max_feet=ceiling-EYE-0.08;
            if feet > max_feet { feet=max_feet; motion.vertical_speed=motion.vertical_speed.min(0.0); }
            let floor=base_height(p.x,p.z,before);
            if motion.vertical_speed <= 0.0 && feet <= floor {
                feet=floor; motion.vertical_speed=0.0; motion.grounded=true;
            }
            p.y=feet+EYE;
        }
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(p: &mut Vec3, m: &mut Motion, d: Vec3, jump: bool) {
        *p=advance(*p,d,jump,1.0/60.0,m);
    }
    #[test]
    fn stage_blocks_walking_but_accepts_jump_and_landing() {
        let mut m=Motion::standing(); let mut p=Vec3::new(0.0,EYE,-4.8);
        for _ in 0..50 { run(&mut p,&mut m,Vec3::new(0.0,0.0,-0.04),false); }
        assert!(p.z >= -5.35+PLAYER_RADIUS-0.01, "{p:?}");
        for i in 0..90 { run(&mut p,&mut m,Vec3::new(0.0,0.0,if i<32 {-0.04} else {0.0}),i==0); }
        assert!(p.z < -5.35 && (p.y-EYE-STAGE).abs()<0.01, "{p:?}");
        assert!(m.grounded);
    }
    #[test]
    fn walk_off_stage_lands_on_floor() {
        let mut m=Motion::standing(); let mut p=Vec3::new(0.0,EYE+STAGE,-5.7);
        for _ in 0..90 { run(&mut p,&mut m,Vec3::new(0.0,0.0,0.025),false); }
        assert!(p.z > -5.0 && (p.y-EYE).abs()<0.01 && m.grounded,"{p:?}");
    }
    #[test]
    fn stairs_reach_gallery_and_descend() {
        let mut m=Motion::standing(); let mut p=Vec3::new(-9.0,EYE,1.0);
        for _ in 0..150 { run(&mut p,&mut m,Vec3::new(0.0,0.0,0.05),false); }
        assert!((p.y-EYE-GALLERY).abs()<0.01,"{p:?}");
        for _ in 0..150 { run(&mut p,&mut m,Vec3::new(0.0,0.0,-0.05),false); }
        assert!((p.y-EYE).abs()<0.01,"{p:?}");
    }
    #[test]
    fn gallery_edge_and_underneath_ceiling_are_solid() {
        let mut m=Motion::standing(); let mut p=Vec3::new(0.0,EYE+GALLERY,5.5);
        for _ in 0..60 { run(&mut p,&mut m,Vec3::new(0.0,0.0,-0.05),false); }
        assert!(p.z>=5.4 && (p.y-EYE-GALLERY).abs()<0.01);
        let mut m=Motion::standing(); let mut p=Vec3::new(0.0,EYE,8.0);
        let mut maximum=p.y;
        for i in 0..100 { run(&mut p,&mut m,Vec3::ZERO,i==0); maximum=maximum.max(p.y); }
        assert!(maximum<=2.65 && (p.y-EYE).abs()<0.01,"{maximum} {p:?}");
    }
    #[test]
    fn no_double_jump() {
        let mut m=Motion::standing(); let mut p=Vec3::new(0.0,EYE,0.0); let mut max=p.y;
        for i in 0..120 { run(&mut p,&mut m,Vec3::ZERO,i==0 || i==15); max=max.max(p.y); }
        assert!(max-EYE < 1.4 && m.grounded,"{max} {p:?}");
    }
}
