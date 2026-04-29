use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rapier_bevy::modes::{Mode, parse_mode, record_duration};
use rapier_bevy::{
    BodyType, ColliderShape, GraphicsPlugin, LockedAxes, ObjectDef, PhysicsStatsPlugin,
    RecordPlugin, SimMode, VisualAppearance, VisualDef, spawn_object,
};

// 1 unit = 1 metro, gravedad -9.81 m/s².
// Canicas de radio 0.08 m — "game scale" que llena visualmente el arena de 1.1 m de ancho.

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
        app.add_plugins(DefaultPlugins)
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(PhysicsStatsPlugin);
    }

    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(GraphicsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_follows_lowest_marble, log_marble_physics))
        .insert_resource(mode)
        .insert_resource(PhysicsLogger {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        });

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

#[derive(Resource)]
struct PhysicsLogger {
    timer: Timer,
}

struct MarbleConfig {
    nickname: &'static str,
    color: Color,
}

fn setup(
    mut commands: Commands,
    mode: Res<SimMode>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_floor(
        &mut commands,
        &mode,
        &asset_server,
        &mut meshes,
        &mut materials,
    );
    spawn_side_walls(
        &mut commands,
        &mode,
        &asset_server,
        &mut meshes,
        &mut materials,
    );
    spawn_zigzag_platforms(
        &mut commands,
        &mode,
        &asset_server,
        &mut meshes,
        &mut materials,
    );
    spawn_marbles(
        &mut commands,
        &mode,
        &asset_server,
        &mut meshes,
        &mut materials,
    );
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
                hz: 0.3,
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
    let arena_height = 4.5;
    let wall_shape = ColliderShape::Box {
        hx: 0.05,
        hy: arena_height / 2.0,
        hz: 0.3,
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
        (-0.1_f32, 2.8_f32, -tilt),
        (0.1, 2.0, tilt),
        (-0.1, 1.2, -tilt),
        (0.1, 0.4, tilt),
    ];

    for (cx, y, tilt_z) in platforms {
        spawn_object(
            commands,
            ObjectDef {
                shape: ColliderShape::Box {
                    hx: 0.50,
                    hy: 0.05,
                    hz: 0.3,
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
) {
    let marbles: &[MarbleConfig] = &[
        MarbleConfig {
            nickname: "Rojo",
            color: Color::srgb(0.90, 0.15, 0.15),
        },
        MarbleConfig {
            nickname: "Azul",
            color: Color::srgb(0.15, 0.35, 0.90),
        },
        MarbleConfig {
            nickname: "Verde",
            color: Color::srgb(0.15, 0.75, 0.20),
        },
        MarbleConfig {
            nickname: "Amarillo",
            color: Color::srgb(0.95, 0.85, 0.10),
        },
        MarbleConfig {
            nickname: "Morado",
            color: Color::srgb(0.55, 0.10, 0.80),
        },
        MarbleConfig {
            nickname: "Naranja",
            color: Color::srgb(0.95, 0.45, 0.05),
        },
        MarbleConfig {
            nickname: "Rosa",
            color: Color::srgb(0.95, 0.40, 0.65),
        },
        MarbleConfig {
            nickname: "Cyan",
            color: Color::srgb(0.10, 0.80, 0.85),
        },
        MarbleConfig {
            nickname: "Blanco",
            color: Color::srgb(0.95, 0.95, 0.95),
        },
    ];
    // Grid 3×3: fila superior primero, de izquierda a derecha
    let grid: [(f32, f32); 9] = [
        (-0.22, 4.1),
        (0.0, 4.1),
        (0.22, 4.1),
        (-0.22, 3.9),
        (0.0, 3.9),
        (0.22, 3.9),
        (-0.22, 3.7),
        (0.0, 3.7),
        (0.22, 3.7),
    ];
    let radius = 0.08;

    for (cfg, (x, y)) in marbles.iter().zip(grid.iter()) {
        let entity = spawn_object(
            commands,
            ObjectDef {
                shape: ColliderShape::Sphere { radius },
                position: Vec3::new(*x, *y, 0.0),
                body: BodyType::Dynamic,
                restitution: Some(0.2),
                friction: Some(0.4),
                linear_damping: Some(0.15),
                angular_damping: Some(0.4),
                ccd: true,
                locked_axes: Some(LockedAxes::TRANSLATION_LOCKED_Z),
                visual: Some(VisualDef {
                    appearance: VisualAppearance::Color(cfg.color),
                    metallic: 0.7,
                    roughness: 0.2,
                }),
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
    }
}

fn log_marble_physics(
    mut logger: ResMut<PhysicsLogger>,
    time: Res<Time>,
    marbles: Query<(&Transform, &Velocity, &Marble)>,
) {
    logger.timer.tick(time.delta());
    if !logger.timer.just_finished() {
        return;
    }

    let t = time.elapsed_secs();
    println!("── {t:.2}s ──────────────────────────────");
    for (transform, velocity, marble) in &marbles {
        let y = transform.translation.y;
        let vy = velocity.linvel.y;
        let speed = velocity.linvel.length();
        println!(
            "  {:>8}  y={:.3}m  vy={:+.2}m/s  speed={:.2}m/s",
            marble.nickname, y, vy, speed
        );
    }
}

fn camera_follows_lowest_marble(
    marbles: Query<&Transform, With<Marble>>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<Marble>)>,
) {
    let cam_z = 2.0;
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
