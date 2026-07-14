use bevy::prelude::*;
use bevy_rapier3d::plugin::PhysicsSet;

pub mod background;
pub mod race_events;
pub mod camera;
pub mod faces;
pub mod finish;
pub mod labels;
pub mod leader;
pub mod marbles;
pub mod staging;
pub mod roster;
pub mod sensors;
pub mod world;

/// Quiénes juegan y el estado de la carrera: elenco, líder, meta y resultado.
pub fn cast_the_race(app: &mut App, marbles: Vec<roster::MarbleConfig>) {
    app.insert_resource(finish::RaceResult::default())
        .insert_resource(finish::FinishLineY::default())
        .insert_resource(leader::RaceLeader::default())
        .insert_resource(roster::Roster(marbles));
}

/// La cadena de liderazgo: quién cruzó la meta, quién va ganando, quién canta —
/// y la cámara que lo sigue (aquí ya no se coloca: se actualiza).
pub fn track_the_leader(app: &mut App) {
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
        camera::camera_follows_lowest_marble.after(PhysicsSet::Writeback),
    );
}
