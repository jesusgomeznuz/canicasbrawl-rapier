use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use rand::RngCore;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rapier_bevy::{
    BakeKey, BodyType, ColliderShape, ObjectDef, SimulationMode, VisualDef, spawn_object,
};

use super::pickups::{
    attach_effect_marker, resolve_slot_variant, should_skip_effect, spawn_invisible_sensor,
    spawn_spinning_icon,
};
use super::structures::{spawn_floor, spawn_wall_segment, tinted_white};
use crate::game::baked_events::BakedEvent;
use crate::game::level::{ModuleData, WorldObject, load_module};
use crate::game::marbles::Marble;

#[derive(Resource)]
pub struct LevelSeed(pub u64);

#[derive(Resource)]
pub struct FinishTarget(pub f32);

#[derive(Resource)]
pub struct LevelGen {
    rng: SmallRng,
    next_top: f32,
    last_module: Option<&'static str>,
    modules_spawned: u32,
    finish_spawned: bool,
    target_secs: f32,
}

impl LevelGen {
    pub fn new(seed: u64, first_module_top: f32, target_secs: f32) -> Self {
        Self {
            rng: SmallRng::seed_from_u64(seed),
            next_top: first_module_top,
            last_module: None,
            modules_spawned: 0,
            finish_spawned: false,
            target_secs,
        }
    }
}

enum LevelGenerationAction {
    GenerateModule,
    SpawnFinishLine,
    DoNothing,
}

fn decide_level_action(
    level_gen: &LevelGen,
    leader_y: f32,
    elapsed_secs: f32,
) -> LevelGenerationAction {
    if level_gen.finish_spawned {
        return LevelGenerationAction::DoNothing;
    }

    let race_time_is_up = elapsed_secs >= level_gen.target_secs;
    if race_time_is_up {
        return LevelGenerationAction::SpawnFinishLine;
    }

    let trigger_distance = 3.0;
    let leader_is_near_the_horizon = leader_y - level_gen.next_top < trigger_distance;
    if leader_is_near_the_horizon {
        return LevelGenerationAction::GenerateModule;
    }
    LevelGenerationAction::DoNothing
}

pub fn generate_level(
    sim_time: Res<Time<Fixed>>,
    marbles: Query<&Transform, With<Marble>>,
    mut level_gen: ResMut<LevelGen>,
    mut events: EventWriter<BakedEvent>,
) {
    let Some(leader_y) = marbles.iter().map(|t| t.translation.y).reduce(f32::min) else {
        return;
    };
    match decide_level_action(&level_gen, leader_y, sim_time.elapsed_secs()) {
        LevelGenerationAction::GenerateModule => {
            let name = pick_module(&mut level_gen);
            let top = level_gen.next_top;
            let module_seed = level_gen.rng.next_u64();
            events.write(BakedEvent::Module {
                name: name.to_string(),
                top,
                seed: module_seed,
            });
            level_gen.next_top = top - module_height(name);
            level_gen.last_module = Some(name);
            level_gen.modules_spawned += 1;
        }
        LevelGenerationAction::SpawnFinishLine => {
            events.write(BakedEvent::Finish { top: level_gen.next_top });
            level_gen.finish_spawned = true;
        }
        LevelGenerationAction::DoNothing => {}
    }
}

fn module_height(name: &str) -> f32 {
    let module_gap = 0.1;
    let ModuleData { objects } = load_module(name);
    let (y_min, y_max) = objects
        .iter()
        .map(|o| o.y_bounds())
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), (mn, mx)| {
            (lo.min(mn), hi.max(mx))
        });
    (y_max - y_min) + module_gap
}

pub fn spawn_level_module(
    name: &str,
    top: f32,
    module_seed: u64,
    obstacle_color: Color,
    commands: &mut Commands,
    mode: &SimulationMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> f32 {
    let module_gap = 0.1;
    let bottom = spawn_module(
        name,
        top,
        obstacle_color,
        module_seed,
        commands,
        mode,
        asset_server,
        meshes,
        materials,
    ) - module_gap;
    spawn_wall_segment(
        top,
        bottom,
        obstacle_color,
        commands,
        mode,
        asset_server,
        meshes,
        materials,
    );
    bottom
}

pub fn close_level_with_finish(
    next_top: f32,
    obstacle_color: Color,
    commands: &mut Commands,
    mode: &SimulationMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let gap_module_to_finish = 0.4;
    let gap_finish_to_floor = 1.0;
    let finish_y = next_top - gap_module_to_finish;
    let floor_y = finish_y - gap_finish_to_floor;
    spawn_wall_segment(
        next_top,
        floor_y,
        obstacle_color,
        commands,
        mode,
        asset_server,
        meshes,
        materials,
    );
    spawn_floor(
        commands,
        mode,
        asset_server,
        meshes,
        materials,
        floor_y,
        obstacle_color,
    );
    crate::game::finish::spawn_finish_line(commands, meshes, materials, asset_server, finish_y);
}

fn pick_module(level_gen: &mut LevelGen) -> &'static str {
    let pool: &[(&'static str, u32)] = &[
        ("crosses", 5),
        ("zigzag", 5),
        ("spheres", 5),
        ("diamonds", 2),
        ("mini_zigzag", 3),
        ("more_stones", 2),
        ("hex_stones", 3),
        ("stones", 0),
        ("bars", 0),
        ("toruses", 5),
        ("bouncy_walls", 5),
    ];
    let weighted: Vec<&'static str> = pool
        .iter()
        .flat_map(|(name, weight)| std::iter::repeat(*name).take(*weight as usize))
        .collect();
    let is_first_module = level_gen.modules_spawned == 0;
    loop {
        let pick = weighted[level_gen.rng.gen_range(0..weighted.len())];
        let repeats_last = Some(pick) == level_gen.last_module;
        let spheres_as_first = is_first_module && pick == "spheres";
        if !repeats_last && !spheres_as_first {
            return pick;
        }
    }
}

fn module_acts_as_gate(name: &str) -> bool {
    matches!(name, "toruses" | "bouncy_walls" | "bars")
}

#[derive(Component, Clone, Copy)]
pub struct ModuleSpan {
    pub bottom: f32,
}

pub fn disable_modules_above_screen(
    camera: Query<(&Projection, &GlobalTransform), With<Camera3d>>,
    modules: Query<(Entity, &ModuleSpan), Without<ColliderDisabled>>,
    mut commands: Commands,
) {
    let Ok((projection, camera_transform)) = camera.single() else {
        return;
    };
    let exit_margin = 0.5;
    for (entity, span) in &modules {
        if crate::game::camera::world_y_above_screen(span.bottom, exit_margin, projection, camera_transform) {
            commands.entity(entity).insert(ColliderDisabled);
        }
    }
}

fn spawn_module(
    name: &str,
    level_top: f32,
    obstacle_color: Color,
    module_seed: u64,
    commands: &mut Commands,
    mode: &SimulationMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> f32 {
    let rng = &mut SmallRng::seed_from_u64(module_seed);
    let body_key = |obj_idx: usize| BakeKey(module_seed ^ ((obj_idx as u64 + 1) << 32));
    let ModuleData { objects, .. } = load_module(name);
    let (y_min, y_max) = objects
        .iter()
        .map(|o| o.y_bounds())
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), (mn, mx)| {
            (lo.min(mn), hi.max(mx))
        });
    let trimmed_height = y_max - y_min;
    let y_offset = level_top - y_max;
    let gate_span = module_acts_as_gate(name).then_some(ModuleSpan {
        bottom: level_top - trimmed_height,
    });
    let tag_if_gate = |commands: &mut Commands, entity: Entity| {
        if let Some(span) = gate_span {
            commands.entity(entity).insert(span);
        }
    };
    for (obj_idx, obj) in objects.iter().enumerate() {
        match obj {
            WorldObject::Box {
                x,
                y,
                hx,
                hy,
                rot,
                angvel,
                border_radius,
                friction,
                restitution,
                bouncy,
            } => {
                let entity = spawn_object(
                    commands,
                    ObjectDef {
                        shape: ColliderShape::Box {
                            hx: *hx,
                            hy: *hy,
                            hz: crate::UNIT / 4.0,
                        },
                        position: Vec3::new(*x, *y + y_offset, 0.0),
                        rotation: Quat::from_rotation_z(*rot),
                        body: if angvel != &[0.0; 3] {
                            BodyType::Kinematic
                        } else {
                            BodyType::Static
                        },
                        angvel: (angvel != &[0.0; 3]).then(|| Vec3::from(*angvel)),
                        visual: Some(VisualDef {
                            border_radius: *border_radius,
                            ..tinted_white(obstacle_color)
                        }),
                        restitution: Some(restitution.unwrap_or(0.05)),
                        friction: Some(friction.unwrap_or(0.15)),
                        ..Default::default()
                    },
                    mode,
                    asset_server,
                    meshes,
                    materials,
                );
                commands.entity(entity).insert(body_key(obj_idx));
                tag_if_gate(commands, entity);
                if *bouncy {
                    commands.entity(entity).insert((
                        ActiveEvents::COLLISION_EVENTS,
                        crate::game::sensors::bouncy::BouncyOnContact,
                    ));
                }
            }
            WorldObject::Sphere {
                x,
                y,
                radius,
                friction,
                restitution,
                bouncy,
            } => {
                let entity = spawn_object(
                    commands,
                    ObjectDef {
                        shape: ColliderShape::Sphere { radius: *radius },
                        position: Vec3::new(*x, *y + y_offset, 0.0),
                        body: BodyType::Static,
                        visual: Some(tinted_white(obstacle_color)),
                        restitution: Some(restitution.unwrap_or(0.05)),
                        friction: Some(friction.unwrap_or(0.15)),
                        ..Default::default()
                    },
                    mode,
                    asset_server,
                    meshes,
                    materials,
                );
                commands.entity(entity).insert(body_key(obj_idx));
                tag_if_gate(commands, entity);
                if *bouncy {
                    commands.entity(entity).insert((
                        ActiveEvents::COLLISION_EVENTS,
                        crate::game::sensors::bouncy::BouncyOnContact,
                    ));
                }
            }
            WorldObject::Mesh {
                x,
                y,
                rot,
                model_name,
                angvel,
                friction,
                restitution,
            } => {
                let entity = spawn_object(
                    commands,
                    ObjectDef {
                        shape: ColliderShape::MeshObject {
                            model_name: model_name.clone(),
                        },
                        position: Vec3::new(*x, *y + y_offset, 0.0),
                        rotation: Quat::from_rotation_z(*rot),
                        body: if angvel != &[0.0; 3] {
                            BodyType::Kinematic
                        } else {
                            BodyType::Static
                        },
                        angvel: (angvel != &[0.0; 3]).then(|| Vec3::from(*angvel)),
                        visual: Some(tinted_white(obstacle_color)),
                        restitution: Some(restitution.unwrap_or(0.05)),
                        friction: Some(friction.unwrap_or(0.15)),
                        ..Default::default()
                    },
                    mode,
                    asset_server,
                    meshes,
                    materials,
                );
                commands.entity(entity).insert(body_key(obj_idx));
                tag_if_gate(commands, entity);
            }
            WorldObject::Image {
                x,
                y,
                w,
                h,
                rot,
                texture,
            } => {
                let half_depth = crate::UNIT / 4.0;
                commands.spawn((
                    Mesh3d(meshes.add(Rectangle::new(*w, *h))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color_texture: Some(asset_server.load(texture.clone())),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..default()
                    })),
                    Transform::from_translation(Vec3::new(*x, *y + y_offset, half_depth + 0.001))
                        .with_rotation(Quat::from_rotation_z(*rot)),
                ));
            }
            WorldObject::Effect {
                x,
                y,
                w,
                h,
                rot,
                variant,
            } => {
                let position = Vec3::new(*x, *y + y_offset, 0.0);
                if should_skip_effect(variant, position.y) {
                    continue;
                }
                let sensor = spawn_invisible_sensor(
                    commands,
                    position,
                    *w,
                    *h,
                    *rot,
                    mode,
                    asset_server,
                    meshes,
                    materials,
                );
                tag_if_gate(commands, sensor);
                attach_effect_marker(commands, sensor, variant);
                spawn_spinning_icon(commands, asset_server, sensor, variant);
            }
            WorldObject::EffectSlot {
                x,
                y,
                w,
                h,
                rot,
                options,
            } => {
                let position = Vec3::new(*x, *y + y_offset, 0.0);
                let Some(variant) = resolve_slot_variant(options, position.y, rng) else {
                    continue;
                };
                let sensor = spawn_invisible_sensor(
                    commands,
                    position,
                    *w,
                    *h,
                    *rot,
                    mode,
                    asset_server,
                    meshes,
                    materials,
                );
                tag_if_gate(commands, sensor);
                attach_effect_marker(commands, sensor, variant);
                spawn_spinning_icon(commands, asset_server, sensor, variant);
            }
        }
    }
    level_top - trimmed_height
}
