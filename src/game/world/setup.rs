use bevy::prelude::*;
use bevy_rapier3d::plugin::context::DefaultRapierContext;
use bevy_rapier3d::prelude::*;
use rapier_bevy::{AssetsLoading, SimulationMode};

use super::level_generation::{FinishTarget, LevelGen};
use super::structures::spawn_wall_segment;
use crate::game::background::palette::ColorPalette;

pub fn setup(
    mut commands: Commands,
    mode: Res<SimulationMode>,
    finish_target: Res<FinishTarget>,
    roster: Res<crate::game::roster::Roster>,
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
    commands.insert_resource(LevelGen::new(first_module_top, finish_target.0));

    crate::game::marbles::spawn_marbles(
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
