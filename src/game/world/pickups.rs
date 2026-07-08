use bevy::prelude::*;
use rand::Rng;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rapier_bevy::{BodyType, ColliderShape, ObjectDef, SimulationMode, spawn_object};

pub fn resolve_slot_variant<'a>(
    options: &'a [String],
    world_y: f32,
    rng: &mut SmallRng,
) -> Option<&'a str> {
    let effect_slot_skip_chance = 0.40_f32;
    if rng.gen_range(0.0_f32..1.0) < effect_slot_skip_chance {
        return None;
    }
    if options.is_empty() {
        let weighted: Vec<&str> = default_effect_weights()
            .iter()
            .flat_map(|(name, w)| std::iter::repeat(*name).take(*w as usize))
            .filter(|v| !should_skip_effect(v, world_y))
            .collect();
        weighted.choose(rng).copied()
    } else {
        let valid: Vec<&str> = options
            .iter()
            .map(|s| s.as_str())
            .filter(|v| !should_skip_effect(v, world_y))
            .collect();
        valid.choose(rng).copied()
    }
}

pub fn spawn_invisible_sensor(
    commands: &mut Commands,
    position: Vec3,
    w: f32,
    h: f32,
    rot: f32,
    mode: &SimulationMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Entity {
    spawn_object(
        commands,
        ObjectDef {
            shape: ColliderShape::Box {
                hx: w / 2.0,
                hy: h / 2.0,
                hz: crate::UNIT / 4.0,
            },
            position,
            rotation: Quat::from_rotation_z(rot),
            body: BodyType::Static,
            sensor: true,
            visual: None,
            ..Default::default()
        },
        mode,
        asset_server,
        meshes,
        materials,
    )
}

pub fn attach_effect_marker(commands: &mut Commands, sensor: Entity, variant: &str) {
    match variant {
        "freeze" => {
            commands.entity(sensor).insert(crate::game::sensors::freeze::FreezeEffect);
        }
        "shrink" => {
            commands.entity(sensor).insert(crate::game::sensors::shrink::ShrinkEffect);
        }
        "swap" => {
            commands.entity(sensor).insert(crate::game::sensors::swap::SwapEffect);
        }
        other => panic!("Variante de effect desconocida: '{}'", other),
    }
}

pub fn spawn_spinning_icon(
    commands: &mut Commands,
    asset_server: &AssetServer,
    sensor: Entity,
    variant: &str,
) {
    let (axis, speed, scale) = icon_tuning_for(variant);
    let scene = asset_server.load(format!("effects/{}.glb#Scene0", variant));
    let icon = commands
        .spawn((
            SceneRoot(scene),
            Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(scale)),
            crate::game::sensors::icons::SpinningIcon { axis, speed },
        ))
        .id();
    commands.entity(sensor).add_child(icon);
}

pub fn should_skip_effect(variant: &str, world_y: f32) -> bool {
    let swap_block_above_y = -3.0_f32;
    variant == "swap" && world_y > swap_block_above_y
}

fn default_effect_weights() -> &'static [(&'static str, u32)] {
    &[("freeze", 4), ("swap", 3), ("shrink", 1)]
}

fn icon_tuning_for(variant: &str) -> (Vec3, f32, f32) {
    match variant {
        "freeze" => (Vec3::Y, 1.0, 13.5),
        "shrink" => (Vec3::Y, 1.0, 0.046),
        "swap" => (Vec3::Z, 2.0, 0.25),
        _ => (Vec3::Y, 1.5, 0.05),
    }
}
