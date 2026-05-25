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
    freezes: Query<&Transform, (With<FreezeEffect>, Without<Marble>)>,
    marbles: Query<(), (With<Marble>, Without<Frozen>)>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let duration = 2.0_f32;
    let Ok((camera, cam_xform)) = camera_q.single() else { return };
    for event in events.read() {
        let CollisionEvent::Started(a, b, _) = event else { continue };
        for (sensor, target) in [(*a, *b), (*b, *a)] {
            let Ok(sensor_xform) = freezes.get(sensor) else { continue };
            if !marbles.contains(target) { continue }
            if !super::camera::world_pos_on_screen(sensor_xform.translation, camera, cam_xform) { continue }
            commands.entity(target).insert((
                Frozen { expires_at: time.elapsed_secs() + duration },
                RigidBody::KinematicPositionBased,
                frozen_groups(),
            ));
            commands.entity(sensor).despawn();
        }
    }
}

pub fn on_shrink_contact(
    mut events: EventReader<CollisionEvent>,
    shrinks: Query<&Transform, (With<ShrinkEffect>, Without<Marble>)>,
    mut marbles: Query<&mut Transform, (With<Marble>, Without<Shrunk>, Without<ShrinkEffect>)>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let duration = 5.0_f32;
    let factor   = 0.5_f32;
    let Ok((camera, cam_xform)) = camera_q.single() else { return };
    for event in events.read() {
        let CollisionEvent::Started(a, b, _) = event else { continue };
        for (sensor, target) in [(*a, *b), (*b, *a)] {
            let Ok(sensor_xform) = shrinks.get(sensor) else { continue };
            if !super::camera::world_pos_on_screen(sensor_xform.translation, camera, cam_xform) { continue }
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

pub fn on_swap_contact(
    mut events: EventReader<CollisionEvent>,
    swaps: Query<&Transform, (With<SwapEffect>, Without<Marble>)>,
    mut marbles: Query<(Entity, &mut Transform), (With<Marble>, Without<SwapEffect>)>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Ok((camera, cam_xform)) = camera_q.single() else { return };
    for event in events.read() {
        let CollisionEvent::Started(a, b, _) = event else { continue };
        for (sensor, target) in [(*a, *b), (*b, *a)] {
            let Ok(sensor_xform) = swaps.get(sensor) else { continue };
            if !super::camera::world_pos_on_screen(sensor_xform.translation, camera, cam_xform) { continue }
            let positions: Vec<(Entity, Vec3)> = marbles.iter()
                .map(|(e, t)| (e, t.translation))
                .collect();
            let Some(target_pos) = positions.iter().find(|(e, _)| *e == target).map(|(_, p)| *p) else { continue };
            let Some((partner, partner_pos)) = find_swap_partner(target, target_pos.y, &positions) else {
                commands.entity(sensor).despawn();
                continue;
            };
            if let Ok((_, mut t)) = marbles.get_mut(target) { t.translation = partner_pos; }
            if let Ok((_, mut t)) = marbles.get_mut(partner) { t.translation = target_pos; }
            spawn_swap_rings(&mut commands, &mut meshes, &mut materials, &time, target, partner);
            commands.entity(sensor).despawn();
        }
    }
}

#[derive(Component)]
pub struct SwapRing {
    pub material: Handle<StandardMaterial>,
    pub spawned_at: f32,
    pub lifetime: f32,
}

fn spawn_swap_rings(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    time: &Time,
    marble_a: Entity,
    marble_b: Entity,
) {
    let lifetime = 1.5_f32;
    let mesh = meshes.add(Torus { minor_radius: 0.010, major_radius: 0.10 });
    let joycon_red = Color::srgba(1.0, 0.235, 0.157, 1.0);
    let joycon_blue = Color::srgba(0.039, 0.725, 0.902, 1.0);
    let face_camera = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);

    for (marble, color) in [(marble_a, joycon_red), (marble_b, joycon_blue)] {
        let material = materials.add(StandardMaterial {
            base_color: color,
            emissive: LinearRgba::from(color) * 2.0,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        let ring = commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_rotation(face_camera),
            SwapRing { material, spawned_at: time.elapsed_secs(), lifetime },
        )).id();
        commands.entity(marble).add_child(ring);
    }
}

pub fn fade_swap_rings(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    rings: Query<(Entity, &SwapRing)>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs();
    for (entity, ring) in &rings {
        let progress = (now - ring.spawned_at) / ring.lifetime;
        if progress >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let alpha = 1.0 - progress;
        if let Some(mat) = materials.get_mut(&ring.material) {
            mat.base_color = mat.base_color.with_alpha(alpha);
            mat.emissive = mat.emissive * alpha;
        }
    }
}

fn find_swap_partner(target: Entity, target_y: f32, all: &[(Entity, Vec3)]) -> Option<(Entity, Vec3)> {
    let immediate_ahead = all.iter()
        .filter(|(e, p)| *e != target && p.y < target_y)
        .max_by(|(_, a), (_, b)| a.y.partial_cmp(&b.y).unwrap())
        .copied();
    if immediate_ahead.is_some() { return immediate_ahead; }

    all.iter()
        .filter(|(e, p)| *e != target && p.y > target_y)
        .min_by(|(_, a), (_, b)| a.y.partial_cmp(&b.y).unwrap())
        .copied()
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
