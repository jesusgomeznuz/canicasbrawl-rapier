use super::level::{ModuleData, WorldObject, load_module};
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
        "crosses",
        "zigzag",
        "spheres",
        "toruses",
        "bouncy_walls",
        "crosses",
        "zigzag",
        "spheres",
        "crosses",
        "zigzag",
        "spheres",
        "crosses",
        "zigzag",
        "spheres",
        "crosses",
        "zigzag",
        "spheres",
        "crosses",
        "zigzag",
        "spheres",
    ];
    let module_gap = 0.1;
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
        ) - module_gap;
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
    let ModuleData { objects, .. } = load_module(name);
    let (y_min, y_max) = objects
        .iter()
        .map(|o| o.y_bounds())
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), (mn, mx)| {
            (lo.min(mn), hi.max(mx))
        });
    let trimmed_height = y_max - y_min;
    let y_offset = level_top - y_max;
    for obj in &objects {
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
                            ..VisualDef::white_matte()
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
                if *bouncy {
                    commands.entity(entity).insert((
                        ActiveEvents::COLLISION_EVENTS,
                        super::bouncy::BouncyOnContact,
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
                        visual: Some(VisualDef::white_matte()),
                        restitution: Some(restitution.unwrap_or(0.05)),
                        friction: Some(friction.unwrap_or(0.15)),
                        ..Default::default()
                    },
                    mode,
                    asset_server,
                    meshes,
                    materials,
                );
                if *bouncy {
                    commands.entity(entity).insert((
                        ActiveEvents::COLLISION_EVENTS,
                        super::bouncy::BouncyOnContact,
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
                spawn_object(
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
                        visual: Some(VisualDef::white_matte()),
                        restitution: Some(restitution.unwrap_or(0.05)),
                        friction: Some(friction.unwrap_or(0.15)),
                        ..Default::default()
                    },
                    mode,
                    asset_server,
                    meshes,
                    materials,
                );
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
            WorldObject::Effect { x, y, w, h, rot, variant } => {
                let position = Vec3::new(*x, *y + y_offset, 0.0);
                if should_skip_effect(variant, position.y) { continue }
                let sensor = spawn_invisible_sensor(
                    commands, position, *w, *h, *rot, mode, asset_server, meshes, materials,
                );
                attach_effect_marker(commands, sensor, variant);
                spawn_spinning_icon(commands, asset_server, sensor, variant);
            }
        }
    }
    level_top - trimmed_height
}

fn should_skip_effect(variant: &str, world_y: f32) -> bool {
    let swap_block_above_y = -3.0_f32;
    variant == "swap" && world_y > swap_block_above_y
}

fn spawn_invisible_sensor(
    commands: &mut Commands,
    position: Vec3,
    w: f32,
    h: f32,
    rot: f32,
    mode: &SimMode,
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

fn attach_effect_marker(commands: &mut Commands, sensor: Entity, variant: &str) {
    match variant {
        "freeze" => { commands.entity(sensor).insert(super::effects::FreezeEffect); }
        "shrink" => { commands.entity(sensor).insert(super::effects::ShrinkEffect); }
        "swap"   => { commands.entity(sensor).insert(super::effects::SwapEffect); }
        other => panic!("Variante de effect desconocida: '{}'", other),
    }
}

fn spawn_spinning_icon(
    commands: &mut Commands,
    asset_server: &AssetServer,
    sensor: Entity,
    variant: &str,
) {
    let (axis, speed, scale) = icon_tuning_for(variant);
    let scene = asset_server.load(format!("effects/{}.glb#Scene0", variant));
    let icon = commands.spawn((
        SceneRoot(scene),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(scale)),
        super::effects::SpinningIcon { axis, speed },
    )).id();
    commands.entity(sensor).add_child(icon);
}

fn icon_tuning_for(variant: &str) -> (Vec3, f32, f32) {
    match variant {
        "freeze" => (Vec3::Y, 1.0, 13.5),
        "shrink" => (Vec3::Y, 1.0, 0.046),
        "swap"   => (Vec3::Z, 2.0, 0.25),
        _ => (Vec3::Y, 1.5, 0.05),
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
