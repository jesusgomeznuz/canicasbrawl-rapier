mod content;
mod game;
mod production;

use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_rapier3d::plugin::PhysicsSet;
use rapier_bevy::{GameAppConfig, SimMode, game_app, preprocess_assets};

// Profundidad Z de canicas y plataformas — temporal mientras se calibran las físicas
pub(crate) const UNIT: f32 = 0.35;

enum Command {
    ProcessModules,
    Preprocess,
    Sim(SimMode, u64),
}

fn parse_command() -> Command {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--process-modules") {
        return Command::ProcessModules;
    }
    if args.iter().any(|a| a == "--preprocess") {
        return Command::Preprocess;
    }
    let mode = if args.iter().any(|a| a == "--sim-raw") { SimMode::Raw } else { SimMode::Precomputed };
    Command::Sim(mode, parse_seed(&args))
}

fn parse_seed(args: &[String]) -> u64 {
    args.iter().position(|a| a == "--seed")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(random_seed)
}

fn random_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64
}

fn main() {
    match parse_command() {
        Command::ProcessModules => content::process_modules::run(),
        Command::Preprocess => preprocess_assets(),
        Command::Sim(mode, seed) => run_sim(mode, seed),
    }
}

fn run_sim(mode: SimMode, seed: u64) {
    println!("Level seed: {}", seed);
    game_app(
        mode,
        GameAppConfig {
            title: "CanicasBrawl",
            resolution: (540.0, 960.0),
        },
    )
    .insert_resource(ClearColor(Color::srgb(0.329, 0.765, 0.980)))
    .insert_resource(production::voice_tracker::VoiceTracker::default())
    .insert_resource(game::finish::RaceResult::default())
    .insert_resource(game::world::LevelSeed(seed))
    .add_systems(
        Startup,
        (
            game::camera::spawn_camera_and_lights,
            game::camera::spawn_leader_crown,
            game::world::setup,
            game::world::set_gravity,
        ),
    )
    .add_systems(Update, (
        production::voice_tracker::track_race_leader,
        game::bouncy::trigger_bouncy_pulse,
        game::bouncy::animate_bounce_pulse,
        game::bouncy::tick_bounce_cooldown,
        game::effects::on_freeze_contact,
        game::effects::try_unfreeze,
        game::effects::on_shrink_contact,
        game::effects::try_unshrink,
        game::effects::on_swap_contact,
        game::effects::fade_swap_rings,
        game::effects::spin_icons,
        game::finish::on_finish_contact,
    ))
    .add_systems(
        PostUpdate,
        game::camera::camera_follows_lowest_marble
            .after(PhysicsSet::Writeback)
            .before(TransformSystem::TransformPropagate),
    )
    .add_systems(
        PostUpdate,
        (
            game::camera::update_marble_labels,
            game::camera::update_leader_crown,
        )
            .chain()
            .after(TransformSystem::TransformPropagate),
    )
    .add_systems(Last, production::voice_tracker::save_voice_tracker_on_exit)
    .run();
}
