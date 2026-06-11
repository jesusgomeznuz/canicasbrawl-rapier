use super::background::ColorPalette;
use super::level::{ModuleData, WorldObject, load_module};
use super::marbles::Marble;
use bevy::prelude::*;
use bevy_rapier3d::plugin::context::DefaultRapierContext;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use rand::RngCore;
use rapier_bevy::BakeEvents;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rapier_bevy::{
    AssetsLoading, BodyType, ColliderShape, ObjectDef, SimMode, VisualAppearance, VisualDef,
    spawn_object,
};

#[derive(Resource)]
pub struct LevelSeed(pub u64);

/// Segundo de simulación en que se cierra el nivel con la finish line. Se deriva de la
/// duración del video (`--record`) menos un margen, para que la meta se cruce unos
/// segundos antes del final. Ver [`decide_level_action`].
#[derive(Resource)]
pub struct FinishTarget(pub f32);

/// Reemplaza `VisualDef::white_matte()` con el color tintado de la paleta activa.
fn tinted_white(color: Color) -> VisualDef {
    VisualDef {
        appearance: VisualAppearance::Color(color),
        roughness: 0.85,
        ..Default::default()
    }
}

pub fn setup(
    mut commands: Commands,
    mode: Res<SimMode>,
    seed: Res<LevelSeed>,
    finish_target: Res<FinishTarget>,
    roster: Res<super::marbles::Roster>,
    palette: Res<ColorPalette>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut assets_loading: Option<ResMut<AssetsLoading>>,
) {
    let spawn_y = 0.0;
    let top_margin = 0.6;
    let walls_top = 2.0;
    let first_module_top = spawn_y - top_margin;

    // Paredes por tramo (sin border_radius para que no se vean las orillas/junturas);
    // cada módulo añade su tramo en generate_level. Un mesh único de toda la columna
    // reventaba el render por las sombras, así que se generan acotados al nivel visible.
    let obstacle_color = palette.obstacle_color();
    spawn_wall_segment(
        walls_top,
        first_module_top,
        obstacle_color,
        &mut commands,
        &mode,
        &asset_server,
        &mut meshes,
        &mut materials,
    );
    commands.insert_resource(LevelGen::new(seed.0, first_module_top, finish_target.0));

    super::marbles::spawn_marbles(
        &mut commands,
        &mode,
        &asset_server,
        &mut meshes,
        &mut materials,
        &roster.0,
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

/// Estado de la generación incremental del nivel. El nivel ya no se arma entero en
/// Startup: `generate_level` añade módulos en runtime conforme baja la canica líder,
/// y cierra con la finish line cuando el reloj se acerca al objetivo.
#[derive(Resource)]
pub struct LevelGen {
    rng: SmallRng,
    next_top: f32,
    last_module: Option<&'static str>,
    modules_spawned: u32,
    finish_spawned: bool,
    target_secs: f32,
}

impl LevelGen {
    fn new(seed: u64, first_module_top: f32, target_secs: f32) -> Self {
        Self {
            rng: SmallRng::seed_from_u64(seed),
            next_top: first_module_top,
            last_module: None,
            modules_spawned: 0,
            finish_spawned: false,
            target_secs,
        }
    }
}

enum LevelGenerationAction {
    GenerateModule,
    SpawnFinishLine,
    DoNothing,
}

fn decide_level_action(
    level_gen: &LevelGen,
    leader_y: f32,
    elapsed_secs: f32,
) -> LevelGenerationAction {
    if level_gen.finish_spawned {
        return LevelGenerationAction::DoNothing;
    }

    // Temprano mejor que nunca: en cuanto se pasa el objetivo de tiempo se cierra el
    // nivel poniendo la meta en el horizonte actual (el fondo de lo generado, así queda
    // limpia al final) y se deja de generar. La líder cae el último tramo y la cruza.
    if elapsed_secs >= level_gen.target_secs {
        return LevelGenerationAction::SpawnFinishLine;
    }

    // Mientras haya tiempo, se mantiene pista por delante de la líder.
    let trigger_distance = 3.0;
    if leader_y - level_gen.next_top < trigger_distance {
        return LevelGenerationAction::GenerateModule;
    }
    LevelGenerationAction::DoNothing
}

pub fn generate_level(
    sim_time: Res<Time<Fixed>>,
    marbles: Query<&Transform, With<Marble>>,
    mut level_gen: ResMut<LevelGen>,
    mut commands: Commands,
    mode: Res<SimMode>,
    palette: Res<ColorPalette>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut bake_events: Option<ResMut<BakeEvents>>,
) {
    let Some(leader_y) = marbles.iter().map(|t| t.translation.y).reduce(f32::min) else {
        return;
    };
    let obstacle_color = palette.obstacle_color();
    match decide_level_action(&level_gen, leader_y, sim_time.elapsed_secs()) {
        LevelGenerationAction::GenerateModule => {
            let name = pick_module(&mut level_gen);
            let top = level_gen.next_top;
            // Cada módulo recibe su propio seed: las variantes dejan de depender del
            // stream global, así el replay reconstruye el módulo exacto desde el evento.
            let module_seed = level_gen.rng.next_u64();
            let bottom = spawn_level_module(
                name,
                top,
                module_seed,
                obstacle_color,
                &mut commands,
                &mode,
                &asset_server,
                &mut meshes,
                &mut materials,
            );
            if let Some(ev) = bake_events.as_deref_mut() {
                ev.0.push(format!("module {name} {top} {module_seed}"));
            }
            level_gen.next_top = bottom;
            level_gen.last_module = Some(name);
            level_gen.modules_spawned += 1;
        }
        LevelGenerationAction::SpawnFinishLine => {
            // La meta cierra el nivel en el horizonte (reemplaza el módulo que tocaría):
            // la líder, ya cerca de él, cae los últimos metros y la cruza. Se deja un
            // hueco antes de la meta (aire tras el último módulo) y otro debajo de ella
            // (para ver caer la cola entre la meta y el suelo).
            close_level_with_finish(
                level_gen.next_top,
                obstacle_color,
                &mut commands,
                &mode,
                &asset_server,
                &mut meshes,
                &mut materials,
            );
            if let Some(ev) = bake_events.as_deref_mut() {
                ev.0.push(format!("finish {}", level_gen.next_top));
            }
            level_gen.finish_spawned = true;
        }
        LevelGenerationAction::DoNothing => {}
    }
}

/// Spawnea un módulo completo (obstáculos + variantes + tramo de pared) de forma
/// reproducible: `module_seed` aísla el RNG de las variantes, así el mismo evento
/// horneado reconstruye el módulo idéntico en replay. Devuelve el nuevo horizonte.
pub(crate) fn spawn_level_module(
    name: &str,
    top: f32,
    module_seed: u64,
    obstacle_color: Color,
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> f32 {
    let module_gap = 0.1;
    let mut rng = SmallRng::seed_from_u64(module_seed);
    let bottom = spawn_module(
        name, top, obstacle_color, &mut rng,
        commands, mode, asset_server, meshes, materials,
    ) - module_gap;
    spawn_wall_segment(top, bottom, obstacle_color, commands, mode, asset_server, meshes, materials);
    bottom
}

/// Cierra el nivel en el horizonte: pared final, suelo y la meta, con aire antes de
/// la meta y debajo de ella (para ver caer la cola de canicas).
pub(crate) fn close_level_with_finish(
    next_top: f32,
    obstacle_color: Color,
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let gap_module_to_finish = 0.4;
    let gap_finish_to_floor = 1.0;
    let finish_y = next_top - gap_module_to_finish;
    let floor_y = finish_y - gap_finish_to_floor;
    spawn_wall_segment(next_top, floor_y, obstacle_color, commands, mode, asset_server, meshes, materials);
    spawn_floor(commands, mode, asset_server, meshes, materials, floor_y, obstacle_color);
    super::finish::spawn_finish_line(commands, meshes, materials, asset_server, finish_y);
}

/// Elige el siguiente módulo del pool ponderado, sin repetir el anterior. El primer
/// módulo nunca es `spheres` (hoy atora las canicas al arrancar; regla de generación,
/// no parche de la física del módulo).
fn pick_module(level_gen: &mut LevelGen) -> &'static str {
    let pool: &[(&'static str, u32)] = &[
        ("crosses", 3),
        ("zigzag", 3),
        ("spheres", 3),
        ("toruses", 1),
        ("bouncy_walls", 1),
    ];
    let weighted: Vec<&'static str> = pool
        .iter()
        .flat_map(|(name, weight)| std::iter::repeat(*name).take(*weight as usize))
        .collect();
    let is_first_module = level_gen.modules_spawned == 0;
    loop {
        let pick = weighted[level_gen.rng.gen_range(0..weighted.len())];
        let repeats_last = Some(pick) == level_gen.last_module;
        let spheres_as_first = is_first_module && pick == "spheres";
        if !repeats_last && !spheres_as_first {
            return pick;
        }
    }
}

fn spawn_module(
    name: &str,
    level_top: f32,
    obstacle_color: Color,
    rng: &mut SmallRng,
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
                            ..tinted_white(obstacle_color)
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
                        visual: Some(tinted_white(obstacle_color)),
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
                        visual: Some(tinted_white(obstacle_color)),
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
                    mode,
                    asset_server,
                    meshes,
                    materials,
                );
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
                    mode,
                    asset_server,
                    meshes,
                    materials,
                );
                attach_effect_marker(commands, sensor, variant);
                spawn_spinning_icon(commands, asset_server, sensor, variant);
            }
        }
    }
    level_top - trimmed_height
}

fn all_sensor_variants() -> &'static [&'static str] {
    &["freeze", "shrink", "swap"]
}

/// Probabilidad de que un EffectSlot sea ignorado completamente (reduce frecuencia general).
const EFFECT_SLOT_SKIP_CHANCE: f32 = 0.40;

/// Pesos por variante: freeze 4×, swap 3×, shrink 1× para contrarrestar la ventaja de shrink.
fn default_effect_weights() -> &'static [(&'static str, u32)] {
    &[("freeze", 4), ("swap", 3), ("shrink", 1)]
}

fn resolve_slot_variant<'a>(
    options: &'a [String],
    world_y: f32,
    rng: &mut SmallRng,
) -> Option<&'a str> {
    if rng.gen_range(0.0_f32..1.0) < EFFECT_SLOT_SKIP_CHANCE {
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
        "freeze" => {
            commands.entity(sensor).insert(super::effects::FreezeEffect);
        }
        "shrink" => {
            commands.entity(sensor).insert(super::effects::ShrinkEffect);
        }
        "swap" => {
            commands.entity(sensor).insert(super::effects::SwapEffect);
        }
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
    let icon = commands
        .spawn((
            SceneRoot(scene),
            Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(scale)),
            super::effects::SpinningIcon { axis, speed },
        ))
        .id();
    commands.entity(sensor).add_child(icon);
}

fn icon_tuning_for(variant: &str) -> (Vec3, f32, f32) {
    match variant {
        "freeze" => (Vec3::Y, 1.0, 13.5),
        "shrink" => (Vec3::Y, 1.0, 0.046),
        "swap" => (Vec3::Z, 2.0, 0.25),
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
    obstacle_color: Color,
) {
    // Grueso a propósito: las canicas llegan a ~20 m/s y un suelo delgado lo atraviesan
    // (tunneling) aunque tengan CCD. Con varios metros de grosor se quedan dentro varios
    // ticks y el solver las frena. El borde superior queda en floor_y.
    let hy = 3.0_f32;
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

fn spawn_wall_segment(
    top: f32,
    bottom: f32,
    obstacle_color: Color,
    commands: &mut Commands,
    mode: &SimMode,
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

    // Sin border_radius: las esquinas redondeadas de cada tramo eran las orillas que
    // delataban las junturas. Rectos y contiguos se ven como una pared continua.
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
