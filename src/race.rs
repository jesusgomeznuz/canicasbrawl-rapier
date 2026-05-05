use bevy::app::AppExit;
use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use crate::marbles::{Marble, MarbleName};
use serde::Serialize;
use std::fs;

const VIDEO_FPS: f32 = 60.0;

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

pub fn track_race_leader(
    marbles: Query<(&Transform, &MarbleName), With<Marble>>,
    frame: Res<FrameCount>,
    mut tracker: ResMut<VoiceTracker>,
) {
    let Some(leader) = find_leader(&marbles) else { return };
    if tracker.leader_changed(leader) {
        let video_secs = frame.0 as f32 / VIDEO_FPS;
        tracker.close_current_segment(video_secs);
        tracker.open_new_segment(leader.to_string(), video_secs);
    }
}

fn find_leader<'w, 's>(
    marbles: &Query<'w, 's, (&Transform, &MarbleName), With<Marble>>,
) -> Option<&'static str> {
    marbles
        .iter()
        .min_by(|(a, _), (b, _)| a.translation.y.partial_cmp(&b.translation.y).unwrap())
        .map(|(_, name)| name.0)
}

pub fn save_voice_tracker_on_exit(
    mut exit_events: EventReader<AppExit>,
    frame: Res<FrameCount>,
    mut tracker: ResMut<VoiceTracker>,
) {
    for _ in exit_events.read() {
        tracker.close_current_segment(frame.0 as f32 / VIDEO_FPS);
        write_voice_tracker_json(&tracker.segments);
    }
}

fn write_voice_tracker_json(segments: &[RaceSegment]) {
    let json = serde_json::to_string_pretty(&VoiceTrackerJson { segments }).unwrap();
    fs::create_dir_all("outputs").ok();
    fs::write("outputs/voice_tracker.json", json).expect("failed to write voice_tracker.json");
    info!("voice_tracker.json saved — {} segments", segments.len());
}
