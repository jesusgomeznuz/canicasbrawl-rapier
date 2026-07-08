use bevy::prelude::*;
use bevy::sprite::ColorMaterial;
use bevy_rapier3d::prelude::*;

use super::badges::{EffectKind, EffectTimerBadge, spawn_badge};
use crate::game::baked_events::BakedEvent;
use crate::game::camera::world_pos_on_screen;
use crate::game::marbles::{Marble, MarbleIndex};

pub fn marble_groups() -> CollisionGroups {
    let (marble, frozen) = (Group::GROUP_1, Group::GROUP_2);
    CollisionGroups::new(marble, Group::all().difference(frozen))
}

pub fn frozen_groups() -> CollisionGroups {
    let (marble, frozen) = (Group::GROUP_1, Group::GROUP_2);
    CollisionGroups::new(frozen, Group::all().difference(marble).difference(frozen))
}

#[derive(Component)]
pub struct FreezeEffect;

#[derive(Component)]
pub struct Frozen {
    pub expires_at: f32,
    pub visual: Entity,
}

#[derive(Component)]
pub struct FreezeTimerMarker;

pub fn on_freeze_contact(
    mut collisions: EventReader<CollisionEvent>,
    freezes: Query<&Transform, (With<FreezeEffect>, Without<Marble>)>,
    marbles: Query<(), (With<Marble>, Without<Frozen>)>,
    camera_q: Query<(&Projection, &GlobalTransform), With<Camera3d>>,
    indices: Query<&MarbleIndex>,
    time: Res<Time>,
    mut events: EventWriter<BakedEvent>,
    mut commands: Commands,
) {
    let duration = 2.0_f32;
    let Ok((projection, camera_transform)) = camera_q.single() else { return };
    for collision in collisions.read() {
        let CollisionEvent::Started(a, b, _) = collision else { continue };
        for (sensor, target) in [(*a, *b), (*b, *a)] {
            let Ok(sensor_transform) = freezes.get(sensor) else { continue };
            if !marbles.contains(target) { continue }
            if !world_pos_on_screen(sensor_transform.translation, projection, camera_transform) { continue }
            let Ok(index) = indices.get(target) else { continue };
            info!("effect: freeze @({:.2},{:.2}) t={:.2}", sensor_transform.translation.x, sensor_transform.translation.y, time.elapsed_secs());
            commands.entity(target).insert((
                RigidBody::KinematicPositionBased,
                frozen_groups(),
            ));
            events.write(BakedEvent::Freeze {
                marble: index.0,
                x: sensor_transform.translation.x,
                y: sensor_transform.translation.y,
                duration,
            });
        }
    }
}

pub fn spawn_frozen_visual(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    marble: Entity,
) -> Entity {
    let icy = StandardMaterial {
        base_color: Color::srgba(0.65, 0.88, 1.0, 0.55),
        emissive: LinearRgba::new(0.2, 0.45, 0.6, 1.0),
        alpha_mode: AlphaMode::Blend,
        ..default()
    };
    let visual = commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.11))),
        MeshMaterial3d(materials.add(icy)),
        Transform::from_xyz(0.0, 0.0, 0.12),
    )).id();
    commands.entity(marble).add_child(visual);
    visual
}

pub fn try_unfreeze(
    time: Res<Time>,
    rapier: ReadRapierContext,
    frozen: Query<(Entity, &Frozen, &Collider, &Transform), With<Marble>>,
    other_marbles: Query<(), (With<Marble>, Without<Frozen>)>,
    mut commands: Commands,
) {
    for (entity, frozen_state, collider, transform) in &frozen {
        if time.elapsed_secs() < frozen_state.expires_at { continue; }
        if let Ok(rapier_context) = rapier.single() {
            let blocked = std::cell::Cell::new(false);
            let filter = QueryFilter::default().exclude_collider(entity);
            rapier_context.intersect_shape(
                transform.translation,
                transform.rotation,
                &*collider.raw,
                filter,
                |hit| {
                    if other_marbles.contains(hit) {
                        blocked.set(true);
                        return false;
                    }
                    true
                },
            );
            if blocked.get() { continue; }
            commands.entity(entity).insert((RigidBody::Dynamic, marble_groups()));
        }
        commands.entity(frozen_state.visual).despawn();
        commands.entity(entity).remove::<Frozen>();
    }
}

pub fn manage_freeze_badges(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    needs_badge: Query<(Entity, &Frozen), (With<Marble>, Without<FreezeTimerMarker>)>,
    lost_freeze: Query<Entity, (With<Marble>, With<FreezeTimerMarker>, Without<Frozen>)>,
    badges: Query<(Entity, &EffectTimerBadge)>,
) {
    for (marble_entity, frozen) in &needs_badge {
        spawn_badge(&mut commands, &time, &mut meshes, &mut color_materials, marble_entity, frozen.expires_at, EffectKind::Freeze);
        commands.entity(marble_entity).insert(FreezeTimerMarker);
    }
    for marble_entity in &lost_freeze {
        for (badge_entity, badge) in &badges {
            if badge.marble == marble_entity && badge.kind == EffectKind::Freeze {
                commands.entity(badge_entity).despawn();
            }
        }
        commands.entity(marble_entity).remove::<FreezeTimerMarker>();
    }
}
