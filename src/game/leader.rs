use bevy::prelude::*;

use super::finish::RaceResult;
use super::marbles::Marble;

/// Fuente única de verdad de quién va ganando la carrera ahora mismo. La corona y el
/// segmento de voz solo leen esto; ninguno recalcula el líder por su cuenta.
#[derive(Resource, Default)]
pub struct RaceLeader {
    pub marble: Option<Entity>,
    locked: bool,
    pending_challenger: Option<Entity>,
    challenger_since: f32,
}

impl RaceLeader {
    fn clear_challenger(&mut self) {
        self.pending_challenger = None;
    }

    /// True cuando el retador lleva sosteniendo la delantera al menos `min_secs`.
    /// La primera vez que aparece solo arranca su cronómetro y devuelve false.
    fn challenge_is_sustained(&mut self, challenger: Entity, now: f32, min_secs: f32) -> bool {
        if self.pending_challenger != Some(challenger) {
            self.pending_challenger = Some(challenger);
            self.challenger_since = now;
            return false;
        }
        now - self.challenger_since >= min_secs
    }
}

pub fn update_race_leader(
    sim_time: Res<Time<Fixed>>,
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
        leader.clear_challenger();
        return;
    }

    // Un retador solo le quita el liderazgo a la canica de adelante si le saca este
    // margen y lo sostiene este tiempo. Evita el parpadeo del arranque, cuando todas
    // caen casi a la misma altura.
    let lead_margin = 0.2;
    let min_lead_secs = 0.5;
    let now = sim_time.elapsed_secs();

    let Some((front_runner, front_y)) = lowest_marble(&marbles) else {
        return;
    };
    let Some(current) = leader.marble else {
        leader.marble = Some(front_runner);
        return;
    };
    if front_runner == current {
        leader.clear_challenger();
        return;
    }

    let current_y = marbles.get(current).map(|(_, t)| t.translation.y);
    let leads_clearly = current_y.is_ok_and(|cy| cy - front_y >= lead_margin);
    if !leads_clearly {
        leader.clear_challenger();
        return;
    }

    if leader.challenge_is_sustained(front_runner, now, min_lead_secs) {
        leader.marble = Some(front_runner);
        leader.clear_challenger();
    }
}

fn lowest_marble(marbles: &Query<(Entity, &Transform), With<Marble>>) -> Option<(Entity, f32)> {
    marbles
        .iter()
        .map(|(e, t)| (e, t.translation.y))
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
}
