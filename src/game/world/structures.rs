use bevy::prelude::*;
use rapier_bevy::{
    ColliderShape, ObjectDef, SimulationMode, VisualAppearance, VisualDef, spawn_object,
};

pub fn spawn_floor(
    commands: &mut Commands,
    mode: &SimulationMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    floor_y: f32,
    obstacle_color: Color,
) {
    let anti_tunneling_half_height = 3.0_f32;
    spawn_object(
        commands,
        ObjectDef {
            shape: ColliderShape::Box {
                hx: 10.0,
                hy: anti_tunneling_half_height,
                hz: 3.0,
            },
            position: Vec3::new(0.0, floor_y - anti_tunneling_half_height, 0.0),
            visual: Some(VisualDef {
                border_radius: Some(0.02),
                ..tinted_white(obstacle_color)
            }),
            restitution: Some(0.05),
            friction: Some(0.7),
            ..Default::default()
        },
        mode,
        asset_server,
        meshes,
        materials,
    );
}

pub fn spawn_wall_segment(
    top: f32,
    bottom: f32,
    obstacle_color: Color,
    commands: &mut Commands,
    mode: &SimulationMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let half_h = (top - bottom) / 2.0;
    let center_y = (top + bottom) / 2.0;
    let half_width = 0.55;
    let wall_shape = ColliderShape::Box {
        hx: 0.05,
        hy: half_h,
        hz: crate::UNIT / 4.0,
    };

    for x_sign in [-1.0_f32, 1.0] {
        spawn_object(
            commands,
            ObjectDef {
                shape: wall_shape.clone(),
                position: Vec3::new(x_sign * (half_width + 0.05), center_y, 0.0),
                visual: Some(tinted_white(obstacle_color)),
                restitution: Some(0.05),
                ..Default::default()
            },
            mode,
            asset_server,
            meshes,
            materials,
        );
    }
}

pub fn tinted_white(color: Color) -> VisualDef {
    VisualDef {
        appearance: VisualAppearance::Color(color),
        roughness: 0.85,
        ..Default::default()
    }
}
