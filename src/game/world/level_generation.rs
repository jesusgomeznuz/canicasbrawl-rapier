use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rapier_bevy::Dice;

use super::modules::{ModuleSpan, module_height, spawn_module};
use super::structures::{spawn_floor, spawn_wall_segment};
use crate::game::marbles::Marble;
use crate::game::race_events::RaceEvent;

#[derive(Resource)]
pub struct LevelSeed(pub u64);

#[derive(Resource)]
pub struct FinishTarget(pub f32);

#[derive(Resource)]
pub struct LevelGen {
    next_top: f32,
    last_module: Option<&'static str>,
    modules_spawned: u32,
    finish_spawned: bool,
    target_secs: f32,
}

impl LevelGen {
    pub fn new(first_module_top: f32, target_secs: f32) -> Self {
        Self {
            next_top: first_module_top,
            last_module: None,
            modules_spawned: 0,
            finish_spawned: false,
            target_secs,
        }
    }
}

pub fn generate_level(
    sim_time: Res<Time<Fixed>>,
    marbles: Query<&Transform, With<Marble>>,
    mut level_gen: ResMut<LevelGen>,
    mut dice: ResMut<Dice>,
    mut events: EventWriter<RaceEvent>,
) {
    let Some(leader_y) = marbles.iter().map(|t| t.translation.y).reduce(f32::min) else {
        return;
    };
    match decide_level_action(&level_gen, leader_y, sim_time.elapsed_secs()) {
        LevelGenerationAction::GenerateModule => {
            let name = pick_module(&level_gen, &mut dice);
            let top = level_gen.next_top;
            let module_seed = dice.next_u64();
            events.write(RaceEvent::Module {
                name: name.to_string(),
                top,
                seed: module_seed,
            });
            level_gen.next_top = top - module_height(name);
            level_gen.last_module = Some(name);
            level_gen.modules_spawned += 1;
        }
        LevelGenerationAction::SpawnFinishLine => {
            events.write(RaceEvent::Finish {
                top: level_gen.next_top,
            });
            level_gen.finish_spawned = true;
        }
        LevelGenerationAction::DoNothing => {}
    }
}

pub fn spawn_level_module(
    name: &str,
    top: f32,
    module_seed: u64,
    obstacle_color: Color,
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> f32 {
    let module_gap = 0.1;
    let bottom = spawn_module(
        name,
        top,
        obstacle_color,
        module_seed,
        commands,
        asset_server,
        meshes,
        materials,
    ) - module_gap;
    spawn_wall_segment(
        top,
        bottom,
        obstacle_color,
        commands,
        asset_server,
        meshes,
        materials,
    );
    bottom
}

pub fn close_level_with_finish(
    next_top: f32,
    obstacle_color: Color,
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let gap_module_to_finish = 0.4;
    let gap_finish_to_floor = 1.0;
    let finish_y = next_top - gap_module_to_finish;
    let floor_y = finish_y - gap_finish_to_floor;
    spawn_wall_segment(
        next_top,
        floor_y,
        obstacle_color,
        commands,
        asset_server,
        meshes,
        materials,
    );
    spawn_floor(
        commands,
        asset_server,
        meshes,
        materials,
        floor_y,
        obstacle_color,
    );
    crate::game::finish::spawn_finish_line(commands, meshes, materials, asset_server, finish_y);
}

pub fn disable_modules_above_screen(
    camera: Query<(&Projection, &GlobalTransform), With<Camera3d>>,
    modules: Query<(Entity, &ModuleSpan), Without<ColliderDisabled>>,
    mut commands: Commands,
) {
    let Ok((projection, camera_transform)) = camera.single() else {
        return;
    };
    let exit_margin = 0.5;
    for (entity, span) in &modules {
        if crate::game::camera::world_y_above_screen(
            span.bottom,
            exit_margin,
            projection,
            camera_transform,
        ) {
            commands.entity(entity).insert(ColliderDisabled);
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

    let race_time_is_up = elapsed_secs >= level_gen.target_secs;
    if race_time_is_up {
        return LevelGenerationAction::SpawnFinishLine;
    }

    let trigger_distance = 3.0;
    let leader_is_near_the_horizon = leader_y - level_gen.next_top < trigger_distance;
    if leader_is_near_the_horizon {
        return LevelGenerationAction::GenerateModule;
    }
    LevelGenerationAction::DoNothing
}

fn pick_module(level_gen: &LevelGen, dice: &mut Dice) -> &'static str {
    let pool: &[(&'static str, u32)] = &[
        ("crosses", 5),
        ("zigzag", 5),
        ("spheres", 5),
        ("diamonds", 2),
        ("mini_zigzag", 3),
        ("more_stones", 2),
        ("hex_stones", 3),
        ("stones", 0),
        ("bars", 0),
        ("toruses", 5),
        ("bouncy_walls", 5),
    ];
    let weighted: Vec<&'static str> = pool
        .iter()
        .flat_map(|(name, weight)| std::iter::repeat(*name).take(*weight as usize))
        .collect();
    let is_first_module = level_gen.modules_spawned == 0;
    loop {
        let pick = weighted[dice.roll(weighted.len())];
        let repeats_last = Some(pick) == level_gen.last_module;
        let spheres_as_first = is_first_module && pick == "spheres";
        if !repeats_last && !spheres_as_first {
            return pick;
        }
    }
}
