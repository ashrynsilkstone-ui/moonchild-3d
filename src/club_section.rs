//! Original Moonchild architecture and materials. Dimensions are in metres.
use std::f32::consts::{PI, TAU};
use bevy::{asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology};

fn textured(materials: &mut Assets<StandardMaterial>, server: &AssetServer, name: &str, metallic: f32) -> Handle<StandardMaterial> {
    let base = server.load(format!("textures/moonchild/{name}_base.png"));
    let normal = server.load(format!("textures/moonchild/{name}_normal.png"));
    let orm: Handle<Image> = server.load(format!("textures/moonchild/{name}_orm.png"));
    materials.add(StandardMaterial {
        base_color_texture: Some(base), normal_map_texture: Some(normal),
        metallic_roughness_texture: Some(orm.clone()), occlusion_texture: Some(orm),
        metallic, perceptual_roughness: 1.0, ..default()
    })
}

pub struct Surfaces {
    pub concrete: Handle<StandardMaterial>,
    pub steel: Handle<StandardMaterial>,
    pub floor: Handle<StandardMaterial>,
    leather: Handle<StandardMaterial>,
    grille: Handle<StandardMaterial>,
}
impl Surfaces {
    pub fn new(materials: &mut Assets<StandardMaterial>, server: &AssetServer) -> Self {
        Self { concrete: textured(materials,server,"concrete",1.0), steel: textured(materials,server,"steel",1.0),
            floor: textured(materials,server,"floor",1.0), leather: textured(materials,server,"leather",1.0),
            grille: textured(materials,server,"grille",1.0) }
    }
}

fn mesh_with_tangents(mut m: Mesh) -> Mesh { let _ = m.generate_tangents(); m }
fn piece(c: &mut Commands, meshes: &mut Assets<Mesh>, shape: impl Into<Mesh>, mat: Handle<StandardMaterial>, transform: Transform) {
    c.spawn((Mesh3d(meshes.add(mesh_with_tangents(shape.into()))), MeshMaterial3d(mat), transform));
}
fn rod(c: &mut Commands, meshes: &mut Assets<Mesh>, mat: Handle<StandardMaterial>, a: Vec3, b: Vec3, radius: f32) {
    let d = b-a;
    if d.length_squared() < 0.0001 { return; }
    piece(c,meshes,Cylinder::new(radius,d.length()),mat,Transform::from_translation((a+b)*0.5).with_rotation(Quat::from_rotation_arc(Vec3::Y,d.normalize())));
}

// Extruded curved gallery; separately normalled faces keep its silhouette crisp.
fn gallery_mesh() -> Mesh {
    let mut positions: Vec<[f32;3]> = vec![];
    let mut normals: Vec<[f32;3]> = vec![];
    let mut uvs: Vec<[f32;2]> = vec![];
    let mut indices: Vec<u32> = vec![];
    let mut quad = |a: Vec3,b: Vec3,c: Vec3,d: Vec3, uv: [[f32;2];4]| {
        let normal = (b-a).cross(c-a).normalize().to_array();
        let i=positions.len() as u32;
        for p in [a,b,c,d] { positions.push(p.to_array()); normals.push(normal); }
        uvs.extend(uv); indices.extend([i,i+1,i+2,i+2,i+1,i+3]);
    };
    for i in 0..80 {
        let x0=-10.3+20.6*i as f32/80.0; let x1=-10.3+20.6*(i+1) as f32/80.0;
        let z0=5.4+0.025*x0*x0; let z1=5.4+0.025*x1*x1;
        let a=Vec3::new(x0,3.0,z0); let b=Vec3::new(x0,3.0,9.55);
        let c=Vec3::new(x1,3.0,z1); let d=Vec3::new(x1,3.0,9.55);
        let uv=[[x0*0.55,z0*0.55],[x0*0.55,9.55*0.55],[x1*0.55,z1*0.55],[x1*0.55,9.55*0.55]];
        quad(a,b,c,d,uv);
        let low=Vec3::new(0.0,-0.28,0.0);
        quad(a+low,c+low,b+low,d+low,uv);
        quad(a,c,a+low,c+low,[[x0*0.55,0.0],[x1*0.55,0.0],[x0*0.55,0.16],[x1*0.55,0.16]]);
    }
    let mut mesh=Mesh::new(PrimitiveTopology::TriangleList,RenderAssetUsages::MAIN_WORLD|RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION,positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL,normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0,uvs)
        .with_inserted_indices(Indices::U32(indices));
    let _=mesh.generate_tangents(); mesh
}

pub fn spawn(c: &mut Commands, meshes: &mut Assets<Mesh>, s: &Surfaces, cyan: Handle<StandardMaterial>, pink: Handle<StandardMaterial>) {
    c.spawn((Mesh3d(meshes.add(gallery_mesh())),MeshMaterial3d(s.floor.clone())));
    // A gentle twenty-step staircase, matching navigation.rs exactly.
    for i in 0..20 {
        let z0=1.2+6.6*i as f32/20.0; let z1=1.2+6.6*(i+1) as f32/20.0;
        let h=3.0*(i+1) as f32/20.0;
        piece(c,meshes,Cuboid::new(1.6,h,z1-z0),s.steel.clone(),Transform::from_xyz(-9.0,h/2.0,(z0+z1)/2.0));
    }
    // Tubular balustrade follows the gallery curve; the stair opening is at the left end.
    let at=|x:f32,y:f32| Vec3::new(x,y,5.4+0.025*x*x);
    for i in 0..48 {
        let x0=-8.15+18.35*i as f32/48.0; let x1=-8.15+18.35*(i+1) as f32/48.0;
        for y in [3.48,3.98] { rod(c,meshes,s.steel.clone(),at(x0,y),at(x1,y),0.035); }
        if i%3==0 { rod(c,meshes,s.steel.clone(),at(x0,3.02),at(x0,4.0),0.035); }
        rod(c,meshes,cyan.clone(),at(x0,3.06),at(x1,3.06),0.018);
    }
    for &x in &[-9.85,-8.15] {
        rod(c,meshes,s.steel.clone(),Vec3::new(x,0.9,1.2),Vec3::new(x,3.9,7.8),0.04);
        for i in 0..7 { let t=i as f32/6.0; let z=1.2+t*6.6; let y=t*3.0;
            rod(c,meshes,s.steel.clone(),Vec3::new(x,y,z),Vec3::new(x,y+0.9,z),0.03); }
    }
    // Two rounded, segmented leather sofas on the upper level.
    let sphere=meshes.add(mesh_with_tangents(Sphere::new(1.0).mesh().uv(40,24)));
    for &cx in &[-4.2,4.2] {
        for (y,z,scale) in [(3.45,8.62,Vec3::new(1.08,0.24,0.52)),(3.92,9.0,Vec3::new(1.12,0.52,0.24))] {
            c.spawn((Mesh3d(sphere.clone()),MeshMaterial3d(s.leather.clone()),Transform::from_xyz(cx,y,z).with_scale(scale)));
        }
        for &dx in &[-1.0,1.0] { c.spawn((Mesh3d(sphere.clone()),MeshMaterial3d(s.leather.clone()),Transform::from_xyz(cx+dx,3.68,8.65).with_scale(Vec3::new(0.24,0.42,0.58)))); }
        piece(c,meshes,Cylinder::new(0.52,0.08),s.steel.clone(),Transform::from_xyz(cx,3.55,7.7));
    }
    // Semicircular bar and counter, behind the dance floor on the right.
    let center=Vec3::new(8.35,0.0,2.5); let segments=36;
    for i in 0..segments {
        let a=PI*0.5+PI*i as f32/segments as f32; let b=PI*0.5+PI*(i+1) as f32/segments as f32;
        let point=|t:f32,r:f32,y:f32| center+Vec3::new(r*t.cos(),y,r*t.sin());
        for (r,y,rad,mat) in [(1.85,1.1,0.12,s.steel.clone()),(1.85,0.14,0.045,pink.clone()),(1.85,0.95,0.045,cyan.clone())] {
            rod(c,meshes,mat,point(a,r,y),point(b,r,y),rad);
        }
        let middle=(a+b)*0.5; let p=point(middle,1.85,0.53);
        piece(c,meshes,Cuboid::new(0.18,1.0,0.17),s.grille.clone(),Transform::from_translation(p).with_rotation(Quat::from_rotation_y(-middle)));
    }
    // Backlit bottle display at the bar's wall side; every object is procedural.
    for &height in &[1.58, 2.13] {
        piece(c,meshes,Cuboid::new(0.38,0.07,3.5),s.steel.clone(),Transform::from_xyz(10.9,height,2.5));
        rod(c,meshes,cyan.clone(),Vec3::new(10.69,height+0.02,0.8),Vec3::new(10.69,height+0.02,4.2),0.018);
        for i in 0..9 {
            let z=0.95+i as f32*0.38;
            let bottle_mat=if i%3==0 { pink.clone() } else { s.steel.clone() };
            piece(c,meshes,Cylinder::new(0.075,0.25),bottle_mat.clone(),Transform::from_xyz(10.9,height+0.17,z));
            piece(c,meshes,Cylinder::new(0.03,0.09),bottle_mat,Transform::from_xyz(10.9,height+0.34,z));
        }
    }
    // Moonchild's suspended double halo and radial strips; no imported logos.
    for (radius,tube,mat) in [(3.4,0.12,s.steel.clone()),(3.15,0.035,cyan.clone()),(3.65,0.035,pink.clone())] {
        piece(c,meshes,Torus::new(radius-tube,radius+tube),mat,Transform::from_xyz(0.0,6.12,0.2));
    }
    for i in 0..16 { let angle=TAU*i as f32/16.0; let (sn,cs)=angle.sin_cos();
        rod(c,meshes,if i%2==0 {cyan.clone()} else {pink.clone()},Vec3::new(cs*0.75,6.13,0.2+sn*0.75),Vec3::new(cs*3.05,6.13,0.2+sn*3.05),0.025); }
}
