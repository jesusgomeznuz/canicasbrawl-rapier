use bevy::prelude::*;

pub mod voice_tracker;

/// El observador de producción: el tracker de voz que alimenta al pipeline.
pub fn setup_production(app: &mut App) {
    app.insert_resource(voice_tracker::VoiceTracker::default());
}
