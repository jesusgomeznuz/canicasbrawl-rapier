use bevy::prelude::*;
use crate::marbles::{Marble, MarbleLabel};

pub fn camera_follows_lowest_marble(
    marbles: Query<&Transform, With<Marble>>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<Marble>)>,
) {
    let camera_z = 2.5;
    let camera_y_offset = 0.2;

    let Some(lowest_y) = marbles.iter().map(|t| t.translation.y).reduce(f32::min) else { return };
    let Ok(mut cam) = camera.single_mut() else { return };

    cam.translation = Vec3::new(0.0, lowest_y + camera_y_offset, camera_z);
    cam.rotation = Quat::IDENTITY;
}

pub fn update_marble_labels(
    marbles: Query<&GlobalTransform, With<Marble>>,
    mut labels: Query<(&mut Transform, &mut TextFont, &MarbleLabel)>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let Ok((camera, cam_gt)) = camera_q.single() else { return };
    let Some(viewport) = camera.logical_viewport_size() else { return };
    // 2.2% del alto del viewport → tamaño relativo igual en ventana y en --record
    let font_size = viewport.y * 0.022;
    for (mut transform, mut font, MarbleLabel(marble_entity)) in &mut labels {
        font.font_size = font_size;
        let Ok(marble_gt) = marbles.get(*marble_entity) else { continue };
        let above = marble_gt.translation() + Vec3::Y * 0.13;
        if let Ok(screen_pos) = camera.world_to_viewport(cam_gt, above) {
            transform.translation.x = screen_pos.x - viewport.x / 2.0;
            transform.translation.y = viewport.y / 2.0 - screen_pos.y;
        }
    }
}
