//! Graphics quality presets (Mittel / Hoch / Ultra), switchable at runtime with F2.
//!
//! Start preset: `MOONCHILD_QUALITY=mittel|hoch|ultra` (default: hoch).
//! - Mittel: 1024 px shadow maps, Gaussian shadow filtering, SSAO medium and
//!   the cheap additive beam cones of the moving heads.
//! - All presets render deferred; Hoch and Ultra add screen-space reflections.
//! - Hoch: 2048 px shadow maps, temporal shadow filtering, SSAO high and real
//!   ray-marched volumetric light (Bevy `VolumetricFog`) in a light haze; the
//!   moving heads then cast shadows, so geometry cuts their beams.
//! - Ultra: 4096 px shadow maps, SSAO ultra, more ray-marching steps, and the
//!   shadowed warm ceiling spots also scatter into the haze.
use bevy::{
    light::{FogVolume, PointLightShadowMap, ShadowFilteringMethod, VolumetricFog, VolumetricLight},
    pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel, ScreenSpaceReflections},
    prelude::*,
};

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Quality { Mittel, Hoch, Ultra }

impl Quality {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "mittel" | "medium" | "low" | "niedrig" => Self::Mittel,
            "ultra" => Self::Ultra,
            _ => Self::Hoch,
        }
    }
    pub fn from_env() -> Self { Self::parse(&std::env::var("MOONCHILD_QUALITY").unwrap_or_default()) }
    pub fn next(self) -> Self { match self { Self::Mittel => Self::Hoch, Self::Hoch => Self::Ultra, Self::Ultra => Self::Mittel } }
    pub fn name(self) -> &'static str { match self { Self::Mittel => "MITTEL", Self::Hoch => "HOCH", Self::Ultra => "ULTRA" } }
    pub fn shadow_map_size(self) -> usize { match self { Self::Mittel => 1024, Self::Hoch => 2048, Self::Ultra => 4096 } }
    /// Ray-marching steps of the volumetric fog, `None` = cheap beam cones instead.
    pub fn volumetric_steps(self) -> Option<u32> { match self { Self::Mittel => None, Self::Hoch => Some(48), Self::Ultra => Some(96) } }
    /// Ray-march steps for screen-space reflections, `None` = off.
    pub fn ssr_steps(self) -> Option<u32> { match self { Self::Mittel => None, Self::Hoch => Some(16), Self::Ultra => Some(32) } }
    pub fn ssao(self) -> ScreenSpaceAmbientOcclusionQualityLevel {
        match self { Self::Mittel => ScreenSpaceAmbientOcclusionQualityLevel::Medium, Self::Hoch => ScreenSpaceAmbientOcclusionQualityLevel::High,
            Self::Ultra => ScreenSpaceAmbientOcclusionQualityLevel::Ultra }
    }
}

/// Show light that scatters into the haze when volumetrics are on.
#[derive(Component)]
pub struct VolumetricSource;
/// Shadowed house spot that only scatters into the haze on Ultra.
#[derive(Component)]
pub struct UltraShaft;
/// Cheap additive beam cone, shown only without volumetrics.
#[derive(Component)]
pub struct FakeBeam;

/// Light club haze filling the hall; only visible to a camera with `VolumetricFog`.
pub fn haze(size: Vec3, centre: Vec3) -> impl Bundle {
    (
        FogVolume {
            density_factor: 0.045,
            absorption: 0.02,
            scattering: 0.6,
            scattering_asymmetry: 0.6,
            light_intensity: 3.0,
            fog_color: Color::srgb(0.92, 0.92, 1.0),
            ..default()
        },
        Transform::from_translation(centre).with_scale(size),
    )
}

pub fn cycle(keys: Res<ButtonInput<KeyCode>>, mut quality: ResMut<Quality>) {
    if keys.just_pressed(KeyCode::F2) { *quality = quality.next(); }
}

/// Applies the preset whenever it changes (and once at startup).
pub fn apply(
    quality: Res<Quality>,
    mut commands: Commands,
    mut shadow_map: ResMut<PointLightShadowMap>,
    camera: Single<(Entity, &mut ScreenSpaceAmbientOcclusion), With<Camera3d>>,
    mut sources: Query<(Entity, &mut SpotLight), With<VolumetricSource>>,
    shafts: Query<Entity, With<UltraShaft>>,
    mut beams: Query<&mut Visibility, With<FakeBeam>>,
) {
    if !quality.is_changed() { return; }
    let q = *quality;
    shadow_map.size = q.shadow_map_size();
    let (cam, mut ssao) = camera.into_inner();
    ssao.quality_level = q.ssao();
    let mut cam = commands.entity(cam);
    cam.insert(if q == Quality::Mittel { ShadowFilteringMethod::Gaussian } else { ShadowFilteringMethod::Temporal });
    match q.volumetric_steps() {
        Some(steps) => { cam.insert(VolumetricFog { step_count: steps, jitter: 0.5, ambient_intensity: 0.0, ..default() }); }
        None => { cam.remove::<VolumetricFog>(); }
    }
    // Screen-space reflections: polished terrazzo, marble, brass and glass
    // reflect the real scene (where it is on screen; the cubemap fills the rest).
    match q.ssr_steps() {
        Some(steps) => { cam.insert(ScreenSpaceReflections { linear_steps: steps, bisection_steps: 6, thickness: 0.12,
            min_perceptual_roughness: 0.0..0.0, max_perceptual_roughness: 0.22..0.32, ..default() }); }
        None => { cam.remove::<ScreenSpaceReflections>(); }
    }
    let volumetric = q.volumetric_steps().is_some();
    // Bevy only scatters lights with shadow maps into the fog; the shadows also
    // let stage, truss and gallery cut the beams like in a real haze.
    for (e, mut spot) in &mut sources {
        spot.shadow_maps_enabled = volumetric;
        if volumetric { commands.entity(e).insert(VolumetricLight); } else { commands.entity(e).remove::<VolumetricLight>(); }
    }
    for e in &shafts {
        if q == Quality::Ultra { commands.entity(e).insert(VolumetricLight); } else { commands.entity(e).remove::<VolumetricLight>(); }
    }
    for mut v in &mut beams { *v = if volumetric { Visibility::Hidden } else { Visibility::Inherited }; }
    info!("Grafikqualitaet: {}", q.name());
}

#[cfg(test)]
mod tests {
    use super::Quality;
    #[test]
    fn quality_presets_parse_and_cycle() {
        assert_eq!(Quality::parse(""), Quality::Hoch);
        assert_eq!(Quality::parse("Ultra"), Quality::Ultra);
        assert_eq!(Quality::parse("mittel"), Quality::Mittel);
        let mut q = Quality::Mittel;
        for _ in 0..3 { q = q.next(); }
        assert_eq!(q, Quality::Mittel);
        assert!(Quality::Mittel.volumetric_steps().is_none() && Quality::Ultra.volumetric_steps().is_some());
    }
}
