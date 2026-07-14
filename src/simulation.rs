use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_rapier3d::plugin::PhysicsSet;
use rapier_bevy::{GameAppConfig, RealCollisions, SimulationMode, random_physics_game_app};

use crate::args::RosterSpec;
use crate::game;
use crate::game::background::palette::ColorPalette;
use crate::game::roster::MarbleConfig;
use crate::production;

pub fn run(mode: SimulationMode, seed: u64, spec: RosterSpec, palette: ColorPalette, video_secs: f32) {
    let roster = resolve_roster(spec);
    println!("Level seed: {}", seed);

    let mut app = random_physics_game_app(
        mode,
        GameAppConfig {
            title: "CanicasBrawl",
            resolution: (540.0, 960.0),
        },
    );
    on_start(&mut app, seed, roster, palette, video_secs);
    on_step(&mut app);
    on_frame_update(&mut app);
    after_frame_update(&mut app);
    on_exit(&mut app);

    app.run();
}

fn on_start(app: &mut App, seed: u64, roster: Vec<MarbleConfig>, palette: ColorPalette, video_secs: f32) {
    game::cast_the_race(app, roster);
    game::world::build_the_world(app, seed, finish_target_secs(video_secs));
    game::background::paint_the_backdrop(app, palette);
    app.add_systems(Startup, game::leader::spawn_crown);
    production::setup_production(app);
}

fn on_step(app: &mut App) {
    game::track_the_leader(app);
    game::race_events::run_the_event_band(app);
    game::sensors::tick_the_sensors(app);
    react_to_real_collisions(app);
}

fn on_frame_update(app: &mut App) {
    game::background::animate_the_backdrop(app);
    game::sensors::show_the_badges(app);
    app.add_systems(Update, production::stall_detector::detect_stall);
}

fn after_frame_update(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (
            game::labels::update_marble_labels,
            game::leader::crown_follows_leader,
            game::sensors::badges::update_badges,
        )
            .after(TransformSystem::TransformPropagate),
    );
}

fn on_exit(app: &mut App) {
    app.add_systems(Last, production::voice_tracker::save_voice_tracker_on_exit);
}

// Etiquetados RealCollisions: el juego declara que nacen de contactos físicos
// reales; el ENGINE los apaga en --play. Aquí nadie pregunta por modos.
fn react_to_real_collisions(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            game::world::level_generation::generate_level,
            game::world::level_generation::disable_modules_above_screen,
            game::sensors::freeze::on_freeze_contact,
            game::sensors::shrink::on_shrink_contact,
            game::sensors::swap::on_swap_contact,
            game::sensors::bouncy::trigger_bouncy_pulse,
        )
            .in_set(RealCollisions)
            .after(PhysicsSet::Writeback)
            .before(game::race_events::emit_race_events_from_timeline),
    );
}

fn resolve_roster(spec: RosterSpec) -> Vec<MarbleConfig> {
    match spec {
        RosterSpec::Default => game::roster::build_roster(None),
        RosterSpec::Characters(names) => game::roster::build_roster(Some(names)),
        RosterSpec::Slots(n) => game::roster::slots_roster(n),
    }
    .unwrap_or_else(|err| {
        eprintln!("Error de roster: {err}");
        std::process::exit(1);
    })
}

fn finish_target_secs(video_secs: f32) -> f32 {
    let tail_of_falling_marbles_secs = 12.0;
    video_secs - tail_of_falling_marbles_secs
}
