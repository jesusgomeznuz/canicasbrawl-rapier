use bevy::prelude::*;
use bevy::transform::TransformSystem;

pub mod marble_timers;
pub mod bouncy;
pub mod freeze;
pub mod icons;
pub mod shrink;
pub mod swap;

/// EL DIRECTOR DE LAS TRAMPAS: cada efecto es dueño de su oficio completo —
/// su oído (el contacto que lo dispara), su reloj (lo que lo revierte) y su
/// insignia. Antes estaban agrupados por CUÁNDO corren, así que freeze vivía
/// partido en tres bloques distintos; ahora vive junto.
///
/// Sueltas quedan las dos que no le pertenecen a ningún efecto, y por eso el
/// nombre dice DE QUIÉN es cada una: los íconos son de la trampa (giran sobre
/// ella, esperando), el cronómetro es de la canica (la persigue en pantalla
/// contando lo que le queda). Ninguna pregunta de qué efecto viene.
pub fn update_sensors(app: &mut App) {
    freeze::update_freeze(app);
    shrink::update_shrink(app);
    swap::update_swap(app);
    bouncy::update_bouncy(app);

    // Girar un ícono es animación, no verdad de la carrera: vive en Update.
    app.add_systems(Update, icons::update_sensor_icons);
    app.add_systems(
        PostUpdate,
        marble_timers::update_marble_timers.after(TransformSystem::TransformPropagate),
    );
}
