use bevy::prelude::*;

use crate::game::world::marbles::{Marble, MarbleName};

/// Cuándo termina la carrera: a los N segundos la meta aparece al horizonte.
/// Regla de la partida — la coloca prepare_the_race; el director la lee.
#[derive(Resource)]
pub struct FinishTarget(pub f32);

#[derive(Component)]
pub struct FinishLine;

#[derive(Resource, Default)]
pub struct FinishLineY(pub Option<f32>);

#[derive(Resource, Default)]
pub struct RaceResult {
    pub finishers: Vec<(Entity, f32)>,
}

pub fn spawn_finish_line(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    y: f32,
) {
    let width = 1.1;
    let height = 0.14;
    let z_front = crate::UNIT / 4.0;
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(width, height))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("img/finish.png")),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, y, z_front),
        FinishLine,
    ));
    commands.insert_resource(FinishLineY(Some(y)));
}

pub fn check_finish_crossing(
    finish_y: Res<FinishLineY>,
    marbles: Query<(Entity, &Transform, &MarbleName), With<Marble>>,
    time: Res<Time<Fixed>>,
    mut result: ResMut<RaceResult>,
) {
    let Some(y) = finish_y.0 else { return };
    for (entity, transform, name) in &marbles {
        if transform.translation.y > y {
            continue;
        }
        if result.finishers.iter().any(|(e, _)| *e == entity) {
            continue;
        }
        let t = time.elapsed_secs();
        result.finishers.push((entity, t));
        let position = result.finishers.len();
        if position == 1 {
            info!("🏁 WINNER: {} at {:.2}s", name.0, t);
        } else {
            info!("   #{} {} at {:.2}s", position, name.0, t);
        }
    }
}
