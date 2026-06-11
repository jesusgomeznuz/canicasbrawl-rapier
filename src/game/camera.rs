use super::leader::RaceLeader;
use super::marbles::{Marble, MarbleLabel, MarbleLabelOutline};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::text::TextLayoutInfo;
use rapier_bevy::OffscreenTarget;

pub fn spawn_camera_and_lights(mut commands: Commands, offscreen: Option<Res<OffscreenTarget>>) {
    let render_target: Option<RenderTarget> = offscreen
        .as_ref()
        .map(|o| RenderTarget::Image(o.image.clone().into()));

    let mut cam3d = Camera::default();
    if let Some(ref t) = render_target {
        cam3d.target = t.clone();
    }
    // Arranca ya en la pose de juego (misma que camera_follows_lowest_marble: z=2.5,
    // mirando recto). La canica más baja descansa ~0.10 bajo el centro de spawn, así
    // que la cámara nace ahí enmarcándola: sin el salto brusco del frame 0.
    let lowest_marble_below_center = 0.10;
    let start_pose = Transform::from_xyz(0.0, -lowest_marble_below_center, 2.5);
    commands
        .spawn((Camera3d::default(), cam3d, Tonemapping::None, start_pose))
        .with_children(|camera| {
            // Tres cuartos desde arriba-derecha: ~46° vertical, ~28° horizontal.
            // Estándar en juegos cartoon/arcade — da volumen a las esferas sin
            // oscurecer las caras. La ambient actúa como luz de relleno suave.
            camera.spawn((
                DirectionalLight {
                    illuminance: 12_000.0,
                    shadows_enabled: true,
                    ..default()
                },
                Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.45, 0.3, 0.0)),
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
    commands.spawn((Camera2d, cam2d, IsDefaultUiCamera));

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
    let Ok(mut cam) = camera.single_mut() else {
        return;
    };

    let target_y = lowest_y + camera_y_offset;
    let new_y = if !*initialized {
        *initialized = true;
        target_y
    } else {
        let alpha = 1.0 - (-time.delta_secs() * sharpness).exp();
        cam.translation.y + (target_y - cam.translation.y) * alpha
    };
    cam.translation = Vec3::new(0.0, new_y, camera_z);
    cam.rotation = Quat::IDENTITY;
}

#[derive(Component)]
pub struct LeaderCrown;

pub fn spawn_leader_crown(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut assets_loading: Option<ResMut<rapier_bevy::AssetsLoading>>,
) {
    let handle: Handle<Image> = asset_server.load("img/crown.png");
    if let Some(al) = assets_loading.as_deref_mut() {
        al.0.push(handle.clone().untyped());
    }
    commands.spawn((
        Sprite {
            image: handle,
            custom_size: Some(Vec2::splat(28.0)),
            ..default()
        },
        Transform::default(),
        Visibility::Hidden,
        LeaderCrown,
    ));
}

pub fn update_leader_crown(
    leader: Res<RaceLeader>,
    labels: Query<(&Transform, &TextLayoutInfo, &MarbleLabel), Without<LeaderCrown>>,
    mut crown: Query<(&mut Transform, &mut Visibility), (With<LeaderCrown>, Without<MarbleLabel>)>,
) {
    let Ok((mut crown_t, mut crown_v)) = crown.single_mut() else {
        return;
    };
    let Some(leader_marble) = leader.marble else {
        *crown_v = Visibility::Hidden;
        return;
    };
    let crown_size = 28.0_f32;
    let gap = 6.0_f32;
    for (label_t, layout, MarbleLabel(marble_entity)) in &labels {
        if *marble_entity == leader_marble {
            crown_t.translation.x =
                label_t.translation.x - layout.size.x / 2.0 - crown_size / 2.0 - gap;
            crown_t.translation.y = label_t.translation.y;
            crown_t.translation.z = label_t.translation.z;
            *crown_v = Visibility::Visible;
            return;
        }
    }
    *crown_v = Visibility::Hidden;
}

/// Aspecto del diseño (vertical 9:16). Fijo a propósito: el aspecto NO se lee del
/// render target porque en bake (headless) no existe viewport, y este check decide
/// FÍSICA (qué sensores disparan) — debe dar lo mismo con ventana, --record o --bake.
const DESIGN_ASPECT: f32 = 9.0 / 16.0;

pub fn world_pos_on_screen(world_pos: Vec3, projection: &Projection, cam_xform: &GlobalTransform) -> bool {
    let Projection::Perspective(persp) = projection else {
        return true;
    };
    // Proyección manual en espacio de cámara: idéntica en todos los modos, a
    // diferencia de world_to_viewport, que requiere un render target vivo.
    let local = cam_xform.affine().inverse().transform_point3(world_pos);
    let depth = -local.z;
    if depth <= 0.0 {
        return false;
    }
    let half_h = depth * (persp.fov * 0.5).tan();
    let half_w = half_h * DESIGN_ASPECT;
    local.x.abs() <= half_w && local.y.abs() <= half_h
}

pub fn update_marble_labels(
    marbles: Query<&GlobalTransform, With<Marble>>,
    mut labels: Query<(&mut Transform, &mut TextFont, &MarbleLabel), Without<MarbleLabelOutline>>,
    mut outlines: Query<(&mut Transform, &mut TextFont, &MarbleLabelOutline), Without<MarbleLabel>>,
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
            transform.translation.z = 0.0;
        }
    }
    for (mut transform, mut font, outline) in &mut outlines {
        font.font_size = font_size;
        let Ok(marble_gt) = marbles.get(outline.marble) else { continue };
        let above = marble_gt.translation() + Vec3::Y * 0.13;
        if let Ok(screen_pos) = camera.world_to_viewport(cam_gt, above) {
            transform.translation.x = screen_pos.x - viewport.x / 2.0 + outline.offset.x;
            transform.translation.y = viewport.y / 2.0 - screen_pos.y + outline.offset.y;
            transform.translation.z = -1.0; // detrás del texto blanco
        }
    }
}
