use bevy::app::AppExit;
use bevy::prelude::*;
use std::time::Instant;

#[derive(Resource, Default)]
pub struct StallDetector {
    last_frame_at: Option<Instant>,
    consecutive_slow_frames: u32,
}

pub fn detect_stall(
    mut detector: ResMut<StallDetector>,
    mut exit: EventWriter<AppExit>,
) {
    let slow_frame_threshold_secs = 0.5_f32;
    let max_consecutive_slow_frames = 5_u32;

    // Wall-time real entre frames, medido con Instant: en --record el tiempo virtual
    // avanza en pasos fijos (ManualDuration), así que Time<Real> no sirve para esto.
    let now = Instant::now();
    let Some(last_frame_at) = detector.last_frame_at.replace(now) else { return };
    let frame_secs = now.duration_since(last_frame_at).as_secs_f32();
    let frame_is_pathological = frame_secs > slow_frame_threshold_secs;

    if frame_is_pathological {
        detector.consecutive_slow_frames += 1;
    } else {
        detector.consecutive_slow_frames = 0;
        return;
    }

    if detector.consecutive_slow_frames >= max_consecutive_slow_frames {
        warn!(
            "Solver atascado — {} frames seguidos > {:.0}ms reales, cerrando para no trabar la máquina",
            detector.consecutive_slow_frames,
            slow_frame_threshold_secs * 1000.0,
        );
        exit.write(AppExit::Success);
    }
}
