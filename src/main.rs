mod camera;
mod level;
mod marbles;
mod process_modules;
mod world;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_rapier3d::prelude::*;
use rapier_bevy::modes::{Mode, parse_mode, record_duration};
use rapier_bevy::{GraphicsPlugin, PhysicsStatsPlugin, RecordPlugin, SimMode};

// Profundidad Z de canicas y plataformas — temporal mientras se calibran las físicas
pub(crate) const UNIT: f32 = 0.35;

fn main() {
    if std::env::args().any(|a| a == "--process-modules") {
        process_modules::run();
        return;
    }
    match parse_mode() {
        Mode::Preprocess => {}
        Mode::Sim(mode) => run_world_mode(mode),
    }
}

fn run_world_mode(mode: SimMode) {
    let record = record_duration();
    let mut app = App::new();

    if record.is_none() {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (540.0, 960.0).into(),
                title: "CanicasBrawl".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(PhysicsStatsPlugin);
    }

    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(GraphicsPlugin)
        .add_systems(Startup, (world::setup, world::set_gravity))
        .add_systems(Update, camera::camera_follows_lowest_marble)
        .add_systems(
            PostUpdate,
            camera::update_marble_labels.after(TransformSystem::TransformPropagate),
        )
        .insert_resource(ClearColor(Color::srgb(0.329, 0.765, 0.980)))
        .insert_resource(mode);

    if let Some(secs) = record {
        app.add_plugins(RecordPlugin { duration_secs: secs });
    }

    app.run();
}
