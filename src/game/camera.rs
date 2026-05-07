use super::marbles::{Marble, MarbleLabel};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use rapier_bevy::OffscreenTarget;

pub fn spawn_camera_and_lights(mut commands: Commands, offscreen: Option<Res<OffscreenTarget>>) {
    let render_target: Option<RenderTarget> = offscreen
        .as_ref()
        .map(|o| RenderTarget::Image(o.image.clone().into()));

    let mut cam3d = Camera::default();
    if let Some(ref t) = render_target {
        cam3d.target = t.clone();
    }
    commands
        .spawn((
            Camera3d::default(),
            cam3d,
            Transform::from_xyz(0.0, 13.0, 22.0).looking_at(Vec3::new(0.0, 12.0, 0.0), Vec3::Y),
        ))
        .with_children(|camera| {
            camera.spawn((
                DirectionalLight {
                    illuminance: 8_000.0,
                    shadows_enabled: true,
                    ..default()
                },
                Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.2, 0.0, 0.0)),
            ));
        });

    // Camera2d renderiza Text2d (nicknames) encima de la escena 3D.
    // ClearColorConfig::None evita que borre el frame 3D ya renderizado.
    let mut cam2d = Camera {
        order: 1,
        clear_color: ClearColorConfig::None,
        ..default()
    };
    if let Some(ref t) = render_target {
        cam2d.target = t.clone();
    }
    commands.spawn((Camera2d, cam2d));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 120.0,
        ..default()
    });
}

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
