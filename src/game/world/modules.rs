use bevy::prelude::*;
use bevy_rapier3d::prelude::ActiveEvents;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rapier_bevy::{
    BodyType, ColliderShape, ObjectDef, TimelineKey, VisualDef, spawn_object,
};

use super::pickups::{
    attach_effect_marker, resolve_slot_variant, should_skip_effect, spawn_invisible_sensor,
    spawn_spinning_icon,
};
use super::structures::tinted_white;

#[derive(serde::Deserialize, Clone)]
#[serde(tag = "kind")]
pub enum WorldObject {
    Box {
        x: f32, y: f32, hx: f32, hy: f32, rot: f32,
        #[serde(default)]
        angvel: [f32; 3],
        #[serde(default)]
        border_radius: Option<f32>,
        #[serde(default)]
        friction: Option<f32>,
        #[serde(default)]
        restitution: Option<f32>,
        #[serde(default)]
        bouncy: bool,
    },
    Sphere {
        x: f32, y: f32, radius: f32,
        #[serde(default)]
        friction: Option<f32>,
        #[serde(default)]
        restitution: Option<f32>,
        #[serde(default)]
        bouncy: bool,
    },
    Mesh {
        x: f32, y: f32, rot: f32, model_name: String,
        #[serde(default)]
        angvel: [f32; 3],
        #[serde(default)]
        friction: Option<f32>,
        #[serde(default)]
        restitution: Option<f32>,
    },
    Image {
        x: f32, y: f32, w: f32, h: f32, rot: f32, texture: String,
    },
    Effect {
        x: f32, y: f32, w: f32, h: f32, rot: f32, variant: String,
    },
    EffectSlot {
        x: f32, y: f32, w: f32, h: f32, rot: f32,
        #[serde(default)]
        options: Vec<String>,
    },
}

#[derive(serde::Deserialize)]
pub struct ModuleData {
    pub objects: Vec<WorldObject>,
}

#[derive(Component, Clone, Copy)]
pub struct ModuleSpan {
    pub bottom: f32,
}

pub fn load_module(name: &str) -> ModuleData {
    let path = format!("assets/modules/{name}.json");
    let json = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("No se encontró {path}"));
    serde_json::from_str(&json).unwrap_or_else(|_| panic!("{path} tiene formato inválido"))
}

pub fn spawn_module(
    name: &str,
    level_top: f32,
    obstacle_color: Color,
    module_seed: u64,
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> f32 {
    let rng = &mut SmallRng::seed_from_u64(module_seed);
    let body_key = |obj_idx: usize| TimelineKey(module_seed ^ ((obj_idx as u64 + 1) << 32));
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
                    asset_server,
                    meshes,
                    materials,
                );
                commands.entity(entity).insert(body_key(obj_idx));
                tag_if_gate(commands, entity);
                if *bouncy {
                    commands.entity(entity).insert((
                        ActiveEvents::COLLISION_EVENTS,
                        crate::game::world::sensors::bouncy::BouncyOnContact,
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
                    asset_server,
                    meshes,
                    materials,
                );
                commands.entity(entity).insert(body_key(obj_idx));
                tag_if_gate(commands, entity);
                if *bouncy {
                    commands.entity(entity).insert((
                        ActiveEvents::COLLISION_EVENTS,
                        crate::game::world::sensors::bouncy::BouncyOnContact,
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

pub fn module_height(name: &str) -> f32 {
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

impl WorldObject {
    pub fn y_bounds(&self) -> (f32, f32) {
        match self {
            WorldObject::Box { y, hx, hy, rot, .. } => {
                let extent = (hx * rot.sin()).abs() + (hy * rot.cos()).abs();
                (y - extent, y + extent)
            }
            WorldObject::Sphere { y, radius, .. } => (y - radius, y + radius),
            WorldObject::Mesh { y, model_name, .. } => {
                let outer_r = torus_outer_radius(model_name).unwrap_or(0.0);
                (y - outer_r, y + outer_r)
            }
            WorldObject::Image { y, w, h, rot, .. } => {
                let extent = (w * 0.5 * rot.sin()).abs() + (h * 0.5 * rot.cos()).abs();
                (y - extent, y + extent)
            }
            WorldObject::Effect { y, w, h, rot, .. } => {
                let extent = (w * 0.5 * rot.sin()).abs() + (h * 0.5 * rot.cos()).abs();
                (y - extent, y + extent)
            }
            WorldObject::EffectSlot { y, w, h, rot, .. } => {
                let extent = (w * 0.5 * rot.sin()).abs() + (h * 0.5 * rot.cos()).abs();
                (y - extent, y + extent)
            }
        }
    }
}

fn module_acts_as_gate(name: &str) -> bool {
    matches!(name, "toruses" | "bouncy_walls" | "bars")
}

fn torus_outer_radius(model_name: &str) -> Option<f32> {
    let ["torus", major, minor] = model_name.split('_').collect::<Vec<_>>()[..] else {
        return None;
    };
    let major_mm: i32 = major.trim_start_matches('R').parse().ok()?;
    let minor_mm: i32 = minor.trim_start_matches('r').parse().ok()?;
    Some((major_mm + minor_mm) as f32 / 1000.0)
}
