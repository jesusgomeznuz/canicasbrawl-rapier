use bevy::prelude::*;
use rapier_bevy::{ReplayEvent, SimulationMode};

use super::background::ColorPalette;
use super::bouncy::{BounceCooldown, BouncePulse, BouncyOnContact};
use super::effects::{
    FreezeEffect, Frozen, ShrinkEffect, Shrunk, SwapEffect, spawn_frozen_visual, spawn_swap_rings,
};
use super::marbles::{Marble, MarbleIndex};
use super::world::{close_level_with_finish, spawn_level_module};

/// Reproduce en replay todo lo que las poses no capturan. El NIVEL también: en
/// replay `generate_level` no corre — cada módulo llega como evento horneado con
/// su propio seed y se reconstruye idéntico por construcción (los módulos
/// estáticos no tienen RigidBody, así que una divergencia ahí sería invisible
/// para el assert de cuerpos; por eso el replay no re-deriva el mundo, lo lee).
/// La utilería de efectos igual: sensor consumido, hielo, anillos, pulso bouncy.
/// Los badges aparecen solos: sus sistemas reaccionan a Frozen/Shrunk.
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
        let parts: Vec<&str> = payload.split_whitespace().collect();
        match parts.as_slice() {
            ["freeze", idx, x, y, dur] => {
                let (Some(marble), Some(x), Some(y), Some(dur)) =
                    (marble_by_index(&marbles, idx), num(x), num(y), num(dur))
                else { continue };
                let visual = spawn_frozen_visual(&mut commands, &mut meshes, &mut materials, marble);
                commands.entity(marble).insert(Frozen {
                    expires_at: time.elapsed_secs() + dur,
                    visual,
                });
                despawn_sensor_near(&mut commands, freeze_sensors.iter(), x, y);
            }
            ["shrink", idx, x, y, dur] => {
                let (Some(marble), Some(x), Some(y), Some(dur)) =
                    (marble_by_index(&marbles, idx), num(x), num(y), num(dur))
                else { continue };
                commands.entity(marble).insert(Shrunk {
                    expires_at: time.elapsed_secs() + dur,
                });
                despawn_sensor_near(&mut commands, shrink_sensors.iter(), x, y);
            }
            ["swap", idx_a, idx_b, x, y] => {
                let (Some(a), Some(b), Some(x), Some(y)) = (
                    marble_by_index(&marbles, idx_a),
                    marble_by_index(&marbles, idx_b),
                    num(x),
                    num(y),
                ) else { continue };
                spawn_swap_rings(&mut commands, &mut meshes, &mut materials, &time, a, b);
                despawn_sensor_near(&mut commands, swap_sensors.iter(), x, y);
            }
            ["module", name, top, module_seed] => {
                let (Some(top), Ok(module_seed)) = (num(top), module_seed.parse::<u64>()) else { continue };
                spawn_level_module(
                    name, top, module_seed, palette.obstacle_color(),
                    &mut commands, &mode, &asset_server, &mut meshes, &mut materials,
                );
            }
            ["finish", next_top] => {
                let Some(next_top) = num(next_top) else { continue };
                close_level_with_finish(
                    next_top, palette.obstacle_color(),
                    &mut commands, &mode, &asset_server, &mut meshes, &mut materials,
                );
            }
            ["bouncy", x, y, amp] => {
                let (Some(x), Some(y), Some(amp)) = (num(x), num(y), num(amp)) else { continue };
                let target = Vec2::new(x, y);
                let hit = bouncys.iter()
                    .filter(|(e, _)| !pulsing.contains(*e))
                    .map(|(e, t)| (e, t.translation.truncate().distance_squared(target)))
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                if let Some((sphere, d2)) = hit {
                    if d2 < 0.25 * 0.25 {
                        commands.entity(sphere).insert(BouncePulse { elapsed: 0.0, amplitude: amp });
                    }
                }
            }
            _ => warn!("replay: evento horneado desconocido: {payload}"),
        }
    }
}

/// try_unfreeze no opera en replay (consulta el contexto de Rapier para no
/// descongelar dentro de otra canica — irrelevante aquí: las poses ya están
/// decididas). Este reemplazo solo expira el visual.
pub fn expire_replay_freezes(
    time: Res<Time>,
    frozen: Query<(Entity, &Frozen), With<Marble>>,
    mut commands: Commands,
) {
    for (marble, f) in &frozen {
        if time.elapsed_secs() < f.expires_at {
            continue;
        }
        commands.entity(f.visual).despawn();
        commands.entity(marble).remove::<Frozen>();
    }
}

fn marble_by_index(
    marbles: &Query<(Entity, &MarbleIndex), With<Marble>>,
    raw: &str,
) -> Option<Entity> {
    let want: usize = raw.parse().ok()?;
    marbles.iter().find(|(_, idx)| idx.0 == want).map(|(e, _)| e)
}

fn num(raw: &str) -> Option<f32> {
    raw.parse().ok()
}

/// El sensor del evento se identifica por posición: la generación del nivel es
/// determinista, así que en replay existe un sensor del mismo tipo en las mismas
/// coordenadas que tuvo en el bake.
fn despawn_sensor_near<'a>(
    commands: &mut Commands,
    sensors: impl Iterator<Item = (Entity, &'a Transform)>,
    x: f32,
    y: f32,
) {
    let target = Vec2::new(x, y);
    let nearest = sensors
        .map(|(e, t)| (e, t.translation.truncate().distance_squared(target)))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    if let Some((sensor, d2)) = nearest {
        if d2 < 0.25 * 0.25 {
            commands.entity(sensor).despawn();
        }
    }
}
