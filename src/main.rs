mod content;
mod game;
mod production;

use bevy::prelude::*;
use bevy::transform::TransformSystem;
use rapier_bevy::{GameAppConfig, SimMode, game_app, preprocess_assets};

// Profundidad Z de canicas y plataformas — temporal mientras se calibran las físicas
pub(crate) const UNIT: f32 = 0.35;

enum Command {
    ProcessModules,
    Preprocess,
    Sim(SimMode),
}

fn parse_command() -> Command {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--process-modules") {
        return Command::ProcessModules;
    }
    if args.iter().any(|a| a == "--preprocess") {
        return Command::Preprocess;
    }
    if args.iter().any(|a| a == "--sim-raw") {
        return Command::Sim(SimMode::Raw);
    }
    Command::Sim(SimMode::Precomputed)
}

fn main() {
    match parse_command() {
        Command::ProcessModules => content::process_modules::run(),
        Command::Preprocess => preprocess_assets(),
        Command::Sim(mode) => run_sim(mode),
    }
}

fn run_sim(mode: SimMode) {
    game_app(
        mode,
        GameAppConfig {
            title: "CanicasBrawl",
            resolution: (540.0, 960.0),
        },
    )
    .insert_resource(ClearColor(Color::srgb(0.329, 0.765, 0.980)))
    .insert_resource(production::voice_tracker::VoiceTracker::default())
    .add_systems(
        Startup,
        (
            game::camera::spawn_camera_and_lights,
            game::world::setup,
            game::world::set_gravity,
        ),
    )
    .add_systems(
        Update,
        (
            game::camera::camera_follows_lowest_marble,
            production::voice_tracker::track_race_leader,
        ),
    )
    .add_systems(
        PostUpdate,
        game::camera::update_marble_labels.after(TransformSystem::TransformPropagate),
    )
    .add_systems(Last, production::voice_tracker::save_voice_tracker_on_exit)
    .run();
}
