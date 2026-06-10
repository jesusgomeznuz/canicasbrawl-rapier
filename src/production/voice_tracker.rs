use bevy::app::AppExit;
use bevy::prelude::*;
use crate::game::leader::RaceLeader;
use crate::game::marbles::{Marble, MarbleName};
use serde::Serialize;
use std::fs;

#[derive(Serialize)]
pub struct RaceSegment {
    pub leader: String,
    pub start_secs: f32,
    pub end_secs: f32,
}

#[derive(Resource, Default)]
pub struct VoiceTracker {
    segments: Vec<RaceSegment>,
    current_leader: Option<String>,
    current_leader_start: f32,
}

impl VoiceTracker {
    fn leader_changed(&self, new_leader: &str) -> bool {
        self.current_leader.as_deref() != Some(new_leader)
    }

    fn open_new_segment(&mut self, leader: String, video_secs: f32) {
        self.current_leader = Some(leader);
        self.current_leader_start = video_secs;
    }

    fn close_current_segment(&mut self, video_secs: f32) {
        let Some(leader) = self.current_leader.take() else { return };
        self.segments.push(RaceSegment {
            leader,
            start_secs: self.current_leader_start,
            end_secs: video_secs,
        });
    }
}

#[derive(Serialize)]
struct VoiceTrackerJson<'a> {
    segments: &'a [RaceSegment],
}

/// Abre un segmento de voz nuevo cada vez que cambia el líder de la carrera. Quién es
/// el líder lo decide [`RaceLeader`]; aquí solo se traduce a segmentos con timestamps.
pub fn track_race_leader(
    leader: Res<RaceLeader>,
    names: Query<&MarbleName, With<Marble>>,
    sim_time: Res<Time<Fixed>>,
    mut tracker: ResMut<VoiceTracker>,
) {
    let Some(marble) = leader.marble else { return };
    let name = match names.get(marble) {
        Ok(n) => n,
        Err(_) => {
            warn!("track_race_leader: leader entity {:?} no tiene MarbleName (t={:.3})", marble, sim_time.elapsed_secs());
            return;
        }
    };
    if tracker.leader_changed(&name.0) {
        let now = sim_time.elapsed_secs();
        info!("track_race_leader: cambio → {} @ t={:.3}", name.0, now);
        tracker.close_current_segment(now);
        tracker.open_new_segment(name.0.clone(), now);
    }
}

pub fn save_voice_tracker_on_exit(
    mut exit_events: EventReader<AppExit>,
    sim_time: Res<Time<Fixed>>,
    mut tracker: ResMut<VoiceTracker>,
) {
    for _ in exit_events.read() {
        tracker.close_current_segment(sim_time.elapsed_secs());
        let segments = clean_ghost_segments(std::mem::take(&mut tracker.segments), 0.09, 2.0);
        write_voice_tracker_json(&segments);
    }
}

/// Solo durante el arranque (`intro_secs`): un líder que no sostiene la delantera al
/// menos `min_secs` no llega a cantar — su tiempo se lo queda quien venía cantando.
/// Ahí las canicas caen en clúster y el "más bajo" parpadea por jitter del solver
/// (clúster medido de 1-5 steps, 0.017–0.083 s; 0.09 lo corta). Pasado el arranque
/// no se filtra nada: a media carrera hasta el mínimo rebase cuenta para la voz.
fn clean_ghost_segments(segments: Vec<RaceSegment>, min_secs: f32, intro_secs: f32) -> Vec<RaceSegment> {
    let mut cleaned: Vec<RaceSegment> = Vec::new();
    let mut unclaimed_start: Option<f32> = None; // fantasmas del arranque, antes del primer líder real

    for seg in segments {
        let in_intro = seg.start_secs < intro_secs;
        let is_ghost = in_intro && seg.end_secs - seg.start_secs < min_secs;

        if is_ghost {
            match cleaned.last_mut() {
                Some(previous) => previous.end_secs = seg.end_secs,
                None => { unclaimed_start.get_or_insert(seg.start_secs); }
            }
            continue;
        }

        let start_secs = unclaimed_start.take().unwrap_or(seg.start_secs);
        match cleaned.last_mut() {
            Some(previous) if previous.leader == seg.leader => previous.end_secs = seg.end_secs,
            _ => cleaned.push(RaceSegment { start_secs, ..seg }),
        }
    }

    cleaned
}

fn write_voice_tracker_json(segments: &[RaceSegment]) {
    let json = serde_json::to_string_pretty(&VoiceTrackerJson { segments }).unwrap();
    fs::create_dir_all("outputs").ok();
    fs::write("outputs/voice_tracker.json", json).expect("failed to write voice_tracker.json");
    info!("voice_tracker.json saved — {} segments", segments.len());
}
