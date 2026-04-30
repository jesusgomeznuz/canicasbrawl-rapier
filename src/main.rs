use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_rapier3d::plugin::context::DefaultRapierContext;
use bevy_rapier3d::prelude::*;
use rapier_bevy::modes::{Mode, parse_mode, record_duration};
use rapier_bevy::{
    AssetsLoading, BodyType, ColliderShape, GraphicsPlugin, LockedAxes, ObjectDef,
    PhysicsStatsPlugin, RecordPlugin, SimMode, VisualAppearance, VisualDef, spawn_object,
};

// 1 unit = 1 metro, gravedad -9.81 m/s².
// Canicas de radio 0.08 m — "game scale" que llena visualmente el arena de 1.1 m de ancho.{}

const UNIT: f32 = 0.35;

fn main() {
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
    spawn_floor(&mut commands, &mode, &asset_server, &mut meshes, &mut materials);
    spawn_side_walls(&mut commands, &mode, &asset_server, &mut meshes, &mut materials);
    spawn_zigzag_platforms(&mut commands, &mode, &asset_server, &mut meshes, &mut materials);
    spawn_marbles(&mut commands, &mode, &asset_server, &mut meshes, &mut materials, &mut assets_loading);
}

fn spawn_floor(
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let half_width = 0.55;
    spawn_object(
        commands,
        ObjectDef {
            shape: ColliderShape::Box {
                hx: half_width + 0.05,
                hy: 0.03,
                hz: UNIT,
            },
            position: Vec3::new(0.0, -0.03, 0.0),
            visual: Some(VisualDef::grass_green()),
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
    let arena_height = 10.0;
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
                visual: Some(VisualDef::white_matte()),
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

fn spawn_zigzag_platforms(
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let tilt = std::f32::consts::FRAC_PI_6; // 30°
    // (center_x, y, tilt_z) — negativo = lado derecho más bajo, canicas caen a la derecha
    let platforms = [
        (-0.25_f32, 2.8_f32, -tilt),
        (0.25, 2.0, tilt),
        (-0.25, 1.2, -tilt),
        (0.25, 0.4, tilt),
    ];

    for (cx, y, tilt_z) in platforms {
        spawn_object(
            commands,
            ObjectDef {
                shape: ColliderShape::Box {
                    hx: 0.50,
                    hy: 0.03,
                    hz: UNIT * 0.5 / 2.0,
                },
                position: Vec3::new(cx, y, 0.0),
                rotation: Quat::from_rotation_z(tilt_z),
                visual: Some(VisualDef::white_matte()),
                restitution: Some(0.05),
                friction: Some(0.15),
                ..Default::default()
            },
            mode,
            asset_server,
            meshes,
            materials,
        );
    }
}

fn spawn_marbles(
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    assets_loading: &mut Option<ResMut<AssetsLoading>>,
) {
    let marbles: &[MarbleConfig] = &[
        MarbleConfig { nickname: "Goku",     color: Color::srgb(0.90, 0.85, 0.60), image: "characters/Goku.png" },
        MarbleConfig { nickname: "Bart",     color: Color::srgb(0.95, 0.85, 0.10), image: "characters/bart.png" },
        MarbleConfig { nickname: "Naruto",   color: Color::srgb(0.95, 0.45, 0.05), image: "characters/naruto.png" },
        MarbleConfig { nickname: "Finn",     color: Color::srgb(0.30, 0.65, 0.90), image: "characters/finn.png" },
        MarbleConfig { nickname: "Rick",     color: Color::srgb(0.55, 0.80, 0.55), image: "characters/rick.png" },
        MarbleConfig { nickname: "Shrek",    color: Color::srgb(0.40, 0.65, 0.20), image: "characters/shrek.png" },
        MarbleConfig { nickname: "Vegeta",   color: Color::srgb(0.55, 0.10, 0.80), image: "characters/vegeta.png" },
        MarbleConfig { nickname: "Patricio", color: Color::srgb(0.95, 0.55, 0.70), image: "characters/patricio.png" },
        MarbleConfig { nickname: "Gumball",  color: Color::srgb(0.30, 0.55, 0.90), image: "characters/gumball.png" },
    ];
    // Grid 3×3: fila superior primero, de izquierda a derecha
    let grid: [(f32, f32); 9] = [
        (-0.25, 4.2),
        (0.0, 4.2),
        (0.25, 4.2),
        (-0.25, 3.9),
        (0.0, 3.9),
        (0.25, 3.9),
        (-0.25, 3.6),
        (0.0, 3.6),
        (0.25, 3.6),
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

        let quad_size = radius * 2.0;
        let bg_handle:  Handle<Image> = asset_server.load("characters/circle_white.png");
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
    let cam_y_offset = 0.8;

    let Some(lowest_y) = marbles.iter().map(|t| t.translation.y).reduce(f32::min) else {
        return;
    };
    let Ok(mut cam) = camera.single_mut() else {
        return;
    };

    cam.translation = Vec3::new(0.0, lowest_y + cam_y_offset, cam_z);
    *cam = cam.looking_at(Vec3::new(0.0, lowest_y, 0.0), Vec3::Y);
}
