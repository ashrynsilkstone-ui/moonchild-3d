//! Moonchild's upgraded club section: curved gallery, floating stairs,
//! velvet and boucle lounges, semicircular bar, circular dance floor, the ceiling halo
//! and a curved DJ booth. All meshes are generated here; dimensions come from
//! `navigation::layout` so that collision matches the visible geometry.
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use bevy::prelude::*;
use crate::look::{tile, Surfaces};
use crate::lod;
use crate::navigation::layout::*;
use crate::shapes::{self, arc_frames, path_frames, rounded_rect, sweep, tube, Frame, MeshData, ProfilePoint};

/// Floor LED strips sit this far above the dance-floor surface (never inside it).
pub const FLOOR_LED_LIFT: f32 = 0.0025;
const LOUNGE_RUG_RADIUS: f32 = 1.58;

pub struct Neon {
    pub cyan: Handle<StandardMaterial>,
    pub magenta: Handle<StandardMaterial>,
    pub blue: Handle<StandardMaterial>,
    pub warm: Handle<StandardMaterial>,
    pub red: Handle<StandardMaterial>,
}

struct Ctx<'a, 'w, 's> { c: &'a mut Commands<'w, 's>, meshes: &'a mut Assets<Mesh> }
impl Ctx<'_, '_, '_> {
    fn put(&mut self, m: MeshData, mat: &Handle<StandardMaterial>) { self.put_at(m, mat, Transform::IDENTITY); }
    fn put_at(&mut self, m: MeshData, mat: &Handle<StandardMaterial>, t: Transform) {
        self.c.spawn((Mesh3d(self.meshes.add(m.build())), MeshMaterial3d(mat.clone()), t));
    }
    fn mesh(&mut self, m: MeshData) -> Handle<Mesh> { self.meshes.add(m.build()) }
    /// Two levels of detail sharing one origin; `aabb` for meshes in world coordinates.
    fn put_lod(&mut self, hi: MeshData, lo: MeshData, mat: &Handle<StandardMaterial>, t: Transform, switch: f32, aabb: bool) {
        let (hi, lo) = (self.meshes.add(hi.build()), self.meshes.add(lo.build()));
        self.spawn_lod(&hi, &lo, mat, t, switch, aabb);
    }
    fn spawn_lod(&mut self, hi: &Handle<Mesh>, lo: &Handle<Mesh>, mat: &Handle<StandardMaterial>, t: Transform, switch: f32, aabb: bool) {
        self.c.spawn((Mesh3d(hi.clone()), MeshMaterial3d(mat.clone()), t, lod::near(switch, aabb)));
        self.c.spawn((Mesh3d(lo.clone()), MeshMaterial3d(mat.clone()), t, lod::far(switch, aabb)));
    }
    fn mesh_pair(&mut self, f: impl Fn(usize) -> MeshData, hi: usize, lo: usize) -> (Handle<Mesh>, Handle<Mesh>) {
        (self.mesh(f(hi)), self.mesh(f(lo)))
    }
}

fn offset(frames: &[Frame], lateral: f32, up: f32) -> Vec<Vec3> {
    frames.iter().map(|f| f.pos + f.side * lateral + f.up * up).collect()
}
fn shifted(profile: Vec<ProfilePoint>, dx: f32, dy: f32) -> Vec<ProfilePoint> {
    profile.into_iter().map(|p| ProfilePoint { p: p.p + Vec2::new(dx, dy), n: p.n }).collect()
}
/// Open vertical panel profile from `y0` to `y1`, facing towards -lateral.
fn panel_profile(lateral: f32, y0: f32, y1: f32) -> Vec<ProfilePoint> { panel_facing(lateral, y0, y1, Vec2::NEG_X) }
/// Open vertical panel whose visible side faces `normal` (in lateral/up coordinates).
fn panel_facing(lateral: f32, y0: f32, y1: f32, normal: Vec2) -> Vec<ProfilePoint> {
    vec![ProfilePoint { p: Vec2::new(lateral, y0), n: normal }, ProfilePoint { p: Vec2::new(lateral, y1), n: normal }]
}
fn at_y(frames: &[Frame], y: f32) -> Vec<Frame> {
    frames.iter().map(|f| Frame { pos: Vec3::new(f.pos.x, y, f.pos.z), ..*f }).collect()
}
fn uv(t: f32) -> f32 { 1.0 / t }

pub fn spawn(c: &mut Commands, meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>, s: &Surfaces, neon: &Neon) {
    let mut cx = Ctx { c, meshes };
    let glass = [
        (0.85, 0.45, 0.10), (0.20, 0.55, 0.25), (0.75, 0.80, 0.85), (0.15, 0.25, 0.75), (0.7, 0.12, 0.2),
    ].map(|(r, g, b)| materials.add(StandardMaterial {
        base_color: Color::srgba(r * 0.7, g * 0.7, b * 0.7, 0.82), alpha_mode: AlphaMode::Blend, perceptual_roughness: 0.06,
        reflectance: 0.7, emissive: LinearRgba::rgb(r * 0.6, g * 0.6, b * 0.6), ..default()
    }));
    let bulb = materials.add(StandardMaterial { base_color: Color::srgb(1.0, 0.9, 0.75), emissive: LinearRgba::rgb(14.0, 10.0, 6.0), ..default() });
    dance_floor(&mut cx, s, neon);
    gallery(&mut cx, s, neon, &bulb);
    stairs(&mut cx, s, neon);
    let leaf_mesh = cx.mesh_pair(|n| shapes::leaf(0.21, 0.07, n, n / 4), 48, 12);
    for (i, &(x, y, z)) in LOUNGES.iter().enumerate() {
        let centre = Vec3::new(x, y, z);
        lounge(&mut cx, s, neon, &bulb, centre, i);
        table_plant(&mut cx, s, &leaf_mesh, centre);
    }
    bar(&mut cx, s, neon, &glass, &bulb);
    halo(&mut cx, s, neon, &bulb);
    booth(&mut cx, s, neon);
    stage_steps(&mut cx, s, neon);
    speakers(&mut cx, s, neon);
}

/// Small foliage arrangement contained within the existing table collision.
fn table_plant(cx: &mut Ctx, s: &Surfaces, mesh: &(Handle<Mesh>, Handle<Mesh>), table: Vec3) {
    let centre = table + Vec3::new(0.23, LOUNGE_TABLE_TOP, -0.10);
    let vase = shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.075, 0.0), Vec2::new(0.085, 0.025),
        Vec2::new(0.065, 0.14), Vec2::new(0.045, 0.16), Vec2::new(0.0, 0.16)], 32, uv(tile::BRASS));
    cx.put_at(vase, &s.brass, Transform::from_translation(centre));
    // Reuse both leaf meshes; retain a simpler visible leaf at a distance.
    for i in 0..9 {
        let angle = i as f32 * 2.399_963_1;
        let r = 0.035 + 0.018 * (i % 3) as f32;
        let pos = centre + Vec3::new(angle.cos() * r, 0.18 + 0.018 * (i % 3) as f32, angle.sin() * r);
        let rotation = Quat::from_rotation_y(-angle + FRAC_PI_2) * Quat::from_rotation_x(0.42 + 0.12 * (i % 3) as f32);
        cx.spawn_lod(&mesh.0, &mesh.1, &s.foliage, Transform::from_translation(pos).with_rotation(rotation), lod::SMALL, false);
    }
}

// ------------------------------------------------------------------ dance floor
fn dance_floor(cx: &mut Ctx, s: &Surfaces, neon: &Neon) {
    let c = Vec3::new(DANCE_CENTER.0, 0.0, DANCE_CENTER.1);
    cx.put(shapes::annulus(c + Vec3::Y * DANCE_TOP, 0.0, DANCE_RADIUS, 0.0, TAU, 192, uv(tile::FLOOR)), &s.dance_floor);
    // Rounded steel nosing around the disc.
    let edge = [Vec2::new(DANCE_RADIUS + 0.06, 0.0), Vec2::new(DANCE_RADIUS + 0.06, 0.012), Vec2::new(DANCE_RADIUS + 0.035, DANCE_TOP),
        Vec2::new(DANCE_RADIUS - 0.01, DANCE_TOP + 0.0005)];
    cx.put_at(shapes::lathe(&edge, 192, uv(tile::STEEL)), &s.steel, Transform::from_translation(c));
    // Concentric and radial LED inlays, lifted clear of the surface.
    let led_y = DANCE_TOP + FLOOR_LED_LIFT;
    for (i, r) in [1.6, 3.2, 4.72].into_iter().enumerate() {
        let mat = if i % 2 == 0 { &neon.cyan } else { &neon.magenta };
        cx.put(shapes::annulus(c + Vec3::Y * led_y, r - 0.025, r + 0.025, 0.0, TAU, 128, 1.0), mat);
    }
    for i in 0..16 {
        let a = TAU * i as f32 / 16.0 + PI / 16.0;
        let (r0, r1) = if i % 2 == 0 { (1.7, 3.1) } else { (3.3, 4.62) };
        cx.put(shapes::annulus(c + Vec3::Y * led_y, r0, r1, a - 0.006, a + 0.006, 1, 1.0),
            if i % 2 == 0 { &neon.magenta } else { &neon.blue });
    }
}

// ------------------------------------------------------------------ gallery
fn gallery_curve(x0: f32, x1: f32, step: f32) -> Vec<Vec3> {
    let n = ((x1 - x0) / step).ceil() as usize;
    (0..=n).map(|i| { let x = x0 + (x1 - x0) * i as f32 / n as f32; Vec3::new(x, 0.0, gallery_front(x)) }).collect()
}

fn gallery(cx: &mut Ctx, s: &Surfaces, neon: &Neon, bulb: &Handle<StandardMaterial>) {
    let front = gallery_curve(-ROOM_X, ROOM_X, 0.12);
    let top: Vec<Vec3> = front.iter().map(|p| Vec3::new(p.x, GALLERY_Y, p.z)).collect();
    let back: Vec<Vec3> = front.iter().map(|p| Vec3::new(p.x, GALLERY_Y, ROOM_Z)).collect();
    cx.put(shapes::ribbon(&top, &back, Vec3::Y, uv(tile::FLOOR)), &s.floor);
    let under_f: Vec<Vec3> = top.iter().map(|p| Vec3::new(p.x, GALLERY_UNDERSIDE, p.z + 0.05)).collect();
    let under_b: Vec<Vec3> = back.iter().map(|p| Vec3::new(p.x, GALLERY_UNDERSIDE, p.z)).collect();
    cx.put(shapes::ribbon(&under_f, &under_b, Vec3::NEG_Y, uv(tile::CONCRETE)), &s.concrete);
    // Fascia: rounded steel band; `side` points towards the back wall.
    let frames = path_frames(&front);
    let fascia = shifted(rounded_rect(0.24, GALLERY_Y - GALLERY_UNDERSIDE + 0.02, 0.07, 5), -0.04, GALLERY_UNDERSIDE - 0.02);
    cx.put(sweep(&fascia, &at_y(&frames, 0.0), true, true, uv(tile::DARKSTEEL)), &s.darksteel);
    cx.put(tube(&offset(&at_y(&frames, 0.0), -0.165, 2.83), 0.016, 10, 1.0), &neon.cyan);
    cx.put(tube(&offset(&at_y(&frames, 0.0), -0.02, GALLERY_UNDERSIDE - 0.035), 0.014, 10, 1.0), &neon.magenta);
    // Balustrade: posts, continuous tubes and perforated infill, open at the stairs.
    for (x0, x1) in [(-ROOM_X + 0.05, STAIR_X.0), (STAIR_X.1, ROOM_X - 0.05)] {
        let curve = gallery_curve(x0, x1, 0.15);
        let f = path_frames(&curve);
        // Brass handrail on top, slim steel rails below.
        cx.put(tube(&offset(&at_y(&f, 0.0), 0.0, GALLERY_Y + 1.05), 0.032, 20, uv(tile::BRASS)), &s.brass);
        for (y, r) in [(GALLERY_Y + 0.98, 0.012), (GALLERY_Y + 0.10, 0.012)] {
            cx.put(tube(&offset(&at_y(&f, 0.0), 0.0, y), r, 14, uv(tile::STEEL)), &s.steel);
        }
        cx.put(sweep(&panel_profile(0.0, GALLERY_Y + 0.12, GALLERY_Y + 0.96), &at_y(&f, 0.0), false, false, uv(tile::GRILLE)), &s.grille);
        let len: f32 = curve.windows(2).map(|w| w[0].distance(w[1])).sum();
        let posts = (len / 1.2).ceil() as usize;
        let mut acc = 0.0; let mut next = 0.0; let mut k = 0;
        for w in curve.windows(2) {
            while acc + w[0].distance(w[1]) >= next && k <= posts {
                let t = (next - acc) / w[0].distance(w[1]).max(1e-5);
                let p = w[0].lerp(w[1], t);
                cx.put(tube(&[Vec3::new(p.x, GALLERY_Y, p.z), Vec3::new(p.x, GALLERY_Y + 1.05, p.z)], 0.024, 12, uv(tile::STEEL)), &s.steel);
                k += 1; next = len * k as f32 / posts as f32;
            }
            acc += w[0].distance(w[1]);
        }
    }
    // Round columns supporting the front edge.
    // Fluted brass columns with plain base and capital.
    let column = shapes::fluted_lathe(&[Vec2::new(0.24, 0.0), Vec2::new(0.24, 0.05), Vec2::new(0.14, 0.12), Vec2::new(0.135, 0.3),
        Vec2::new(0.135, GALLERY_UNDERSIDE - 0.3), Vec2::new(0.15, GALLERY_UNDERSIDE - 0.12), Vec2::new(0.22, GALLERY_UNDERSIDE)], 12, 0.016, 10, uv(tile::BRASS));
    let column = cx.mesh(column);
    for &x in &GALLERY_COLUMNS {
        cx.c.spawn((Mesh3d(column.clone()), MeshMaterial3d(s.brass_polished.clone()), Transform::from_xyz(x, 0.0, column_z(x))));
    }
    // Recessed warm downlights under the gallery for the lower lounges.
    let disc = cx.mesh(shapes::lathe(&[Vec2::new(0.0, -0.01), Vec2::new(0.11, -0.01), Vec2::new(0.13, 0.0)], 24, 1.0));
    for &x in &[-8.0, -4.6, -1.4, 1.4, 4.6, 8.0] {
        let z = 8.6;
        cx.c.spawn((Mesh3d(disc.clone()), MeshMaterial3d(bulb.clone()), Transform::from_xyz(x, GALLERY_UNDERSIDE, z)));
        cx.c.spawn((SpotLight { intensity: 26_000.0, color: Color::srgb(1.0, 0.85, 0.66), range: 7.0, inner_angle: 0.55, outer_angle: 1.05,
            shadow_maps_enabled: x.abs() > 4.0 && x.abs() < 5.0, contact_shadows_enabled: true, ..default() },
            Transform::from_xyz(x, GALLERY_UNDERSIDE - 0.03, z).looking_at(Vec3::new(x, 0.0, z), Vec3::Z)));
    }
}

// ------------------------------------------------------------------ stairs
fn stairs(cx: &mut Ctx, s: &Surfaces, neon: &Neon) {
    let (x0, x1) = STAIR_X; let (z0, z1) = STAIR_Z;
    let width = x1 - x0 - 0.12; let xc = (x0 + x1) * 0.5;
    let depth = (z1 - z0) / STAIR_STEPS as f32;
    let tread = cx.mesh(shapes::rounded_box(Vec3::new(width, 0.05, depth + 0.02), 0.015, 2, uv(tile::TREAD)));
    for i in 0..STAIR_STEPS {
        let h = GALLERY_Y * (i + 1) as f32 / STAIR_STEPS as f32;
        let z = z0 + depth * (i as f32 + 0.5);
        cx.c.spawn((Mesh3d(tread.clone()), MeshMaterial3d(s.tread.clone()), Transform::from_xyz(xc, h - 0.025, z)));
        // Amber nosing light on every step.
        cx.put(tube(&[Vec3::new(x0 + 0.15, h - 0.04, z - depth * 0.5 - 0.008), Vec3::new(x1 - 0.15, h - 0.04, z - depth * 0.5 - 0.008)], 0.007, 6, 1.0), &neon.warm);
    }
    // Landing between the last step and the curved gallery edge.
    cx.put_at(shapes::rounded_box(Vec3::new(x1 - x0, 0.3, 1.12), 0.03, 2, uv(tile::TREAD)), &s.tread,
        Transform::from_xyz(xc, GALLERY_Y - 0.152, z1 + 0.54));
    // Two sloped stringers carry the treads.
    let rise = |z: f32| ((z - z0) / (z1 - z0)).clamp(0.0, 1.0) * GALLERY_Y;
    for &x in &[x0 + 0.04, x1 - 0.04] {
        let pts: Vec<Vec3> = (0..=24).map(|i| { let z = z0 - 0.15 + (z1 - z0 + 0.15) * i as f32 / 24.0; Vec3::new(x, rise(z) - 0.16, z) }).collect();
        let f: Vec<Frame> = (0..pts.len()).map(|i| {
            let t = (pts[(i + 1).min(pts.len() - 1)] - pts[i.saturating_sub(1)]).normalize();
            Frame { pos: pts[i], side: Vec3::X, up: t.cross(Vec3::X).normalize() }
        }).collect();
        cx.put(sweep(&shifted(rounded_rect(0.08, 0.3, 0.02, 2), 0.0, -0.15), &f, true, true, uv(tile::DARKSTEEL)), &s.darksteel);
        // Handrail tube following the slope, continuing level to the gallery edge.
        let top_z = gallery_front(x);
        let mut rail: Vec<Vec3> = (0..=16).map(|i| { let z = z0 + (z1 - z0) * i as f32 / 16.0; Vec3::new(x, rise(z) + 0.95, z) }).collect();
        rail.push(Vec3::new(x, GALLERY_Y + 0.95, top_z.max(z1 + 0.2)));
        cx.put(tube(&rail, 0.028, 12, uv(tile::STEEL)), &s.steel);
        let glow: Vec<Vec3> = rail.iter().map(|p| *p - Vec3::Y * 0.05).collect();
        cx.put(tube(&glow, 0.008, 6, 1.0), &neon.cyan);
        for i in 0..=6 {
            let z = z0 + 0.2 + (top_z.max(z1 + 0.2) - z0 - 0.25) * i as f32 / 6.0;
            let y = rise(z);
            cx.put(tube(&[Vec3::new(x, y, z), Vec3::new(x, y + 0.95, z)], 0.02, 10, uv(tile::STEEL)), &s.steel);
        }
    }
}

// ------------------------------------------------------------------ lounges
/// Circle of points around the Y axis, for rings and loops.
fn ring_points(c: Vec3, radius: f32, n: usize) -> Vec<Vec3> {
    (0..=n).map(|i| { let a = TAU * i as f32 / n as f32; c + Vec3::new(a.cos() * radius, 0.0, a.sin() * radius) }).collect()
}

/// Bend a mesh built around the origin (x = along the arc, z = outward,
/// y = up) onto a circle of radius `r` around `centre`, centred at angle `a_mid`.
/// Cushions follow the curve instead of leaving wedge gaps.
fn bend(mut m: MeshData, centre: Vec3, r: f32, a_mid: f32) -> MeshData {
    for (p, n) in m.positions.iter_mut().zip(m.normals.iter_mut()) {
        let (x, y, z) = (p[0], p[1], p[2]);
        let a = a_mid + x / r;
        let dir = Vec3::new(a.cos(), 0.0, a.sin());
        let tan = Vec3::new(-a.sin(), 0.0, a.cos());
        *p = (centre + dir * (r + z) + Vec3::Y * y).to_array();
        *n = (tan * n[0] + Vec3::Y * n[1] + dir * n[2]).normalize_or(Vec3::Y).to_array();
    }
    // (tangent, up, radial) is left-handed: bending reverses triangle winding.
    // Restore outward-facing triangles so back-face culling cannot hide the upholstery.
    for triangle in m.indices.chunks_exact_mut(3) { triangle.swap(1, 2); }
    m
}
/// Rotate a mesh about its local X axis (tilt a back cushion).
fn tilt(mut m: MeshData, angle: f32, pivot: Vec3) -> MeshData {
    let q = Quat::from_rotation_x(angle);
    for (p, n) in m.positions.iter_mut().zip(m.normals.iter_mut()) {
        *p = (pivot + q * (Vec3::from(*p) - pivot)).to_array();
        *n = (q * Vec3::from(*n)).to_array();
    }
    m
}
fn moved(mut m: MeshData, by: Vec3) -> MeshData {
    for p in m.positions.iter_mut() { *p = (Vec3::from(*p) + by).to_array(); }
    m
}

/// Luxury curved "cloud" sofa: upholstered base on a brass kick, soft
/// superquadric seat and back cushions bent along the arc, round bolsters,
/// throw pillows, a marble and fluted-brass coffee table and opal pendants.
/// Gallery sofas are wine velvet with boucle pillows, lower ones cream boucle
/// with velvet pillows.
fn lounge(cx: &mut Ctx, s: &Surfaces, neon: &Neon, bulb: &Handle<StandardMaterial>, centre: Vec3, index: usize) {
    let (a0, a1) = LOUNGE_ARC;
    let r = LOUNGE_RADIUS;
    let upper = centre.y > 1.0;
    let (fabric, fabric_tile, accent, accent_tile) = if upper { (&s.leather_burgundy, tile::LEATHER_BURGUNDY, &s.boucle, tile::BOUCLE) }
        else { (&s.boucle, tile::BOUCLE, &s.velvet, tile::VELVET) };
    let arc = |b0: f32, b1: f32, n: usize| arc_frames(centre, r, b0, b1, n);
    // Low-profile woven rug, entirely below seating and table; it does not
    // introduce a new obstacle and follows the lounge floor elevation.
    cx.put(shapes::annulus(centre + Vec3::Y * 0.004, 0.0, LOUNGE_RUG_RADIUS, 0.0, TAU, 128, uv(tile::ACOUSTIC)), &s.acoustic_charcoal);
    cx.put(tube(&ring_points(centre + Vec3::Y * 0.009, LOUNGE_RUG_RADIUS - 0.01, 128), 0.004, 8, uv(tile::DARKSTEEL)), &s.darksteel);
    // Brass kick plinth and a warm light gap under the upholstered base.
    cx.put(sweep(&shifted(rounded_rect(0.6, 0.07, 0.015, 3), 0.07, 0.0), &arc(a0 + 0.02, a1 - 0.02, 96), true, true, uv(tile::BRASS)), &s.brass);
    cx.put(tube(&offset(&arc(a0 + 0.06, a1 - 0.06, 96), -0.24, 0.035), 0.009, 8, 1.0), if upper { &neon.magenta } else { &neon.red });
    let base = |seg: usize, steps: usize| sweep(&shifted(rounded_rect(0.74, 0.2, 0.07, seg), 0.06, 0.07), &arc(a0, a1, steps), true, true, uv(fabric_tile));
    cx.put_lod(base(8, 128), base(3, 32), fabric, Transform::IDENTITY, lod::LARGE, true);
    // Low upholstered back frame with brass piping along the top.
    let back = |seg: usize, steps: usize| sweep(&shifted(rounded_rect(0.14, 0.62, 0.06, seg), 0.4, 0.07), &arc(a0, a1, steps), true, true, uv(fabric_tile));
    cx.put_lod(back(8, 128), back(3, 32), fabric, Transform::IDENTITY, lod::LARGE, true);
    cx.put(tube(&offset(&arc(a0, a1, 128), 0.4, 0.695), 0.009, 10, uv(tile::BRASS)), &s.brass);

    // Seat and back cushions, bent onto the arc.
    let count = 6;
    let span = (a1 - a0 - 0.16) / count as f32;
    let len = span * r;
    for k in 0..count {
        let am = a0 + 0.08 + span * (k as f32 + 0.5);
        let seat = |res: usize| bend(moved(shapes::cushion(Vec3::new(len * 0.5 - 0.01, 0.085, 0.3), 0.22, 0.5, res, uv(fabric_tile)),
            Vec3::new(0.0, 0.355, -0.02)), centre, r, am);
        cx.put_lod(seat(80), seat(14), fabric, Transform::IDENTITY, lod::LARGE, true);
        let backc = |res: usize| bend(tilt(moved(shapes::cushion(Vec3::new(len * 0.5 - 0.015, 0.22, 0.09), 0.25, 0.6, res, uv(fabric_tile)),
            Vec3::new(0.0, 0.68, 0.25)), 0.2, Vec3::new(0.0, 0.45, 0.25)), centre, r, am);
        cx.put_lod(backc(80), backc(14), fabric, Transform::IDENTITY, lod::LARGE, true);
    }
    // Fine piping follows the actual curved silhouette of the upholstery.
    let seam_frames = arc(a0 + 0.10, a1 - 0.10, 128);
    cx.put(tube(&offset(&seam_frames, -0.265, 0.425), 0.0045, 10, uv(tile::BRASS)), &s.brass);
    // Round bolsters closing both ends.
    for (a, dir) in [(a0, 1.0f32), (a1, -1.0)] {
        let am = a + dir * 0.04;
        let bolster = |res: usize| bend(moved(shapes::cushion(Vec3::new(0.09, 0.2, 0.36), 0.6, 0.7, res, uv(fabric_tile)), Vec3::new(0.0, 0.47, 0.04)), centre, r, am);
        cx.put_lod(bolster(64), bolster(12), fabric, Transform::IDENTITY, lod::LARGE, true);
    }
    // Throw pillows in the accent fabric, leaning against the back cushions.
    for (k, (frac, yaw)) in [(0.18f32, 0.25f32), (0.5, -0.15), (0.82, 0.2)].into_iter().enumerate() {
        let am = a0 + (a1 - a0) * frac;
        let pillow = |res: usize| {
            let m = shapes::cushion(Vec3::new(0.2, 0.2, 0.065), 0.15, 0.45, res, uv(accent_tile));
            // Stand the pillow up and lean it back against the cushions.
            let m = tilt(m, 0.28, Vec3::ZERO);
            let q = Quat::from_rotation_z(yaw);
            let mut m = m;
            for (p, n) in m.positions.iter_mut().zip(m.normals.iter_mut()) { *p = (q * Vec3::from(*p)).to_array(); *n = (q * Vec3::from(*n)).to_array(); }
            bend(moved(m, Vec3::new(0.0, 0.64, 0.1 + 0.02 * k as f32)), centre, r, am)
        };
        cx.put_lod(pillow(56), pillow(10), accent, Transform::IDENTITY, lod::MEDIUM, true);
    }

    // Coffee table: fluted brass drum and an eased marble top.
    let top = LOUNGE_TABLE_TOP;
    let tr = LOUNGE_TABLE_RADIUS;
    let drum = |flutes: usize, sides: usize| shapes::fluted_lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.25, 0.0), Vec2::new(0.25, 0.005),
        Vec2::new(0.25, top - 0.05), Vec2::new(0.0, top - 0.05)], flutes, 0.012, sides, uv(tile::BRASS));
    cx.put_lod(drum(28, 6), drum(14, 2), &s.brass_polished, Transform::from_translation(centre), lod::MEDIUM, false);
    let slab = |n: usize| shapes::lathe(&[Vec2::new(0.0, top - 0.05), Vec2::new(tr - 0.02, top - 0.05), Vec2::new(tr, top - 0.035), Vec2::new(tr, top - 0.015),
        Vec2::new(tr - 0.015, top), Vec2::new(0.0, top)], n, uv(tile::MARBLE));
    cx.put_lod(slab(160), slab(28), &s.marble, Transform::from_translation(centre), lod::MEDIUM, false);
    cx.put(tube(&ring_points(centre + Vec3::Y * 0.02, 0.29, 64), 0.007, 6, 1.0), &neon.warm);
    // Brass lantern with a glowing core.
    cx.put_at(shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.06, 0.0), Vec2::new(0.06, 0.012), Vec2::new(0.0, 0.012)], 32, uv(tile::BRASS)),
        &s.brass, Transform::from_translation(centre + Vec3::Y * top));
    cx.put_at(shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.03, 0.0), Vec2::new(0.032, 0.08), Vec2::new(0.0, 0.085)], 24, 1.0), bulb,
        Transform::from_translation(centre + Vec3::Y * (top + 0.012)));
    cx.put_at(shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.055, 0.0), Vec2::new(0.058, 0.13), Vec2::new(0.05, 0.135), Vec2::new(0.0, 0.135)], 40, 1.0),
        &s.smoked_glass, Transform::from_translation(centre + Vec3::Y * (top + 0.012)));
    cx.c.spawn((PointLight { intensity: 3_500.0, range: 3.0, color: Color::srgb(1.0, 0.68, 0.38), shadow_maps_enabled: false, ..default() },
        Transform::from_translation(centre + Vec3::Y * (top + 0.2))));
    // Cluster of opal globe pendants above each gallery lounge.
    if upper {
        let globe = cx.mesh(shapes::lathe(&(0..=24).map(|i| { let a = -FRAC_PI_2 + PI * i as f32 / 24.0; Vec2::new(a.cos() * 0.14, 0.14 + a.sin() * 0.14) }).collect::<Vec<_>>(), 48, 1.0));
        // Hung above head height on the gallery (eye level about 4.8 m).
        for (dx, dz, drop) in [(0.0f32, 0.0f32, 1.65f32), (0.32, 0.12, 1.45), (-0.28, 0.18, 1.8)] {
            let p = Vec3::new(centre.x + dx, CEILING - drop, centre.z + dz);
            cx.c.spawn((Mesh3d(globe.clone()), MeshMaterial3d(s.opal.clone()), Transform::from_translation(p)));
            cx.put(tube(&[p + Vec3::Y * 0.28, Vec3::new(p.x, CEILING, p.z)], 0.006, 8, 1.0), &s.brass);
            cx.put_at(shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.035, 0.0), Vec2::new(0.035, 0.03), Vec2::new(0.0, 0.03)], 24, uv(tile::BRASS)),
                &s.brass, Transform::from_translation(p + Vec3::Y * 0.27));
        }
        let y = CEILING - 1.85;
        cx.c.spawn((SpotLight { intensity: 40_000.0, color: Color::srgb(1.0, 0.82, 0.62), range: 8.0, inner_angle: 0.45, outer_angle: 0.95,
            shadow_maps_enabled: index == 0, contact_shadows_enabled: true, ..default() },
            Transform::from_xyz(centre.x, y - 0.05, centre.z).looking_at(Vec3::new(centre.x, 0.0, centre.z), Vec3::Z)));
    }
}

// ------------------------------------------------------------------ bar
fn bar(cx: &mut Ctx, s: &Surfaces, neon: &Neon, glass: &[Handle<StandardMaterial>; 5], _bulb: &Handle<StandardMaterial>) {
    let c = Vec3::new(BAR_CENTER.0, 0.0, BAR_CENTER.1);
    let r = BAR_RADIUS;
    // Path: wing -> semicircle -> wing, `side` points into the service area.
    let mut path: Vec<Vec3> = (0..=8).map(|i| Vec3::new(BAR_WING_END - (BAR_WING_END - c.x) * i as f32 / 8.0, 0.0, c.z + r)).collect();
    for i in 1..96 { let a = FRAC_PI_2 + PI * i as f32 / 96.0; path.push(c + Vec3::new(a.cos(), 0.0, a.sin()) * r); }
    path.extend((0..=8).map(|i| Vec3::new(c.x + (BAR_WING_END - c.x) * i as f32 / 8.0, 0.0, c.z - r)));
    let f = path_frames(&path);
    // Core body, then vertically fluted smoked-oak cladding on the guest side.
    cx.put(sweep(&shifted(rounded_rect(0.5, BAR_TOP - 0.14, 0.04, 3), 0.3, 0.08), &f, true, true, uv(tile::DARKSTEEL)), &s.darksteel);
    let outward: Vec<Frame> = f.iter().map(|fr| Frame { pos: fr.pos + fr.side * 0.04, side: -fr.side, up: fr.up }).collect();
    let skin = |pitch: f32| shapes::fluted_skin(&outward, 0.13, BAR_TOP - 0.1, pitch, 0.02, uv(tile::OAK));
    cx.put_lod(skin(0.065), skin(0.26), &s.oak, Transform::IDENTITY, lod::LARGE, true);
    // Brass kick plinth and a recessed warm line under the top.
    cx.put(sweep(&shifted(rounded_rect(0.05, 0.13, 0.012, 3), 0.03, 0.0), &f, true, true, uv(tile::BRASS)), &s.brass);
    cx.put(tube(&offset(&f, 0.0, 0.03), 0.01, 8, 1.0), &neon.cyan);
    cx.put(tube(&offset(&f, -0.04, BAR_TOP - 0.1), 0.009, 8, 1.0), &neon.warm);
    // Marble counter with a softly eased, overhanging edge, brass trim below.
    cx.put(sweep(&shifted(rounded_rect(0.8, 0.07, 0.03, 6), 0.25, BAR_TOP - 0.07), &f, true, true, uv(tile::MARBLE)), &s.marble);
    cx.put(tube(&offset(&f, -0.12, BAR_TOP - 0.075), 0.007, 8, uv(tile::BRASS)), &s.brass);
    // Brass foot rail on brackets.
    cx.put(tube(&offset(&f, -0.3, 0.22), 0.024, 16, uv(tile::BRASS)), &s.brass);
    for fr in f.iter().step_by(10) {
        let a = fr.pos + fr.side * -0.3 + Vec3::Y * 0.22; let b = fr.pos + fr.side * 0.02 + Vec3::Y * 0.14;
        cx.put(tube(&[a, b], 0.012, 10, 1.0), &s.brass);
    }

    // Back bar: fluted oak cabinet with marble top, three backlit arched niches
    // framed in brass, smoked-glass shelves and bottles.
    let wall_x = ROOM_X;
    cx.put_at(shapes::rounded_box(Vec3::new(0.6, 0.95, 3.6), 0.04, 3, uv(tile::DARKSTEEL)), &s.darksteel, Transform::from_xyz(wall_x - 0.3, 0.475, c.z));
    let cab: Vec<Frame> = path_frames(&[Vec3::new(wall_x - 0.6, 0.0, c.z + 1.8), Vec3::new(wall_x - 0.6, 0.0, c.z - 1.8)]);
    let cab: Vec<Frame> = cab.into_iter().map(|fr| Frame { side: Vec3::NEG_X, ..fr }).collect();
    cx.put(shapes::fluted_skin(&cab, 0.06, 0.94, 0.065, 0.018, uv(tile::OAK)), &s.oak);
    cx.put_at(shapes::rounded_box(Vec3::new(0.66, 0.05, 3.66), 0.02, 3, uv(tile::MARBLE)), &s.marble, Transform::from_xyz(wall_x - 0.32, 0.975, c.z));
    // Oak wall behind the niches.
    let wall: Vec<Frame> = path_frames(&[Vec3::new(wall_x - 0.015, 0.0, c.z + 1.8), Vec3::new(wall_x - 0.015, 0.0, c.z - 1.8)])
        .into_iter().map(|fr| Frame { side: Vec3::NEG_X, ..fr }).collect();
    cx.put(shapes::fluted_skin(&wall, 1.0, 3.0, 0.065, 0.012, uv(tile::OAK)), &s.oak);
    let (bottle, bottle_lo) = cx.mesh_pair(|n| shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.034, 0.0), Vec2::new(0.037, 0.012), Vec2::new(0.037, 0.19), Vec2::new(0.03, 0.225),
        Vec2::new(0.014, 0.26), Vec2::new(0.012, 0.31), Vec2::new(0.015, 0.318), Vec2::new(0.0, 0.322)], n, 1.0), 36, 8);
    // The sleeve is a separate, single UV-wrapped surface over the glass.
    let sleeve = cx.mesh_pair(|n| {
        let mut m = shapes::lathe(&[Vec2::new(0.0377, 0.0), Vec2::new(0.0377, 0.10)], n, 1.0);
        for uv in &mut m.uvs { uv[0] /= TAU * 0.05; uv[1] /= 0.10; }
        m
    }, 48, 12);
    let cap = cx.mesh_pair(|n| shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.015, 0.0),
        Vec2::new(0.015, 0.022), Vec2::new(0.0, 0.022)], n, uv(tile::BRASS)), 32, 8);
    for (ni, dz) in [-1.15f32, 0.0, 1.15].into_iter().enumerate() {
        let base = Vec3::new(wall_x - 0.04, 1.08, c.z + dz);
        cx.put(shapes::arch_panel(base, Vec3::Z, Vec3::NEG_X, 0.98, 1.82, uv(tile::BACKLIGHT)), &s.backlight_amber);
        let outline = shapes::arch_outline(base + Vec3::NEG_X * 0.03, Vec3::Z, 1.02, 1.84, 40);
        cx.put(tube(&outline, 0.016, 12, uv(tile::BRASS)), &s.brass);
        for (k, y) in [1.32f32, 1.76, 2.2].into_iter().enumerate() {
            cx.put_at(shapes::rounded_box(Vec3::new(0.3, 0.018, 0.9), 0.008, 2, 1.0), &s.smoked_glass, Transform::from_xyz(wall_x - 0.2, y, c.z + dz));
            for i in 0..4 {
                let z = c.z + dz - 0.33 + i as f32 * 0.22;
                let scale = 0.85 + 0.3 * (((i * 7 + k * 3 + ni) % 5) as f32 / 4.0);
                let x = wall_x - 0.2 + if i % 2 == 0 { 0.05 } else { -0.04 };
                cx.spawn_lod(&bottle, &bottle_lo, &glass[(i + k + ni) % 5],
                    Transform::from_xyz(x, y + 0.01, z).with_scale(Vec3::new(1.0, scale, 1.0)), lod::SMALL, false);
                cx.spawn_lod(&sleeve.0, &sleeve.1, &s.bottle_label,
                    Transform::from_xyz(x, y + 0.01 + 0.065 * scale, z).with_scale(Vec3::new(1.0, scale, 1.0)), lod::SMALL, false);
                cx.spawn_lod(&cap.0, &cap.1, &s.brass,
                    Transform::from_xyz(x, y + 0.01 + 0.30 * scale, z).with_scale(Vec3::new(1.0, scale, 1.0)), lod::SMALL, false);
            }
        }
    }
    // Bar stools: brass trumpet base and seat shell, velvet cushion and backrest.
    let base = cx.mesh_pair(|n| shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.2, 0.0), Vec2::new(0.205, 0.01), Vec2::new(0.19, 0.022), Vec2::new(0.06, 0.05),
        Vec2::new(0.024, 0.13), Vec2::new(0.022, 0.62), Vec2::new(0.07, 0.64), Vec2::new(0.0, 0.645)], n, uv(tile::BRASS)), 64, 12);
    let shell = cx.mesh_pair(|n| shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.19, 0.0), Vec2::new(0.205, 0.015), Vec2::new(0.205, 0.045), Vec2::new(0.0, 0.045)], n, uv(tile::BRASS)), 64, 14);
    let cushion = cx.mesh_pair(|n| moved(shapes::cushion(Vec3::new(0.19, 0.05, 0.19), 0.9, 0.5, n, uv(tile::VELVET)), Vec3::Y * 0.04), 64, 14);
    let backrest = cx.mesh_pair(|n| sweep(&rounded_rect(0.05, 0.19, 0.022, n / 8), &arc_frames(Vec3::ZERO, 0.18, -1.15, 1.15, n), true, true, uv(tile::VELVET)), 48, 10);
    let footring = cx.mesh_pair(|n| tube(&ring_points(Vec3::ZERO, 0.17, n), 0.01, n / 5, uv(tile::BRASS)), 64, 20);
    let spoke = cx.mesh(tube(&[Vec3::new(0.02, 0.0, 0.0), Vec3::new(0.165, 0.0, 0.0)], 0.008, 8, 1.0));
    for &a in &BAR_STOOL_ANGLES {
        let p = c + Vec3::new(a.cos(), 0.0, a.sin()) * (r + 0.48);
        let facing = Quat::from_rotation_y(-a);
        let m = lod::MEDIUM;
        cx.spawn_lod(&base.0, &base.1, &s.brass, Transform::from_translation(p), m, false);
        cx.spawn_lod(&shell.0, &shell.1, &s.brass, Transform::from_translation(p + Vec3::Y * 0.64), m, false);
        cx.spawn_lod(&cushion.0, &cushion.1, &s.velvet, Transform::from_translation(p + Vec3::Y * 0.675), m, false);
        cx.spawn_lod(&backrest.0, &backrest.1, &s.velvet, Transform::from_translation(p + Vec3::Y * 0.72).with_rotation(facing), m, false);
        cx.spawn_lod(&footring.0, &footring.1, &s.brass, Transform::from_translation(p + Vec3::Y * 0.3), m, false);
        for k in 0..3 {
            cx.c.spawn((Mesh3d(spoke.clone()), MeshMaterial3d(s.brass.clone()),
                Transform::from_translation(p + Vec3::Y * 0.3).with_rotation(Quat::from_rotation_y(TAU * k as f32 / 3.0 - a))));
        }
    }
    // Opal globe pendants on brass rods over the counter; the middle spot casts shadows.
    let globe = cx.mesh(shapes::lathe(&(0..=24).map(|i| { let a = -FRAC_PI_2 + PI * i as f32 / 24.0; Vec2::new(a.cos() * 0.13, 0.13 + a.sin() * 0.13) }).collect::<Vec<_>>(), 48, 1.0));
    let cap = cx.mesh(shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.04, 0.0), Vec2::new(0.04, 0.035), Vec2::new(0.0, 0.035)], 24, uv(tile::BRASS)));
    for (i, &a) in [2.25f32, 2.75, PI, 3.53, 4.03].iter().enumerate() {
        let p = c + Vec3::new(a.cos(), 0.0, a.sin()) * (r + 0.05);
        let y = 2.35 + if i % 2 == 0 { 0.0 } else { 0.18 };
        cx.c.spawn((Mesh3d(globe.clone()), MeshMaterial3d(s.opal.clone()), Transform::from_xyz(p.x, y, p.z)));
        cx.c.spawn((Mesh3d(cap.clone()), MeshMaterial3d(s.brass.clone()), Transform::from_xyz(p.x, y + 0.25, p.z)));
        cx.put(tube(&[Vec3::new(p.x, y + 0.28, p.z), Vec3::new(p.x, CEILING, p.z)], 0.006, 8, 1.0), &s.brass);
        if i % 2 == 0 {
            cx.c.spawn((SpotLight { intensity: 30_000.0, color: Color::srgb(1.0, 0.8, 0.58), range: 6.0, inner_angle: 0.35, outer_angle: 0.85,
                shadow_maps_enabled: i == 2, contact_shadows_enabled: true, ..default() },
                Transform::from_xyz(p.x, y - 0.03, p.z).looking_at(Vec3::new(p.x, 0.0, p.z), Vec3::Z)));
        }
    }
    // Two hand-sized stemmed glasses and a steel cocktail shaker, all resting
    // on the existing bar footprint, use shared turned meshes.
    let stem = cx.mesh(shapes::lathe(&[Vec2::new(0.0,0.0), Vec2::new(0.045,0.0), Vec2::new(0.045,0.008),
        Vec2::new(0.010,0.015), Vec2::new(0.008,0.11), Vec2::new(0.025,0.12),
        Vec2::new(0.065,0.20), Vec2::new(0.061,0.205)], 48, 1.0));
    let shaker = cx.mesh(shapes::lathe(&[Vec2::new(0.0,0.0),Vec2::new(0.04,0.0),Vec2::new(0.05,0.015),
        Vec2::new(0.047,0.19),Vec2::new(0.038,0.22),Vec2::new(0.025,0.225),Vec2::new(0.0,0.225)],48,uv(tile::STEEL)));
    for a in [2.38f32, 3.74] {
        let p = c + Vec3::new(a.cos(), 0.0, a.sin()) * (r + 0.05) + Vec3::Y * BAR_TOP;
        cx.c.spawn((Mesh3d(stem.clone()), MeshMaterial3d(glass[2].clone()), Transform::from_translation(p)));
    }
    let p = c + Vec3::new(PI.cos(), 0.0, PI.sin()) * (r + 0.05) + Vec3::Y * BAR_TOP;
    cx.c.spawn((Mesh3d(shaker), MeshMaterial3d(s.steel.clone()), Transform::from_translation(p)));
    // Warm glow from the back bar onto the counter.
    cx.c.spawn((PointLight { intensity: 22_000.0, range: 5.0, color: Color::srgb(1.0, 0.55, 0.25), shadow_maps_enabled: false, ..default() },
        Transform::from_xyz(wall_x - 0.8, 1.9, c.z)));
}

// ------------------------------------------------------------------ ceiling halo
fn halo(cx: &mut Ctx, s: &Surfaces, neon: &Neon, bulb: &Handle<StandardMaterial>) {
    let (hx, hy, hz) = RING_CENTER;
    let c = Vec3::new(hx, hy, hz);
    let r = RING_RADIUS;
    let ring = shifted(rounded_rect(0.5, 0.24, 0.09, 5), 0.0, 0.0);
    cx.put(sweep(&ring, &arc_frames(c, r, 0.0, TAU, 240), true, false, uv(tile::DARKSTEEL)), &s.darksteel);
    // Neon rings underneath and an inner floating halo.
    for (rr, y, mat) in [(r - 0.16, -0.012, &neon.cyan), (r + 0.16, -0.012, &neon.magenta), (r - 0.6, 0.06, &neon.cyan)] {
        let pts: Vec<Vec3> = (0..=240).map(|i| { let a = TAU * i as f32 / 240.0; c + Vec3::new(a.cos() * rr, y, a.sin() * rr) }).collect();
        cx.put(tube(&pts, if rr < r - 0.3 { 0.035 } else { 0.022 }, 10, 1.0), mat);
    }
    // Hub with a warm downlight.
    cx.put_at(shapes::lathe(&[Vec2::new(0.0, -0.08), Vec2::new(0.62, -0.08), Vec2::new(0.7, -0.02), Vec2::new(0.7, 0.12), Vec2::new(0.55, 0.2), Vec2::new(0.0, 0.22)], 48, uv(tile::DARKSTEEL)),
        &s.darksteel, Transform::from_translation(c));
    cx.put_at(shapes::annulus(Vec3::ZERO, 0.0, 0.5, 0.0, TAU, 40, 1.0), bulb, Transform::from_translation(c - Vec3::Y * 0.085).with_rotation(Quat::from_rotation_x(PI)));
    // Twelve radial spokes with alternating light strips.
    for i in 0..12 {
        let a = TAU * i as f32 / 12.0;
        let d = Vec3::new(a.cos(), 0.0, a.sin());
        let (p0, p1) = (c + d * 0.72, c + d * (r - 0.24));
        let len = p0.distance(p1);
        let rot = Quat::from_rotation_y(-a);
        cx.put_at(shapes::rounded_box(Vec3::new(len, 0.08, 0.1), 0.03, 3, uv(tile::DARKSTEEL)), &s.darksteel,
            Transform::from_translation((p0 + p1) * 0.5 + Vec3::Y * 0.04).with_rotation(rot));
        cx.put(tube(&[p0 - Vec3::Y * 0.005, p1 - Vec3::Y * 0.005], 0.016, 8, 1.0), if i % 2 == 0 { &neon.cyan } else { &neon.magenta });
    }
    // Suspension cables.
    for i in 0..6 {
        let a = TAU * i as f32 / 6.0 + 0.26;
        let p = c + Vec3::new(a.cos(), 0.0, a.sin()) * r;
        cx.put(tube(&[p + Vec3::Y * 0.12, Vec3::new(p.x, CEILING, p.z)], 0.008, 6, 1.0), &s.steel);
    }
    // Radial light strips across the ceiling, continuing the halo outwards.
    for i in 0..20 {
        let a = TAU * i as f32 / 20.0;
        let d = Vec3::new(a.cos(), 0.0, a.sin());
        let limit = ((ROOM_X - 0.4) / d.x.abs().max(1e-3)).min((ROOM_Z - 0.4 - if d.z > 0.0 { hz } else { -hz }) / d.z.abs().max(1e-3));
        let (r0, r1) = (r + 0.7, limit.min(9.5));
        if r1 <= r0 + 0.5 { continue; }
        let y = CEILING - 0.03;
        let a = Vec3::new(hx, y, hz) + d * r0;
        let b = Vec3::new(hx, y, hz) + d * r1;
        cx.put(tube(&[a + Vec3::Y * 0.025, b + Vec3::Y * 0.025], 0.034, 12, uv(tile::DARKSTEEL)), &s.darksteel);
        cx.put(tube(&[a, b], 0.018, 12, 1.0), if i % 2 == 0 { &neon.blue } else { &neon.magenta });
    }
    // Warm house spots mounted on the ring.
    for i in 0..6 {
        let a = TAU * i as f32 / 6.0;
        let p = c + Vec3::new(a.cos(), 0.0, a.sin()) * r - Vec3::Y * 0.13;
        cx.put_at(shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.1, 0.0), Vec2::new(0.11, 0.02), Vec2::new(0.11, 0.18), Vec2::new(0.0, 0.18)], 20, uv(tile::DARKSTEEL)),
            &s.darksteel, Transform::from_translation(p - Vec3::Y * 0.06));
        cx.put_at(shapes::annulus(Vec3::ZERO, 0.0, 0.08, 0.0, TAU, 16, 1.0), bulb, Transform::from_translation(p - Vec3::Y * 0.065).with_rotation(Quat::from_rotation_x(PI)));
        cx.c.spawn((SpotLight { intensity: 70_000.0, color: Color::srgb(1.0, 0.94, 0.84), range: 14.0, inner_angle: 0.40, outer_angle: 0.80,
            shadow_maps_enabled: true, contact_shadows_enabled: true, ..default() },
            Transform::from_translation(p - Vec3::Y * 0.08).looking_at(Vec3::new(p.x * 0.85, 0.0, p.z), Vec3::Z)));
    }
    cx.c.spawn((SpotLight { intensity: 110_000.0, color: Color::srgb(1.0, 0.95, 0.86), range: 16.0, inner_angle: 0.45, outer_angle: 0.85,
        shadow_maps_enabled: true, contact_shadows_enabled: true, ..default() },
        Transform::from_translation(c - Vec3::Y * 0.1).looking_at(Vec3::new(hx, 0.0, hz), Vec3::Z)));
}

/// Moving-head fixtures hung from the halo. Returns their positions.
pub fn moving_head_positions() -> Vec<Vec3> {
    let (hx, hy, hz) = RING_CENTER;
    (0..6).map(|i| {
        let a = TAU * i as f32 / 6.0 + PI / 6.0;
        Vec3::new(hx + a.cos() * RING_RADIUS, hy - 0.42, hz + a.sin() * RING_RADIUS)
    }).collect()
}

/// Mesh and yoke for a moving head (spawn as a child of the light so it turns with the beam).
pub fn moving_head_meshes(meshes: &mut Assets<Mesh>, s: &Surfaces, lens: Handle<StandardMaterial>)
    -> Vec<(Handle<Mesh>, Handle<StandardMaterial>, Transform, Option<bevy::camera::visibility::VisibilityRange>)> {
    let body = |n: usize| shapes::lathe(&[Vec2::new(0.0, -0.18), Vec2::new(0.1, -0.18), Vec2::new(0.13, -0.1), Vec2::new(0.14, 0.12), Vec2::new(0.12, 0.17), Vec2::new(0.0, 0.17)], n, uv(tile::DARKSTEEL)).build();
    let lens_mesh = shapes::annulus(Vec3::ZERO, 0.0, 0.11, 0.0, TAU, 32, 1.0).build();
    // Lathe axis Y -> beam axis -Z.
    let to_beam = Quat::from_rotation_x(-FRAC_PI_2);
    vec![
        (meshes.add(body(48)), s.darksteel.clone(), Transform::from_rotation(to_beam), Some(lod::near(lod::MEDIUM, false))),
        (meshes.add(body(12)), s.darksteel.clone(), Transform::from_rotation(to_beam), Some(lod::far(lod::MEDIUM, false))),
        (meshes.add(lens_mesh), lens, Transform::from_xyz(0.0, 0.0, -0.172).with_rotation(Quat::from_rotation_x(FRAC_PI_2)), None),
    ]
}

pub fn spawn_head_yokes(c: &mut Commands, meshes: &mut Assets<Mesh>, s: &Surfaces) {
    let yoke = meshes.add(shapes::rounded_box(Vec3::new(0.36, 0.3, 0.12), 0.04, 3, uv(tile::DARKSTEEL)).build());
    let base = meshes.add(shapes::rounded_box(Vec3::new(0.32, 0.1, 0.32), 0.03, 3, uv(tile::DARKSTEEL)).build());
    for p in moving_head_positions() {
        c.spawn((Mesh3d(base.clone()), MeshMaterial3d(s.darksteel.clone()), Transform::from_translation(p + Vec3::Y * 0.33)));
        c.spawn((Mesh3d(yoke.clone()), MeshMaterial3d(s.darksteel.clone()), Transform::from_translation(p + Vec3::Y * 0.16)
            .with_rotation(Quat::from_rotation_y(p.z.atan2(p.x)))));
    }
}

// ------------------------------------------------------------------ DJ booth and stage steps
fn booth(cx: &mut Ctx, s: &Surfaces, neon: &Neon) {
    let centre = Vec3::new(0.0, STAGE_Y, BOOTH_CENTER_Z);
    let (a0, a1) = (FRAC_PI_2 - BOOTH_HALF_ANGLE, FRAC_PI_2 + BOOTH_HALF_ANGLE);
    let f = arc_frames(centre, BOOTH_RADIUS, a0, a1, 96);
    let h = BOOTH_TOP - STAGE_Y;
    cx.put(sweep(&shifted(rounded_rect(0.84, h, 0.12, 6), -0.07, 0.0), &f, true, true, uv(tile::DARKSTEEL)), &s.darksteel);
    // Backlit perforated front, slightly in front of the body.
    let inner = arc_frames(centre, BOOTH_RADIUS, a0 + 0.02, a1 - 0.02, 96);
    cx.put(sweep(&panel_facing(0.36, 0.16, h - 0.22, Vec2::X), &inner, false, false, uv(tile::BACKLIGHT)), &s.backlight_cyan);
    cx.put(sweep(&panel_facing(0.375, 0.14, h - 0.2, Vec2::X), &inner, false, false, uv(tile::GRILLE)), &s.grille);
    // Vertical brass ribs and fine edging add depth to the booth face.
    for frame in f.iter().step_by(8) {
        let p = frame.pos + frame.side * 0.395;
        cx.put(tube(&[p + Vec3::Y * 0.17, p + Vec3::Y * (h - 0.19)], 0.008, 12, uv(tile::BRASS)), &s.brass);
    }
    cx.put(tube(&offset(&f, 0.395, h - 0.17), 0.012, 12, uv(tile::BRASS)), &s.brass);
    cx.put(tube(&offset(&f, 0.34, 0.04), 0.014, 8, 1.0), &neon.magenta);
    cx.put(tube(&offset(&f, 0.36, h - 0.06), 0.012, 8, 1.0), &neon.cyan);
}

fn stage_steps(cx: &mut Ctx, s: &Surfaces, neon: &Neon) {
    let (x_low, x_high) = (STAGE_STEPS_X.1, STAGE_STEPS_X.0);
    let n = STAGE_STEP_COUNT;
    let run = (x_low - x_high) / n as f32;
    let width = STAGE_STEPS_Z.1 - STAGE_STEPS_Z.0;
    let zc = (STAGE_STEPS_Z.0 + STAGE_STEPS_Z.1) * 0.5;
    for i in 0..n {
        // Step i spans from the stage edge to its own nosing, stacked like a solid block.
        let h = STAGE_Y * (i + 1) as f32 / n as f32;
        let nosing = x_low - run * i as f32;
        let length = nosing - x_high;
        cx.put_at(shapes::rounded_box(Vec3::new(length, h, width), 0.02, 2, uv(tile::TREAD)), &s.tread,
            Transform::from_xyz(x_high + length * 0.5, h * 0.5, zc));
        cx.put(tube(&[Vec3::new(nosing + 0.004, h - 0.02, STAGE_STEPS_Z.0 + 0.08), Vec3::new(nosing + 0.004, h - 0.02, STAGE_STEPS_Z.1 - 0.08)], 0.007, 6, 1.0), &neon.warm);
    }
}

fn speakers(cx: &mut Ctx, s: &Surfaces, neon: &Neon) {
    let cabinet = cx.mesh(shapes::rounded_box(Vec3::new(0.83, 1.02, 0.78), 0.06, 8, uv(tile::DARKSTEEL)));
    let cone = cx.mesh(shapes::lathe(&[Vec2::new(0.0, -0.05), Vec2::new(0.05, -0.045), Vec2::new(0.2, 0.0), Vec2::new(0.25, 0.02), Vec2::new(0.27, 0.025)], 64, 1.0));
    let front = |x: f32| path_frames(&[Vec3::new(x + 0.37, 0.0, -8.1), Vec3::new(x - 0.37, 0.0, -8.1)]);
    for &x in &SPEAKERS_X {
        for k in 0..2 {
            let y = STAGE_Y + 0.51 + k as f32 * 1.04;
            cx.c.spawn((Mesh3d(cabinet.clone()), MeshMaterial3d(s.darksteel.clone()), Transform::from_xyz(x, y, -8.5)));
            cx.c.spawn((Mesh3d(cone.clone()), MeshMaterial3d(s.darksteel.clone()),
                Transform::from_xyz(x, y - 0.08, -8.13).with_rotation(Quat::from_rotation_x(FRAC_PI_2))));
            cx.put(sweep(&panel_profile(0.0, y - 0.46, y + 0.44), &front(x), false, false, uv(tile::GRILLE)), &s.grille);
            cx.put(tube(&[Vec3::new(x - 0.3, y + 0.43, -8.09), Vec3::new(x + 0.3, y + 0.43, -8.09)], 0.012, 8, 1.0), &neon.magenta);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bent_upholstery_faces_outward() {
        for (half, round, puff) in [(Vec3::new(0.3, 0.085, 0.3), 0.22, 0.5),
            (Vec3::new(0.2, 0.2, 0.065), 0.15, 0.45)] {
            let m = bend(shapes::cushion(half, round, puff, 56, 1.0), Vec3::new(6.0, 0.0, 8.15), 1.2, 1.0);
            let mut checked = 0;
            for t in m.indices.chunks_exact(3) {
                let a = Vec3::from(m.positions[t[0] as usize]);
                let b = Vec3::from(m.positions[t[1] as usize]);
                let c = Vec3::from(m.positions[t[2] as usize]);
                let face = (b - a).cross(c - a);
                let normal = Vec3::from(m.normals[t[0] as usize]);
                if face.length_squared() > 1.0e-12 {
                    assert!(face.dot(normal) > -1.0e-6, "bent cushion face points inward");
                    checked += 1;
                }
            }
            assert!(checked > 100);
        }
    }
    #[test]
    fn lounge_rugs_fit_the_room_and_gallery() {
        for &(x, y, z) in &LOUNGES {
            assert!(x.abs() + LOUNGE_RUG_RADIUS < ROOM_X);
            assert!(z.abs() + LOUNGE_RUG_RADIUS < ROOM_Z);
            if y > 1.0 {
                let outer_x = x.abs() + LOUNGE_RUG_RADIUS;
                assert!(z - LOUNGE_RUG_RADIUS > gallery_front(outer_x));
            }
        }
    }
    #[test]
    fn floor_leds_sit_above_dance_floor() {
        assert!(DANCE_TOP + FLOOR_LED_LIFT > DANCE_TOP + 0.001);
    }
}
