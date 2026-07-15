use bevy::prelude::*;
use rapier_bevy::{GameAppConfig, game_app};

use crate::args::RosterSpec;
use crate::game;
use crate::game::scene::background::palette::ColorPalette;
use crate::game::race::roster::MarbleConfig;
use crate::production;

pub fn run(seed: u64, spec: RosterSpec, palette: ColorPalette, video_secs: f32) {
    let roster = resolve_roster(spec);
    println!("Level seed: {}", seed);

    let mut app = game_app(
        GameAppConfig {
            title: "CanicasBrawl",
            resolution: (540.0, 960.0),
            seed,
        },
    );
    // La banda de eventos se conecta al armar la mesa: estructura del engine
    // (replay → record) + la escenografía del juego como único músico propio.
    rapier_bevy::run_the_event_band::<game::race_events::RaceEvent, _>(
        &mut app,
        game::world::staging::stage_race_events,
    );
    on_start(&mut app, seed, roster, palette, video_secs);
    on_update(&mut app);
    on_exit(&mut app);

    app.run();
}

fn on_start(app: &mut App, seed: u64, roster: Vec<MarbleConfig>, palette: ColorPalette, video_secs: f32) {
    game::race::prepare_the_race(app, roster, finish_target_secs(video_secs));
    game::world::build_the_world(app);
    game::scene::prepare_the_scene(app, palette, seed);
}

// El loop central: todo lo que se repite, agrupado por semántica y ordenado
// como se cuenta la carrera. Cada acto declara su ritmo adentro con el
// vocabulario de Bevy (FixedUpdate = la verdad de la carrera, Update = lo que
// se anima, PostUpdate = los que persiguen posiciones ya firmes).
fn on_update(app: &mut App) {
    // EL MUNDO trabaja
    game::world::spawn_next_module(app);
    game::world::update_sensors(app);
    // LA CARRERA se mide
    game::race::follow_the_leader(app);
    // LA ESCENA respira
    game::scene::update_scene(app);
}

fn on_exit(app: &mut App) {
    app.add_systems(Last, production::voice_tracker::save_voice_tracker_on_exit);
}

fn resolve_roster(spec: RosterSpec) -> Vec<MarbleConfig> {
    match spec {
        RosterSpec::Default => game::race::roster::build_roster(None),
        RosterSpec::Characters(names) => game::race::roster::build_roster(Some(names)),
        RosterSpec::Slots(n) => game::race::roster::slots_roster(n),
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
