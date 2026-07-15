use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_rapier3d::plugin::PhysicsSet;

pub mod faces;
pub mod finish;
pub mod labels;
pub mod leader;
pub mod roster;

/// Se pactan las reglas de la partida: el elenco (quiénes juegan), el marcador
/// (líder, meta y resultado, aún vacíos), cuándo termina la carrera — y de
/// paso producción enciende su micrófono (viaja aquí por peso, no por oficio:
/// inicializar un vector no merece acto propio; su trabajo vive en production/).
pub fn prepare_the_race(app: &mut App, marbles: Vec<roster::MarbleConfig>, finish_target_secs: f32) {
    app.insert_resource(crate::production::voice_tracker::VoiceTracker::default())
        .insert_resource(finish::RaceResult::default())
        .insert_resource(finish::FinishLineY::default())
        .insert_resource(finish::FinishTarget(finish_target_secs))
        .insert_resource(leader::RaceLeader::default())
        .insert_resource(roster::Roster(marbles));
}

/// El acto completo del liderazgo: quién cruzó la meta, quién va ganando,
/// quién canta (la verdad, a paso de física), la cámara que lo sigue, y la
/// corona que se pega al líder cuando las posiciones del frame ya quedaron
/// firmes (la corona se COLOCA en build_the_world, como la cámara).
/// Solo se sigue a UNO: al líder — las canicas compiten por serlo.
pub fn follow_the_leader(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            finish::check_finish_crossing,
            leader::update_race_leader,
            crate::production::voice_tracker::track_race_leader,
        )
            .chain()
            .after(PhysicsSet::Writeback),
    );
    app.add_systems(
        FixedUpdate,
        crate::game::scene::camera::camera_follows_lowest_marble.after(PhysicsSet::Writeback),
    );
    app.add_systems(
        PostUpdate,
        leader::crown_follows_leader.after(TransformSystem::TransformPropagate),
    );
}
