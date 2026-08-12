use bevy::prelude::*;
use bevy::transform::TransformSystem;

pub mod badges;
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
/// Sueltas quedan las dos que no le pertenecen a ningún efecto: la insignia se
/// mueve igual sea de freeze o de shrink, y spin_icons ni siquiera sabe que
/// existen los sensores — gira cualquier cosa que traiga un SpinningIcon.
pub fn update_sensors(app: &mut App) {
    freeze::update_freeze(app);
    shrink::update_shrink(app);
    swap::update_swap(app);
    bouncy::update_bouncy(app);

    // Girar un ícono es animación, no verdad de la carrera: vive en Update.
    app.add_systems(Update, icons::spin_icons);
    app.add_systems(
        PostUpdate,
        badges::update_badges.after(TransformSystem::TransformPropagate),
    );
}
