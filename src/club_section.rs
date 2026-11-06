//! Moonchild's upgraded club section: curved gallery, floating stairs,
//! leather lounges, semicircular bar, circular dance floor, the ceiling halo
//! and a curved DJ booth. All meshes are generated here; dimensions come from
//! `navigation::layout` so that collision matches the visible geometry.
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use bevy::prelude::*;
use crate::look::{tile, Surfaces};
use crate::navigation::layout::*;
use crate::shapes::{self, arc_frames, path_frames, rounded_rect, sweep, tube, Frame, MeshData, ProfilePoint};

/// Floor LED strips sit this far above the dance-floor surface (never inside it).
pub const FLOOR_LED_LIFT: f32 = 0.0025;

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
    for (i, &(x, y, z)) in LOUNGES.iter().enumerate() { lounge(&mut cx, s, neon, &bulb, Vec3::new(x, y, z), i); }
    bar(&mut cx, s, neon, &glass, &bulb);
    halo(&mut cx, s, neon, &bulb);
    booth(&mut cx, s, neon);
    stage_steps(&mut cx, s, neon);
    speakers(&mut cx, s, neon);
}

// ------------------------------------------------------------------ dance floor
fn dance_floor(cx: &mut Ctx, s: &Surfaces, neon: &Neon) {
    let c = Vec3::new(DANCE_CENTER.0, 0.0, DANCE_CENTER.1);
    cx.put(shapes::annulus(c + Vec3::Y * DANCE_TOP, 0.0, DANCE_RADIUS, 0.0, TAU, 128, uv(tile::FLOOR)), &s.dance_floor);
    // Rounded steel nosing around the disc.
    let edge = [Vec2::new(DANCE_RADIUS + 0.06, 0.0), Vec2::new(DANCE_RADIUS + 0.06, 0.012), Vec2::new(DANCE_RADIUS + 0.035, DANCE_TOP),
        Vec2::new(DANCE_RADIUS - 0.01, DANCE_TOP + 0.0005)];
    cx.put_at(shapes::lathe(&edge, 160, uv(tile::STEEL)), &s.steel, Transform::from_translation(c));
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
        for (y, r) in [(GALLERY_Y + 1.05, 0.032), (GALLERY_Y + 0.98, 0.012), (GALLERY_Y + 0.10, 0.012)] {
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
    let column = shapes::lathe(&[Vec2::new(0.24, 0.0), Vec2::new(0.24, 0.05), Vec2::new(0.14, 0.12), Vec2::new(0.13, 0.3),
        Vec2::new(0.13, GALLERY_UNDERSIDE - 0.3), Vec2::new(0.15, GALLERY_UNDERSIDE - 0.12), Vec2::new(0.22, GALLERY_UNDERSIDE)], 28, uv(tile::STEEL));
    let column = cx.mesh(column);
    for &x in &GALLERY_COLUMNS {
        cx.c.spawn((Mesh3d(column.clone()), MeshMaterial3d(s.steel.clone()), Transform::from_xyz(x, 0.0, column_z(x))));
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
fn lounge(cx: &mut Ctx, s: &Surfaces, neon: &Neon, bulb: &Handle<StandardMaterial>, centre: Vec3, index: usize) {
    let (a0, a1) = LOUNGE_ARC;
    let r = LOUNGE_RADIUS;
    // Plinth with a magenta glow underneath.
    let plinth = shifted(rounded_rect(0.62, 0.22, 0.04, 3), 0.05, 0.02);
    cx.put(sweep(&plinth, &arc_frames(centre, r, a0, a1, 48), true, true, uv(tile::DARKSTEEL)), &s.darksteel);
    cx.put(tube(&offset(&arc_frames(centre, r - 0.27, a0 + 0.03, a1 - 0.03, 48), 0.0, 0.03), 0.012, 8, 1.0), if centre.y > 1.0 { &neon.magenta } else { &neon.red });
    // Segmented seat and back cushions with rounded profiles.
    let cushions = 5;
    let gap = 0.012;
    for k in 0..cushions {
        let b0 = a0 + (a1 - a0) * k as f32 / cushions as f32 + gap;
        let b1 = a0 + (a1 - a0) * (k + 1) as f32 / cushions as f32 - gap;
        let seat = shifted(rounded_rect(0.6, 0.2, 0.08, 5), 0.02, 0.22);
        cx.put(sweep(&seat, &arc_frames(centre, r, b0, b1, 14), true, true, uv(tile::LEATHER)), &s.leather);
        let back = shifted(rounded_rect(0.2, 0.52, 0.09, 5), 0.36, 0.34);
        cx.put(sweep(&back, &arc_frames(centre, r, b0, b1, 14), true, true, uv(tile::LEATHER)), &s.leather);
    }
    // Rolled armrests at both ends.
    for &a in &[a0, a1] {
        let d = Vec3::new(a.cos(), 0.0, a.sin());
        let p0 = centre + d * (r - 0.28) + Vec3::Y * 0.5; let p1 = centre + d * (r + 0.36) + Vec3::Y * 0.5;
        cx.put(tube(&[p0, p1], 0.09, 18, uv(tile::LEATHER)), &s.leather);
    }
    // Round pedestal table with a lit glass candle.
    let table = shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.3, 0.0), Vec2::new(0.3, 0.025), Vec2::new(0.06, 0.06), Vec2::new(0.045, 0.1),
        Vec2::new(0.045, 0.62), Vec2::new(0.08, 0.66), Vec2::new(0.44, 0.67), Vec2::new(0.46, 0.69), Vec2::new(0.44, 0.71), Vec2::new(0.0, 0.71)], 40, uv(tile::DARKSTEEL));
    cx.put_at(table, &s.darksteel, Transform::from_translation(centre));
    cx.put_at(shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.035, 0.0), Vec2::new(0.04, 0.06), Vec2::new(0.0, 0.06)], 16, 1.0), bulb,
        Transform::from_translation(centre + Vec3::Y * 0.71));
    cx.c.spawn((PointLight { intensity: 3_500.0, range: 3.0, color: Color::srgb(1.0, 0.68, 0.38), shadow_maps_enabled: false, ..default() },
        Transform::from_translation(centre + Vec3::Y * 0.85)));
    // A warm pendant above each upper lounge.
    if centre.y > 1.0 {
        let shade = shapes::lathe(&[Vec2::new(0.02, 0.0), Vec2::new(0.24, 0.0), Vec2::new(0.25, 0.02), Vec2::new(0.12, 0.2), Vec2::new(0.03, 0.26)], 32, uv(tile::DARKSTEEL));
        let y = CEILING - 2.2;
        cx.put_at(shade, &s.darksteel, Transform::from_translation(Vec3::new(centre.x, y, centre.z)));
        cx.put(tube(&[Vec3::new(centre.x, y + 0.26, centre.z), Vec3::new(centre.x, CEILING, centre.z)], 0.006, 6, 1.0), &s.darksteel);
        cx.put_at(shapes::annulus(Vec3::ZERO, 0.0, 0.22, 0.0, TAU, 24, 1.0), bulb,
            Transform::from_xyz(centre.x, y + 0.01, centre.z).with_rotation(Quat::from_rotation_x(PI)));
        cx.c.spawn((SpotLight { intensity: 40_000.0, color: Color::srgb(1.0, 0.82, 0.62), range: 8.0, inner_angle: 0.45, outer_angle: 0.95,
            shadow_maps_enabled: index == 0, contact_shadows_enabled: true, ..default() },
            Transform::from_xyz(centre.x, y - 0.02, centre.z).looking_at(Vec3::new(centre.x, 0.0, centre.z), Vec3::Z)));
    }
}

// ------------------------------------------------------------------ bar
fn bar(cx: &mut Ctx, s: &Surfaces, neon: &Neon, glass: &[Handle<StandardMaterial>; 5], bulb: &Handle<StandardMaterial>) {
    let c = Vec3::new(BAR_CENTER.0, 0.0, BAR_CENTER.1);
    let r = BAR_RADIUS;
    // Path: wing -> semicircle -> wing, `side` points into the service area.
    let mut path: Vec<Vec3> = (0..=8).map(|i| Vec3::new(BAR_WING_END - (BAR_WING_END - c.x) * i as f32 / 8.0, 0.0, c.z + r)).collect();
    for i in 1..48 { let a = FRAC_PI_2 + PI * i as f32 / 48.0; path.push(c + Vec3::new(a.cos(), 0.0, a.sin()) * r); }
    path.extend((0..=8).map(|i| Vec3::new(c.x + (BAR_WING_END - c.x) * i as f32 / 8.0, 0.0, c.z - r)));
    let f = path_frames(&path);
    // Body, plinth, backlit perforated front, counter top, padded armrest and foot rail.
    cx.put(sweep(&shifted(rounded_rect(0.5, BAR_TOP - 0.14, 0.04, 3), 0.3, 0.08), &f, true, true, uv(tile::DARKSTEEL)), &s.darksteel);
    cx.put(sweep(&panel_profile(0.03, 0.12, 0.98), &f, false, false, uv(tile::BACKLIGHT)), &s.backlight_magenta);
    cx.put(sweep(&panel_profile(0.0, 0.1, 1.0), &f, false, false, uv(tile::GRILLE)), &s.grille);
    cx.put(sweep(&shifted(rounded_rect(0.78, 0.06, 0.025, 4), 0.26, BAR_TOP - 0.06), &f, true, true, uv(tile::FLOOR)), &s.floor);
    cx.put(tube(&offset(&f, -0.08, BAR_TOP + 0.03), 0.045, 18, uv(tile::LEATHER)), &s.leather);
    cx.put(tube(&offset(&f, -0.06, BAR_TOP - 0.085), 0.01, 8, 1.0), &neon.warm);
    cx.put(tube(&offset(&f, 0.02, 0.035), 0.012, 8, 1.0), &neon.cyan);
    cx.put(tube(&offset(&f, -0.3, 0.22), 0.025, 12, uv(tile::STEEL)), &s.steel);
    for fr in f.iter().step_by(6) {
        let a = fr.pos + fr.side * -0.3 + Vec3::Y * 0.22; let b = fr.pos + fr.side * 0.02 + Vec3::Y * 0.14;
        cx.put(tube(&[a, b], 0.014, 8, 1.0), &s.steel);
    }
    // Back bar: lower cabinet, glowing amber panel, steel shelves and glass bottles.
    let wall_x = ROOM_X;
    cx.put_at(shapes::rounded_box(Vec3::new(0.6, 0.95, 3.6), 0.04, 3, uv(tile::DARKSTEEL)), &s.darksteel, Transform::from_xyz(wall_x - 0.3, 0.475, c.z));
    cx.put_at(shapes::rounded_box(Vec3::new(0.64, 0.04, 3.64), 0.015, 2, uv(tile::FLOOR)), &s.floor, Transform::from_xyz(wall_x - 0.32, 0.97, c.z));
    cx.put(sweep(&panel_facing(0.0, 1.05, 2.75, Vec2::X), &path_frames(&[Vec3::new(wall_x - 0.02, 0.0, c.z - 1.7), Vec3::new(wall_x - 0.02, 0.0, c.z + 1.7)]), false, false, uv(tile::BACKLIGHT)), &s.backlight_amber);
    let bottle = cx.mesh(shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.034, 0.0), Vec2::new(0.037, 0.012), Vec2::new(0.037, 0.19), Vec2::new(0.03, 0.225),
        Vec2::new(0.014, 0.26), Vec2::new(0.012, 0.31), Vec2::new(0.015, 0.318), Vec2::new(0.0, 0.322)], 20, 1.0));
    let mut k = 0;
    for &y in &[1.28, 1.74, 2.2] {
        cx.put_at(shapes::rounded_box(Vec3::new(0.32, 0.03, 3.3), 0.012, 2, uv(tile::STEEL)), &s.steel, Transform::from_xyz(wall_x - 0.2, y, c.z));
        cx.put(tube(&[Vec3::new(wall_x - 0.36, y - 0.02, c.z - 1.62), Vec3::new(wall_x - 0.36, y - 0.02, c.z + 1.62)], 0.007, 6, 1.0), &neon.cyan);
        for i in 0..14 {
            let z = c.z - 1.5 + i as f32 * 0.23;
            let scale = 0.85 + 0.3 * (((i * 7 + k * 3) % 5) as f32 / 4.0);
            cx.c.spawn((Mesh3d(bottle.clone()), MeshMaterial3d(glass[(i + k) % 5].clone()),
                Transform::from_xyz(wall_x - 0.2 + if i % 2 == 0 { 0.05 } else { -0.04 }, y + 0.015, z).with_scale(Vec3::new(1.0, scale, 1.0))));
        }
        k += 1;
    }
    // Bar stools around the curve.
    let stool = shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.21, 0.0), Vec2::new(0.22, 0.012), Vec2::new(0.2, 0.025), Vec2::new(0.03, 0.035),
        Vec2::new(0.025, 0.06), Vec2::new(0.025, 0.66), Vec2::new(0.12, 0.68), Vec2::new(0.0, 0.69)], 28, uv(tile::STEEL));
    let stool = cx.mesh(stool);
    let cushion = cx.mesh(shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.17, 0.0), Vec2::new(0.195, 0.025), Vec2::new(0.2, 0.06),
        Vec2::new(0.185, 0.09), Vec2::new(0.12, 0.105), Vec2::new(0.0, 0.11)], 32, uv(tile::LEATHER)));
    let footring = cx.mesh(tube(&(0..=32).map(|i| { let a = TAU * i as f32 / 32.0; Vec3::new(a.cos() * 0.16, 0.0, a.sin() * 0.16) }).collect::<Vec<_>>(), 0.011, 8, 1.0));
    for &a in &BAR_STOOL_ANGLES {
        let p = c + Vec3::new(a.cos(), 0.0, a.sin()) * (r + 0.48);
        cx.c.spawn((Mesh3d(stool.clone()), MeshMaterial3d(s.steel.clone()), Transform::from_translation(p)));
        cx.c.spawn((Mesh3d(cushion.clone()), MeshMaterial3d(s.leather.clone()), Transform::from_translation(p + Vec3::Y * 0.68)));
        cx.c.spawn((Mesh3d(footring.clone()), MeshMaterial3d(s.steel.clone()), Transform::from_translation(p + Vec3::Y * 0.3)));
    }
    // Three pendant lamps above the counter; the middle one casts shadows.
    let shade = cx.mesh(shapes::lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.16, 0.0), Vec2::new(0.17, 0.015), Vec2::new(0.1, 0.24), Vec2::new(0.02, 0.3)], 32, uv(tile::STEEL)));
    for (i, &a) in [2.4f32, PI, 3.88].iter().enumerate() {
        let p = c + Vec3::new(a.cos(), 0.0, a.sin()) * (r + 0.05);
        let y = 2.35;
        cx.c.spawn((Mesh3d(shade.clone()), MeshMaterial3d(s.steel.clone()), Transform::from_xyz(p.x, y, p.z)));
        cx.put(tube(&[Vec3::new(p.x, y + 0.3, p.z), Vec3::new(p.x, CEILING, p.z)], 0.005, 6, 1.0), &s.darksteel);
        cx.put_at(shapes::annulus(Vec3::ZERO, 0.0, 0.15, 0.0, TAU, 20, 1.0), bulb, Transform::from_xyz(p.x, y + 0.01, p.z).with_rotation(Quat::from_rotation_x(PI)));
        cx.c.spawn((SpotLight { intensity: 30_000.0, color: Color::srgb(1.0, 0.8, 0.58), range: 6.0, inner_angle: 0.35, outer_angle: 0.85,
            shadow_maps_enabled: i == 1, contact_shadows_enabled: true, ..default() },
            Transform::from_xyz(p.x, y - 0.02, p.z).looking_at(Vec3::new(p.x, 0.0, p.z), Vec3::Z)));
    }
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
    cx.put(sweep(&ring, &arc_frames(c, r, 0.0, TAU, 160), true, false, uv(tile::DARKSTEEL)), &s.darksteel);
    // Neon rings underneath and an inner floating halo.
    for (rr, y, mat) in [(r - 0.16, -0.012, &neon.cyan), (r + 0.16, -0.012, &neon.magenta), (r - 0.6, 0.06, &neon.cyan)] {
        let pts: Vec<Vec3> = (0..=160).map(|i| { let a = TAU * i as f32 / 160.0; c + Vec3::new(a.cos() * rr, y, a.sin() * rr) }).collect();
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
        cx.put(tube(&[Vec3::new(hx, y, hz) + d * r0, Vec3::new(hx, y, hz) + d * r1], 0.018, 8, 1.0), if i % 2 == 0 { &neon.blue } else { &neon.magenta });
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
pub fn moving_head_meshes(meshes: &mut Assets<Mesh>, s: &Surfaces, lens: Handle<StandardMaterial>) -> Vec<(Handle<Mesh>, Handle<StandardMaterial>, Transform)> {
    let body = shapes::lathe(&[Vec2::new(0.0, -0.18), Vec2::new(0.1, -0.18), Vec2::new(0.13, -0.1), Vec2::new(0.14, 0.12), Vec2::new(0.12, 0.17), Vec2::new(0.0, 0.17)], 24, uv(tile::DARKSTEEL)).build();
    let lens_mesh = shapes::annulus(Vec3::ZERO, 0.0, 0.11, 0.0, TAU, 20, 1.0).build();
    // Lathe axis Y -> beam axis -Z.
    let to_beam = Quat::from_rotation_x(-FRAC_PI_2);
    vec![
        (meshes.add(body), s.darksteel.clone(), Transform::from_rotation(to_beam)),
        (meshes.add(lens_mesh), lens, Transform::from_xyz(0.0, 0.0, -0.172).with_rotation(Quat::from_rotation_x(FRAC_PI_2))),
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
    let f = arc_frames(centre, BOOTH_RADIUS, a0, a1, 40);
    let h = BOOTH_TOP - STAGE_Y;
    cx.put(sweep(&shifted(rounded_rect(0.84, h, 0.12, 6), -0.07, 0.0), &f, true, true, uv(tile::DARKSTEEL)), &s.darksteel);
    // Backlit perforated front, slightly in front of the body.
    let inner = arc_frames(centre, BOOTH_RADIUS, a0 + 0.02, a1 - 0.02, 40);
    cx.put(sweep(&panel_facing(0.36, 0.16, h - 0.22, Vec2::X), &inner, false, false, uv(tile::BACKLIGHT)), &s.backlight_cyan);
    cx.put(sweep(&panel_facing(0.375, 0.14, h - 0.2, Vec2::X), &inner, false, false, uv(tile::GRILLE)), &s.grille);
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
    let cabinet = cx.mesh(shapes::rounded_box(Vec3::new(0.83, 1.02, 0.78), 0.06, 4, uv(tile::DARKSTEEL)));
    let cone = cx.mesh(shapes::lathe(&[Vec2::new(0.0, -0.05), Vec2::new(0.05, -0.045), Vec2::new(0.2, 0.0), Vec2::new(0.25, 0.02), Vec2::new(0.27, 0.025)], 32, 1.0));
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
    fn floor_leds_sit_above_dance_floor() {
        assert!(DANCE_TOP + FLOOR_LED_LIFT > DANCE_TOP + 0.001);
    }
}
