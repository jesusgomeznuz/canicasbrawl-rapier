use bevy::prelude::*;
use bevy_rapier3d::plugin::context::DefaultRapierContext;
use bevy_rapier3d::prelude::*;
use rapier_bevy::{AssetsLoading, BodyType, ColliderShape, ObjectDef, SimMode, VisualDef, spawn_object};
use crate::level::PlatformData;

pub fn setup(
    mut commands: Commands,
    mode: Res<SimMode>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut assets_loading: Option<ResMut<AssetsLoading>>,
) {
    let level = crate::level::load_level();
    spawn_floor(&mut commands, &mode, &asset_server, &mut meshes, &mut materials, level.floor_y.unwrap_or(0.0));
    spawn_side_walls(&mut commands, &mode, &asset_server, &mut meshes, &mut materials);
    spawn_platforms(&mut commands, &mode, &asset_server, &mut meshes, &mut materials, &level.platforms);
    crate::marbles::spawn_marbles(&mut commands, &mode, &asset_server, &mut meshes, &mut materials, level.spawn.as_ref(), &mut assets_loading);
}

pub fn set_gravity(mut config: Query<&mut RapierConfiguration, With<DefaultRapierContext>>) {
    if let Ok(mut cfg) = config.single_mut() {
        cfg.gravity = Vec3::new(0.0, -3.0, 0.0);
    }
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
            shape: ColliderShape::Box { hx: 10.0, hy, hz: 3.0 },
            position: Vec3::new(0.0, floor_y - hy, 0.0),
            visual: Some(VisualDef { border_radius: Some(0.02), ..VisualDef::white_matte() }),
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
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let half_width = 0.55;
    let arena_height = 50.0;
    let wall_hz = crate::UNIT / 4.0;
    let wall_shape = ColliderShape::Box { hx: 0.05, hy: arena_height / 2.0, hz: wall_hz };

    for x_sign in [-1.0_f32, 1.0] {
        spawn_object(
            commands,
            ObjectDef {
                shape: wall_shape.clone(),
                position: Vec3::new(x_sign * (half_width + 0.05), arena_height / 2.0, 0.0),
                visual: Some(VisualDef { border_radius: Some(0.02), ..VisualDef::white_matte() }),
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

fn spawn_platforms(
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    platforms: &[PlatformData],
) {
    for p in platforms {
        spawn_platform(commands, mode, asset_server, meshes, materials, p);
    }
}

fn spawn_platform(
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    p: &PlatformData,
) {
    let spinning = p.angvel_z != 0.0;
    let entity = spawn_object(
        commands,
        ObjectDef {
            shape: ColliderShape::Box { hx: p.hx, hy: p.hy, hz: crate::UNIT / 4.0 },
            position: Vec3::new(p.x, p.y, 0.0),
            rotation: Quat::from_rotation_z(p.rot),
            body: if spinning { BodyType::Kinematic } else { BodyType::Static },
            visual: Some(VisualDef { border_radius: Some(0.02), ..VisualDef::white_matte() }),
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
