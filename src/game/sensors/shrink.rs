use bevy::prelude::*;
use bevy::sprite::ColorMaterial;
use bevy_rapier3d::prelude::CollisionEvent;
use rapier_bevy::BakeEvents;

use super::badges::{EffectKind, EffectTimerBadge, spawn_badge};
use crate::game::baked_events::BakedEvent;
use crate::game::camera::world_pos_on_screen;
use crate::game::marbles::{Marble, MarbleIndex};

#[derive(Component)]
pub struct ShrinkEffect;

#[derive(Component)]
pub struct Shrunk {
    pub expires_at: f32,
}

#[derive(Component)]
pub struct ShrinkTimerMarker;

pub fn on_shrink_contact(
    mut events: EventReader<CollisionEvent>,
    shrinks: Query<&Transform, (With<ShrinkEffect>, Without<Marble>)>,
    mut marbles: Query<&mut Transform, (With<Marble>, Without<Shrunk>, Without<ShrinkEffect>)>,
    camera_q: Query<(&Projection, &GlobalTransform), With<Camera3d>>,
    indices: Query<&MarbleIndex>,
    mut bake_events: Option<ResMut<BakeEvents>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let duration = 5.0_f32;
    let factor   = 0.5_f32;
    let Ok((projection, camera_transform)) = camera_q.single() else { return };
    for event in events.read() {
        let CollisionEvent::Started(a, b, _) = event else { continue };
        for (sensor, target) in [(*a, *b), (*b, *a)] {
            let Ok(sensor_transform) = shrinks.get(sensor) else { continue };
            if !world_pos_on_screen(sensor_transform.translation, projection, camera_transform) { continue }
            if let Ok(mut transform) = marbles.get_mut(target) {
                info!("effect: shrink @({:.2},{:.2}) t={:.2}", sensor_transform.translation.x, sensor_transform.translation.y, time.elapsed_secs());
                if let (Some(events), Ok(index)) = (bake_events.as_deref_mut(), indices.get(target)) {
                    events.0.push(BakedEvent::Shrink {
                        marble: index.0,
                        x: sensor_transform.translation.x,
                        y: sensor_transform.translation.y,
                        duration,
                    }.payload());
                }
                transform.scale = Vec3::splat(factor);
                commands.entity(target).insert(Shrunk {
                    expires_at: time.elapsed_secs() + duration,
                });
                commands.entity(sensor).despawn();
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

pub fn manage_shrink_badges(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    needs_badge: Query<(Entity, &Shrunk), (With<Marble>, Without<ShrinkTimerMarker>)>,
    lost_shrink: Query<Entity, (With<Marble>, With<ShrinkTimerMarker>, Without<Shrunk>)>,
    badges: Query<(Entity, &EffectTimerBadge)>,
) {
    for (marble_entity, shrunk) in &needs_badge {
        spawn_badge(&mut commands, &time, &mut meshes, &mut color_materials, marble_entity, shrunk.expires_at, EffectKind::Shrink);
        commands.entity(marble_entity).insert(ShrinkTimerMarker);
    }
    for marble_entity in &lost_shrink {
        for (badge_entity, badge) in &badges {
            if badge.marble == marble_entity && badge.kind == EffectKind::Shrink {
                commands.entity(badge_entity).despawn();
            }
        }
        commands.entity(marble_entity).remove::<ShrinkTimerMarker>();
    }
}
