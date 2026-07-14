use bevy::prelude::*;
use bevy_rapier3d::plugin::PhysicsSet;

pub mod badges;
pub mod bouncy;
pub mod freeze;
pub mod icons;
pub mod shrink;
pub mod swap;

/// El oficio completo del sensor: el oído y los relojes.
/// El oído (on_*_contact) aplica el efecto físico y emite su RaceEvent; declara
/// `EventReader<CollisionEvent>`, así que donde no hay choques se duerme solo.
/// Los relojes descongelan, devuelven el tamaño y apagan anillos y pulsos.
pub fn run_the_sensors(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            freeze::on_freeze_contact,
            shrink::on_shrink_contact,
            swap::on_swap_contact,
            bouncy::trigger_bouncy_pulse,
        )
            .after(PhysicsSet::Writeback)
            .before(crate::game::race_events::emit_race_events_from_timeline),
    );
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
