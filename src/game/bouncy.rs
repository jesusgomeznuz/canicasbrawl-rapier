use bevy::prelude::*;
use bevy_rapier3d::prelude::{CollisionEvent, Velocity};

#[derive(Component)]
pub struct BouncyOnContact;

#[derive(Component)]
pub struct BouncePulse {
    pub elapsed: f32,
    pub amplitude: f32,
}

#[derive(Component)]
pub struct BounceCooldown {
    pub remaining: f32,
}

pub fn trigger_bouncy_pulse(
    mut events: EventReader<CollisionEvent>,
    bouncy: Query<&Transform, With<BouncyOnContact>>,
    already_pulsing: Query<(), Or<(With<BouncePulse>, With<BounceCooldown>)>>,
    movers: Query<(&Transform, &Velocity)>,
    mut commands: Commands,
) {
    let base_amp     = 0.04_f32;
    let speed_scale  = 0.05_f32;
    let max_amp      = 0.28_f32;

    for event in events.read() {
        if let CollisionEvent::Started(a, b, _) = event {
            for (sphere, other) in [(*a, *b), (*b, *a)] {
                let Ok(sphere_t) = bouncy.get(sphere) else { continue };
                if already_pulsing.contains(sphere) { continue }
                let closing = movers.get(other).ok().map(|(t, v)| {
                    let dir = (t.translation - sphere_t.translation).normalize_or_zero();
                    (-v.linvel.dot(dir)).max(0.0)
                }).unwrap_or(0.0);
                let amp = (base_amp + closing * speed_scale).min(max_amp);
                commands.entity(sphere).insert(BouncePulse { elapsed: 0.0, amplitude: amp });
            }
        }
    }
}

pub fn animate_bounce_pulse(
    time: Res<Time>,
    mut commands: Commands,
    mut pulses: Query<(Entity, &mut BouncePulse, &mut Transform)>,
) {
    let duration = 0.18_f32;
    for (entity, mut pulse, mut transform) in &mut pulses {
        pulse.elapsed += time.delta_secs();
        if pulse.elapsed >= duration {
            transform.scale = Vec3::ONE;
            commands.entity(entity)
                .remove::<BouncePulse>()
                .insert(BounceCooldown { remaining: 0.5 });
        } else {
            let t = pulse.elapsed / duration;
            let s = 1.0 + pulse.amplitude * (t * std::f32::consts::PI).sin();
            transform.scale = Vec3::splat(s);
        }
    }
}

pub fn tick_bounce_cooldown(
    time: Res<Time>,
    mut commands: Commands,
    mut cooldowns: Query<(Entity, &mut BounceCooldown)>,
) {
    for (entity, mut cd) in &mut cooldowns {
        cd.remaining -= time.delta_secs();
        if cd.remaining <= 0.0 {
            commands.entity(entity).remove::<BounceCooldown>();
        }
    }
}
