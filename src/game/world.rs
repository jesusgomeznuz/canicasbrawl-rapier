use super::level::{ModuleData, load_module};
use bevy::prelude::*;
use bevy_rapier3d::plugin::context::DefaultRapierContext;
use bevy_rapier3d::prelude::*;
use rapier_bevy::{
    AssetsLoading, BodyType, ColliderShape, ObjectDef, SimMode, VisualDef, spawn_object,
};

pub fn setup(
    mut commands: Commands,
    mode: Res<SimMode>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut assets_loading: Option<ResMut<AssetsLoading>>,
) {
    let (spawn_y, level_bottom) = spawn_level(
        &mut commands,
        &mode,
        &asset_server,
        &mut meshes,
        &mut materials,
    );
    spawn_side_walls(
        level_bottom,
        &mut commands,
        &mode,
        &asset_server,
        &mut meshes,
        &mut materials,
    );
    super::marbles::spawn_marbles(
        &mut commands,
        &mode,
        &asset_server,
        &mut meshes,
        &mut materials,
        0.0,
        spawn_y,
        &mut assets_loading,
    );
}

pub fn set_gravity(mut config: Query<&mut RapierConfiguration, With<DefaultRapierContext>>) {
    if let Ok(mut cfg) = config.single_mut() {
        cfg.gravity = Vec3::new(0.0, -3.0, 0.0);
    }
}

fn spawn_level(
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> (f32, f32) {
    let spawn_y = 0.0;
    let modules = [
        "crosses", "zigzag", "crosses", "zigzag", "crosses", "zigzag", "crosses", "zigzag",
        "crosses", "zigzag", "crosses", "zigzag", "crosses", "zigzag", "crosses", "zigzag",
        "crosses", "zigzag", "crosses", "zigzag", "crosses", "zigzag", "crosses", "zigzag",
        "crosses", "zigzag",
    ];
    let mut next_top = spawn_y;
    for name in modules {
        next_top = spawn_module(
            name,
            next_top,
            commands,
            mode,
            asset_server,
            meshes,
            materials,
        );
    }
    let level_bottom = next_top - 0.5;
    spawn_floor(
        commands,
        mode,
        asset_server,
        meshes,
        materials,
        level_bottom,
    );

    (spawn_y, level_bottom)
}

fn spawn_module(
    name: &str,
    level_top: f32,
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> f32 {
    let ModuleData { height, platforms } = load_module(name);
    let y_offset = level_top - height;
    for p in &platforms {
        let spinning = p.angvel_z != 0.0;
        let entity = spawn_object(
            commands,
            ObjectDef {
                shape: ColliderShape::Box {
                    hx: p.hx,
                    hy: p.hy,
                    hz: crate::UNIT / 4.0,
                },
                position: Vec3::new(p.x, p.y + y_offset, 0.0),
                rotation: Quat::from_rotation_z(p.rot),
                body: if spinning {
                    BodyType::Kinematic
                } else {
                    BodyType::Static
                },
                visual: Some(VisualDef {
                    border_radius: p.border_radius.or(Some(0.02)),
                    ..VisualDef::white_matte()
                }),
                restitution: Some(0.05),
                friction: Some(0.15),
                ..Default::default()
            },
            mode,
            asset_server,
            meshes,
            materials,
        );
        if spinning {
            commands.entity(entity).insert(Velocity {
                linvel: Vec3::ZERO,
                angvel: Vec3::new(0.0, 0.0, p.angvel_z),
            });
        }
    }
    y_offset
}

fn spawn_floor(
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    floor_y: f32,
) {
    let hy = 0.03_f32;
    spawn_object(
        commands,
        ObjectDef {
            shape: ColliderShape::Box {
                hx: 10.0,
                hy,
                hz: 3.0,
            },
            position: Vec3::new(0.0, floor_y - hy, 0.0),
            visual: Some(VisualDef {
                border_radius: Some(0.02),
                ..VisualDef::white_matte()
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

fn spawn_side_walls(
    level_bottom: f32,
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let top_y = 2.0; // margen sobre el spawn
    let half_h = (top_y - level_bottom) / 2.0;
    let center_y = (top_y + level_bottom) / 2.0;
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
                visual: Some(VisualDef {
                    border_radius: Some(0.02),
                    ..VisualDef::white_matte()
                }),
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
