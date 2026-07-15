use bevy::prelude::*;
use bevy_rapier3d::plugin::PhysicsSet;

pub mod level_generation;
pub mod marbles;
pub mod modules;
pub mod pickups;
pub mod sensors;
pub mod setup;
pub mod staging;
pub mod structures;

pub use sensors::run_the_sensors;

/// El escenario donde se corre: pura obra física — arena, gravedad, y la
/// cámara con sus luces y la corona, que también son entidades que se colocan
/// en el mundo (actualizarlas ya es oficio de track_the_leader). Cero
/// pizarras: construir significa construir.
pub fn build_the_world(app: &mut App) {
    app.add_systems(
        Startup,
        (
            setup::setup,
            setup::set_gravity,
            crate::game::scene::camera::spawn_camera_and_lights,
            crate::game::race::leader::spawn_crown,
        ),
    );
}

/// El director trabaja durante la carrera: tira los dados del engine para
/// decidir el siguiente módulo y apaga colisionadores que ya salieron de
/// pantalla. Declara `ResMut<Dice>`: donde no hay dados en la mesa (la suerte
/// ya está escrita en la partitura) se duerme solo, y los módulos llegan como
/// eventos a la escenografía.
pub fn generate_the_level(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            level_generation::generate_level,
            level_generation::disable_modules_above_screen,
        )
            .after(PhysicsSet::Writeback)
            .before(rapier_bevy::EventBand),
    );
}
