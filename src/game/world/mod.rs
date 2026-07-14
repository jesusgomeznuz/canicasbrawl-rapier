use bevy::prelude::*;

pub mod level_generation;
pub mod modules;
pub mod pickups;
pub mod setup;
pub mod structures;

/// El escenario donde se corre: semilla, meta, suelo, gravedad — y la cámara
/// con sus luces, que también es una entidad que se coloca en el mundo.
pub fn build_the_world(app: &mut App, seed: u64, finish_target_secs: f32) {
    app.insert_resource(level_generation::LevelSeed(seed))
        .insert_resource(level_generation::FinishTarget(finish_target_secs))
        .add_systems(
            Startup,
            (
                setup::setup,
                setup::set_gravity,
                super::camera::spawn_camera_and_lights,
            ),
        );
}
