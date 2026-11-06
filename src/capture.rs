//! Optional screenshot tour for visual checks from the running game.
//!
//! Start with `MOONCHILD_SHOTS=<directory> cargo run --release`;
//! `MOONCHILD_SHOT_FILTER=<text>` limits the tour to matching viewpoints.
//! The camera visits fixed viewpoints, waits until the scene has settled,
//! measures the average frame time over a short window (V-Sync is disabled
//! in this mode), saves an in-engine screenshot and finally quits.
use std::{fs, path::PathBuf, time::Instant};
use bevy::{prelude::*, render::view::screenshot::{save_to_disk, Screenshot}};

pub struct Shot { pub name: &'static str, pub eye: Vec3, pub target: Vec3 }

pub fn viewpoints() -> Vec<Shot> {
    vec![
        Shot { name: "01_entrance", eye: Vec3::new(0.0, 1.78, 6.6), target: Vec3::new(0.0, 2.3, -7.0) },
        Shot { name: "02_gallery_overview", eye: Vec3::new(-6.0, 4.78, 8.6), target: Vec3::new(2.0, 1.0, -3.0) },
        Shot { name: "03_bar", eye: Vec3::new(3.4, 1.7, 4.6), target: Vec3::new(8.8, 1.1, 2.2) },
        Shot { name: "04_lounge_upper", eye: Vec3::new(0.3, 4.75, 6.3), target: Vec3::new(-4.8, 3.3, 8.4) },
        Shot { name: "05_ceiling_ring", eye: Vec3::new(-2.5, 1.6, 4.6), target: Vec3::new(0.0, 6.0, 0.2) },
        Shot { name: "06_dj_booth", eye: Vec3::new(1.6, 2.4, -3.0), target: Vec3::new(0.0, 1.9, -8.0) },
        Shot { name: "07_floor_detail", eye: Vec3::new(-2.0, 1.2, 2.5), target: Vec3::new(0.5, 0.0, -0.5) },
        Shot { name: "08_stairs", eye: Vec3::new(-5.0, 1.75, 0.0), target: Vec3::new(-9.0, 1.6, 4.8) },
        Shot { name: "09_lounge_lower", eye: Vec3::new(1.6, 1.6, 6.0), target: Vec3::new(6.0, 0.6, 8.3) },
        Shot { name: "11_dj_gear_close", eye: Vec3::new(0.9, 2.75, -6.6), target: Vec3::new(0.0, 2.1, -7.85) },
        Shot { name: "12_stage_steps", eye: Vec3::new(10.5, 1.8, -3.5), target: Vec3::new(7.2, 0.6, -7.0) },
        Shot { name: "10_from_stage", eye: Vec3::new(-3.0, 2.74, -6.0), target: Vec3::new(1.0, 2.0, 6.0) },
        // Same view with the show lights off: only the warm house lighting remains.
        Shot { name: "13_house_light_only", eye: Vec3::new(-3.0, 2.74, -6.0), target: Vec3::new(1.0, 2.0, 6.0) },
    ]
}

#[derive(Resource)]
pub struct Tour { dir: PathBuf, shots: Vec<Shot>, index: usize, frame: u32, timer: Option<Instant>, report: String }

const SETTLE_FIRST: u32 = 360;
const SETTLE: u32 = 45;
const MEASURE: u32 = 120;

impl Tour {
    pub fn from_env() -> Option<Self> {
        let dir = PathBuf::from(std::env::var_os("MOONCHILD_SHOTS")?);
        fs::create_dir_all(&dir).ok()?;
        let filter = std::env::var("MOONCHILD_SHOT_FILTER").unwrap_or_default();
        let shots = viewpoints().into_iter().filter(|s| s.name.contains(filter.as_str())).collect();
        Some(Self { dir, shots, index: 0, frame: 0, timer: None, report: String::new() })
    }
}

pub fn run_tour(mut commands: Commands, tour: Option<ResMut<Tour>>, mut show: ResMut<crate::Show>,
    mut camera: Query<&mut Transform, With<Camera3d>>, mut exit: MessageWriter<AppExit>) {
    let Some(mut tour) = tour else { return };
    let Ok(mut transform) = camera.single_mut() else { return };
    if tour.index >= tour.shots.len() {
        tour.frame += 1;
        if tour.frame == 30 {
            let path = tour.dir.join("frame_times.txt");
            let _ = fs::write(&path, &tour.report);
            info!("Screenshot tour finished:\n{}", tour.report);
            exit.write(AppExit::Success);
        }
        return;
    }
    let (name, eye, target) = { let s = &tour.shots[tour.index]; (s.name, s.eye, s.target) };
    *transform = Transform::from_translation(eye).looking_at(target, Vec3::Y);
    show.enabled = !name.ends_with("house_light_only");
    tour.frame += 1;
    let settle = if tour.index == 0 { SETTLE_FIRST } else { SETTLE };
    if tour.frame == settle { tour.timer = Some(Instant::now()); }
    if tour.frame == settle + MEASURE {
        let ms = tour.timer.map(|t| t.elapsed().as_secs_f64() * 1000.0 / MEASURE as f64).unwrap_or(0.0);
        let line = format!("{:<22} {:>7.2} ms/frame  ~{:>6.1} fps\n", name, ms, 1000.0 / ms.max(0.001));
        let path = tour.dir.join(format!("{name}.png"));
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
        tour.report.push_str(&line);
        tour.index += 1;
        tour.frame = 0;
    }
}
