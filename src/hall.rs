//! Moonchild hall shell: wall fins and backlit bays, cornice, ceiling ducts,
//! the stage back wall with its truss portal, the stage riser front and the
//! crescent-moon sign. Everything is generated geometry; the only collision
//! relevant parts (the two truss towers) are mirrored in `navigation`.
use std::f32::consts::{FRAC_PI_2, PI};
use bevy::prelude::*;
use crate::club_section::Neon;
use crate::look::{tile, Surfaces};
use crate::lod;
use crate::navigation::layout::*;
use crate::shapes::{self, MeshData};

/// Hanging positions of the four stage-truss moving heads.
pub fn stage_head_positions() -> Vec<Vec3> {
    [-4.5, -1.5, 1.5, 4.5].iter().map(|&x| Vec3::new(x, TRUSS_Y - 0.5, TRUSS_Z + 0.02)).collect()
}

struct B<'a, 'w, 's> { c: &'a mut Commands<'w, 's>, meshes: &'a mut Assets<Mesh> }
impl B<'_, '_, '_> {
    /// Rounded box centred at `pos`; UVs in metres scaled by `1/tile`.
    fn bx(&mut self, size: Vec3, r: f32, mat: &Handle<StandardMaterial>, pos: Vec3, tile_m: f32) {
        let mesh = self.meshes.add(shapes::rounded_box(size, r.min(size.min_element() * 0.45), 2, 1.0 / tile_m).build());
        self.c.spawn((Mesh3d(mesh), MeshMaterial3d(mat.clone()), Transform::from_translation(pos)));
    }
    fn put(&mut self, m: MeshData, mat: &Handle<StandardMaterial>) {
        self.c.spawn((Mesh3d(self.meshes.add(m.build())), MeshMaterial3d(mat.clone())));
    }
}

fn merge(into: &mut MeshData, other: MeshData) {
    let base = into.positions.len() as u32;
    into.positions.extend(other.positions);
    into.normals.extend(other.normals);
    into.uvs.extend(other.uvs);
    into.indices.extend(other.indices.into_iter().map(|i| i + base));
}

/// Square box truss between `a` and `b` with chords and zig-zag bracing.
fn truss(a: Vec3, b: Vec3, half: f32, chord_sides: usize, brace_sides: usize) -> MeshData {
    let axis = (b - a).normalize();
    let up = if axis.y.abs() > 0.9 { Vec3::Z } else { Vec3::Y };
    let u = axis.cross(up).normalize() * half;
    let v = axis.cross(u).normalize() * half;
    let corners = [u + v, u - v, -u - v, -u + v];
    let mut m = MeshData::default();
    for c in corners { merge(&mut m, shapes::tube(&[a + c, b + c], 0.022, chord_sides, 1.0)); }
    let len = a.distance(b);
    let n = (len / 0.3).round().max(1.0) as usize;
    for k in 0..n {
        let (p0, p1) = (a.lerp(b, k as f32 / n as f32), a.lerp(b, (k + 1) as f32 / n as f32));
        for f in 0..4 {
            let (c0, c1) = (corners[f], corners[(f + 1) % 4]);
            let (s, e) = if k % 2 == 0 { (c0, c1) } else { (c1, c0) };
            merge(&mut m, shapes::tube(&[p0 + s, p1 + e], 0.009, brace_sides, 1.0));
        }
    }
    m
}

/// Crescent in the XY plane (tips at top and bottom), visible from -Z.
fn crescent(center: Vec3, radius: f32, thickness: f32) -> MeshData {
    let n = 48;
    let outer: Vec<Vec3> = (0..=n).map(|i| { let a = FRAC_PI_2 + PI * i as f32 / n as f32; center + Vec3::new(a.cos(), a.sin(), 0.0) * radius }).collect();
    let inner: Vec<Vec3> = (0..=n).map(|i| { let a = FRAC_PI_2 + PI * i as f32 / n as f32;
        center + Vec3::new(a.cos() * radius * (1.0 - thickness), a.sin() * radius, 0.0) }).collect();
    shapes::ribbon(&outer, &inner, Vec3::NEG_Z, 1.0)
}

/// Dimmed copy of a backlight material for large wall bays, so they glow instead of glare.
fn dimmed(materials: &mut Assets<StandardMaterial>, src: &Handle<StandardMaterial>, k: f32) -> Handle<StandardMaterial> {
    let mut m = materials.get(src).cloned().unwrap_or_default();
    let e = m.emissive;
    m.emissive = LinearRgba::rgb(e.red * k, e.green * k, e.blue * k);
    m.base_color = Color::srgb(0.12, 0.12, 0.13);
    materials.add(m)
}

struct Bays { cyan: Handle<StandardMaterial>, magenta: Handle<StandardMaterial>, amber: Handle<StandardMaterial> }

pub fn spawn(c: &mut Commands, meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>, s: &Surfaces, neon: &Neon) {
    let bays = Bays {
        cyan: dimmed(materials, &s.backlight_cyan, 0.3),
        magenta: dimmed(materials, &s.backlight_magenta, 0.3),
        amber: dimmed(materials, &s.backlight_amber, 0.45),
    };
    let mut b = B { c, meshes };
    walls(&mut b, s, &bays, neon);
    ceiling(&mut b, s, neon);
    back_wall(&mut b, s, &bays, neon);
    stage_front(&mut b, s, neon);
}

// ------------------------------------------------------------------ walls
fn walls(b: &mut B, s: &Surfaces, bay: &Bays, neon: &Neon) {
    let len_x = ROOM_X * 2.0 - 0.1;
    let len_z = ROOM_Z * 2.0 - 0.1;
    // Skirting and cornice with a warm cove line on all four walls.
    for sx in [-1.0, 1.0] {
        b.bx(Vec3::new(0.12, 0.26, len_z), 0.03, &s.darksteel, Vec3::new(sx * (ROOM_X - 0.06), 0.13, 0.0), tile::DARKSTEEL);
        b.bx(Vec3::new(0.3, 0.24, len_z), 0.05, &s.steel, Vec3::new(sx * (ROOM_X - 0.15), CEILING - 0.14, 0.0), tile::STEEL);
        b.put(shapes::tube(&[Vec3::new(sx * (ROOM_X - 0.31), CEILING - 0.14, -ROOM_Z + 0.1), Vec3::new(sx * (ROOM_X - 0.31), CEILING - 0.14, ROOM_Z - 0.1)], 0.016, 8, 1.0), &neon.warm);
    }
    for sz in [-1.0, 1.0] {
        b.bx(Vec3::new(len_x, 0.26, 0.12), 0.03, &s.darksteel, Vec3::new(0.0, 0.13, sz * (ROOM_Z - 0.06)), tile::DARKSTEEL);
        b.bx(Vec3::new(len_x, 0.24, 0.3), 0.05, &s.steel, Vec3::new(0.0, CEILING - 0.14, sz * (ROOM_Z - 0.15)), tile::STEEL);
        b.put(shapes::tube(&[Vec3::new(-ROOM_X + 0.1, CEILING - 0.14, sz * (ROOM_Z - 0.31)), Vec3::new(ROOM_X - 0.1, CEILING - 0.14, sz * (ROOM_Z - 0.31))], 0.016, 8, 1.0), &neon.warm);
    }

    // Side walls: steel fins, backlit upper bays, perforated lower bays.
    let bays: [(f32, f32); 3] = [(-9.5, -8.3), (-7.3, -0.45), (0.45, 9.0)];
    for sx in [-1.0f32, 1.0] {
        let upper = if sx < 0.0 { &bay.cyan } else { &bay.magenta };
        for (z0, z1) in bays {
            let n = ((z1 - z0) / 1.5).round().max(1.0) as usize;
            for k in 0..=n {
                let z = z0 + (z1 - z0) * k as f32 / n as f32;
                b.bx(Vec3::new(0.16, 6.2, 0.16), 0.04, &s.darksteel, Vec3::new(sx * (ROOM_X - 0.08), 3.4, z), tile::DARKSTEEL);
            }
            for k in 0..n {
                let zc = z0 + (z1 - z0) * (k as f32 + 0.5) / n as f32;
                let w = (z1 - z0) / n as f32 - 0.2;
                b.bx(Vec3::new(0.03, 2.4, w), 0.01, upper, Vec3::new(sx * (ROOM_X - 0.03), 4.95, zc), tile::BACKLIGHT);
                b.bx(Vec3::new(0.025, 2.4, w), 0.005, &s.grille, Vec3::new(sx * (ROOM_X - 0.075), 4.95, zc), tile::GRILLE);
                // The back bar occupies the lower part of the right wall.
                if sx > 0.0 && zc > 0.0 && zc < 5.1 { continue; }
                // Lower bays: vertically fluted smoked-oak wainscot.
                let x = sx * (ROOM_X - 0.02);
                let fr = shapes::path_frames(&[Vec3::new(x, 0.0, zc - w * 0.5), Vec3::new(x, 0.0, zc + w * 0.5)])
                    .into_iter().map(|f| shapes::Frame { side: Vec3::new(-sx, 0.0, 0.0), ..f }).collect::<Vec<_>>();
                b.put(shapes::fluted_skin(&fr, 0.3, 2.45, 0.07, 0.02, 1.0 / tile::OAK), &s.oak);
                b.bx(Vec3::new(0.05, 0.03, w + 0.02), 0.01, &s.brass, Vec3::new(sx * (ROOM_X - 0.03), 2.47, zc), tile::BRASS);
            }
        }
    }
    // Entrance wall: amber panels with fins, a crescent moon in the middle.
    for k in 0..=10 {
        let x = -10.0 + 2.0 * k as f32;
        if (x.abs() - 10.0).abs() < 0.1 { continue; }
        b.bx(Vec3::new(0.16, 6.2, 0.16), 0.04, &s.darksteel, Vec3::new(x, 3.4, ROOM_Z - 0.08), tile::DARKSTEEL);
    }
    for k in 0..10 {
        let x = -9.0 + 2.0 * k as f32;
        if x.abs() < 1.5 { continue; }
        b.bx(Vec3::new(1.8, 2.4, 0.03), 0.01, &bay.amber, Vec3::new(x, 4.95, ROOM_Z - 0.03), tile::BACKLIGHT);
        b.bx(Vec3::new(1.8, 2.4, 0.025), 0.005, &s.grille, Vec3::new(x, 4.95, ROOM_Z - 0.075), tile::GRILLE);
        let fr = shapes::path_frames(&[Vec3::new(x - 0.9, 0.0, ROOM_Z - 0.02), Vec3::new(x + 0.9, 0.0, ROOM_Z - 0.02)])
            .into_iter().map(|f| shapes::Frame { side: Vec3::NEG_Z, ..f }).collect::<Vec<_>>();
        b.put(shapes::fluted_skin(&fr, 0.3, 2.45, 0.07, 0.02, 1.0 / tile::OAK), &s.oak);
        b.bx(Vec3::new(1.82, 0.03, 0.05), 0.01, &s.brass, Vec3::new(x, 2.47, ROOM_Z - 0.03), tile::BRASS);
    }
    let moon = Vec3::new(0.0, 4.9, ROOM_Z - 0.05);
    b.put(crescent(moon, 1.25, 0.55), &neon.magenta);
    b.put(crescent(moon + Vec3::Z * 0.004, 1.34, 0.62), &s.darksteel);
}

// ------------------------------------------------------------------ ceiling
fn ceiling(b: &mut B, s: &Surfaces, _neon: &Neon) {
    // Two large ventilation ducts with flanges and hangers, below the beams.
    for sx in [-1.0f32, 1.0] {
        let x = sx * 7.6;
        b.put(shapes::tube(&[Vec3::new(x, 6.1, -9.3), Vec3::new(x, 6.1, 9.3)], 0.3, 20, 1.0 / tile::STEEL), &s.steel);
        for k in 0..13 {
            let z = -9.0 + 1.5 * k as f32;
            b.put(shapes::tube(&[Vec3::new(x, 6.1, z - 0.04), Vec3::new(x, 6.1, z + 0.04)], 0.335, 20, 1.0 / tile::DARKSTEEL), &s.darksteel);
            if k % 2 == 0 {
                b.put(shapes::tube(&[Vec3::new(x - 0.2, 6.35, z), Vec3::new(x - 0.2, 6.9, z)], 0.012, 6, 1.0), &s.darksteel);
                b.put(shapes::tube(&[Vec3::new(x + 0.2, 6.35, z), Vec3::new(x + 0.2, 6.9, z)], 0.012, 6, 1.0), &s.darksteel);
            }
        }
    }
    // Cable tray above the entrance side of the room.
    b.bx(Vec3::new(0.6, 0.08, 17.0), 0.02, &s.grille, Vec3::new(0.0, 6.88, 0.0), tile::GRILLE);
}

// ------------------------------------------------------------------ stage back wall
fn back_wall(b: &mut B, s: &Surfaces, bay: &Bays, neon: &Neon) {
    let zw = -ROOM_Z;
    // Backlit acoustic panels either side of the LED wall.
    for sx in [-1.0f32, 1.0] {
        for (i, x) in [6.8f32, 8.7, 10.6].into_iter().enumerate() {
            let mat = if i == 1 { &bay.magenta } else { &bay.cyan };
            b.bx(Vec3::new(1.7, 4.6, 0.04), 0.01, mat, Vec3::new(sx * x, 3.6, zw + 0.05), tile::BACKLIGHT);
            b.bx(Vec3::new(1.7, 4.6, 0.03), 0.005, &s.grille, Vec3::new(sx * x, 3.6, zw + 0.11), tile::GRILLE);
            b.bx(Vec3::new(0.12, 4.8, 0.14), 0.03, &s.darksteel, Vec3::new(sx * (x - 0.95), 3.6, zw + 0.08), tile::DARKSTEEL);
            // Layered arched surrounds add depth without projecting into the stage.
            let base = Vec3::new(sx * x, 1.32, zw + 0.16);
            let outline = shapes::arch_outline(base, Vec3::X, 1.62, 4.42, 32);
            b.put(shapes::tube(&outline, 0.027, 12, 1.0 / tile::BRASS), &s.brass);
            let inner = shapes::arch_outline(base + Vec3::new(0.0,0.10,0.012), Vec3::X, 1.44, 4.18, 32);
            b.put(shapes::tube(&inner, 0.009, 8, 1.0), &neon.warm);
        }
        b.bx(Vec3::new(0.12, 4.8, 0.14), 0.03, &s.darksteel, Vec3::new(sx * 11.55, 3.6, zw + 0.08), tile::DARKSTEEL);
    }
    // Concrete plinth with a LED line below the LED wall.
    b.bx(Vec3::new(12.2, 0.5, 0.22), 0.05, &s.darksteel, Vec3::new(0.0, 2.35, zw + 0.11), tile::DARKSTEEL);
    b.put(shapes::tube(&[Vec3::new(-5.9, 2.58, zw + 0.23), Vec3::new(5.9, 2.58, zw + 0.23)], 0.014, 8, 1.0), &neon.cyan);

    // Truss portal: two towers and a cross beam, with hanging LED bars.
    // Two levels of detail: round chords up close, 4/3-sided tubes from afar.
    let portal = |chord: usize, brace: usize| {
        let mut t = MeshData::default();
        for sx in [-1.0f32, 1.0] {
            merge(&mut t, truss(Vec3::new(sx * TRUSS_X, STAGE_Y, TRUSS_Z), Vec3::new(sx * TRUSS_X, TRUSS_Y, TRUSS_Z), 0.15, chord, brace));
        }
        merge(&mut t, truss(Vec3::new(-TRUSS_X, TRUSS_Y, TRUSS_Z), Vec3::new(TRUSS_X, TRUSS_Y, TRUSS_Z), 0.15, chord, brace));
        t
    };
    let (hi, lo) = (b.meshes.add(portal(12, 8).build()), b.meshes.add(portal(4, 3).build()));
    b.c.spawn((Mesh3d(hi), MeshMaterial3d(s.darksteel.clone()), lod::near(lod::LARGE, true)));
    b.c.spawn((Mesh3d(lo), MeshMaterial3d(s.darksteel.clone()), lod::far(lod::LARGE, true)));
    for sx in [-1.0f32, 1.0] {
        b.bx(Vec3::new(0.6, 0.06, 0.6), 0.02, &s.darksteel, Vec3::new(sx * TRUSS_X, STAGE_Y + 0.03, TRUSS_Z), tile::DARKSTEEL);
    }
    for (i, x) in [-5.4f32, -2.7, 0.0, 2.7, 5.4].into_iter().enumerate() {
        b.bx(Vec3::new(1.2, 0.12, 0.14), 0.03, &s.darksteel, Vec3::new(x, TRUSS_Y - 0.28, TRUSS_Z + 0.2), tile::DARKSTEEL);
        let mat = if i % 2 == 0 { &neon.cyan } else { &neon.magenta };
        b.put(shapes::tube(&[Vec3::new(x - 0.5, TRUSS_Y - 0.28, TRUSS_Z + 0.28), Vec3::new(x + 0.5, TRUSS_Y - 0.28, TRUSS_Z + 0.28)], 0.02, 8, 1.0), mat);
    }
    // Hangers for the stage moving heads.
    for p in stage_head_positions() {
        b.put(shapes::tube(&[Vec3::new(p.x, TRUSS_Y - 0.15, p.z), Vec3::new(p.x, p.y + 0.3, p.z)], 0.03, 10, 1.0 / tile::DARKSTEEL), &s.darksteel);
        b.bx(Vec3::new(0.34, 0.1, 0.3), 0.03, &s.darksteel, Vec3::new(p.x, p.y + 0.3, p.z), tile::DARKSTEEL);
    }
}

// ------------------------------------------------------------------ stage riser
fn stage_front(b: &mut B, s: &Surfaces, _neon: &Neon) {
    let z = STAGE_Z.1 + 0.015;
    for i in 0..8 {
        let x = -6.1 + 1.75 * i as f32;
        let mat = if i % 2 == 0 { &s.backlight_magenta } else { &s.backlight_cyan };
        b.bx(Vec3::new(1.5, 0.56, 0.03), 0.01, mat, Vec3::new(x, 0.42, z), tile::BACKLIGHT);
        b.bx(Vec3::new(1.5, 0.56, 0.02), 0.005, &s.grille, Vec3::new(x, 0.42, z + 0.025), tile::GRILLE);
        b.bx(Vec3::new(0.12, 0.8, 0.05), 0.02, &s.steel, Vec3::new(x + 0.875, 0.45, z + 0.01), tile::STEEL);
    }
    b.bx(Vec3::new(0.12, 0.8, 0.05), 0.02, &s.steel, Vec3::new(-6.975, 0.45, z + 0.01), tile::STEEL);
    // Side cladding projects less than 3 cm beyond the existing stage footprint;
    // this stays within the player radius and clears the right-hand stage steps.
    for sx in [-1.0f32, 1.0] {
        let x = sx * (STAGE_X + 0.004);
        let frames = shapes::path_frames(&[Vec3::new(x, 0.0, STAGE_Z.0 + 0.05), Vec3::new(x, 0.0, STAGE_Z.1 - 0.05)])
            .into_iter().map(|f| shapes::Frame { side: Vec3::new(sx, 0.0, 0.0), ..f }).collect::<Vec<_>>();
        b.put(shapes::fluted_skin(&frames, 0.10, 0.84, 0.14, 0.018, 1.0 / tile::OAK), &s.oak);
        for y in [0.10, 0.84] {
            b.put(shapes::tube(&[Vec3::new(x, y, STAGE_Z.0 + 0.04), Vec3::new(x, y, STAGE_Z.1 - 0.04)],
                0.012, 8, 1.0 / tile::BRASS), &s.brass);
        }
    }
}
