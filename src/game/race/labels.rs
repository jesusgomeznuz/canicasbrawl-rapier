use bevy::prelude::*;
use bevy::transform::TransformSystem;

use crate::game::world::marbles::Marble;

/// Se actualizan las etiquetas de las canicas: proyectan mundo→pantalla
/// cuando todas las posiciones del frame ya quedaron firmes.
pub fn update_labels(app: &mut App) {
    app.add_systems(
        PostUpdate,
        update_marble_labels.after(TransformSystem::TransformPropagate),
    );
}

#[derive(Component)]
pub struct MarbleLabel(pub Entity);

#[derive(Component)]
pub struct MarbleLabelOutline {
    pub marble: Entity,
    pub offset: Vec2,
}

pub fn spawn_marble_label(
    commands: &mut Commands,
    asset_server: &AssetServer,
    marble_entity: Entity,
    nickname: &str,
) {
    let font = asset_server.load("fonts/DMSans-Medium.ttf");
    for &(offset_x, offset_y) in &[(-1.5f32, 0.0f32), (1.5, 0.0), (0.0, -1.5), (0.0, 1.5)] {
        commands.spawn((
            Text2d::new(nickname),
            TextFont {
                font: font.clone(),
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgba(0.0, 0.0, 0.0, 0.88)),
            TextLayout::new_with_justify(JustifyText::Center),
            MarbleLabelOutline {
                marble: marble_entity,
                offset: Vec2::new(offset_x, offset_y),
            },
        ));
    }
    commands.spawn((
        Text2d::new(nickname),
        TextFont {
            font,
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(JustifyText::Center),
        MarbleLabel(marble_entity),
    ));
}

pub fn update_marble_labels(
    marbles: Query<&GlobalTransform, With<Marble>>,
    mut labels: Query<(&mut Transform, &mut TextFont, &MarbleLabel), Without<MarbleLabelOutline>>,
    mut outlines: Query<(&mut Transform, &mut TextFont, &MarbleLabelOutline), Without<MarbleLabel>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let Ok((camera, camera_global_transform)) = camera_q.single() else { return };
    let Some(viewport) = camera.logical_viewport_size() else { return };
    // 2.2% del alto del viewport → tamaño relativo igual en ventana y en --record
    let font_size = viewport.y * 0.022;
    for (mut transform, mut font, MarbleLabel(marble_entity)) in &mut labels {
        font.font_size = font_size;
        let Ok(marble_global_transform) = marbles.get(*marble_entity) else { continue };
        let above = marble_global_transform.translation() + Vec3::Y * 0.13;
        if let Ok(screen_pos) = camera.world_to_viewport(camera_global_transform, above) {
            transform.translation.x = screen_pos.x - viewport.x / 2.0;
            transform.translation.y = viewport.y / 2.0 - screen_pos.y;
            transform.translation.z = 0.0;
        }
    }
    for (mut transform, mut font, outline) in &mut outlines {
        font.font_size = font_size;
        let Ok(marble_global_transform) = marbles.get(outline.marble) else { continue };
        let above = marble_global_transform.translation() + Vec3::Y * 0.13;
        if let Ok(screen_pos) = camera.world_to_viewport(camera_global_transform, above) {
            transform.translation.x = screen_pos.x - viewport.x / 2.0 + outline.offset.x;
            transform.translation.y = viewport.y / 2.0 - screen_pos.y + outline.offset.y;
            transform.translation.z = -1.0; // detrás del texto blanco
        }
    }
}
