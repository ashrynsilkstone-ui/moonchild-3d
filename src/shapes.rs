//! Procedural mesh builders for Moonchild's curved architecture.
//!
//! All builders produce smooth normals, UVs in metres multiplied by a
//! material-specific scale (so a texture keeps the same physical size on
//! every object) and MikkTSpace tangents for normal mapping.
use std::f32::consts::TAU;
use bevy::{asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology};

#[derive(Default)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn vertex(&mut self, p: Vec3, n: Vec3, uv: Vec2) -> u32 {
        self.positions.push(p.to_array());
        self.normals.push(n.normalize_or(Vec3::Y).to_array());
        self.uvs.push(uv.to_array());
        (self.positions.len() - 1) as u32
    }
    pub fn tri(&mut self, a: u32, b: u32, c: u32) { self.indices.extend([a, b, c]); }
    /// Grid of `rows x cols` vertices starting at `start`, connected as quads.
    /// Winding is chosen so that `cross(d_row, d_col)` faces outward.
    fn grid(&mut self, start: u32, rows: usize, cols: usize, wrap_cols: bool) {
        let ccount = if wrap_cols { cols } else { cols - 1 };
        for r in 0..rows - 1 {
            for c in 0..ccount {
                let c1 = (c + 1) % cols;
                let a = start + (r * cols + c) as u32;
                let b = start + (r * cols + c1) as u32;
                let d = start + ((r + 1) * cols + c) as u32;
                let e = start + ((r + 1) * cols + c1) as u32;
                self.indices.extend([a, d, b, b, d, e]);
            }
        }
    }
    pub fn build(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
            .with_inserted_indices(Indices::U32(self.indices));
        let _ = mesh.generate_tangents();
        mesh
    }
}

/// A point of a 2D cross-section: `p.x` lateral, `p.y` up, with its outward normal.
#[derive(Clone, Copy)]
pub struct ProfilePoint { pub p: Vec2, pub n: Vec2 }

/// Rounded rectangle profile (counter-clockwise, starting bottom-right),
/// `w` wide, `h` high, centred on x, with y from 0 to `h`.
pub fn rounded_rect(w: f32, h: f32, r: f32, seg: usize) -> Vec<ProfilePoint> {
    let r = r.min(w * 0.5).min(h * 0.5);
    let corners = [
        (Vec2::new(w * 0.5 - r, r), -0.25),
        (Vec2::new(w * 0.5 - r, h - r), 0.0),
        (Vec2::new(-w * 0.5 + r, h - r), 0.25),
        (Vec2::new(-w * 0.5 + r, r), 0.5),
    ];
    let mut out = vec![];
    for (c, start) in corners {
        for i in 0..=seg {
            let a = (start + 0.25 * i as f32 / seg as f32) * TAU;
            let n = Vec2::new(a.cos(), a.sin());
            out.push(ProfilePoint { p: c + n * r, n });
        }
    }
    out
}

/// Profile from an explicit polyline with smooth normals (for open profiles).
pub fn smooth_profile(points: &[Vec2], closed: bool) -> Vec<ProfilePoint> {
    let n = points.len();
    (0..n).map(|i| {
        let prev = if i == 0 { if closed { points[n - 1] } else { points[0] } } else { points[i - 1] };
        let next = if i == n - 1 { if closed { points[0] } else { points[n - 1] } } else { points[i + 1] };
        let t = (next - prev).normalize_or(Vec2::X);
        // Counter-clockwise profiles have their outside on the right of travel.
        ProfilePoint { p: points[i], n: Vec2::new(t.y, -t.x) }
    }).collect()
}

/// A frame along a sweep path: position, lateral axis and up axis.
#[derive(Clone, Copy)]
pub struct Frame { pub pos: Vec3, pub side: Vec3, pub up: Vec3 }

/// Frames along a horizontal circular arc; `side` points away from the centre.
pub fn arc_frames(center: Vec3, radius: f32, a0: f32, a1: f32, segments: usize) -> Vec<Frame> {
    (0..=segments).map(|i| {
        let a = a0 + (a1 - a0) * i as f32 / segments as f32;
        let dir = Vec3::new(a.cos(), 0.0, a.sin());
        Frame { pos: center + dir * radius, side: dir, up: Vec3::Y }
    }).collect()
}

/// Frames along an arbitrary horizontal path, side = right of travel direction.
pub fn path_frames(points: &[Vec3]) -> Vec<Frame> {
    let n = points.len();
    (0..n).map(|i| {
        let a = points[i.saturating_sub(1)];
        let b = points[(i + 1).min(n - 1)];
        let t = (b - a).normalize_or(Vec3::X);
        let side = t.cross(Vec3::Y).normalize_or(Vec3::Z);
        Frame { pos: points[i], side, up: Vec3::Y }
    }).collect()
}

/// Sweep a closed or open profile along frames. `caps` closes both ends with flat fans.
pub fn sweep(profile: &[ProfilePoint], frames: &[Frame], closed_profile: bool, caps: bool, uv_scale: f32) -> MeshData {
    let mut m = MeshData::default();
    // Arc-length along the profile for the U coordinate.
    let mut prof_len = vec![0.0];
    for i in 1..profile.len() { prof_len.push(prof_len[i - 1] + profile[i].p.distance(profile[i - 1].p)); }
    let mut path_len = vec![0.0];
    for i in 1..frames.len() { path_len.push(path_len[i - 1] + frames[i].pos.distance(frames[i - 1].pos)); }
    // Orientation check: cross(d_path, d_profile) must face along the outward normal.
    let start = m.positions.len() as u32;
    for (fi, f) in frames.iter().enumerate() {
        for (pi, pp) in profile.iter().enumerate() {
            let pos = f.pos + f.side * pp.p.x + f.up * pp.p.y;
            let n = f.side * pp.n.x + f.up * pp.n.y;
            m.vertex(pos, n, Vec2::new(path_len[fi], prof_len[pi]) * uv_scale);
        }
    }
    let cols = profile.len();
    m.grid(start, frames.len(), cols, closed_profile);
    // Fix winding by testing the first quad against its normal.
    if m.indices.len() >= 3 {
        let i = &m.indices;
        let (a, b, c) = (Vec3::from(m.positions[i[0] as usize]), Vec3::from(m.positions[i[1] as usize]), Vec3::from(m.positions[i[2] as usize]));
        let n = Vec3::from(m.normals[i[0] as usize]);
        if (b - a).cross(c - a).dot(n) < 0.0 {
            for t in m.indices.chunks_mut(3) { t.swap(1, 2); }
        }
    }
    if caps {
        for (end, f) in [(0usize, frames[0]), (1, frames[frames.len() - 1])] {
            let forward = f.up.cross(f.side).normalize();
            let normal = if end == 0 { forward } else { -forward };
            let centre: Vec2 = profile.iter().map(|p| p.p).sum::<Vec2>() / profile.len() as f32;
            let c = m.vertex(f.pos + f.side * centre.x + f.up * centre.y, normal, centre * uv_scale);
            let ring: Vec<u32> = profile.iter().map(|pp| m.vertex(f.pos + f.side * pp.p.x + f.up * pp.p.y, normal, pp.p * uv_scale)).collect();
            for i in 0..ring.len() {
                let j = (i + 1) % ring.len();
                let (a, b) = (Vec3::from(m.positions[ring[i] as usize]), Vec3::from(m.positions[ring[j] as usize]));
                let cp = Vec3::from(m.positions[c as usize]);
                if (a - cp).cross(b - cp).dot(normal) >= 0.0 { m.tri(c, ring[i], ring[j]); } else { m.tri(c, ring[j], ring[i]); }
            }
        }
    }
    m
}

/// A round tube along a 3D polyline, with parallel-transported frames and rounded end caps.
pub fn tube(points: &[Vec3], radius: f32, sides: usize, uv_scale: f32) -> MeshData {
    let n = points.len();
    let tangent = |i: usize| (points[(i + 1).min(n - 1)] - points[i.saturating_sub(1)]).normalize_or(Vec3::X);
    let t0 = tangent(0);
    let mut side = if t0.y.abs() < 0.9 { t0.cross(Vec3::Y).normalize() } else { t0.cross(Vec3::X).normalize() };
    let mut frames = vec![];
    let mut prev_t = t0;
    for i in 0..n {
        let t = tangent(i);
        let rot = Quat::from_rotation_arc(prev_t, t);
        side = (rot * side).normalize();
        let up = side.cross(t).normalize();
        frames.push(Frame { pos: points[i], side, up });
        prev_t = t;
    }
    let profile: Vec<ProfilePoint> = (0..sides).map(|i| {
        let a = TAU * i as f32 / sides as f32;
        let n = Vec2::new(a.cos(), a.sin());
        ProfilePoint { p: n * radius, n }
    }).collect();
    let mut m = sweep(&profile, &frames, true, false, uv_scale);
    // Hemispherical end caps.
    for (idx, dir) in [(0usize, -tangent(0)), (n - 1, tangent(n - 1))] {
        let f = frames[idx];
        let rings = 4;
        let start = m.positions.len() as u32;
        for r in 0..=rings {
            let phi = (r as f32 / rings as f32) * std::f32::consts::FRAC_PI_2;
            for s in 0..sides {
                let a = TAU * s as f32 / sides as f32;
                let radial = f.side * a.cos() + f.up * a.sin();
                let nrm = radial * phi.cos() + dir * phi.sin();
                m.vertex(f.pos + nrm * radius, nrm, Vec2::new(a * radius, phi * radius) * uv_scale);
            }
        }
        let before = m.indices.len();
        m.grid(start, rings + 1, sides, true);
        let i = &m.indices[before..];
        let (a, b, c) = (Vec3::from(m.positions[i[0] as usize]), Vec3::from(m.positions[i[1] as usize]), Vec3::from(m.positions[i[2] as usize]));
        if (b - a).cross(c - a).dot(Vec3::from(m.normals[i[0] as usize])) < 0.0 {
            for t in m.indices[before..].chunks_mut(3) { t.swap(1, 2); }
        }
    }
    m
}

/// Surface of revolution around Y. `profile` holds (radius, height) from bottom to top.
pub fn lathe(profile: &[Vec2], sides: usize, uv_scale: f32) -> MeshData {
    let pts = smooth_profile(profile, false);
    let mut m = MeshData::default();
    let mut len = vec![0.0];
    for i in 1..profile.len() { len.push(len[i - 1] + profile[i].distance(profile[i - 1])); }
    for (i, pp) in pts.iter().enumerate() {
        for s in 0..=sides {
            let a = TAU * s as f32 / sides as f32;
            let dir = Vec3::new(a.cos(), 0.0, a.sin());
            // smooth_profile puts the normal on the right of travel: outward for bottom-to-top.
            let n2 = pp.n;
            m.vertex(dir * pp.p.x + Vec3::Y * pp.p.y, dir * n2.x + Vec3::Y * n2.y,
                Vec2::new(a * pp.p.x.max(0.05), len[i]) * uv_scale);
        }
    }
    m.grid(0, pts.len(), sides + 1, false);
    let i = &m.indices;
    if !i.is_empty() {
        let (a, b, c) = (Vec3::from(m.positions[i[0] as usize]), Vec3::from(m.positions[i[1] as usize]), Vec3::from(m.positions[i[2] as usize]));
        if (b - a).cross(c - a).dot(Vec3::from(m.normals[i[0] as usize])) < 0.0 {
            for t in m.indices.chunks_mut(3) { t.swap(1, 2); }
        }
    }
    m
}

/// Box with rounded edges and corners. UVs are box-projected in metres.
pub fn rounded_box(size: Vec3, radius: f32, seg: usize, uv_scale: f32) -> MeshData {
    let half = size * 0.5;
    let r = radius.min(half.min_element() * 0.999);
    let inner = half - Vec3::splat(r);
    let mut m = MeshData::default();
    // Coordinates along one face axis: rounded zone, flat zone, rounded zone.
    let axis = |h: f32, inn: f32| -> Vec<f32> {
        let mut v = vec![];
        for i in 0..=seg { let t = i as f32 / seg as f32; v.push(-h + (h - inn) * (1.0 - (t * std::f32::consts::FRAC_PI_2).cos())); }
        for i in 0..=seg { let t = i as f32 / seg as f32; v.push(inn + (h - inn) * (t * std::f32::consts::FRAC_PI_2).sin()); }
        v
    };
    let faces = [(Vec3::X, Vec3::Z, Vec3::Y), (Vec3::NEG_X, Vec3::NEG_Z, Vec3::Y), (Vec3::Y, Vec3::X, Vec3::Z),
                 (Vec3::NEG_Y, Vec3::X, Vec3::NEG_Z), (Vec3::Z, Vec3::NEG_X, Vec3::Y), (Vec3::NEG_Z, Vec3::X, Vec3::Y)];
    for (n, u, v) in faces {
        let hu = (u * half).abs().max_element(); let hv = (v * half).abs().max_element();
        let iu = (u * inner).abs().max_element(); let iv = (v * inner).abs().max_element();
        let hn = (n * half).abs().max_element();
        let us = axis(hu, iu); let vs = axis(hv, iv);
        let start = m.positions.len() as u32;
        for &b in &vs {
            for &a in &us {
                let q = n * hn + u * a + v * b;
                let c = q.clamp(-inner, inner);
                let d = q - c;
                let nrm = if d.length_squared() > 1e-10 { d.normalize() } else { n };
                let p = c + nrm * r;
                m.vertex(p, nrm, Vec2::new(a, b) * uv_scale);
            }
        }
        m.grid(start, vs.len(), us.len(), false);
    }
    // Ensure outward winding per triangle using the vertex normals.
    for t in m.indices.chunks_mut(3) {
        let (a, b, c) = (Vec3::from(m.positions[t[0] as usize]), Vec3::from(m.positions[t[1] as usize]), Vec3::from(m.positions[t[2] as usize]));
        let nn = Vec3::from(m.normals[t[0] as usize]) + Vec3::from(m.normals[t[1] as usize]) + Vec3::from(m.normals[t[2] as usize]);
        if (b - a).cross(c - a).dot(nn) < 0.0 { t.swap(1, 2); }
    }
    m
}

/// Flat horizontal ring/annulus sector, facing up, with planar world UVs.
pub fn annulus(center: Vec3, r0: f32, r1: f32, a0: f32, a1: f32, segments: usize, uv_scale: f32) -> MeshData {
    let mut m = MeshData::default();
    for i in 0..=segments {
        let a = a0 + (a1 - a0) * i as f32 / segments as f32;
        let d = Vec3::new(a.cos(), 0.0, a.sin());
        for r in [r0, r1] {
            let p = center + d * r;
            m.vertex(p, Vec3::Y, Vec2::new(p.x, p.z) * uv_scale);
        }
    }
    for i in 0..segments as u32 {
        let (a, b, c, d) = (i * 2, i * 2 + 1, i * 2 + 2, i * 2 + 3);
        m.indices.extend([a, b, c, c, b, d]);
    }
    for t in m.indices.chunks_mut(3) {
        let (a, b, c) = (Vec3::from(m.positions[t[0] as usize]), Vec3::from(m.positions[t[1] as usize]), Vec3::from(m.positions[t[2] as usize]));
        if (b - a).cross(c - a).y < 0.0 { t.swap(1, 2); }
    }
    m
}

/// Horizontal strip between two curves sampled at the same parameters, facing `normal`.
pub fn ribbon(a: &[Vec3], b: &[Vec3], normal: Vec3, uv_scale: f32) -> MeshData {
    let mut m = MeshData::default();
    for (p, q) in a.iter().zip(b) {
        m.vertex(*p, normal, Vec2::new(p.x, p.z) * uv_scale);
        m.vertex(*q, normal, Vec2::new(q.x, q.z) * uv_scale);
    }
    for i in 0..(a.len() - 1) as u32 {
        let (p0, q0, p1, q1) = (i * 2, i * 2 + 1, i * 2 + 2, i * 2 + 3);
        m.indices.extend([p0, q0, p1, p1, q0, q1]);
    }
    for t in m.indices.chunks_mut(3) {
        let (x, y, z) = (Vec3::from(m.positions[t[0] as usize]), Vec3::from(m.positions[t[1] as usize]), Vec3::from(m.positions[t[2] as usize]));
        if (y - x).cross(z - x).dot(normal) < 0.0 { t.swap(1, 2); }
    }
    m
}

/// Signed power used by superquadrics.
fn spow(v: f32, e: f32) -> f32 { v.signum() * v.abs().powf(e) }

/// Upholstery cushion as a superquadric: `half` extents, `e_round` controls
/// how boxy the horizontal outline is (0.2 boxy .. 1.0 round), `e_puff` how
/// soft the vertical profile is. Dense, smooth and watertight; box-projected
/// UVs in metres. `res` is the number of segments around.
pub fn cushion(half: Vec3, e_round: f32, e_puff: f32, res: usize, uv_scale: f32) -> MeshData {
    let mut m = MeshData::default();
    let rings = (res / 2).max(4);
    for i in 0..=rings {
        let u = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / rings as f32;
        let (su, cu) = u.sin_cos();
        for j in 0..=res {
            let v = -std::f32::consts::PI + TAU * j as f32 / res as f32;
            let (sv, cv) = v.sin_cos();
            let p = Vec3::new(half.x * spow(cu, e_puff) * spow(cv, e_round), half.y * spow(su, e_puff), half.z * spow(cu, e_puff) * spow(sv, e_round));
            // Normal of the implicit superquadric surface.
            let n = Vec3::new(spow(cu, 2.0 - e_puff) * spow(cv, 2.0 - e_round) / half.x, spow(su, 2.0 - e_puff) / half.y,
                spow(cu, 2.0 - e_puff) * spow(sv, 2.0 - e_round) / half.z);
            let n = if n.length_squared() < 1e-12 { Vec3::new(0.0, su.signum(), 0.0) } else { n.normalize() };
            let a = n.abs();
            let uv = if a.y >= a.x && a.y >= a.z { Vec2::new(p.x, p.z) } else if a.x >= a.z { Vec2::new(p.z, -p.y) } else { Vec2::new(p.x, -p.y) };
            m.vertex(p, n, uv * uv_scale);
        }
    }
    m.grid(0, rings + 1, res + 1, false);
    fix_winding(&mut m);
    m
}

/// Lathe with vertical flutes: radius `r(y)` from `profile` minus a rounded
/// groove repeated `flutes` times around. For fluted brass bases and columns.
pub fn fluted_lathe(profile: &[Vec2], flutes: usize, depth: f32, sides_per_flute: usize, uv_scale: f32) -> MeshData {
    let mut m = MeshData::default();
    let sides = flutes * sides_per_flute;
    let mut len = vec![0.0];
    for i in 1..profile.len() { len.push(len[i - 1] + profile[i].distance(profile[i - 1])); }
    let groove = |a: f32| { let t = (a * flutes as f32 / TAU).fract(); depth * (std::f32::consts::PI * t).sin().powf(0.6) };
    for (i, pp) in profile.iter().enumerate() {
        for k in 0..=sides {
            let a = TAU * k as f32 / sides as f32;
            let r = (pp.x - if pp.x > depth * 2.0 { groove(a) } else { 0.0 }).max(0.0);
            m.vertex(Vec3::new(a.cos() * r, pp.y, a.sin() * r), Vec3::ZERO, Vec2::new(a * pp.x.max(0.05), len[i]) * uv_scale);
        }
    }
    m.grid(0, profile.len(), sides + 1, false);
    recompute_normals(&mut m);
    fix_winding_outward(&mut m);
    m
}

/// Vertically fluted cladding following horizontal `frames` (side = outward),
/// from `y0` to `y1`. Flutes are convex half-rounds of `pitch` width standing
/// `depth` proud of the frames. Used for oak wall panels and the bar front.
pub fn fluted_skin(frames: &[Frame], y0: f32, y1: f32, pitch: f32, depth: f32, uv_scale: f32) -> MeshData {
    // Resample the path densely so each flute gets several columns.
    let mut pts = vec![]; let mut sides = vec![]; let mut dist = vec![]; let mut acc = 0.0;
    for w in frames.windows(2) {
        let l = w[0].pos.distance(w[1].pos);
        let steps = ((l / (pitch / 8.0)).ceil() as usize).max(1);
        for k in 0..steps {
            let t = k as f32 / steps as f32;
            pts.push(w[0].pos.lerp(w[1].pos, t)); sides.push(w[0].side.lerp(w[1].side, t).normalize()); dist.push(acc + l * t);
        }
        acc += l;
    }
    if let Some(f) = frames.last() { pts.push(f.pos); sides.push(f.side); dist.push(acc); }
    let mut m = MeshData::default();
    for y in [y0, y1] {
        for (i, p) in pts.iter().enumerate() {
            let t = (dist[i] / pitch).fract();
            let bump = depth * (1.0 - (2.0 * t - 1.0).powi(2)).max(0.0).sqrt();
            m.vertex(Vec3::new(p.x, y, p.z) + sides[i] * bump, sides[i], Vec2::new(dist[i], -y) * uv_scale);
        }
    }
    m.grid(0, 2, pts.len(), false);
    recompute_normals(&mut m);
    // Orientation: normals must face the outward side.
    let flip = m.indices.chunks(3).next().map(|t| {
        let (a, b, c) = (Vec3::from(m.positions[t[0] as usize]), Vec3::from(m.positions[t[1] as usize]), Vec3::from(m.positions[t[2] as usize]));
        (b - a).cross(c - a).dot(sides[t[0] as usize % pts.len()]) < 0.0
    }).unwrap_or(false);
    if flip { for t in m.indices.chunks_mut(3) { t.swap(1, 2); } }
    recompute_normals(&mut m);
    m
}

/// Flat arch panel (rectangle with a semicircular top) in the plane spanned
/// by `right` and Y, centred on `base` (bottom middle), facing `normal`.
pub fn arch_panel(base: Vec3, right: Vec3, normal: Vec3, width: f32, height: f32, uv_scale: f32) -> MeshData {
    arch_ribbon(&arch_outline(base, right, width, height, 24), normal, uv_scale)
}

/// Outline of an arch from the bottom-left corner over the top to the bottom-right corner.
pub fn arch_outline(base: Vec3, right: Vec3, width: f32, height: f32, arc_steps: usize) -> Vec<Vec3> {
    let r = width * 0.5;
    let spring = (height - r).max(0.0);
    let mut pts = vec![base - right * r, base - right * r + Vec3::Y * spring];
    for k in 1..arc_steps {
        let a = std::f32::consts::PI * (1.0 - k as f32 / arc_steps as f32);
        pts.push(base + right * (r * a.cos()) + Vec3::Y * (spring + r * a.sin()));
    }
    pts.push(base + right * r + Vec3::Y * spring);
    pts.push(base + right * r);
    pts
}

fn arch_ribbon(outline: &[Vec3], normal: Vec3, uv_scale: f32) -> MeshData {
    // Pair the left half with the mirrored right half, bottom to top.
    let n = outline.len();
    let half = n / 2;
    let left: Vec<Vec3> = outline[..half + (n % 2)].to_vec();
    let right: Vec<Vec3> = outline[half..].iter().rev().cloned().collect();
    let k = left.len().min(right.len());
    let mut m = MeshData::default();
    let right_dir = (outline[n - 1] - outline[0]).normalize_or(Vec3::X);
    for i in 0..k {
        for p in [left[i], right[i]] {
            m.vertex(p, normal, Vec2::new(p.dot(right_dir), -p.y) * uv_scale);
        }
    }
    for i in 0..(k - 1) as u32 {
        let (a, b, c, d) = (i * 2, i * 2 + 1, i * 2 + 2, i * 2 + 3);
        m.indices.extend([a, b, c, c, b, d]);
    }
    for t in m.indices.chunks_mut(3) {
        let (x, y, z) = (Vec3::from(m.positions[t[0] as usize]), Vec3::from(m.positions[t[1] as usize]), Vec3::from(m.positions[t[2] as usize]));
        if (y - x).cross(z - x).dot(normal) < 0.0 { t.swap(1, 2); }
    }
    m
}

/// Smooth vertex normals from the triangle faces (area weighted).
fn recompute_normals(m: &mut MeshData) {
    let mut acc = vec![Vec3::ZERO; m.positions.len()];
    for t in m.indices.chunks(3) {
        let (a, b, c) = (Vec3::from(m.positions[t[0] as usize]), Vec3::from(m.positions[t[1] as usize]), Vec3::from(m.positions[t[2] as usize]));
        let f = (b - a).cross(c - a);
        for &i in t { acc[i as usize] += f; }
    }
    for (n, a) in m.normals.iter_mut().zip(acc) { *n = a.normalize_or(Vec3::Y).to_array(); }
}

/// Make triangle winding agree with the stored normals.
fn fix_winding(m: &mut MeshData) {
    for t in m.indices.chunks_mut(3) {
        let (a, b, c) = (Vec3::from(m.positions[t[0] as usize]), Vec3::from(m.positions[t[1] as usize]), Vec3::from(m.positions[t[2] as usize]));
        let n = Vec3::from(m.normals[t[0] as usize]) + Vec3::from(m.normals[t[1] as usize]) + Vec3::from(m.normals[t[2] as usize]);
        if (b - a).cross(c - a).dot(n) < 0.0 { t.swap(1, 2); }
    }
}

/// For lathes: wind so that faces point away from the Y axis, then rebuild normals.
fn fix_winding_outward(m: &mut MeshData) {
    let mut flips = 0i32;
    for t in m.indices.chunks(3) {
        let (a, b, c) = (Vec3::from(m.positions[t[0] as usize]), Vec3::from(m.positions[t[1] as usize]), Vec3::from(m.positions[t[2] as usize]));
        let centre = (a + b + c) / 3.0;
        let radial = Vec3::new(centre.x, 0.0, centre.z);
        let d = (b - a).cross(c - a).dot(radial);
        if d < 0.0 { flips += 1 } else if d > 0.0 { flips -= 1 }
    }
    if flips > 0 { for t in m.indices.chunks_mut(3) { t.swap(1, 2); } }
    recompute_normals(m);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn outward(m: &MeshData) -> bool {
        m.indices.chunks(3).all(|t| {
            let (a, b, c) = (Vec3::from(m.positions[t[0] as usize]), Vec3::from(m.positions[t[1] as usize]), Vec3::from(m.positions[t[2] as usize]));
            let n = Vec3::from(m.normals[t[0] as usize]);
            let face = (b - a).cross(c - a);
            face.length() < 1e-7 || face.dot(n) > -1e-6
        })
    }
    #[test]
    fn builders_wind_outward() {
        assert!(outward(&rounded_box(Vec3::new(1.0, 0.5, 0.8), 0.1, 4, 1.0)));
        assert!(outward(&lathe(&[Vec2::new(0.3, 0.0), Vec2::new(0.3, 0.5), Vec2::new(0.1, 0.8)], 24, 1.0)));
        assert!(outward(&sweep(&rounded_rect(0.5, 0.4, 0.08, 4), &arc_frames(Vec3::ZERO, 2.0, 0.0, 1.0, 16), true, true, 1.0)));
        assert!(outward(&cushion(Vec3::new(0.4, 0.1, 0.3), 0.3, 0.6, 48, 1.0)));
        assert!(outward(&fluted_lathe(&[Vec2::new(0.3, 0.0), Vec2::new(0.3, 0.5)], 16, 0.02, 4, 1.0)));
        assert!(outward(&arch_panel(Vec3::ZERO, Vec3::X, Vec3::Z, 1.0, 2.0, 1.0)));
        let skin = fluted_skin(&path_frames(&[Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0)]), 0.0, 1.0, 0.1, 0.02, 1.0);
        assert!(outward(&skin));
    }
}
