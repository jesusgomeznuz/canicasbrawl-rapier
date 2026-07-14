use bevy::prelude::*;
use bevy_rapier3d::plugin::PhysicsSet;

pub mod badges;
pub mod bouncy;
pub mod freeze;
pub mod icons;
pub mod shrink;
pub mod swap;

/// Los relojes de los efectos: descongelar, crecer de vuelta, apagar anillos y pulsos.
pub fn tick_the_sensors(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            freeze::try_unfreeze,
            shrink::try_unshrink,
            swap::fade_swap_rings,
            icons::spin_icons,
            bouncy::animate_bounce_pulse,
            bouncy::tick_bounce_cooldown,
        )
            .after(PhysicsSet::Writeback),
    );
}

/// Las insignias sobre las canicas bajo efecto.
pub fn show_the_badges(app: &mut App) {
    app.add_systems(
        Update,
        (freeze::manage_freeze_badges, shrink::manage_shrink_badges),
    );
}
