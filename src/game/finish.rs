use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rapier_bevy::{
    BodyType, ColliderShape, ObjectDef, SimMode, VisualAppearance, VisualDef, spawn_object,
};

use super::marbles::{Marble, MarbleName};

#[derive(Component)]
pub struct FinishLine;

#[derive(Resource, Default)]
pub struct RaceResult {
    pub finishers: Vec<(Entity, f32)>,
}

pub fn spawn_finish_line(
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    y: f32,
) {
    let entity = spawn_object(
        commands,
        ObjectDef {
            shape: ColliderShape::Box {
                hx: 0.55,
                hy: 0.025,
                hz: crate::UNIT / 4.0,
            },
            position: Vec3::new(0.0, y, 0.0),
            body: BodyType::Static,
            sensor: true,
            visual: Some(VisualDef {
                appearance: VisualAppearance::Color(Color::srgb(1.0, 0.85, 0.1)),
                ..VisualDef::white_matte()
            }),
            ..Default::default()
        },
        mode,
        asset_server,
        meshes,
        materials,
    );
    commands.entity(entity).insert((FinishLine, ActiveEvents::COLLISION_EVENTS));
}

pub fn on_finish_contact(
    mut events: EventReader<CollisionEvent>,
    finish: Query<(), With<FinishLine>>,
    marbles: Query<&MarbleName, With<Marble>>,
    time: Res<Time>,
    mut result: ResMut<RaceResult>,
) {
    for event in events.read() {
        let CollisionEvent::Started(a, b, _) = event else { continue };
        for (sensor, target) in [(*a, *b), (*b, *a)] {
            if !finish.contains(sensor) { continue }
            let Ok(name) = marbles.get(target) else { continue };
            if result.finishers.iter().any(|(e, _)| *e == target) { continue }
            let t = time.elapsed_secs();
            result.finishers.push((target, t));
            let position = result.finishers.len();
            if position == 1 {
                info!("🏁 WINNER: {} at {:.2}s", name.0, t);
            } else {
                info!("   #{} {} at {:.2}s", position, name.0, t);
            }
        }
    }
}
