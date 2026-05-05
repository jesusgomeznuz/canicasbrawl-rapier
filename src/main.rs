use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_rapier3d::plugin::context::DefaultRapierContext;
use bevy_rapier3d::prelude::*;
use rapier_bevy::modes::{Mode, parse_mode, record_duration};
use rapier_bevy::{
    AssetsLoading, BodyType, ColliderShape, GraphicsPlugin, LockedAxes, ObjectDef,
    PhysicsStatsPlugin, RecordPlugin, SimMode, VisualAppearance, VisualDef, spawn_object,
};

const UNIT: f32 = 0.35;

#[derive(serde::Deserialize)]
struct PlatformData {
    x: f32, y: f32, hx: f32, hy: f32, rot: f32, angvel_z: f32,
}

#[derive(serde::Deserialize)]
struct SpawnData {
    cx: f32, cy: f32,
}

#[derive(serde::Deserialize)]
struct LevelData {
    platforms: Vec<PlatformData>,
    floor_y: Option<f32>,
    spawn: Option<SpawnData>,
}

fn load_level() -> LevelData {
    let json = std::fs::read_to_string("assets/levels/level_01.json")
        .expect("No se encontró assets/levels/level_01.json — corre: cargo run -- --process-figma");
    serde_json::from_str(&json).expect("level_01.json tiene formato inválido")
}

fn main() {
    if std::env::args().any(|a| a == "--process-figma") {
        let status = std::process::Command::new("python3")
            .arg("figma_export.py")
            .status()
            .expect("No se pudo ejecutar python3. ¿Está instalado?");
        std::process::exit(if status.success() { 0 } else { 1 });
    }
    match parse_mode() {
        Mode::Preprocess => {}
        Mode::Sim(mode) => run_world_mode(mode),
    }
}

fn run_world_mode(mode: SimMode) {
    let record = record_duration();
    let mut app = App::new();

    if record.is_none() {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (540.0, 960.0).into(),
                title: "CanicasBrawl".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(PhysicsStatsPlugin);
    }

    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(GraphicsPlugin)
        .add_systems(Startup, (setup, set_gravity))
        .add_systems(Update, camera_follows_lowest_marble)
        .add_systems(
            PostUpdate,
            update_marble_labels.after(TransformSystem::TransformPropagate),
        )
        .insert_resource(ClearColor(Color::srgb(0.329, 0.765, 0.980)))
        .insert_resource(mode);

    if let Some(secs) = record {
        app.add_plugins(RecordPlugin {
            duration_secs: secs,
        });
    }

    app.run();
}

#[derive(Component)]
struct Marble {
    pub nickname: &'static str,
}

#[derive(Component)]
struct MarbleLabel(Entity);

struct MarbleConfig {
    nickname: &'static str,
    color: Color,
    image: &'static str,
}

fn setup(
    mut commands: Commands,
    mode: Res<SimMode>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut assets_loading: Option<ResMut<AssetsLoading>>,
) {
    let level = load_level();
    spawn_floor(&mut commands, &mode, &asset_server, &mut meshes, &mut materials,
        level.floor_y.unwrap_or(0.0));
    spawn_side_walls(&mut commands, &mode, &asset_server, &mut meshes, &mut materials);
    for p in &level.platforms {
        spawn_platform(&mut commands, &mode, &asset_server, &mut meshes, &mut materials,
            p.x, p.y, p.hx, p.hy, p.rot, p.angvel_z);
    }
    spawn_marbles(&mut commands, &mode, &asset_server, &mut meshes, &mut materials,
        level.spawn.as_ref(), &mut assets_loading);
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
    let wall_shape = ColliderShape::Box {
        hx: 0.05,
        hy: arena_height / 2.0,
        hz: UNIT / 2.0 / 2.0,
    };
    let wall_y = arena_height / 2.0;

    for x_sign in [-1.0_f32, 1.0] {
        spawn_object(
            commands,
            ObjectDef {
                shape: wall_shape.clone(),
                position: Vec3::new(x_sign * (half_width + 0.05), wall_y, 0.0),
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


fn spawn_platform(
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    x: f32,
    y: f32,
    hx: f32,
    hy: f32,
    rotation: f32,
    angvel_z: f32,
) {
    let spinning = angvel_z != 0.0;
    let entity = spawn_object(
        commands,
        ObjectDef {
            shape: ColliderShape::Box {
                hx,
                hy,
                hz: UNIT * 0.5 / 2.0,
            },
            position: Vec3::new(x, y, 0.0),
            rotation: Quat::from_rotation_z(rotation),
            body: if spinning {
                BodyType::Kinematic
            } else {
                BodyType::Static
            },
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
            angvel: Vec3::new(0.0, 0.0, angvel_z),
        });
    }
}

fn spawn_marbles(
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    spawn: Option<&SpawnData>,
    assets_loading: &mut Option<ResMut<AssetsLoading>>,
) {
    let marbles: &[MarbleConfig] = &[
        MarbleConfig {
            nickname: "Goku",
            color: Color::srgb(0.90, 0.85, 0.60),
            image: "characters/Goku.png",
        },
        MarbleConfig {
            nickname: "Bart",
            color: Color::srgb(0.95, 0.85, 0.10),
            image: "characters/bart.png",
        },
        MarbleConfig {
            nickname: "Naruto",
            color: Color::srgb(0.95, 0.45, 0.05),
            image: "characters/naruto.png",
        },
        MarbleConfig {
            nickname: "Finn",
            color: Color::srgb(0.30, 0.65, 0.90),
            image: "characters/finn.png",
        },
        MarbleConfig {
            nickname: "Rick",
            color: Color::srgb(0.55, 0.80, 0.55),
            image: "characters/rick.png",
        },
        MarbleConfig {
            nickname: "Shrek",
            color: Color::srgb(0.40, 0.65, 0.20),
            image: "characters/shrek.png",
        },
        MarbleConfig {
            nickname: "Vegeta",
            color: Color::srgb(0.55, 0.10, 0.80),
            image: "characters/vegeta.png",
        },
        MarbleConfig {
            nickname: "Patricio",
            color: Color::srgb(0.95, 0.55, 0.70),
            image: "characters/patricio.png",
        },
        MarbleConfig {
            nickname: "Gumball",
            color: Color::srgb(0.30, 0.55, 0.90),
            image: "characters/gumball.png",
        },
    ];
    // Grid 3×3 centrado en spawn_area (si está en el JSON) o posición por defecto
    let (cx, cy) = spawn.map(|s| (s.cx, s.cy)).unwrap_or((0.0, 10.3));
    let dx = 0.25;
    let dy = 0.30;
    let grid: [(f32, f32); 9] = [
        (cx - dx, cy + dy), (cx, cy + dy), (cx + dx, cy + dy),
        (cx - dx, cy),      (cx, cy),      (cx + dx, cy),
        (cx - dx, cy - dy), (cx, cy - dy), (cx + dx, cy - dy),
    ];
    let radius = 0.09;
    let half_depth = UNIT / 2.0 / 2.0;

    for (cfg, (x, y)) in marbles.iter().zip(grid.iter()) {
        let entity = spawn_object(
            commands,
            ObjectDef {
                shape: ColliderShape::Cylinder {
                    half_height: half_depth,
                    radius,
                    axis: Vec3::Z,
                },
                position: Vec3::new(*x, *y, 0.0),
                body: BodyType::Dynamic,
                restitution: Some(0.6),
                friction: Some(0.4),
                linear_damping: Some(0.15),
                angular_damping: Some(0.4),
                ccd: true,
                locked_axes: Some(
                    LockedAxes::TRANSLATION_LOCKED_Z
                        | LockedAxes::ROTATION_LOCKED_X
                        | LockedAxes::ROTATION_LOCKED_Y,
                ),
                visual: Some(VisualDef::white_matte()),
                ..Default::default()
            },
            mode,
            asset_server,
            meshes,
            materials,
        );

        commands.entity(entity).insert(Marble {
            nickname: cfg.nickname,
        });

        commands.spawn((
            Text2d::new(cfg.nickname),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::BLACK),
            TextLayout::new_with_justify(JustifyText::Center),
            MarbleLabel(entity),
        ));

        let quad_size = radius * 2.0;
        let bg_handle: Handle<Image> = asset_server.load("characters/circle_white.png");
        let img_handle: Handle<Image> = asset_server.load(cfg.image);
        if let Some(al) = assets_loading.as_deref_mut() {
            al.0.push(bg_handle.clone().untyped());
            al.0.push(img_handle.clone().untyped());
        }
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Mesh3d(meshes.add(Rectangle::new(quad_size, quad_size))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(bg_handle),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })),
                Transform::from_xyz(0.0, 0.0, half_depth + 0.001),
            ));
            parent.spawn((
                Mesh3d(meshes.add(Rectangle::new(quad_size, quad_size))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(img_handle),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })),
                Transform::from_xyz(0.0, 0.0, half_depth + 0.002),
            ));
        });
    }
}

fn update_marble_labels(
    marbles: Query<&GlobalTransform, With<Marble>>,
    mut labels: Query<(&mut Transform, &mut TextFont, &MarbleLabel)>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let Ok((camera, cam_gt)) = camera_q.single() else {
        return;
    };
    let Some(viewport) = camera.logical_viewport_size() else {
        return;
    };
    // 2.2% del alto del viewport → tamaño relativo igual en ventana y en --record
    let font_size = viewport.y * 0.022;
    for (mut transform, mut font, MarbleLabel(marble_entity)) in &mut labels {
        font.font_size = font_size;
        let Ok(marble_gt) = marbles.get(*marble_entity) else {
            continue;
        };
        let above = marble_gt.translation() + Vec3::Y * 0.13;
        if let Ok(screen_pos) = camera.world_to_viewport(cam_gt, above) {
            transform.translation.x = screen_pos.x - viewport.x / 2.0;
            transform.translation.y = viewport.y / 2.0 - screen_pos.y;
        }
    }
}

fn set_gravity(mut config: Query<&mut RapierConfiguration, With<DefaultRapierContext>>) {
    if let Ok(mut cfg) = config.single_mut() {
        cfg.gravity = Vec3::new(0.0, -3.0, 0.0);
    }
}

fn camera_follows_lowest_marble(
    marbles: Query<&Transform, With<Marble>>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<Marble>)>,
) {
    let cam_z = 2.5;
    let cam_y_offset = 0.2;

    let Some(lowest_y) = marbles.iter().map(|t| t.translation.y).reduce(f32::min) else {
        return;
    };
    let Ok(mut cam) = camera.single_mut() else {
        return;
    };

    cam.translation = Vec3::new(0.0, lowest_y + cam_y_offset, cam_z);
    cam.rotation = Quat::IDENTITY;
}
