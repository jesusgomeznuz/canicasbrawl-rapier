use super::marbles::Marble;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use rapier_bevy::OffscreenTarget;

pub fn spawn_camera_and_lights(mut commands: Commands, offscreen: Option<Res<OffscreenTarget>>) {
    let render_target = offscreen
        .as_ref()
        .map(|offscreen_target| RenderTarget::Image(offscreen_target.image.clone().into()))
        .unwrap_or_default();

    let world_camera = Camera {
        target: render_target.clone(),
        ..default()
    };
    let lowest_marble_below_center = 0.10;
    let start_pose = Transform::from_xyz(0.0, -lowest_marble_below_center, 2.5);
    commands
        .spawn((Camera3d::default(), world_camera, Tonemapping::None, start_pose))
        .with_children(|camera| {
            camera.spawn((
                DirectionalLight {
                    illuminance: 12_000.0,
                    shadows_enabled: true,
                    ..default()
                },
                Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.45, 0.3, 0.0)),
            ));
        });

    let overlay_camera = Camera {
        order: 1,
        clear_color: ClearColorConfig::None,
        target: render_target,
        ..default()
    };
    commands.spawn((Camera2d, overlay_camera, IsDefaultUiCamera));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 90.0,
        ..default()
    });
}

pub fn camera_follows_lowest_marble(
    time: Res<Time>,
    marbles: Query<&Transform, With<Marble>>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<Marble>)>,
    mut initialized: Local<bool>,
) {
    let camera_z = 2.5;
    let camera_y_offset = 0.2;
    let sharpness = 10.0;

    let Some(lowest_y) = marbles.iter().map(|t| t.translation.y).reduce(f32::min) else {
        return;
    };
    let Ok(mut camera_transform) = camera.single_mut() else {
        return;
    };

    let target_y = lowest_y + camera_y_offset;
    let new_y = if !*initialized {
        *initialized = true;
        target_y
    } else {
        let alpha = 1.0 - (-time.delta_secs() * sharpness).exp();
        camera_transform.translation.y + (target_y - camera_transform.translation.y) * alpha
    };
    camera_transform.translation = Vec3::new(0.0, new_y, camera_z);
    camera_transform.rotation = Quat::IDENTITY;
}

pub fn world_pos_on_screen(world_position: Vec3, projection: &Projection, camera_transform: &GlobalTransform) -> bool {
    let design_aspect_9_16 = 9.0 / 16.0;
    let Projection::Perspective(perspective) = projection else {
        return true;
    };
    let point_in_camera_space = camera_transform.affine().inverse().transform_point3(world_position);
    let depth = -point_in_camera_space.z;
    if depth <= 0.0 {
        return false;
    }
    let half_height = depth * (perspective.fov * 0.5).tan();
    let half_width = half_height * design_aspect_9_16;
    point_in_camera_space.x.abs() <= half_width && point_in_camera_space.y.abs() <= half_height
}

pub fn world_y_above_screen(
    world_y: f32,
    margin: f32,
    projection: &Projection,
    camera_transform: &GlobalTransform,
) -> bool {
    let Projection::Perspective(perspective) = projection else {
        return false;
    };
    let point_in_camera_space = camera_transform
        .affine()
        .inverse()
        .transform_point3(Vec3::new(0.0, world_y, 0.0));
    let depth = -point_in_camera_space.z;
    if depth <= 0.0 {
        return false;
    }
    let half_height = depth * (perspective.fov * 0.5).tan();
    point_in_camera_space.y > half_height + margin
}
