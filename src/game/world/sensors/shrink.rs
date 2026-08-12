use bevy::prelude::*;
use bevy::sprite::ColorMaterial;
use bevy_rapier3d::prelude::CollisionEvent;
use bevy_rapier3d::plugin::PhysicsSet;

use super::marble_timers::{EffectKind, MarbleTimer, spawn_marble_timer};
use crate::game::race_events::RaceEvent;
use crate::game::scene::camera::world_pos_on_screen;
use crate::game::world::marbles::{Marble, MarbleIndex};

/// El oficio completo de ENCOGER: el oído, el reloj que devuelve el tamaño, y
/// la insignia del tiempo restante.
pub fn update_shrink(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        on_shrink_contact
            .after(PhysicsSet::Writeback)
            .before(rapier_bevy::EventBand),
    );
    app.add_systems(FixedUpdate, try_unshrink.after(PhysicsSet::Writeback));
    app.add_systems(Update, manage_shrink_timer);
}

#[derive(Component)]
pub struct ShrinkEffect;

#[derive(Component)]
pub struct Shrunk {
    pub expires_at: f32,
}

#[derive(Component)]
pub struct ShrinkTimerMarker;

pub fn on_shrink_contact(
    mut collisions: EventReader<CollisionEvent>,
    shrinks: Query<&Transform, (With<ShrinkEffect>, Without<Marble>)>,
    mut marbles: Query<&mut Transform, (With<Marble>, Without<Shrunk>, Without<ShrinkEffect>)>,
    camera_q: Query<(&Projection, &GlobalTransform), With<Camera3d>>,
    indices: Query<&MarbleIndex>,
    time: Res<Time>,
    mut events: EventWriter<RaceEvent>,
) {
    let duration = 5.0_f32;
    let factor   = 0.5_f32;
    let Ok((projection, camera_transform)) = camera_q.single() else { return };
    for collision in collisions.read() {
        let CollisionEvent::Started(a, b, _) = collision else { continue };
        for (sensor, target) in [(*a, *b), (*b, *a)] {
            let Ok(sensor_transform) = shrinks.get(sensor) else { continue };
            if !world_pos_on_screen(sensor_transform.translation, projection, camera_transform) { continue }
            if let Ok(mut transform) = marbles.get_mut(target) {
                let Ok(index) = indices.get(target) else { continue };
                info!("effect: shrink @({:.2},{:.2}) t={:.2}", sensor_transform.translation.x, sensor_transform.translation.y, time.elapsed_secs());
                transform.scale = Vec3::splat(factor);
                events.write(RaceEvent::Shrink {
                    marble: index.0,
                    x: sensor_transform.translation.x,
                    y: sensor_transform.translation.y,
                    duration,
                });
            }
        }
    }
}

pub fn try_unshrink(
    time: Res<Time>,
    mut commands: Commands,
    mut marbles: Query<(Entity, &Shrunk, &mut Transform)>,
) {
    for (entity, shrunk_state, mut transform) in &mut marbles {
        if time.elapsed_secs() >= shrunk_state.expires_at {
            transform.scale = Vec3::ONE;
            commands.entity(entity).remove::<Shrunk>();
        }
    }
}

pub fn manage_shrink_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    needs_timer: Query<(Entity, &Shrunk), (With<Marble>, Without<ShrinkTimerMarker>)>,
    lost_shrink: Query<Entity, (With<Marble>, With<ShrinkTimerMarker>, Without<Shrunk>)>,
    timers: Query<(Entity, &MarbleTimer)>,
) {
    for (marble_entity, shrunk) in &needs_timer {
        spawn_marble_timer(&mut commands, &time, &mut meshes, &mut color_materials, marble_entity, shrunk.expires_at, EffectKind::Shrink);
        commands.entity(marble_entity).insert(ShrinkTimerMarker);
    }
    for marble_entity in &lost_shrink {
        for (timer_entity, timer) in &timers {
            if timer.marble == marble_entity && timer.kind == EffectKind::Shrink {
                commands.entity(timer_entity).despawn();
            }
        }
        commands.entity(marble_entity).remove::<ShrinkTimerMarker>();
    }
}
