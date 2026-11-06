//! Moonchild 3D v0.1 - fully procedural, no paid assets.
mod club_section;
mod navigation;
mod dj_gear;

use std::f32::consts::PI;
use bevy::{
    camera::Hdr,
    image::{ImageAddressMode, ImageSamplerDescriptor},
    input::mouse::AccumulatedMouseMotion,
    mesh::VertexAttributeValues,
    post_process::bloom::Bloom,
    prelude::*,
};

const W: f32 = 24.0;
const D: f32 = 20.0;
const H: f32 = 7.0;

#[derive(Resource)]
struct Show { mode: usize, speed: f32, enabled: bool, beat: f32 }
impl Default for Show {
    fn default() -> Self { Self { mode: 0, speed: 1.0, enabled: true, beat: 130.0 } }
}
#[derive(Component)]
struct Player { yaw: f32, pitch: f32, motion: navigation::Motion }
#[derive(Component)]
struct MovingHead { index: usize }
#[derive(Component)]
struct Led { index: usize }
#[derive(Component)]
struct PulseLight { index: usize }
#[derive(Component)]
struct StatusText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..ImageSamplerDescriptor::linear()
            },
        }).set(WindowPlugin {
            primary_window: Some(Window {
                title: "MOONCHILD | Rust 3D Club v0.1".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(GlobalAmbientLight { color: Color::srgb(0.80, 0.73, 0.65), brightness: 220.0, ..default() })
        .init_resource::<Show>()
        .add_systems(Startup, setup)
        .add_systems(Update, (walk, controls, animate_lights, update_hud))
        .run();
}

fn pbr(
    materials: &mut Assets<StandardMaterial>,
    color: Color,
    metallic: f32,
    roughness: f32,
    reflectance: f32,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: color,
        metallic,
        perceptual_roughness: roughness,
        reflectance,
        ..default()
    })
}
fn glow(materials: &mut Assets<StandardMaterial>, r: f32, g: f32, b: f32) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial { base_color: Color::srgb(r.min(1.0),g.min(1.0),b.min(1.0)), emissive: LinearRgba::rgb(r*9.0,g*9.0,b*9.0), unlit: false, ..default() })
}
fn block(cmd: &mut Commands, meshes: &mut Assets<Mesh>, material: Handle<StandardMaterial>, pos: Vec3, size: Vec3) {
    let mut mesh: Mesh = Cuboid::new(size.x,size.y,size.z).into();
    // Use world-sized planar UVs on the six cuboid faces, instead of stretching
    // a single 256 px tile across an entire wall or the dance floor.
    let projected = match (mesh.attribute(Mesh::ATTRIBUTE_POSITION), mesh.attribute(Mesh::ATTRIBUTE_NORMAL)) {
        (Some(VertexAttributeValues::Float32x3(points)), Some(VertexAttributeValues::Float32x3(normals))) => {
            Some(points.iter().zip(normals).map(|(p,n)| {
                if n[1].abs() > 0.5 { [p[0]*0.5, p[2]*0.5] }
                else if n[0].abs() > 0.5 { [p[2]*0.5, p[1]*0.5] }
                else { [p[0]*0.5, p[1]*0.5] }
            }).collect::<Vec<[f32;2]>>())
        },
        _ => None,
    };
    if let Some(uv) = projected { mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv); }
    let _ = mesh.generate_tangents();
    cmd.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material), Transform::from_translation(pos)));
}
fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, asset_server: Res<AssetServer>) {
    let surfaces = club_section::Surfaces::new(&mut materials, &asset_server);
    // Physically based materials: glossy dance floor, brushed steel and matte concrete.
    let dark = pbr(&mut materials, Color::srgb(0.035,0.039,0.055), 0.0, 0.88, 0.20);
    let wall = surfaces.concrete.clone();
    let floor = surfaces.floor.clone();
    let metal = surfaces.steel.clone();
    let stage = pbr(&mut materials, Color::srgb(0.14,0.14,0.18), 0.14, 0.38, 0.55);
    let glossy_floor = surfaces.floor.clone();
    let violet = glow(&mut materials, 0.58,0.10,1.0);
    let pink = glow(&mut materials, 1.0,0.06,0.42);
    let cyan = glow(&mut materials, 0.01,0.8,1.0);
    let blue = glow(&mut materials, 0.04,0.14,1.0);
    let white = glow(&mut materials, 0.85,0.85,1.0);

    // Enclosed industrial warehouse: room 24m x 20m x 7m
    block(&mut commands,&mut meshes,floor.clone(),Vec3::new(0.0,-0.16,0.0),Vec3::new(W,0.32,D));
    // Separate polished dance-floor inset: its top is just above the main floor.
    block(&mut commands,&mut meshes,glossy_floor,Vec3::new(0.0,0.017,0.35),Vec3::new(14.5,0.03,9.1));
    block(&mut commands,&mut meshes,wall.clone(),Vec3::new(0.0,H/2.0,-D/2.0),Vec3::new(W,H,0.3));
    block(&mut commands,&mut meshes,wall.clone(),Vec3::new(-W/2.0,H/2.0,0.0),Vec3::new(0.3,H,D));
    block(&mut commands,&mut meshes,wall.clone(),Vec3::new(W/2.0,H/2.0,0.0),Vec3::new(0.3,H,D));
    block(&mut commands,&mut meshes,wall.clone(),Vec3::new(0.0,H/2.0,D/2.0),Vec3::new(W,H,0.3));
    block(&mut commands,&mut meshes,dark.clone(),Vec3::new(0.0,H+0.12,0.0),Vec3::new(W,0.25,D));
    // Concrete columns, metal beams and suspended ceiling trusses
    for &x in &[-10.6,10.6] {
        for &z in &[-7.8,0.0,7.8] { block(&mut commands,&mut meshes,metal.clone(),Vec3::new(x,3.45,z),Vec3::new(0.42,6.9,0.42)); }
    }
    for &z in &[-7.5,-2.0,3.5] {
        block(&mut commands,&mut meshes,metal.clone(),Vec3::new(0.0,6.54,z),Vec3::new(22.0,0.22,0.23));
        for &x in &[-8.0,0.0,8.0] {
            block(&mut commands,&mut meshes,metal.clone(),Vec3::new(x,6.15,z),Vec3::new(0.12,0.72,0.12));
        }
    }

    // Raised DJ stage at the far end, including decks, mixers, speakers, LED wall.
    block(&mut commands,&mut meshes,stage.clone(),Vec3::new(0.0,0.48,-7.55),Vec3::new(14.0,0.96,4.4));
    // Capsule profile softens the front and ends of the DJ booth.
    let mut booth_mesh: Mesh = Capsule3d::new(0.55, 5.7).into();
    let _ = booth_mesh.generate_tangents();
    commands.spawn((Mesh3d(meshes.add(booth_mesh)), MeshMaterial3d(dark.clone()),
        Transform::from_xyz(0.0,1.54,-8.0).with_rotation(Quat::from_rotation_z(PI / 2.0))));
    block(&mut commands,&mut meshes,cyan.clone(),Vec3::new(0.0,1.1,-7.43),Vec3::new(6.9,0.09,0.08));
    dj_gear::spawn(&mut commands, &mut meshes, &mut materials, &asset_server);
    for &x in &[-6.2,-5.25,5.25,6.2] {
        for y in 0..2 {
            block(&mut commands,&mut meshes,dark.clone(),Vec3::new(x,1.75+y as f32*1.12,-8.50),Vec3::new(0.83,1.02,0.78));
            block(&mut commands,&mut meshes,pink.clone(),Vec3::new(x,1.75+y as f32*1.12,-8.09),Vec3::new(0.37,0.04,0.04));
        }
    }
    // LED tiles, individually animated via material emissive color.
    for col in 0..20 {
        for row in 0..6 {
            let material = materials.add(StandardMaterial { base_color: Color::BLACK, emissive: LinearRgba::rgb(2.0,0.1,3.0), unlit: false, ..default() });
            commands.spawn((Mesh3d(meshes.add(Cuboid::new(0.49,0.49,0.07))), MeshMaterial3d(material),
                Transform::from_xyz((col as f32-9.5)*0.53, 3.15+row as f32*0.54,-9.76), Led { index: col*6+row }));
        }
    }

    // Alternating luminous lines around floor, side wall, lounge and bar.
    for i in 0..12 {
        let x = (i as f32-5.5)*1.50;
        block(&mut commands,&mut meshes,if i%2==0 {cyan.clone()} else {violet.clone()},Vec3::new(x,0.038,0.35),Vec3::new(0.11,0.008,9.1));
    }
    for &x in &[-11.6,11.6] {
        for &y in &[1.0,3.3,5.9] {
            block(&mut commands,&mut meshes,if y<3.0 {pink.clone()} else {blue.clone()},Vec3::new(x,y,0.0),Vec3::new(0.08,0.08,17.5));
        }
    }
    club_section::spawn(&mut commands, &mut meshes, &surfaces, cyan.clone(), pink.clone());
    for i in 0..5 {
        let x = -7.0 + i as f32 * 3.5;
        block(&mut commands,&mut meshes,metal.clone(),Vec3::new(x,6.05,-1.2),Vec3::new(0.65,0.35,0.65));
        commands.spawn((
            SpotLight { intensity: 180_000.0, color: Color::srgb(0.8,0.2,1.0), range: 35.0,
                inner_angle: 0.11, outer_angle: 0.38, shadow_maps_enabled: false, ..default() },
            Transform::from_xyz(x,5.6,-1.2).looking_at(Vec3::new(x,0.0,-1.0),Vec3::Z),
            MovingHead { index: i },
        ));
    }
    for (i,(x,z)) in [(-8.0,-4.0),(8.0,-4.0),(-8.0,5.0),(8.0,5.0)].iter().enumerate() {
        commands.spawn((PointLight { intensity: 95_000.0, range: 13.0, color: Color::srgb(0.35,0.12,1.0), shadow_maps_enabled: false, ..default() },
            Transform::from_xyz(*x,4.0,*z), PulseLight { index: i }));
    }
    // Permanent warm-white house lights, independent of the show-light toggle.
    // These illuminate the surfaces so PBR highlights remain visible even in Chill mode.
    for &(x,z) in &[
        (-8.0,-2.5),(-4.0,-1.0),(0.0,0.5),(4.0,-1.0),(8.0,-2.5),
        (-8.0,4.0),(-4.0,5.0),(0.0,5.5),(4.0,5.0),(8.0,4.0),
    ] {
        block(&mut commands,&mut meshes,metal.clone(),
            Vec3::new(x,6.27,z),Vec3::new(0.62,0.22,0.62));
        block(&mut commands,&mut meshes,white.clone(),
            Vec3::new(x,6.14,z),Vec3::new(0.36,0.035,0.36));
        commands.spawn((
            SpotLight {
                intensity: 95_000.0,
                color: Color::srgb(1.0,0.96,0.88),
                range: 18.0,
                inner_angle: 0.50,
                outer_angle: 0.78,
                shadow_maps_enabled: (x == 0.0 && z == 0.5) || (x == 8.0 && z == 4.0),
                ..default()
            },
            Transform::from_xyz(x,6.08,z)
                .looking_at(Vec3::new(x,0.0,z),Vec3::Z),
        ));
    }
    // Extra stage wash lights; subtle warm white illumination for the DJ deck.
    for &x in &[-4.5,0.0,4.5] {
        commands.spawn((
            SpotLight {
                intensity: 75_000.0,
                color: Color::srgb(1.0,0.90,0.77),
                range: 14.0,
                inner_angle: 0.44,
                outer_angle: 0.72,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_xyz(x,5.9,-5.8)
                .looking_at(Vec3::new(x*0.6,1.8,-8.0),Vec3::Y),
        ));
    }
    for &x in &[-10.8,10.8] {
        for &z in &[-7.0,0.0,7.0] {
            block(&mut commands,&mut meshes,white.clone(),Vec3::new(x,6.71,z),Vec3::new(0.25,0.13,0.25));
        }
    }
    // Camera starts at entrance looking toward DJ stage (-Z).
    commands.spawn((Camera3d::default(), Hdr, Bloom::NATURAL,
        Transform::from_xyz(0.0,1.78,7.5).looking_at(Vec3::new(0.0,2.5,-7.0),Vec3::Y),
        Player { yaw: 0.0, pitch: 0.045, motion: navigation::Motion::standing() }));
    // Overlay and instructions.
    commands.spawn((
        Text::new("MOONCHILD  |  3D CLUB   •   WASD bewegen  •  Leertaste springen  •  rechte Maustaste + Maus umsehen\n1 Neon  2 Rave  3 Chill  4 Strobe   |   L Licht an/aus   |   +/- Tempo  |  ESC schließen"),
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
    if keys.just_pressed(KeyCode::KeyL) { show.enabled=!show.enabled; }
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) { show.speed=(show.speed+0.2).min(3.0); show.beat=(show.beat+5.0).min(200.0); }
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) { show.speed=(show.speed-0.2).max(0.2); show.beat=(show.beat-5.0).max(60.0); }
    if keys.just_pressed(KeyCode::Escape) { exit.write(AppExit::Success); }
}
fn animate_lights(time: Res<Time>, show: Res<Show>,
    mut heads: Query<(&MovingHead,&mut Transform,&mut SpotLight)>,
    mut points: Query<(&PulseLight,&mut PointLight)>,
    leds: Query<(&Led,&MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>) {
    let t = time.elapsed_secs() * show.speed;
    let beat_phase = time.elapsed_secs() * show.beat / 60.0 * 2.0 * PI;
    let pulse = ((beat_phase.sin()+1.0)*0.5).powi(6);
    let colors = match show.mode {
        0 => [(0.75,0.12,1.0),(0.04,0.8,1.0),(0.95,0.08,0.44)],
        1 => [(1.0,0.08,0.10),(0.05,0.18,1.0),(0.85,0.92,1.0)],
        2 => [(0.15,0.35,1.0),(0.1,0.75,0.65),(0.55,0.15,0.9)],
        _ => [(1.0,1.0,1.0),(1.0,0.07,0.15),(0.2,0.3,1.0)],
    };
    for (head,mut trans,mut light) in &mut heads {
        let i=head.index as f32;
        let target=Vec3::new((t*(0.55+i*0.08)+i).sin()*7.0,0.0,(t*0.7+i*1.7).cos()*4.0);
        *trans=Transform::from_translation(Vec3::new(-7.0+i*3.5,5.6,-1.2)).looking_at(target,Vec3::Y);
        let (r,g,b)=colors[head.index%3];
        light.color=Color::srgb(r,g,b);
        light.intensity=if !show.enabled { 0.0 } else if show.mode==3 { if pulse>0.38 { 280_000.0 } else { 500.0 } } else { 110_000.0+65_000.0*pulse };
        light.outer_angle=if show.mode==2 { 0.52 } else { 0.23+(t+i).sin().abs()*0.22 };
        light.inner_angle=light.outer_angle*0.7;
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
fn update_hud(show: Res<Show>, mut hud: Single<&mut Text,With<StatusText>>) {
    let mode=["NEON","RAVE","CHILL","STROBE"][show.mode];
    hud.0=format!("MOONCHILD  •  LICHT: {} {}  •  BPM: {:.0}  •  SPEED: {:.1}x",mode,if show.enabled {"ON"} else {"OFF"},show.beat,show.speed);
}
