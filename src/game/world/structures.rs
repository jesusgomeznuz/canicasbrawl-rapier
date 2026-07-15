use bevy::prelude::*;
use bevy_rapier3d::plugin::context::DefaultRapierContext;
use bevy_rapier3d::prelude::RapierConfiguration;
use rapier_bevy::{
    ColliderShape, ObjectDef, VisualAppearance, VisualDef, spawn_object,
};

use crate::game::scene::background::palette::ColorPalette;

/// Los muros iniciales de la arena: del tope hasta donde empezará el primer
/// módulo (la geometría la dicta level_generation::first_module_top).
pub fn spawn_walls(
    mut commands: Commands,
    palette: Res<ColorPalette>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let walls_top = 2.0;
    spawn_wall_segment(
        walls_top,
        super::level_generation::first_module_top(),
        palette.obstacle_color(),
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
    );
}

/// La gravedad del escenario.
pub fn set_gravity(mut config: Query<&mut RapierConfiguration, With<DefaultRapierContext>>) {
    if let Ok(mut cfg) = config.single_mut() {
        cfg.gravity = Vec3::new(0.0, -3.0, 0.0);
    }
}

pub fn spawn_floor(
    commands: &mut Commands,
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
