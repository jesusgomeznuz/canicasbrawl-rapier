use bevy::prelude::*;

pub mod voice_tracker;

/// Producción enciende su micrófono: el tracker de voz que alimenta al pipeline.
pub fn initialize_voice_tracker(app: &mut App) {
    app.insert_resource(voice_tracker::VoiceTracker::default());
}
