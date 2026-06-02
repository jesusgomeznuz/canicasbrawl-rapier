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
    let Ok(name) = names.get(marble) else { return };
    if tracker.leader_changed(&name.0) {
        let now = sim_time.elapsed_secs();
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
        write_voice_tracker_json(&tracker.segments);
    }
}

fn write_voice_tracker_json(segments: &[RaceSegment]) {
    let json = serde_json::to_string_pretty(&VoiceTrackerJson { segments }).unwrap();
    fs::create_dir_all("outputs").ok();
    fs::write("outputs/voice_tracker.json", json).expect("failed to write voice_tracker.json");
    info!("voice_tracker.json saved — {} segments", segments.len());
}
