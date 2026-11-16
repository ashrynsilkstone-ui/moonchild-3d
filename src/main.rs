//! Moonchild 3D - fully procedural, no paid assets.
mod capture;
mod club_section;
mod dj_gear;
mod hall;
mod lod;
mod look;
mod navigation;
mod quality;
mod shapes;

use std::f32::consts::PI;
use bevy::{
    image::{ImageAddressMode, ImageSamplerDescriptor},
    input::mouse::AccumulatedMouseMotion,
    mesh::VertexAttributeValues,
    prelude::*,
};
use bevy::{asset::RenderAssetUsages, mesh::Indices, render::render_resource::PrimitiveTopology};
use navigation::layout::{CEILING, RING_CENTER, ROOM_X, ROOM_Z};

const W: f32 = ROOM_X * 2.0 + 0.3;
const D: f32 = ROOM_Z * 2.0 + 0.3;
const H: f32 = CEILING;

#[derive(Resource)]
struct Show { mode: usize, speed: f32, enabled: bool, beat: f32 }
impl Default for Show {
    fn default() -> Self { Self { mode: 0, speed: 1.0, enabled: true, beat: 130.0 } }
}
#[derive(Component)]
struct Player { yaw: f32, pitch: f32, motion: navigation::Motion }
#[derive(Component)]
struct MovingHead { index: usize, base: Vec3, stage: bool }
/// Additive light cone below a moving head (a visual beam, not volumetric fog).
#[derive(Component)]
struct Beam;

/// Number of light programs: Neon, Rave, Chill, Strobe, Faecher, Kegel, Chase.
const PROGRAMS: usize = 7;
const PROGRAM_NAMES: [&str; PROGRAMS] = ["NEON", "RAVE", "CHILL", "STROBE", "FAECHER", "KEGEL", "CHASE"];
#[derive(Component)]
struct Led { index: usize }
#[derive(Component)]
struct PulseLight { index: usize }
#[derive(Component)]
struct StatusText;

/// Repeating, trilinear sampler used for every tiling texture.
pub fn repeat_sampler() -> ImageSamplerDescriptor {
    ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    }
}

fn main() {
    let tour = capture::Tour::from_env();
    let present_mode = if tour.is_some() { bevy::window::PresentMode::AutoNoVsync } else { bevy::window::PresentMode::AutoVsync };
    let mut app = App::new();
    app
        .add_plugins(DefaultPlugins.set(ImagePlugin { default_sampler: repeat_sampler() }).set(WindowPlugin {
            primary_window: Some(Window {
                title: "MOONCHILD | Rust 3D Club".into(),
                resolution: (1440, 900).into(),
                present_mode,
                ..default()
            }),
            ..default()
        }))
        // Calm warm-white base light: the club must stay readable in every show mode.
        .insert_resource(GlobalAmbientLight { color: Color::srgb(0.80, 0.73, 0.65), brightness: 270.0, ..default() })
        .init_resource::<Show>()
        .init_resource::<look::MipQueue>()
        .insert_resource(quality::Quality::from_env())
        // Deferred shading: required for screen-space reflections (SSR).
        .insert_resource(bevy::pbr::DefaultOpaqueRendererMethod::deferred())
        .add_systems(Startup, setup)
        .add_systems(Update, (walk, controls, animate_lights, update_hud, look::generate_mipmaps, quality::cycle, quality::apply.after(quality::cycle)))
        .add_systems(Update, capture::run_tour.after(walk));
    if let Some(tour) = tour { app.insert_resource(tour); }
    app.run();
}

fn glow(materials: &mut Assets<StandardMaterial>, r: f32, g: f32, b: f32) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial { base_color: Color::srgb(r.min(1.0),g.min(1.0),b.min(1.0)), emissive: LinearRgba::rgb(r*9.0,g*9.0,b*9.0), unlit: false, ..default() })
}
/// Cuboid with world-sized planar UVs (`uv_scale` repeats per metre).
fn block(cmd: &mut Commands, meshes: &mut Assets<Mesh>, material: Handle<StandardMaterial>, pos: Vec3, size: Vec3, uv_scale: f32) {
    let mut mesh: Mesh = Cuboid::new(size.x,size.y,size.z).into();
    let projected = match (mesh.attribute(Mesh::ATTRIBUTE_POSITION), mesh.attribute(Mesh::ATTRIBUTE_NORMAL)) {
        (Some(VertexAttributeValues::Float32x3(points)), Some(VertexAttributeValues::Float32x3(normals))) => {
            Some(points.iter().zip(normals).map(|(p,n)| {
                let w = Vec3::from(*p) + pos;
                if n[1].abs() > 0.5 { [w.x*uv_scale, w.z*uv_scale] }
                else if n[0].abs() > 0.5 { [w.z*uv_scale, -w.y*uv_scale] }
                else { [w.x*uv_scale, -w.y*uv_scale] }
            }).collect::<Vec<[f32;2]>>())
        },
        _ => None,
    };
    if let Some(uv) = projected { mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv); }
    let _ = mesh.generate_tangents();
    cmd.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material), Transform::from_translation(pos)));
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>, mut mip_queue: ResMut<look::MipQueue>, asset_server: Res<AssetServer>) {
    let surfaces = look::Surfaces::new(&mut materials, &asset_server, &mut mip_queue);
    let s = &surfaces;
    let neon = club_section::Neon {
        cyan: glow(&mut materials, 0.01, 0.8, 1.0),
        magenta: glow(&mut materials, 1.0, 0.06, 0.42),
        blue: glow(&mut materials, 0.08, 0.2, 1.0),
        warm: glow(&mut materials, 1.0, 0.62, 0.3),
        red: glow(&mut materials, 1.0, 0.05, 0.06),
    };
    let white = glow(&mut materials, 0.85,0.85,1.0);
    let concrete_uv = 1.0 / look::tile::CONCRETE;
    let steel_uv = 1.0 / look::tile::DARKSTEEL;

    // Enclosed industrial hall: floor, four walls and a dark ceiling.
    block(&mut commands,&mut meshes,s.floor.clone(),Vec3::new(0.0,-0.16,0.0),Vec3::new(W,0.32,D),1.0 / look::tile::FLOOR);
    block(&mut commands,&mut meshes,s.concrete.clone(),Vec3::new(0.0,H/2.0,-D/2.0),Vec3::new(W,H,0.3),concrete_uv);
    block(&mut commands,&mut meshes,s.concrete.clone(),Vec3::new(-W/2.0,H/2.0,0.0),Vec3::new(0.3,H,D),concrete_uv);
    block(&mut commands,&mut meshes,s.concrete.clone(),Vec3::new(W/2.0,H/2.0,0.0),Vec3::new(0.3,H,D),concrete_uv);
    block(&mut commands,&mut meshes,s.concrete.clone(),Vec3::new(0.0,H/2.0,D/2.0),Vec3::new(W,H,0.3),concrete_uv);
    block(&mut commands,&mut meshes,s.concrete.clone(),Vec3::new(0.0,H+0.12,0.0),Vec3::new(W,0.25,D),concrete_uv);
    // Rounded steel columns and ceiling beams.
    let column = meshes.add(shapes::rounded_box(Vec3::new(0.42, H, 0.42), 0.06, 3, steel_uv).build());
    for &(x, z) in &navigation::layout::ROOM_COLUMNS {
        commands.spawn((Mesh3d(column.clone()), MeshMaterial3d(s.darksteel.clone()), Transform::from_xyz(x, H * 0.5, z)));
    }
    let beam = meshes.add(shapes::rounded_box(Vec3::new(22.0, 0.24, 0.24), 0.05, 3, steel_uv).build());
    for &z in &[-7.5,-2.0,3.5] {
        commands.spawn((Mesh3d(beam.clone()), MeshMaterial3d(s.darksteel.clone()), Transform::from_xyz(0.0, 6.62, z)));
    }

    // Raised DJ stage at the far end with a tread-plate edge.
    block(&mut commands,&mut meshes,s.darksteel.clone(),Vec3::new(0.0,0.46,-7.55),Vec3::new(14.0,0.92,4.4),steel_uv);
    block(&mut commands,&mut meshes,s.tread.clone(),Vec3::new(0.0,0.94,-7.55),Vec3::new(14.0,0.04,4.4),1.0 / look::tile::TREAD);
    commands.spawn((Mesh3d(meshes.add(shapes::tube(&[Vec3::new(-6.95,0.9,-5.33),Vec3::new(6.95,0.9,-5.33)],0.014,8,1.0).build())), MeshMaterial3d(neon.cyan.clone())));
    dj_gear::spawn(&mut commands, &mut meshes, &mut materials, &asset_server);
    // LED tiles, individually animated via material emissive colour, in a rounded frame.
    commands.spawn((Mesh3d(meshes.add(shapes::rounded_box(Vec3::new(11.0,3.75,0.12),0.05,3,steel_uv).build())),
        MeshMaterial3d(s.darksteel.clone()), Transform::from_xyz(0.0,4.5,-9.82)));
    let tile_mesh = meshes.add(shapes::rounded_box(Vec3::new(0.5,0.5,0.06),0.035,2,1.0).build());
    for col in 0..20 {
        for row in 0..6 {
            let material = materials.add(StandardMaterial { base_color: Color::BLACK, emissive: LinearRgba::rgb(2.0,0.1,3.0), unlit: false, ..default() });
            commands.spawn((Mesh3d(tile_mesh.clone()), MeshMaterial3d(material),
                Transform::from_xyz((col as f32-9.5)*0.53, 3.15+row as f32*0.54,-9.74), Led { index: col*6+row }));
        }
    }
    // Neon bands along the side walls; the low right band stops at the bar.
    let band = |x: f32, y: f32, z0: f32, z1: f32| shapes::tube(&[Vec3::new(x,y,z0),Vec3::new(x,y,z1)],0.03,8,1.0).build();
    for &x in &[-ROOM_X+0.25, ROOM_X-0.25] {
        for &y in &[1.0, 5.9] {
            let mat = if y<3.0 { neon.magenta.clone() } else { neon.blue.clone() };
            if x > 0.0 && y < 3.0 {
                for (z0,z1) in [(-8.6,0.2),(4.8,8.6)] { commands.spawn((Mesh3d(meshes.add(band(x,y,z0,z1))), MeshMaterial3d(mat.clone()))); }
            } else {
                commands.spawn((Mesh3d(meshes.add(band(x,y,-8.6,8.6))), MeshMaterial3d(mat)));
            }
        }
    }
    club_section::spawn(&mut commands, &mut meshes, &mut materials, s, &neon);

    hall::spawn(&mut commands, &mut meshes, &mut materials, s, &neon);

    // Show lighting: six moving heads on the halo and four on the stage truss (no shadows, they move).
    let lens = glow(&mut materials, 0.9, 0.9, 1.0);
    let parts = club_section::moving_head_meshes(&mut meshes, s, lens);
    club_section::spawn_head_yokes(&mut commands, &mut meshes, s);
    let beam_mesh = meshes.add(beam_cone());
    let heads = club_section::moving_head_positions().into_iter().enumerate().map(|(i, p)| (i, p, false))
        .chain(hall::stage_head_positions().into_iter().enumerate().map(|(i, p)| (i, p, true)));
    for (i, base, stage) in heads {
        let beam_material = materials.add(StandardMaterial {
            base_color: Color::srgba(0.8, 0.2, 1.0, 0.0), unlit: true, alpha_mode: AlphaMode::Add,
            cull_mode: None, double_sided: true, fog_enabled: false, ..default()
        });
        let rest = if stage { Vec3::new(0.0, 0.0, 1.0) } else { Vec3::new(base.x * 0.3, 0.0, base.z * 0.3) };
        commands.spawn((
            SpotLight { intensity: 180_000.0, color: Color::srgb(0.8,0.2,1.0), range: 35.0,
                inner_angle: 0.11, outer_angle: 0.38, shadow_maps_enabled: false, ..default() },
            Transform::from_translation(base).looking_at(rest, Vec3::Z),
            MovingHead { index: i, base, stage },
            quality::VolumetricSource,
        )).with_children(|c| {
            for (mesh, mat, t, range) in &parts {
                // The fixture body surrounds the light; it must not shadow its own beam.
                let mut part = c.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), *t, bevy::light::NotShadowCaster));
                if let Some(range) = range { part.insert(range.clone()); }
            }
            c.spawn((Mesh3d(beam_mesh.clone()), MeshMaterial3d(beam_material), Transform::from_xyz(0.0, 0.0, -0.17),
                Beam, quality::FakeBeam, bevy::light::NotShadowCaster));
        });
    }
    for (i,(x,z)) in [(-8.0,-4.0),(8.0,-4.0),(-8.0,5.0),(8.0,5.0)].iter().enumerate() {
        commands.spawn((PointLight { intensity: 95_000.0, range: 13.0, color: Color::srgb(0.35,0.12,1.0), shadow_maps_enabled: false, ..default() },
            Transform::from_xyz(*x,4.0,*z), PulseLight { index: i }));
    }
    // Permanent warm-white house lights outside the halo, independent of the show toggle.
    let can = meshes.add(shapes::lathe(&[Vec2::new(0.0,0.0),Vec2::new(0.16,0.0),Vec2::new(0.17,0.02),Vec2::new(0.17,0.25),Vec2::new(0.0,0.25)],24,steel_uv).build());
    let disc = meshes.add(shapes::annulus(Vec3::ZERO,0.0,0.13,0.0,std::f32::consts::TAU,20,1.0).build());
    for &(x,z) in &[(-8.3,-3.0),(8.3,-3.0),(-6.5,4.2),(5.6,5.4),(-3.6,-6.0),(3.6,-6.0)] {
        let y = CEILING - 0.3;
        commands.spawn((Mesh3d(can.clone()), MeshMaterial3d(s.darksteel.clone()), Transform::from_xyz(x,y,z)));
        commands.spawn((Mesh3d(disc.clone()), MeshMaterial3d(white.clone()), Transform::from_xyz(x,y-0.005,z).with_rotation(Quat::from_rotation_x(PI))));
        commands.spawn((
            SpotLight { intensity: 85_000.0, color: Color::srgb(1.0,0.95,0.86), range: 16.0, inner_angle: 0.45, outer_angle: 0.82,
                shadow_maps_enabled: true, contact_shadows_enabled: true, ..default() },
            // Vertical spots need an up vector that is not parallel to the beam.
            Transform::from_xyz(x,y-0.02,z).looking_at(Vec3::new(x,0.0,z),Vec3::Z),
            quality::UltraShaft,
        ));
    }
    // Warm stage washes for the DJ booth.
    for &x in &[-4.5,0.0,4.5] {
        commands.spawn((
            SpotLight { intensity: 75_000.0, color: Color::srgb(1.0,0.90,0.77), range: 14.0, inner_angle: 0.44, outer_angle: 0.72,
                shadow_maps_enabled: x == 0.0, contact_shadows_enabled: true, ..default() },
            Transform::from_xyz(x,5.9,-5.2).looking_at(Vec3::new(x*0.6,1.8,-8.0),Vec3::Y),
        ));
    }

    // Club haze for the volumetric light (Hoch/Ultra).
    commands.spawn(quality::haze(Vec3::new(W - 0.4, H - 0.1, D - 0.4), Vec3::new(0.0, H * 0.5, 0.0)));

    // Camera starts beside the dance floor, looking towards the DJ stage (-Z).
    commands.spawn((look::camera_bundle(&mut images),
        Transform::from_xyz(0.0,navigation::EYE,6.6).looking_at(Vec3::new(0.0,2.3,-7.0),Vec3::Y),
        Player { yaw: 0.0, pitch: 0.03, motion: navigation::Motion::standing() }));
    // Overlay and instructions (ASCII only: the default UI font lacks some glyphs).
    commands.spawn((
        Text::new("MOONCHILD  |  3D CLUB  |  WASD gehen  |  Shift schneller  |  Leertaste springen  |  rechte Maustaste + Maus umsehen\n1 Neon  2 Rave  3 Chill  4 Strobe  5 Faecher  6 Kegel  7 Chase  |  L Show-Licht an/aus  |  +/- Tempo  |  F2 Grafik  |  ESC beenden"),
        TextFont { font_size: FontSize::Px(17.0), ..default() },
        TextColor(Color::srgb(0.87,0.91,1.0)),
        Node { position_type: PositionType::Absolute, top: px(16), left: px(18), ..default() },
    ));
    commands.spawn((
        Text::new("LICHTMODUS: NEON   |   BPM: 130"),
        TextFont { font_size: FontSize::Px(18.0), ..default() },
        TextColor(Color::srgb(0.5,0.95,1.0)),
        Node { position_type: PositionType::Absolute, bottom: px(17), left: px(18), ..default() },
        StatusText,
    ));
}

fn walk(keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>, time: Res<Time>, player: Single<(&mut Transform, &mut Player)>) {
    let (mut t,mut p) = player.into_inner();
    if mouse.pressed(MouseButton::Right) {
        p.yaw -= motion.delta.x * 0.003;
        p.pitch = (p.pitch - motion.delta.y * 0.003).clamp(-1.30,1.30);
    }
    t.rotation = Quat::from_rotation_y(p.yaw) * Quat::from_rotation_x(p.pitch);
    let forward = Vec3::new(-p.yaw.sin(),0.0,-p.yaw.cos());
    let right = Vec3::new(p.yaw.cos(),0.0,-p.yaw.sin());
    let mut direction = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) { direction+=forward; }
    if keys.pressed(KeyCode::KeyS) { direction-=forward; }
    if keys.pressed(KeyCode::KeyD) { direction+=right; }
    if keys.pressed(KeyCode::KeyA) { direction-=right; }
    let speed = if keys.pressed(KeyCode::ShiftLeft) { 9.0 } else { 4.7 };
    let dt = time.delta_secs().min(0.1);
    let movement = direction.normalize_or_zero() * speed * dt;
    t.translation = navigation::advance(t.translation, movement, keys.just_pressed(KeyCode::Space), dt, &mut p.motion);
}
fn controls(keys: Res<ButtonInput<KeyCode>>, mut show: ResMut<Show>, mut exit: MessageWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Digit1) { show.mode=0; }
    if keys.just_pressed(KeyCode::Digit2) { show.mode=1; }
    if keys.just_pressed(KeyCode::Digit3) { show.mode=2; }
    if keys.just_pressed(KeyCode::Digit4) { show.mode=3; }
    if keys.just_pressed(KeyCode::Digit5) { show.mode=4; }
    if keys.just_pressed(KeyCode::Digit6) { show.mode=5; }
    if keys.just_pressed(KeyCode::Digit7) { show.mode=6; }
    if keys.just_pressed(KeyCode::KeyL) { show.enabled=!show.enabled; }
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) { show.speed=(show.speed+0.2).min(3.0); show.beat=(show.beat+5.0).min(200.0); }
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) { show.speed=(show.speed-0.2).max(0.2); show.beat=(show.beat-5.0).max(60.0); }
    if keys.just_pressed(KeyCode::Escape) { exit.write(AppExit::Success); }
}
/// Up vector that is never parallel to the beam direction.
fn safe_up(dir: Vec3) -> Vec3 { if dir.normalize_or(Vec3::NEG_Y).y.abs() > 0.95 { Vec3::Z } else { Vec3::Y } }

/// Unit cone along -Z (apex at the origin, radius 1 at length 1) whose vertex
/// alpha fades from the apex to the base; scaled per frame to match a spot.
fn beam_cone() -> Mesh {
    const SIDES: usize = 28;
    const RINGS: usize = 10;
    let mut positions = vec![]; let mut normals = vec![]; let mut colors = vec![]; let mut indices: Vec<u32> = vec![];
    for r in 0..=RINGS {
        let d = r as f32 / RINGS as f32;
        let alpha = 0.6 * (1.0 - d).powf(1.6) + 0.05;
        for k in 0..=SIDES {
            let a = 2.0 * PI * k as f32 / SIDES as f32;
            positions.push([a.cos() * d, a.sin() * d, -d]);
            normals.push([a.cos(), a.sin(), 0.0]);
            colors.push([1.0, 1.0, 1.0, alpha]);
        }
    }
    for r in 0..RINGS {
        for k in 0..SIDES {
            let (a, b) = ((r * (SIDES + 1) + k) as u32, (r * (SIDES + 1) + k + 1) as u32);
            let (c, d) = (a + (SIDES + 1) as u32, b + (SIDES + 1) as u32);
            indices.extend([a, c, b, b, c, d]);
        }
    }
    let uvs: Vec<[f32; 2]> = positions.iter().map(|p| [p[0], p[1]]).collect();
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U32(indices))
}

/// Floor target of head `i` of `n` for a light program at time `t`.
fn aim(mode: usize, stage: bool, i: usize, n: usize, t: f32) -> Vec3 {
    let (cx, _, cz) = RING_CENTER;
    let fi = i as f32;
    let k = fi - (n as f32 - 1.0) * 0.5;
    match mode {
        // Fan: a line of beams that spreads and sweeps front to back.
        4 => Vec3::new(cx + k * 2.4 * (1.0 + 0.35 * (t * 0.6).sin()), 0.0, cz + (t * 0.7).sin() * 5.5),
        // Cone: all beams on one circle that breathes in and out while it turns.
        5 => {
            let a = t * 0.8 + fi * 2.0 * PI / n as f32;
            let r = 1.2 + 3.8 * (0.5 + 0.5 * (t * 0.45).sin());
            Vec3::new(cx + a.cos() * r, 0.0, cz + a.sin() * r)
        }
        // Chase: fixed positions, the brightness jumps from head to head.
        6 => {
            let a = fi * 2.0 * PI / n as f32 + t * 0.1;
            let r = if stage { 3.0 } else { 4.2 };
            Vec3::new(cx + a.cos() * r, 0.0, cz + a.sin() * r)
        }
        _ if stage => Vec3::new(cx + (t * 0.5 + fi * 1.3).sin() * 6.0, 0.0, cz + 1.0 + (t * 0.4 + fi * 1.9).cos() * 4.0),
        _ => Vec3::new(cx + (t * (0.55 + fi * 0.08) + fi).sin() * 6.0, 0.0, cz + (t * 0.7 + fi * 1.7).cos() * 4.5),
    }
}

fn animate_lights(time: Res<Time>, show: Res<Show>,
    mut heads: Query<(&MovingHead, &mut Transform, &mut SpotLight, &Children), Without<Beam>>,
    mut beams: Query<(&MeshMaterial3d<StandardMaterial>, &mut Transform), (With<Beam>, Without<MovingHead>)>,
    mut points: Query<(&PulseLight,&mut PointLight)>,
    leds: Query<(&Led,&MeshMaterial3d<StandardMaterial>), Without<Beam>>,
    mut materials: ResMut<Assets<StandardMaterial>>) {
    let t = time.elapsed_secs() * show.speed;
    let beats = time.elapsed_secs() * show.beat / 60.0;
    let beat_phase = beats * 2.0 * PI;
    let pulse = ((beat_phase.sin()+1.0)*0.5).powi(6);
    let colors = match show.mode {
        0 => [(0.75,0.12,1.0),(0.04,0.8,1.0),(0.95,0.08,0.44)],
        1 => [(1.0,0.08,0.10),(0.05,0.18,1.0),(0.85,0.92,1.0)],
        2 => [(0.15,0.35,1.0),(0.1,0.75,0.65),(0.55,0.15,0.9)],
        4 => [(0.1,0.9,1.0),(0.9,0.2,1.0),(0.2,0.4,1.0)],
        5 => [(1.0,0.45,0.1),(1.0,0.1,0.45),(0.8,0.2,1.0)],
        6 => [(0.1,1.0,0.5),(0.1,0.7,1.0),(1.0,0.9,0.2)],
        _ => [(1.0,1.0,1.0),(1.0,0.07,0.15),(0.2,0.3,1.0)],
    };
    for (head,mut trans,mut light,children) in &mut heads {
        let n = if head.stage { 4 } else { 6 };
        let target = aim(show.mode, head.stage, head.index, n, t);
        let up = safe_up(target - head.base);
        *trans=Transform::from_translation(head.base).looking_at(target,up);
        let (r,g,b)=colors[(head.index + head.stage as usize)%3];
        light.color=Color::srgb(r,g,b);
        let i=head.index as f32;
        light.intensity=if !show.enabled { 0.0 } else { match show.mode {
            3 => if pulse>0.38 { 280_000.0 } else { 500.0 },
            6 => if beats.floor() as usize % n == head.index { 260_000.0 * (1.0 - 0.6 * beats.fract()) } else { 6_000.0 },
            _ => 110_000.0+65_000.0*pulse,
        } };
        light.outer_angle=match show.mode { 2 => 0.52, 4 => 0.20, 5 => 0.30, 6 => 0.17, _ => 0.23+(t+i).sin().abs()*0.22 };
        light.inner_angle=light.outer_angle*0.7;
        // Beam cone: from the lens down to the floor target.
        let len = (target - head.base).length() - 0.17;
        let radius = len * (light.outer_angle * 0.85).tan();
        let level = (light.intensity / 180_000.0).min(1.0);
        for child in children.iter() {
            let Ok((handle, mut beam)) = beams.get_mut(child) else { continue };
            beam.scale = Vec3::new(radius, radius, len);
            if let Some(mut material) = materials.get_mut(&handle.0) {
                material.base_color = Color::srgba(r, g, b, 0.13 * level);
            }
        }
    }
    for (point,mut light) in &mut points {
        let (r,g,b)=colors[(point.index+1)%3];
        light.color=Color::srgb(r,g,b);
        light.intensity=if show.enabled { 32_000.0+70_000.0*pulse } else { 0.0 };
    }
    for (led,mat_handle) in &leds {
        if let Some(mut material)=materials.get_mut(&mat_handle.0) {
            let x=(led.index / 6) as f32;
            let y=(led.index % 6) as f32;
            let wave=((x*0.39+y*0.54-t*2.6).sin()+1.0)*0.5;
            let (r,g,b)=colors[(led.index/17 + ((t*0.6) as usize))%3];
            let power=if show.enabled { 1.2+wave*9.0 + pulse*4.0 } else { 0.02 };
            material.emissive=LinearRgba::rgb(r*power,g*power,b*power);
        }
    }
}
fn update_hud(show: Res<Show>, quality: Res<quality::Quality>, mut hud: Single<&mut Text,With<StatusText>>) {
    let mode=PROGRAM_NAMES[show.mode];
    hud.0=format!("MOONCHILD  |  LICHT: {} {}  |  BPM: {:.0}  |  SPEED: {:.1}x  |  GRAFIK: {}",mode,if show.enabled {"ON"} else {"OFF"},show.beat,show.speed,quality.name());
}

#[cfg(test)]
mod tests {
    #[test]
    fn strobe_is_not_the_default_mode() { assert_eq!(super::Show::default().mode, 0); }
    #[test]
    fn every_program_aims_at_a_finite_floor_point() {
        for mode in 0..super::PROGRAMS {
            for stage in [false, true] {
                for i in 0..6 {
                    let p = super::aim(mode, stage, i, if stage { 4 } else { 6 }, 3.7);
                    assert!(p.is_finite() && p.y == 0.0 && p.x.abs() < 12.0 && p.z.abs() < 10.0, "{mode} {stage} {i} {p:?}");
                }
            }
        }
        assert_eq!(super::PROGRAM_NAMES.len(), super::PROGRAMS);
    }
}
