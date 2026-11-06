//! Original Moonchild DJ equipment: textured faceplates plus raised hardware.
use bevy::prelude::*;

fn object(c: &mut Commands, meshes: &mut Assets<Mesh>, shape: impl Into<Mesh>, material: Handle<StandardMaterial>, transform: Transform) {
    c.spawn((Mesh3d(meshes.add(shape)), MeshMaterial3d(material), transform));
}

pub fn spawn(c: &mut Commands, meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>, server: &AssetServer) {
    let casing=materials.add(StandardMaterial { base_color: Color::srgb(0.045,0.052,0.065), metallic:0.55, perceptual_roughness:0.42, ..default() });
    let metal=materials.add(StandardMaterial { base_color: Color::srgb(0.27,0.30,0.33), metallic:0.88, perceptual_roughness:0.30, ..default() });
    let rubber=materials.add(StandardMaterial { base_color: Color::srgb(0.09,0.10,0.11), metallic:0.04, perceptual_roughness:0.65, ..default() });
    let deck_face=materials.add(StandardMaterial { base_color_texture:Some(server.load("textures/dj/deck_base.png")), metallic:0.10, perceptual_roughness:0.55, ..default() });
    let mixer_face=materials.add(StandardMaterial { base_color_texture:Some(server.load("textures/dj/mixer_base.png")), metallic:0.10, perceptual_roughness:0.58, ..default() });
    let deck_mesh=meshes.add(Plane3d::default().mesh().size(1.50,0.88));
    let mixer_mesh=meshes.add(Plane3d::default().mesh().size(1.20,0.86));
    let wheel_ring=meshes.add(Torus::new(0.229,0.240));
    let knob=meshes.add(Cylinder::new(0.023,0.035));
    // Instruments rest on top of the curved booth, rather than floating at stage height.
    let center_z=-7.80;
    for &center_x in &[-2.0,2.0] {
        object(c,meshes,Cuboid::new(1.53,0.16,0.91),casing.clone(),Transform::from_xyz(center_x,2.16,center_z));
        c.spawn((Mesh3d(deck_mesh.clone()),MeshMaterial3d(deck_face.clone()),Transform::from_xyz(center_x,2.242,center_z)));
        let wheel_x=center_x+(510.0/1536.0-0.5)*1.50;
        let wheel_z=center_z+(550.0/900.0-0.5)*0.88;
        c.spawn((Mesh3d(wheel_ring.clone()),MeshMaterial3d(metal.clone()),Transform::from_xyz(wheel_x,2.255,wheel_z)));
        for &(x,y) in &[(908.0,735.0),(1015.0,735.0)] {
            let px=center_x+(x/1536.0-0.5)*1.50;
            let pz=center_z+(y/900.0-0.5)*0.88;
            c.spawn((Mesh3d(knob.clone()),MeshMaterial3d(rubber.clone()),Transform::from_xyz(px,2.26,pz)));
        }
    }
    object(c,meshes,Cuboid::new(1.23,0.16,0.89),casing,Transform::from_xyz(0.0,2.16,center_z));
    c.spawn((Mesh3d(mixer_mesh),MeshMaterial3d(mixer_face),Transform::from_xyz(0.0,2.242,center_z)));
    // Raised gain/EQ/filter knobs and the four channel fader caps.
    for &x in &[230.0,410.0,590.0,770.0] {
        for y in [170.0,258.0,346.0,434.0,522.0] {
            let px=(x/1200.0-0.5)*1.20;
            let pz=center_z+(y/860.0-0.5)*0.86;
            c.spawn((Mesh3d(knob.clone()),MeshMaterial3d(rubber.clone()),Transform::from_xyz(px,2.26,pz)));
        }
    }
    for (i,x) in [230.0,410.0,590.0,770.0].iter().enumerate() {
        let y=715.0-(i%3) as f32*20.0;
        object(c,meshes,Cuboid::new(0.066,0.025,0.029),metal.clone(),Transform::from_xyz((x/1200.0-0.5)*1.20,2.26,center_z+(y/860.0-0.5)*0.86));
    }
}
