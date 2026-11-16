//! Original Moonchild DJ station. All dimensions are metres.
//!
//! Authoring space: +Z is the operator/front edge. The whole station turns by
//! PI so its controls face the DJ behind the booth (-Z), not the dance floor.
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use bevy::{image::ImageLoaderSettings, prelude::*};
use crate::{lod, look::MipQueue, navigation::layout::BOOTH_TOP, shapes::{self, MeshData}};

const CENTER_Z: f32 = -7.80;
const DECK_WIDTH: f32 = 0.33;
const MIXER_WIDTH: f32 = 0.43;
const DEPTH: f32 = 0.45;
const FACE_Y: f32 = 0.067;
const DECK_CENTERS: [f32; 2] = [-0.415, 0.415];
const CHANNEL_X: [f32; 4] = [-0.145, -0.075, -0.005, 0.065];
const SCREEN_TILT: f32 = 0.44;

fn station_rotation() -> Quat { Quat::from_rotation_y(PI) }
fn world_position(local: Vec3) -> Vec3 { Vec3::new(0.0, BOOTH_TOP, CENTER_Z) + station_rotation() * local }

/// Explicit UVs: image top is at the back (-Z); its bottom is nearest the DJ.
fn panel(width: f32, depth: f32) -> MeshData {
    let mut m = MeshData::default();
    for (x, z, u, v) in [(-0.5,-0.5,0.0,0.0),(0.5,-0.5,1.0,0.0),(-0.5,0.5,0.0,1.0),(0.5,0.5,1.0,1.0)] {
        m.vertex(Vec3::new(x * width, 0.0, z * depth), Vec3::Y, Vec2::new(u,v));
    }
    m.tri(0,2,1); m.tri(1,2,3);
    m
}

struct Rig<'a, 'w, 's> { c: &'a mut Commands<'w,'s>, meshes: &'a mut Assets<Mesh> }
impl Rig<'_, '_, '_> {
    fn mesh(&mut self, m: MeshData) -> Handle<Mesh> { self.meshes.add(m.build()) }
    fn place(&mut self, mesh: &Handle<Mesh>, mat: &Handle<StandardMaterial>, p: Vec3, q: Quat, detail: bool) {
        let mut e = self.c.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()),
            Transform::from_translation(world_position(p)).with_rotation(station_rotation() * q)));
        // Fine controls vanish at a distance; the textured panels and shells remain.
        if detail { e.insert((lod::near(lod::SMALL, false), bevy::light::NotShadowCaster)); }
    }
    fn put(&mut self, mesh: &Handle<Mesh>, mat: &Handle<StandardMaterial>, p: Vec3, detail: bool) {
        self.place(mesh, mat, p, Quat::IDENTITY, detail);
    }
}

fn map(server: &AssetServer, queue: &mut MipQueue, path: String, data: bool, normal: bool) -> Handle<Image> {
    let image = if data {
        server.load_builder().with_settings(|s: &mut ImageLoaderSettings| s.is_srgb = false).load(path)
    } else { server.load(path) };
    queue.watch_image(&image, normal);
    image
}

fn face_material(materials: &mut Assets<StandardMaterial>, server: &AssetServer, queue: &mut MipQueue, name: &str, emission: bool) -> Handle<StandardMaterial> {
    let base = map(server, queue, format!("textures/dj/{name}_base.png"), false, false);
    let normal = map(server, queue, format!("textures/dj/{name}_normal.png"), true, true);
    let orm = map(server, queue, format!("textures/dj/{name}_orm.png"), true, false);
    let emissive = if emission { Some(map(server, queue, format!("textures/dj/{name}_emissive.png"), false, false)) } else { None };
    materials.add(StandardMaterial { base_color_texture: Some(base), normal_map_texture: Some(normal),
        metallic_roughness_texture: Some(orm.clone()), occlusion_texture: Some(orm),
        metallic: 1.0, perceptual_roughness: 1.0, emissive_texture: emissive,
        emissive: if emission { LinearRgba::rgb(0.7,0.7,0.7) } else { LinearRgba::rgb(0.0,0.0,0.0) }, ..default() })
}

fn cable(rig: &mut Rig, rubber: &Handle<StandardMaterial>, x: f32) {
    let p0 = Vec3::new(x,0.034,-DEPTH*0.5-0.005);
    let p1 = p0 + Vec3::new(0.0,0.0,-0.055);
    let p2 = p0 + Vec3::new(0.025,0.012,-0.10);
    let p3 = p0 + Vec3::new(0.04,-0.065,-0.13);
    let points: Vec<Vec3> = (0..=24).map(|i| {
        let t = i as f32 / 24.0; let u = 1.0-t;
        p0*u*u*u + p1*3.0*u*u*t + p2*3.0*u*t*t + p3*t*t*t
    }).collect();
    let mesh = rig.mesh(shapes::tube(&points,0.0028,10,1.0));
    rig.put(&mesh,rubber,Vec3::ZERO,true);
}

pub fn spawn(c: &mut Commands, meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>, server: &AssetServer, queue: &mut MipQueue) {
    let shell = materials.add(StandardMaterial { base_color: Color::srgb(0.035,0.040,0.046), metallic:0.55, perceptual_roughness:0.42, ..default() });
    let metal = materials.add(StandardMaterial { base_color: Color::srgb(0.29,0.31,0.33), metallic:0.9, perceptual_roughness:0.27, ..default() });
    let rubber = materials.add(StandardMaterial { base_color: Color::srgb(0.023,0.026,0.030), perceptual_roughness:0.72, ..default() });
    let white = materials.add(StandardMaterial { base_color: Color::srgb(0.68,0.72,0.73), perceptual_roughness:0.45, ..default() });
    let cyan = materials.add(StandardMaterial { base_color: Color::srgb(0.06,0.24,0.28), emissive: LinearRgba::rgb(0.08,1.4,1.7), perceptual_roughness:0.45, ..default() });
    let blue = materials.add(StandardMaterial { base_color: Color::srgb(0.035,0.07,0.12), emissive: LinearRgba::rgb(0.04,0.17,0.32), perceptual_roughness:0.5, ..default() });
    let green = materials.add(StandardMaterial { base_color: Color::srgb(0.08,0.25,0.14), emissive: LinearRgba::rgb(0.1,1.2,0.3), ..default() });
    let deck_face = face_material(materials,server,queue,"deck",true);
    let mixer_face = face_material(materials,server,queue,"mixer",true);
    let jog_face = face_material(materials,server,queue,"jog",false);
    let display_map = map(server,queue,"textures/dj/display_base.png".into(),false,false);
    let display_mat = materials.add(StandardMaterial { base_color: Color::BLACK, emissive_texture: Some(display_map),
        emissive: LinearRgba::rgb(1.15,1.15,1.15), perceptual_roughness:0.32, ..default() });
    // A focused work light illuminates the operator's controls.
    c.spawn((SpotLight { intensity: 35_000.0, range: 5.0, color: Color::srgb(1.0,0.95,0.86),
        inner_angle:0.38, outer_angle:0.68, shadow_maps_enabled:false, ..default() },
        Transform::from_xyz(0.0,3.9,CENTER_Z).looking_at(Vec3::new(0.0,BOOTH_TOP+0.1,CENTER_Z),Vec3::Z)));
    let mut rig = Rig { c, meshes };
    let deck_body = rig.mesh(shapes::rounded_box(Vec3::new(DECK_WIDTH,0.055,DEPTH),0.012,8,1.0));
    let mixer_body = rig.mesh(shapes::rounded_box(Vec3::new(MIXER_WIDTH,0.055,DEPTH),0.012,8,1.0));
    let deck_panel = rig.mesh(panel(DECK_WIDTH-0.010,DEPTH-0.010));
    let mixer_panel = rig.mesh(panel(MIXER_WIDTH-0.010,DEPTH-0.010));
    let foot = rig.mesh(shapes::lathe(&[Vec2::new(0.0,-0.004),Vec2::new(0.012,-0.004),Vec2::new(0.012,0.004),Vec2::new(0.0,0.004)],24,1.0));
    let knob = rig.mesh(shapes::fluted_lathe(&[Vec2::new(0.0,0.0),Vec2::new(0.0105,0.0),Vec2::new(0.011,0.003),Vec2::new(0.010,0.018),Vec2::new(0.0,0.018)],14,0.0013,4,1.0));
    let browse = rig.mesh(shapes::fluted_lathe(&[Vec2::new(0.0,0.0),Vec2::new(0.014,0.0),Vec2::new(0.014,0.018),Vec2::new(0.012,0.023),Vec2::new(0.0,0.023)],18,0.0014,4,1.0));
    let knob_mark = rig.mesh(shapes::rounded_box(Vec3::new(0.0014,0.001,0.005),0.0003,2,1.0));
    let fader = rig.mesh(shapes::rounded_box(Vec3::new(0.032,0.009,0.020),0.0025,5,1.0));
    let fader_mark = rig.mesh(shapes::rounded_box(Vec3::new(0.022,0.001,0.0015),0.0004,2,1.0));
    let pad = rig.mesh(shapes::rounded_box(Vec3::new(0.044,0.006,0.020),0.003,5,1.0));
    let cue_cap = rig.mesh(shapes::lathe(&[Vec2::new(0.0,0.0),Vec2::new(0.014,0.0),Vec2::new(0.014,0.006),Vec2::new(0.0,0.006)],40,1.0));
    let cue_ring = rig.mesh(shapes::lathe(&[Vec2::new(0.014,0.0),Vec2::new(0.0157,0.0),Vec2::new(0.0157,0.002),Vec2::new(0.014,0.002)],48,1.0));
    let screw = rig.mesh(shapes::lathe(&[Vec2::new(0.0,0.0),Vec2::new(0.0027,0.0),Vec2::new(0.0027,0.001),Vec2::new(0.0,0.001)],16,1.0));
    let vent = rig.mesh(shapes::rounded_box(Vec3::new(0.001,0.004,0.018),0.0004,2,1.0));
    let jack = rig.mesh(shapes::lathe(&[Vec2::new(0.004,0.0),Vec2::new(0.006,0.0),Vec2::new(0.006,0.004),Vec2::new(0.004,0.004)],24,1.0));

    for &cx in &DECK_CENTERS {
        rig.put(&deck_body,&shell,Vec3::new(cx,0.0385,0.0),false);
        rig.put(&deck_panel,&deck_face,Vec3::new(cx,FACE_Y,0.0),false);
        for x in [-0.13f32,0.13] { for z in [-0.19f32,0.19] {
            rig.put(&foot,&rubber,Vec3::new(cx+x,0.005,z),true);
        }}
        for x in [-0.148f32,0.148] { for z in [-0.204f32,0.204] {
            rig.put(&screw,&metal,Vec3::new(cx+x,FACE_Y+0.001,z),true);
        }}
        for sx in [-1.0f32,1.0] { for z in [-0.11f32,-0.08,-0.05,-0.02,0.01,0.04] {
            rig.put(&vent,&rubber,Vec3::new(cx+sx*(DECK_WIDTH*0.5-0.0003),0.038,z),true);
        }}
        // A raised jog wheel with a machined rim and its own mapped top disc.
        let wheel = rig.mesh(shapes::lathe(&[Vec2::new(0.0,0.0),Vec2::new(0.091,0.0),Vec2::new(0.093,0.005),
            Vec2::new(0.092,0.021),Vec2::new(0.083,0.026),Vec2::new(0.0,0.026)],96,1.0));
        let rim = rig.mesh(shapes::lathe(&[Vec2::new(0.088,0.0),Vec2::new(0.092,0.0),Vec2::new(0.092,0.004),Vec2::new(0.088,0.004)],96,1.0));
        let mut top = shapes::annulus(Vec3::ZERO,0.0,0.081,0.0,TAU,96,1.0);
        for uv in &mut top.uvs { uv[0]=uv[0]/0.162+0.5; uv[1]=uv[1]/0.162+0.5; }
        let top = rig.mesh(top);
        let wheel_p=Vec3::new(cx-0.025,FACE_Y+0.001,0.04);
        rig.put(&wheel,&rubber,wheel_p,false);
        rig.put(&rim,&metal,wheel_p+Vec3::Y*0.018,false);
        rig.put(&top,&jog_face,wheel_p+Vec3::Y*0.0265,false);
        for i in 0..4 {
            rig.put(&pad,if i==0 { &cyan } else { &blue },Vec3::new(cx-0.12+i as f32*0.06,FACE_Y+0.004,-0.075),true);
        }
        for (z,light) in [(0.10,&cyan),(0.17,&green)] {
            rig.put(&cue_cap,&rubber,Vec3::new(cx-0.138,FACE_Y+0.001,z),true);
            rig.put(&cue_ring,light,Vec3::new(cx-0.138,FACE_Y+0.006,z),true);
        }
        rig.put(&browse,&rubber,Vec3::new(cx+0.125,FACE_Y+0.001,-0.155),true);
        rig.put(&fader,&metal,Vec3::new(cx+0.130,FACE_Y+0.006,0.055),true);
        rig.put(&fader_mark,&rubber,Vec3::new(cx+0.130,FACE_Y+0.011,0.055),true);
        // Tilted display: the outward normal has a +Z operator-facing component.
        let q=Quat::from_rotation_x(SCREEN_TILT);
        let screen_p=Vec3::new(cx-0.015,FACE_Y+0.041,-0.159);
        let housing=rig.mesh(shapes::rounded_box(Vec3::new(0.248,0.014,0.118),0.007,6,1.0));
        let screen=rig.mesh(panel(0.229,0.093));
        rig.place(&housing,&shell,screen_p-q*Vec3::Y*0.008,q,false);
        rig.place(&screen,&display_mat,screen_p,q,false);
        for x in [-0.07f32,0.07] {
            rig.place(&jack,&metal,Vec3::new(cx+x,0.034,-DEPTH*0.5-0.002),Quat::from_rotation_x(FRAC_PI_2),true);
            cable(&mut rig,&rubber,cx+x);
        }
    }

    rig.put(&mixer_body,&shell,Vec3::new(0.0,0.0385,0.0),false);
    rig.put(&mixer_panel,&mixer_face,Vec3::new(0.0,FACE_Y,0.0),false);
    for x in [-0.18f32,0.18] { for z in [-0.19f32,0.19] { rig.put(&foot,&rubber,Vec3::new(x,0.005,z),true); }}
    for (ch,&x) in CHANNEL_X.iter().enumerate() {
        for z in [-0.155f32,-0.095,-0.035,0.025] {
            rig.put(&knob,&rubber,Vec3::new(x,FACE_Y+0.001,z),true);
            rig.put(&knob_mark,&white,Vec3::new(x,FACE_Y+0.0195,z-0.004),true);
        }
        let z=0.137-(ch%3) as f32*0.015;
        rig.put(&fader,&metal,Vec3::new(x,FACE_Y+0.006,z),true);
        rig.put(&fader_mark,&rubber,Vec3::new(x,FACE_Y+0.011,z),true);
        let cap=rig.mesh(shapes::rounded_box(Vec3::new(0.027,0.005,0.012),0.002,4,1.0));
        rig.put(&cap,if ch==1 { &cyan } else { &blue },Vec3::new(x,FACE_Y+0.003,0.074),true);
    }
    for z in [-0.15f32,-0.075,0.0] {
        rig.put(&browse,&rubber,Vec3::new(0.16,FACE_Y+0.001,z),true);
        rig.put(&knob_mark,&white,Vec3::new(0.16,FACE_Y+0.0245,z-0.005),true);
    }
    let cross=rig.mesh(shapes::rounded_box(Vec3::new(0.022,0.010,0.029),0.003,5,1.0));
    rig.put(&cross,&metal,Vec3::new(-0.025,FACE_Y+0.006,0.198),true);
    for x in [-0.16f32,0.16] {
        rig.place(&jack,&metal,Vec3::new(x,0.034,-DEPTH*0.5-0.002),Quat::from_rotation_x(FRAC_PI_2),true);
        cable(&mut rig,&rubber,x);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::navigation::layout::{BOOTH_RADIUS, BOOTH_CENTER_Z};
    #[test]
    fn controls_and_displays_face_the_dj_behind_the_booth() {
        assert!((station_rotation()*Vec3::Z).dot(Vec3::NEG_Z)>0.999);
        let normal=station_rotation()*Quat::from_rotation_x(SCREEN_TILT)*Vec3::Y;
        assert!(normal.y>0.8 && normal.z < -0.3);
        assert!(world_position(Vec3::Z*DEPTH*0.5).z < CENTER_Z);
    }
    #[test]
    fn all_device_corners_rest_inside_the_curved_booth_top() {
        for (cx,width) in [(DECK_CENTERS[0],DECK_WIDTH),(0.0,MIXER_WIDTH),(DECK_CENTERS[1],DECK_WIDTH)] {
            for x in [-width*0.5,width*0.5] { for z in [-DEPTH*0.5,DEPTH*0.5] {
                let p=world_position(Vec3::new(cx+x,0.0,z));
                let radius=Vec2::new(p.x,p.z-BOOTH_CENTER_Z).length();
                assert!((BOOTH_RADIUS-0.48..BOOTH_RADIUS+0.34).contains(&radius));
                assert!((p.y-BOOTH_TOP).abs()<1e-5);
            }}
        }
    }
    #[test]
    fn faceplate_uv_top_is_at_the_rear_and_triangles_face_up() {
        let m=panel(DECK_WIDTH,DEPTH);
        assert_eq!(m.uvs[0],[0.0,0.0]);
        assert!(m.positions[0][2]<0.0 && m.positions[2][2]>0.0);
        for t in m.indices.chunks_exact(3) {
            let a=Vec3::from(m.positions[t[0] as usize]);
            let b=Vec3::from(m.positions[t[1] as usize]);
            let c=Vec3::from(m.positions[t[2] as usize]);
            assert!((b-a).cross(c-a).y>0.0);
        }
    }
}
