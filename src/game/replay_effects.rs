use bevy::prelude::*;
use rapier_bevy::{ReplayEvent, SimulationMode};

use super::background::palette::ColorPalette;
use super::baked_events::BakedEvent;
use super::sensors::bouncy::{BounceCooldown, BouncePulse, BouncyOnContact};
use super::sensors::freeze::{FreezeEffect, Frozen, spawn_frozen_visual};
use super::sensors::shrink::{ShrinkEffect, Shrunk};
use super::sensors::swap::{SwapEffect, spawn_swap_rings};
use super::marbles::{Marble, MarbleIndex};
use super::world::level_generation::{close_level_with_finish, spawn_level_module};

pub fn apply_replay_effects(
    mut events: EventReader<ReplayEvent>,
    marbles: Query<(Entity, &MarbleIndex), With<Marble>>,
    freeze_sensors: Query<(Entity, &Transform), With<FreezeEffect>>,
    shrink_sensors: Query<(Entity, &Transform), With<ShrinkEffect>>,
    swap_sensors: Query<(Entity, &Transform), With<SwapEffect>>,
    bouncys: Query<(Entity, &Transform), With<BouncyOnContact>>,
    pulsing: Query<(), Or<(With<BouncePulse>, With<BounceCooldown>)>>,
    mode: Res<SimulationMode>,
    palette: Res<ColorPalette>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for ReplayEvent(payload) in events.read() {
        match BakedEvent::parse(payload) {
            BakedEvent::Freeze { marble, x, y, duration } => {
                let Some(marble) = marble_by_index(&marbles, marble) else { continue };
                let visual = spawn_frozen_visual(&mut commands, &mut meshes, &mut materials, marble);
                commands.entity(marble).insert(Frozen {
                    expires_at: time.elapsed_secs() + duration,
                    visual,
                });
                despawn_sensor_near(&mut commands, freeze_sensors.iter(), x, y);
            }
            BakedEvent::Shrink { marble, x, y, duration } => {
                let Some(marble) = marble_by_index(&marbles, marble) else { continue };
                commands.entity(marble).insert(Shrunk {
                    expires_at: time.elapsed_secs() + duration,
                });
                despawn_sensor_near(&mut commands, shrink_sensors.iter(), x, y);
            }
            BakedEvent::Swap { marble_a, marble_b, x, y } => {
                let (Some(a), Some(b)) = (
                    marble_by_index(&marbles, marble_a),
                    marble_by_index(&marbles, marble_b),
                ) else { continue };
                spawn_swap_rings(&mut commands, &mut meshes, &mut materials, &time, a, b);
                despawn_sensor_near(&mut commands, swap_sensors.iter(), x, y);
            }
            BakedEvent::Module { name, top, seed } => {
                spawn_level_module(
                    &name, top, seed, palette.obstacle_color(),
                    &mut commands, &mode, &asset_server, &mut meshes, &mut materials,
                );
            }
            BakedEvent::Finish { top } => {
                close_level_with_finish(
                    top, palette.obstacle_color(),
                    &mut commands, &mode, &asset_server, &mut meshes, &mut materials,
                );
            }
            BakedEvent::Bouncy { x, y, amplitude } => {
                let target = Vec2::new(x, y);
                let hit = bouncys.iter()
                    .filter(|(entity, _)| !pulsing.contains(*entity))
                    .map(|(entity, transform)| (entity, transform.translation.truncate().distance_squared(target)))
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                if let Some((sphere, distance_squared)) = hit {
                    if distance_squared < 0.25 * 0.25 {
                        commands.entity(sphere).insert(BouncePulse { elapsed: 0.0, amplitude });
                    }
                }
            }
        }
    }
}

pub fn expire_replay_freezes(
    time: Res<Time>,
    frozen: Query<(Entity, &Frozen), With<Marble>>,
    mut commands: Commands,
) {
    for (marble, frozen_state) in &frozen {
        if time.elapsed_secs() < frozen_state.expires_at {
            continue;
        }
        commands.entity(frozen_state.visual).despawn();
        commands.entity(marble).remove::<Frozen>();
    }
}

fn marble_by_index(
    marbles: &Query<(Entity, &MarbleIndex), With<Marble>>,
    want: usize,
) -> Option<Entity> {
    marbles.iter().find(|(_, index)| index.0 == want).map(|(entity, _)| entity)
}

fn despawn_sensor_near<'a>(
    commands: &mut Commands,
    sensors: impl Iterator<Item = (Entity, &'a Transform)>,
    x: f32,
    y: f32,
) {
    let target = Vec2::new(x, y);
    let nearest = sensors
        .map(|(entity, transform)| (entity, transform.translation.truncate().distance_squared(target)))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    if let Some((sensor, distance_squared)) = nearest {
        if distance_squared < 0.25 * 0.25 {
            commands.entity(sensor).despawn();
        }
    }
}
