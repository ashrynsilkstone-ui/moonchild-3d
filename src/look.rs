//! Moonchild materials, mipmaps, environment reflections and camera post-processing.
use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    asset::RenderAssetUsages,
    camera::Hdr,
    image::{ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    light::GeneratedEnvironmentMapLight,
    pbr::{ContactShadows, ScreenSpaceAmbientOcclusion},
    post_process::bloom::Bloom,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension},
};

/// Physical tile size of each texture set (metres per texture repeat).
/// Mesh builders multiply metre coordinates with `1.0 / tile`.
pub mod tile {
    pub const CONCRETE: f32 = 2.0;
    pub const FLOOR: f32 = 2.0;
    pub const STEEL: f32 = 1.0;
    pub const DARKSTEEL: f32 = 1.0;
    pub const LEATHER: f32 = 0.6;
    pub const GRILLE: f32 = 0.4;
    pub const TREAD: f32 = 0.5;
    pub const BACKLIGHT: f32 = 1.0;
}

#[derive(Resource, Default)]
pub struct MipQueue(Vec<(AssetId<Image>, bool)>);

pub struct Surfaces {
    pub concrete: Handle<StandardMaterial>,
    pub steel: Handle<StandardMaterial>,
    pub darksteel: Handle<StandardMaterial>,
    pub floor: Handle<StandardMaterial>,
    pub dance_floor: Handle<StandardMaterial>,
    pub leather: Handle<StandardMaterial>,
    pub grille: Handle<StandardMaterial>,
    pub tread: Handle<StandardMaterial>,
    pub backlight_cyan: Handle<StandardMaterial>,
    pub backlight_magenta: Handle<StandardMaterial>,
    pub backlight_amber: Handle<StandardMaterial>,
}

struct Maps { base: Handle<Image>, normal: Handle<Image>, orm: Handle<Image> }

fn load_maps(server: &AssetServer, queue: &mut MipQueue, name: &str) -> Maps {
    let base: Handle<Image> = server.load(format!("textures/moonchild/{name}_base.png"));
    // Data textures are linear, unlike the sRGB base-colour image.
    let normal: Handle<Image> = server.load_builder().with_settings(|s: &mut ImageLoaderSettings| s.is_srgb = false)
        .load(format!("textures/moonchild/{name}_normal.png"));
    let orm: Handle<Image> = server.load_builder().with_settings(|s: &mut ImageLoaderSettings| s.is_srgb = false)
        .load(format!("textures/moonchild/{name}_orm.png"));
    queue.0.push((base.id(), false));
    queue.0.push((normal.id(), true));
    queue.0.push((orm.id(), false));
    Maps { base, normal, orm }
}

fn pbr_from(maps: &Maps) -> StandardMaterial {
    StandardMaterial {
        base_color_texture: Some(maps.base.clone()),
        normal_map_texture: Some(maps.normal.clone()),
        metallic_roughness_texture: Some(maps.orm.clone()),
        occlusion_texture: Some(maps.orm.clone()),
        // Texture values are multiplied with these factors.
        metallic: 1.0,
        perceptual_roughness: 1.0,
        ..default()
    }
}

impl Surfaces {
    pub fn new(materials: &mut Assets<StandardMaterial>, server: &AssetServer, queue: &mut MipQueue) -> Self {
        let concrete = load_maps(server, queue, "concrete");
        let floor = load_maps(server, queue, "floor");
        let steel = load_maps(server, queue, "steel");
        let darksteel = load_maps(server, queue, "darksteel");
        let leather = load_maps(server, queue, "leather");
        let grille = load_maps(server, queue, "grille");
        let tread = load_maps(server, queue, "treadplate");
        let back = load_maps(server, queue, "backlight");
        let back_emissive: Handle<Image> = server.load("textures/moonchild/backlight_emissive.png");
        queue.0.push((back_emissive.id(), false));
        let backlight = |materials: &mut Assets<StandardMaterial>, c: LinearRgba| materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.25, 0.27),
            emissive_texture: Some(back_emissive.clone()),
            emissive: c,
            ..pbr_from(&back)
        });
        Self {
            concrete: materials.add(StandardMaterial { base_color: Color::srgb(1.12, 1.10, 1.08), ..pbr_from(&concrete) }),
            floor: materials.add(pbr_from(&floor)),
            // The circular dance floor is darker and more polished than the surrounding floor.
            dance_floor: materials.add(StandardMaterial { base_color: Color::srgb(0.55, 0.55, 0.62), perceptual_roughness: 0.55, ..pbr_from(&floor) }),
            steel: materials.add(pbr_from(&steel)),
            darksteel: materials.add(pbr_from(&darksteel)),
            leather: materials.add(pbr_from(&leather)),
            grille: materials.add(StandardMaterial {
                alpha_mode: AlphaMode::Mask(0.5), double_sided: true, cull_mode: None, ..pbr_from(&grille)
            }),
            tread: materials.add(pbr_from(&tread)),
            backlight_cyan: backlight(materials, LinearRgba::rgb(0.2, 2.6, 3.4)),
            backlight_magenta: backlight(materials, LinearRgba::rgb(3.2, 0.25, 1.6)),
            backlight_amber: backlight(materials, LinearRgba::rgb(1.5, 0.42, 0.12)),
        }
    }
}

/// PNG files carry only one mip level. Without mipmaps, fine texture detail
/// shimmers and produces moiré at a distance, so a box-filtered chain is
/// built on the CPU once per image after loading.
pub fn generate_mipmaps(mut events: MessageReader<AssetEvent<Image>>, queue: Res<MipQueue>, mut images: ResMut<Assets<Image>>) {
    for event in events.read() {
        let AssetEvent::LoadedWithDependencies { id } = event else { continue };
        let Some(&(_, is_normal)) = queue.0.iter().find(|(i, _)| i == id) else { continue };
        let Some(mut image) = images.get_mut(*id) else { continue };
        if image.texture_descriptor.mip_level_count > 1 { continue; }
        let format = image.texture_descriptor.format;
        let srgb = match format {
            TextureFormat::Rgba8UnormSrgb => true,
            TextureFormat::Rgba8Unorm => false,
            _ => continue,
        };
        let (w, h) = (image.width() as usize, image.height() as usize);
        let Some(data) = image.data.as_ref() else { continue };
        let to_lin: Vec<f32> = (0..256).map(|v| {
            let c = v as f32 / 255.0;
            if srgb { if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) } } else { c }
        }).collect();
        let from_lin = |v: f32| -> u8 {
            let v = v.clamp(0.0, 1.0);
            let c = if srgb { if v <= 0.0031308 { v * 12.92 } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 } } else { v };
            (c * 255.0 + 0.5) as u8
        };
        let mut level: Vec<f32> = data.iter().enumerate()
            .map(|(i, &b)| if i % 4 == 3 { b as f32 / 255.0 } else { to_lin[b as usize] }).collect();
        let mut out = data.clone();
        let (mut lw, mut lh, mut count) = (w, h, 1u32);
        while lw > 1 || lh > 1 {
            let (nw, nh) = ((lw / 2).max(1), (lh / 2).max(1));
            let mut next = vec![0.0f32; nw * nh * 4];
            for y in 0..nh {
                for x in 0..nw {
                    for c in 0..4 {
                        let mut s = 0.0;
                        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                            let sx = (x * 2 + dx).min(lw - 1); let sy = (y * 2 + dy).min(lh - 1);
                            s += level[(sy * lw + sx) * 4 + c];
                        }
                        next[(y * nw + x) * 4 + c] = s * 0.25;
                    }
                    if is_normal {
                        let p = &mut next[(y * nw + x) * 4..(y * nw + x) * 4 + 3];
                        let v = Vec3::new(p[0] * 2.0 - 1.0, p[1] * 2.0 - 1.0, p[2] * 2.0 - 1.0).normalize_or(Vec3::Z);
                        p[0] = v.x * 0.5 + 0.5; p[1] = v.y * 0.5 + 0.5; p[2] = v.z * 0.5 + 0.5;
                    }
                }
            }
            out.extend(next.iter().enumerate().map(|(i, &v)| if i % 4 == 3 { (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8 } else { from_lin(v) }));
            level = next; lw = nw; lh = nh; count += 1;
        }
        image.data = Some(out);
        image.texture_descriptor.mip_level_count = count;
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor { anisotropy_clamp: 16, ..crate::repeat_sampler() });
    }
}

fn f16_bits(v: f32) -> u16 {
    let bits = v.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exp = ((bits >> 23) & 0xff) as i32 - 127 + 15;
    let mant = bits & 0x7f_ffff;
    if v == 0.0 || exp <= 0 { return sign; }
    if exp >= 31 { return sign | 0x7bff; }
    sign | ((exp as u16) << 10) | ((mant >> 13) as u16)
}

/// Simplified, procedurally painted HDR cubemap of the club's surroundings:
/// dark concrete, warm ceiling downlights, cyan/magenta neon bands, the LED wall
/// at -Z and the warm bar at +X. It feeds Bevy's runtime-filtered environment
/// lighting, so glossy floors and steel pick up plausible coloured reflections.
/// It is not a live reflection of the actual scene.
fn environment_cubemap(images: &mut Assets<Image>) -> Handle<Image> {
    const N: usize = 128;
    let mut data = Vec::with_capacity(N * N * 6 * 8);
    let face_dir = |face: usize, u: f32, v: f32| -> Vec3 {
        match face {
            0 => Vec3::new(1.0, -v, -u), 1 => Vec3::new(-1.0, -v, u),
            2 => Vec3::new(u, 1.0, v), 3 => Vec3::new(u, -1.0, -v),
            4 => Vec3::new(u, -v, 1.0), _ => Vec3::new(-u, -v, -1.0),
        }.normalize()
    };
    let band = |y: f32, centre: f32, width: f32| (-((y - centre) / width).powi(2)).exp();
    for face in 0..6 {
        for py in 0..N {
            for px in 0..N {
                let u = (px as f32 + 0.5) / N as f32 * 2.0 - 1.0;
                let v = (py as f32 + 0.5) / N as f32 * 2.0 - 1.0;
                let d = face_dir(face, u, v);
                let mut c = Vec3::new(0.03, 0.026, 0.022) * (1.0 + 0.5 * d.y.max(0.0));
                if d.y > 0.55 {
                    // Grid of warm ceiling downlights.
                    let p = Vec2::new(d.x, d.z) / d.y * 3.0;
                    let cell = (p - p.round()).length();
                    c += Vec3::new(1.0, 0.86, 0.68) * 3.6 * (-(cell / 0.11).powi(2)).exp();
                    // Circular neon halo overhead.
                    let r = (d.x * d.x + d.z * d.z).sqrt() / d.y;
                    c += Vec3::new(0.1, 0.9, 1.2) * 0.7 * (-((r - 0.72) / 0.03).powi(2)).exp();
                    c += Vec3::new(1.2, 0.08, 0.6) * 0.5 * (-((r - 0.80) / 0.025).powi(2)).exp();
                }
                let horizontal = d.y.abs() < 0.6;
                if horizontal {
                    c += Vec3::new(1.1, 0.05, 0.45) * 0.45 * band(d.y, -0.08, 0.02);
                    c += Vec3::new(0.1, 0.25, 1.3) * 0.5 * band(d.y, 0.22, 0.015);
                    c += Vec3::new(0.05, 0.85, 1.1) * 0.55 * band(d.y, 0.05, 0.012);
                }
                // LED wall towards -Z, warm bar towards +X.
                if d.z < -0.85 && d.y > 0.0 && d.y < 0.3 { c += Vec3::new(0.9, 0.25, 1.3) * 0.12 * (-d.z - 0.85) * 6.6; }
                if d.x > 0.7 && d.y > -0.25 && d.y < 0.12 { c += Vec3::new(1.4, 0.45, 0.12) * 1.2 * (d.x - 0.7) * 3.3; }
                // Floor: dark with a faint reflection of the ring.
                if d.y < -0.3 { c *= 0.6; }
                for ch in [c.x, c.y, c.z, 1.0] { data.extend_from_slice(&f16_bits(ch).to_le_bytes()); }
            }
        }
    }
    let mut image = Image::new(Extent3d { width: N as u32, height: N as u32, depth_or_array_layers: 6 },
        TextureDimension::D2, data, TextureFormat::Rgba16Float, RenderAssetUsages::RENDER_WORLD);
    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
    image.texture_view_descriptor = Some(TextureViewDescriptor { dimension: Some(TextureViewDimension::Cube), ..default() });
    images.add(image)
}

/// Camera with HDR, controlled bloom, TAA, SSAO, contact shadows and environment reflections.
pub fn camera_bundle(images: &mut Assets<Image>) -> impl Bundle {
    (
        Camera3d::default(),
        Hdr,
        Msaa::Off,
        TemporalAntiAliasing::default(),
        ScreenSpaceAmbientOcclusion::default(),
        ContactShadows { linear_steps: 16, thickness: 0.08, length: 0.35 },
        Bloom { intensity: 0.07, low_frequency_boost: 0.3, ..Bloom::NATURAL },
        GeneratedEnvironmentMapLight { environment_map: environment_cubemap(images), intensity: 450.0, ..default() },
    )
}
