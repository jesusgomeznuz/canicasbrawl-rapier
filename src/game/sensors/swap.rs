use bevy::prelude::*;
use bevy_rapier3d::prelude::CollisionEvent;

use crate::game::race_events::RaceEvent;
use crate::game::camera::world_pos_on_screen;
use crate::game::marbles::{Marble, MarbleIndex};

#[derive(Component)]
pub struct SwapEffect;

#[derive(Component)]
pub struct SwapRing {
    pub material: Handle<StandardMaterial>,
    pub spawned_at: f32,
    pub lifetime: f32,
}

pub fn on_swap_contact(
    mut collisions: EventReader<CollisionEvent>,
    swaps: Query<&Transform, (With<SwapEffect>, Without<Marble>)>,
    mut marbles: Query<(Entity, &mut Transform), (With<Marble>, Without<SwapEffect>)>,
    camera_q: Query<(&Projection, &GlobalTransform), With<Camera3d>>,
    indices: Query<&MarbleIndex>,
    time: Res<Time>,
    mut events: EventWriter<RaceEvent>,
    mut commands: Commands,
) {
    let Ok((projection, camera_transform)) = camera_q.single() else { return };
    for collision in collisions.read() {
        let CollisionEvent::Started(a, b, _) = collision else { continue };
        for (sensor, target) in [(*a, *b), (*b, *a)] {
            let Ok(sensor_transform) = swaps.get(sensor) else { continue };
            if !world_pos_on_screen(sensor_transform.translation, projection, camera_transform) { continue }
            let positions: Vec<(Entity, Vec3)> = marbles.iter()
                .map(|(entity, transform)| (entity, transform.translation))
                .collect();
            let Some(target_position) = positions.iter().find(|(entity, _)| *entity == target).map(|(_, position)| *position) else { continue };
            let Some((partner, partner_position)) = find_swap_partner(target, target_position.y, &positions) else {
                commands.entity(sensor).despawn();
                continue;
            };
            let (Ok(index_a), Ok(index_b)) = (indices.get(target), indices.get(partner)) else { continue };
            info!("effect: swap @({:.2},{:.2}) t={:.2}", sensor_transform.translation.x, sensor_transform.translation.y, time.elapsed_secs());
            if let Ok((_, mut transform)) = marbles.get_mut(target) { transform.translation = partner_position; }
            if let Ok((_, mut transform)) = marbles.get_mut(partner) { transform.translation = target_position; }
            events.write(RaceEvent::Swap {
                marble_a: index_a.0,
                marble_b: index_b.0,
                x: sensor_transform.translation.x,
                y: sensor_transform.translation.y,
            });
        }
    }
}

pub fn spawn_swap_rings(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    time: &Time,
    marble_a: Entity,
    marble_b: Entity,
) {
    let lifetime = 1.5_f32;
    let mesh = meshes.add(Torus { minor_radius: 0.010, major_radius: 0.10 });
    let purple = Color::srgba(0.7, 0.3, 0.95, 1.0);
    let face_camera = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);

    for marble in [marble_a, marble_b] {
        let color = purple;
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
        if let Some(material) = materials.get_mut(&ring.material) {
            material.base_color = material.base_color.with_alpha(alpha);
            material.emissive = material.emissive * alpha;
        }
    }
}

fn find_swap_partner(target: Entity, target_y: f32, all: &[(Entity, Vec3)]) -> Option<(Entity, Vec3)> {
    let immediate_ahead = all.iter()
        .filter(|(entity, position)| *entity != target && position.y < target_y)
        .max_by(|(_, a), (_, b)| a.y.partial_cmp(&b.y).unwrap())
        .copied();
    if immediate_ahead.is_some() { return immediate_ahead; }

    all.iter()
        .filter(|(entity, position)| *entity != target && position.y > target_y)
        .min_by(|(_, a), (_, b)| a.y.partial_cmp(&b.y).unwrap())
        .copied()
}
