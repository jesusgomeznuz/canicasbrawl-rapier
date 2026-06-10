use bevy::prelude::*;

use super::finish::RaceResult;
use super::marbles::Marble;

/// Fuente única de verdad de quién va ganando la carrera ahora mismo. La corona y el
/// segmento de voz solo leen esto; ninguno recalcula el líder por su cuenta.
///
/// El líder es estrictamente la canica más baja, sin suavización: los rebases
/// relámpago (≤0.45 s medidos) los absorbe la limpieza de fantasmas del voice
/// tracker al guardar, no este resource.
#[derive(Resource, Default)]
pub struct RaceLeader {
    pub marble: Option<Entity>,
    locked: bool,
}

pub fn update_race_leader(
    marbles: Query<(Entity, &Transform), With<Marble>>,
    result: Res<RaceResult>,
    mut leader: ResMut<RaceLeader>,
) {
    if leader.locked {
        return;
    }

    // El primero en cruzar la meta se queda con el liderazgo para siempre.
    if let Some((winner, _)) = result.finishers.first() {
        leader.marble = Some(*winner);
        leader.locked = true;
        return;
    }

    if let Some((front_runner, _)) = lowest_marble(&marbles) {
        leader.marble = Some(front_runner);
    }
}

fn lowest_marble(marbles: &Query<(Entity, &Transform), With<Marble>>) -> Option<(Entity, f32)> {
    marbles
        .iter()
        .map(|(e, t)| (e, t.translation.y))
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
}
