use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use super::marbles::Marble;

pub const MARBLE_GROUP: Group = Group::GROUP_1;
pub const FROZEN_GROUP: Group = Group::GROUP_2;

pub fn marble_groups() -> CollisionGroups {
    CollisionGroups::new(MARBLE_GROUP, Group::all().difference(FROZEN_GROUP))
}

pub fn frozen_groups() -> CollisionGroups {
    CollisionGroups::new(FROZEN_GROUP, Group::all().difference(MARBLE_GROUP).difference(FROZEN_GROUP))
}

#[derive(Component)]
pub struct FreezeEffect;

#[derive(Component)]
pub struct Frozen {
    pub expires_at: f32,
}

#[derive(Component)]
pub struct ShrinkEffect;

#[derive(Component)]
pub struct SwapEffect;

#[derive(Component)]
pub struct Shrunk {
    pub expires_at: f32,
}

#[derive(Component)]
pub struct SpinningIcon {
    pub axis: Vec3,
    pub speed: f32,
}

pub fn spin_icons(time: Res<Time>, mut icons: Query<(&SpinningIcon, &mut Transform)>) {
    for (icon, mut transform) in &mut icons {
        transform.rotate_axis(Dir3::new(icon.axis).unwrap_or(Dir3::Y), icon.speed * time.delta_secs());
    }
}

pub fn on_freeze_contact(
    mut events: EventReader<CollisionEvent>,
    freezes: Query<(), With<FreezeEffect>>,
    marbles: Query<(), (With<Marble>, Without<Frozen>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let duration = 2.0_f32;
    for event in events.read() {
        let CollisionEvent::Started(a, b, _) = event else { continue };
        for (sensor, target) in [(*a, *b), (*b, *a)] {
            if freezes.contains(sensor) && marbles.contains(target) {
                commands.entity(target).insert((
                    Frozen { expires_at: time.elapsed_secs() + duration },
                    RigidBody::KinematicPositionBased,
                    frozen_groups(),
                ));
                commands.entity(sensor).despawn();
            }
        }
    }
}

pub fn on_shrink_contact(
    mut events: EventReader<CollisionEvent>,
    shrinks: Query<(), With<ShrinkEffect>>,
    mut marbles: Query<&mut Transform, (With<Marble>, Without<Shrunk>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let duration = 5.0_f32;
    let factor   = 0.5_f32;
    for event in events.read() {
        let CollisionEvent::Started(a, b, _) = event else { continue };
        for (sensor, target) in [(*a, *b), (*b, *a)] {
            if shrinks.contains(sensor) {
                if let Ok(mut transform) = marbles.get_mut(target) {
                    transform.scale = Vec3::splat(factor);
                    commands.entity(target).insert(Shrunk {
                        expires_at: time.elapsed_secs() + duration,
                    });
                    commands.entity(sensor).despawn();
                }
            }
        }
    }
}

pub fn on_swap_contact(
    mut events: EventReader<CollisionEvent>,
    swaps: Query<(), With<SwapEffect>>,
    mut marbles: Query<(Entity, &mut Transform), With<Marble>>,
    mut commands: Commands,
) {
    for event in events.read() {
        let CollisionEvent::Started(a, b, _) = event else { continue };
        for (sensor, target) in [(*a, *b), (*b, *a)] {
            if !swaps.contains(sensor) { continue }
            let positions: Vec<(Entity, Vec3)> = marbles.iter()
                .map(|(e, t)| (e, t.translation))
                .collect();
            let Some(target_pos) = positions.iter().find(|(e, _)| *e == target).map(|(_, p)| *p) else { continue };
            let behind = positions.iter()
                .filter(|(e, p)| *e != target && p.y < target_pos.y)
                .max_by(|(_, a), (_, b)| a.y.partial_cmp(&b.y).unwrap());
            let Some((other_entity, other_pos)) = behind.copied() else {
                commands.entity(sensor).despawn();
                continue;
            };
            if let Ok((_, mut t)) = marbles.get_mut(target) { t.translation = other_pos; }
            if let Ok((_, mut t)) = marbles.get_mut(other_entity) { t.translation = target_pos; }
            commands.entity(sensor).despawn();
        }
    }
}

pub fn try_unshrink(
    time: Res<Time>,
    mut commands: Commands,
    mut marbles: Query<(Entity, &Shrunk, &mut Transform)>,
) {
    for (entity, s, mut transform) in &mut marbles {
        if time.elapsed_secs() >= s.expires_at {
            transform.scale = Vec3::ONE;
            commands.entity(entity).remove::<Shrunk>();
        }
    }
}

pub fn try_unfreeze(
    time: Res<Time>,
    rapier: ReadRapierContext,
    frozen: Query<(Entity, &Frozen, &Collider, &Transform), With<Marble>>,
    other_marbles: Query<(), (With<Marble>, Without<Frozen>)>,
    mut commands: Commands,
) {
    let Ok(ctx) = rapier.single() else { return };
    for (entity, f, collider, transform) in &frozen {
        if time.elapsed_secs() < f.expires_at { continue; }
        let blocked = std::cell::Cell::new(false);
        let filter = QueryFilter::default().exclude_collider(entity);
        ctx.intersect_shape(
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
        if !blocked.get() {
            commands.entity(entity).insert((
                RigidBody::Dynamic,
                marble_groups(),
            )).remove::<Frozen>();
        }
    }
}
