//! Optional screenshot tour for visual checks from the running game.
//!
//! Start with `MOONCHILD_SHOTS=<directory> cargo run --release`;
//! `MOONCHILD_SHOT_FILTER=<text>[,<text>...]` limits the tour to matching viewpoints,
//! `MOONCHILD_SHOT_FAST=1` shortens the waits for slow software renderers.
//! The camera visits fixed viewpoints, waits until the scene has settled,
//! measures the average frame time over a short window (V-Sync is disabled
//! in this mode), saves an in-engine screenshot and finally quits.
use std::{fs, path::PathBuf, time::Instant};
use bevy::{prelude::*, render::view::screenshot::{save_to_disk, Screenshot}};

pub struct Shot { pub name: &'static str, pub eye: Vec3, pub target: Vec3, pub mode: usize }

pub fn viewpoints() -> Vec<Shot> {
    vec![
        Shot { name: "01_entrance", eye: Vec3::new(0.0, 1.78, 6.6), target: Vec3::new(0.0, 2.3, -7.0), mode: 0 },
        Shot { name: "02_gallery_overview", eye: Vec3::new(-6.0, 4.78, 8.6), target: Vec3::new(2.0, 1.0, -3.0), mode: 0 },
        Shot { name: "03_bar", eye: Vec3::new(3.4, 1.7, 4.6), target: Vec3::new(8.8, 1.1, 2.2), mode: 0 },
        Shot { name: "04_lounge_upper", eye: Vec3::new(0.3, 4.75, 6.3), target: Vec3::new(-4.8, 3.3, 8.4), mode: 0 },
        Shot { name: "05_ceiling_ring", eye: Vec3::new(-2.5, 1.6, 4.6), target: Vec3::new(0.0, 6.0, 0.2), mode: 0 },
        Shot { name: "06_dj_booth", eye: Vec3::new(1.6, 2.4, -3.0), target: Vec3::new(0.0, 1.9, -8.0), mode: 0 },
        Shot { name: "07_floor_detail", eye: Vec3::new(-2.0, 1.2, 2.5), target: Vec3::new(0.5, 0.0, -0.5), mode: 0 },
        Shot { name: "08_stairs", eye: Vec3::new(-5.0, 1.75, 0.0), target: Vec3::new(-9.0, 1.6, 4.8), mode: 0 },
        Shot { name: "09_lounge_lower", eye: Vec3::new(1.6, 1.6, 6.0), target: Vec3::new(6.0, 0.6, 8.3), mode: 0 },
        Shot { name: "44_dj_operator", eye: Vec3::new(0.0, 2.94, -8.68), target: Vec3::new(0.0, 2.15, -7.80), mode: 7 },
        Shot { name: "45_dj_detail", eye: Vec3::new(-0.52, 2.57, -8.30), target: Vec3::new(-0.22, 2.17, -7.80), mode: 7 },
        Shot { name: "11_dj_gear_close", eye: Vec3::new(0.9, 2.75, -6.6), target: Vec3::new(0.0, 2.1, -7.85), mode: 0 },
        Shot { name: "12_stage_steps", eye: Vec3::new(10.5, 1.8, -3.5), target: Vec3::new(7.2, 0.6, -7.0), mode: 0 },
        Shot { name: "10_from_stage", eye: Vec3::new(-3.0, 2.74, -6.0), target: Vec3::new(1.0, 2.0, 6.0), mode: 0 },
        // Same view with the show lights off: only the warm house lighting remains.
        Shot { name: "13_house_light_only", eye: Vec3::new(-3.0, 2.74, -6.0), target: Vec3::new(1.0, 2.0, 6.0), mode: 0 },
        // New hall areas and light programs (3 Chill, 4 Fan, 5 Cone, 6 Chase).
        Shot { name: "14_stage_backwall", eye: Vec3::new(0.0, 1.9, -2.5), target: Vec3::new(0.0, 4.2, -9.6), mode: 0 },
        Shot { name: "15_side_wall", eye: Vec3::new(8.0, 1.8, -4.0), target: Vec3::new(-11.8, 3.0, 2.0), mode: 2 },
        Shot { name: "16_entrance_wall", eye: Vec3::new(0.0, 1.8, -2.0), target: Vec3::new(0.0, 4.5, 9.8), mode: 0 },
        Shot { name: "17_program_fan", eye: Vec3::new(-6.0, 1.8, 6.0), target: Vec3::new(1.0, 2.4, -4.0), mode: 4 },
        Shot { name: "18_program_cone", eye: Vec3::new(-6.0, 1.8, 6.0), target: Vec3::new(1.0, 2.4, -4.0), mode: 5 },
        Shot { name: "19_program_chase", eye: Vec3::new(-6.0, 1.8, 6.0), target: Vec3::new(1.0, 2.4, -4.0), mode: 6 },
        Shot { name: "30_lounge_stage", eye: Vec3::new(0.0, 2.2, 3.8), target: Vec3::new(0.0, 2.7, -8.0), mode: 7 },
        Shot { name: "40_overhaul_upper", eye: Vec3::new(-6.6, 4.4, 6.0), target: Vec3::new(-4.8, 3.6, 8.15), mode: 7 },
        Shot { name: "31_lounge_table", eye: Vec3::new(4.2, 1.55, 5.8), target: Vec3::new(6.0, 0.6, 8.15), mode: 7 },
    ]
}

#[derive(Resource)]
pub struct Tour { dir: PathBuf, shots: Vec<Shot>, index: usize, frame: u32, timer: Option<Instant>, report: String }

const SETTLE_FIRST: u32 = 360;
const SETTLE: u32 = 45;
const MEASURE: u32 = 120;
/// `MOONCHILD_SHOT_FAST=1` shortens waits for slow software renderers; frame times are then meaningless.
fn fast() -> bool { std::env::var_os("MOONCHILD_SHOT_FAST").is_some() }
fn waits() -> (u32, u32, u32) { if fast() { (110, 12, 2) } else { (SETTLE_FIRST, SETTLE, MEASURE) } }

impl Tour {
    pub fn from_env() -> Option<Self> {
        let dir = PathBuf::from(std::env::var_os("MOONCHILD_SHOTS")?);
        fs::create_dir_all(&dir).ok()?;
        let filter = std::env::var("MOONCHILD_SHOT_FILTER").unwrap_or_default();
        // Comma-separated: a shot is taken if its name contains any of the parts.
        let parts: Vec<&str> = filter.split(',').map(str::trim).collect();
        let shots = viewpoints().into_iter().filter(|s| parts.iter().any(|p| s.name.contains(p))).collect();
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
    show.mode = tour.shots[tour.index].mode;
    tour.frame += 1;
    let (first, other, measure) = waits();
    let settle = if tour.index == 0 { first } else { other };
    if tour.frame == settle { tour.timer = Some(Instant::now()); }
    if tour.frame == settle + measure {
        let ms = tour.timer.map(|t| t.elapsed().as_secs_f64() * 1000.0 / measure as f64).unwrap_or(0.0);
        let line = format!("{:<22} {:>7.2} ms/frame  ~{:>6.1} fps\n", name, ms, 1000.0 / ms.max(0.001));
        let path = tour.dir.join(format!("{name}.png"));
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
        tour.report.push_str(&line);
        tour.index += 1;
        tour.frame = 0;
    }
}
