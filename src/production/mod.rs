use bevy::prelude::*;

pub mod stall_detector;
pub mod voice_tracker;

/// Los observadores de producción: el tracker de voz y el vigía de cuelgues.
pub fn setup_production(app: &mut App) {
    app.insert_resource(voice_tracker::VoiceTracker::default())
        .insert_resource(stall_detector::StallDetector::default());
}
