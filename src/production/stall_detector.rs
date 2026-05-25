use bevy::app::AppExit;
use bevy::prelude::*;

use crate::game::marbles::Marble;

#[derive(Resource, Default)]
pub struct StallDetector {
    last_lowest_y: Option<f32>,
    last_progress_secs: Option<f32>,
}

pub fn detect_stall(
    time: Res<Time>,
    marbles: Query<&Transform, With<Marble>>,
    mut detector: ResMut<StallDetector>,
    mut exit: EventWriter<AppExit>,
) {
    let stall_timeout = 5.0_f32;
    let progress_epsilon = 0.05_f32;

    let Some(lowest_y) = marbles.iter().map(|t| t.translation.y).reduce(f32::min) else { return };
    let now = time.elapsed_secs();

    let made_progress = match detector.last_lowest_y {
        Some(previous) => lowest_y < previous - progress_epsilon,
        None => true,
    };

    if made_progress {
        detector.last_lowest_y = Some(lowest_y);
        detector.last_progress_secs = Some(now);
        return;
    }

    let Some(last_progress) = detector.last_progress_secs else { return };
    if now - last_progress > stall_timeout {
        warn!("Stall detectado — sin progreso en {:.1}s, cerrando", stall_timeout);
        exit.write(AppExit::Success);
    }
}
