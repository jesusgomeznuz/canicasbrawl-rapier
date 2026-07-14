use bevy::prelude::*;
use rapier_bevy::{GameAppConfig, SimulationMode, random_physics_game_app};

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
            seed,
        },
    );
    on_start(&mut app, seed, roster, palette, video_secs);
    on_update(&mut app);
    on_exit(&mut app);

    app.run();
}

fn on_start(app: &mut App, seed: u64, roster: Vec<MarbleConfig>, palette: ColorPalette, video_secs: f32) {
    game::cast_the_race(app, roster);
    game::world::build_the_world(app, seed, finish_target_secs(video_secs));
    game::background::paint_the_backdrop(app, palette);
    production::setup_production(app);
}

// El loop central: todo lo que se repite, agrupado por semántica. Cada acto
// declara su ritmo adentro con el vocabulario de Bevy (FixedUpdate = la verdad
// de la carrera, Update = lo que se anima, PostUpdate = los que persiguen
// posiciones ya firmes).
fn on_update(app: &mut App) {
    game::track_the_leader(app);
    game::race_events::run_the_event_band(app);
    game::sensors::run_the_sensors(app);
    game::world::generate_the_level(app);
    game::background::animate_the_backdrop(app);
    game::labels::follow_the_marbles(app);
}

fn on_exit(app: &mut App) {
    app.add_systems(Last, production::voice_tracker::save_voice_tracker_on_exit);
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
