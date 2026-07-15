use bevy::prelude::*;
use bevy_rapier3d::plugin::PhysicsSet;

pub mod level_generation;
pub mod marbles;
pub mod modules;
pub mod pickups;
pub mod sensors;
pub mod staging;
pub mod structures;

pub use sensors::update_sensors;

/// El EDIFICIO del nacimiento: los pisos de la obra, todos a la vista — muros,
/// gravedad, el primer módulo (el director tira los dados por primera vez),
/// las canicas (el mundo le pone cuerpo al elenco), y la cámara y la corona,
/// que también son entidades que se colocan en el mundo (actualizarlas ya es
/// oficio de follow_the_leader). Cero pizarras: construir significa construir.
pub fn build_world(app: &mut App) {
    app.add_systems(
        Startup,
        (
            structures::spawn_walls,
            structures::set_gravity,
            level_generation::spawn_first_module,
            marbles::spawn_marbles,
            crate::game::scene::camera::spawn_camera_and_lights,
            crate::game::race::leader::spawn_crown,
        ),
    );
}

/// El mundo se mantiene: el director tira los dados para el siguiente módulo
/// y se apagan los colisionadores que ya salieron de pantalla. Declara
/// `ResMut<Dice>`: donde no hay dados en la mesa (la suerte ya está escrita
/// en la partitura) se duerme solo, y los módulos llegan como eventos.
pub fn update_world(app: &mut App) {
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
