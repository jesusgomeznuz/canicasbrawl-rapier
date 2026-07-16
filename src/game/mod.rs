//! LA PORTADA DEL JUEGO: su vida en tres fases y sus directores.
//!   race/  LA CARRERA: quiénes corren y quién gana (elenco, juez, reglas)
//!   world/ EL MUNDO: la realidad interactuable (pista, trampas, cuerpos)
//!   scene/ LA ESCENA: lo que se ve sin tocarse (telón, encuadre)
//! race_events.rs es la aduana — vocabulario transversal, de nadie y de todos.

use bevy::prelude::*;
use rapier_bevy::{GameAppConfig, game_app};

use crate::args::RosterSpec;
use crate::game::race::roster::MarbleConfig;
use crate::game::scene::background::palette::ColorPalette;
use crate::production;

pub mod race;
pub mod race_events;
pub mod scene;
pub mod world;

/// La vida del juego. En `--write-timeline` este MISMO flujo corre headless y
/// el engine lo fotografía: este juego se escribe jugándose a sí mismo.
pub fn run(seed: u64, spec: RosterSpec, palette: ColorPalette, video_secs: f32) {
    let roster = resolve_roster(spec);
    println!("Level seed: {}", seed);

    let mut app = game_app(GameAppConfig {
        title: "CanicasBrawl",
        resolution: (540.0, 960.0),
        seed,
    });
    // La banda de eventos se conecta al armar la mesa: estructura del engine
    // (replay → record) + la escenografía del juego como único músico propio.
    rapier_bevy::run_the_event_band::<race_events::RaceEvent, _>(
        &mut app,
        world::staging::stage_race_events,
    );
    on_start(&mut app, seed, roster, palette, video_secs);
    on_update(&mut app);
    on_exit(&mut app);

    app.run();
}

fn on_start(app: &mut App, seed: u64, roster: Vec<MarbleConfig>, palette: ColorPalette, video_secs: f32) {
    world::build_world(app);
    scene::build_scene(app, palette, seed);
    race::build_race(app, roster, finish_target_secs(video_secs));
}

// El loop central: todo lo que se repite, agrupado por semántica y ordenado
// como se cuenta la carrera. Cada acto declara su ritmo adentro con el
// vocabulario de Bevy (FixedUpdate = la verdad de la carrera, Update = lo que
// se anima, PostUpdate = los que persiguen posiciones ya firmes).
fn on_update(app: &mut App) {
    world::update_world(app);
    world::update_sensors(app);
    scene::update_scene(app);
    race::follow_the_leader(app);
}

fn on_exit(app: &mut App) {
    app.add_systems(Last, production::voice_tracker::save_voice_tracker_on_exit);
}

fn resolve_roster(spec: RosterSpec) -> Vec<MarbleConfig> {
    match spec {
        RosterSpec::Default => race::roster::build_roster(None),
        RosterSpec::Characters(names) => race::roster::build_roster(Some(names)),
        RosterSpec::Slots(n) => race::roster::slots_roster(n),
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
