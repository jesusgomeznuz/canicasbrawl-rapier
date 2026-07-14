use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_rapier3d::plugin::PhysicsSet;

pub mod badges;
pub mod bouncy;
pub mod freeze;
pub mod icons;
pub mod shrink;
pub mod swap;

/// El oficio completo del sensor: el oído, los relojes y las insignias.
/// El oído (on_*_contact) aplica el efecto físico y emite su RaceEvent; declara
/// `EventReader<CollisionEvent>`, así que donde no hay choques se duerme solo.
/// Los relojes descongelan, devuelven el tamaño y apagan anillos y pulsos.
/// Las insignias nacen y mueren con el efecto (Update) y persiguen a su canica
/// cuando las posiciones del frame ya quedaron firmes (PostUpdate).
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
            .before(rapier_bevy::EventBand),
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
    app.add_systems(
        Update,
        (freeze::manage_freeze_badges, shrink::manage_shrink_badges),
    );
    app.add_systems(
        PostUpdate,
        badges::update_badges.after(TransformSystem::TransformPropagate),
    );
}
