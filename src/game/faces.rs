use bevy::prelude::*;
use rapier_bevy::AssetsLoading;

pub fn attach_marble_face(
    commands: &mut Commands,
    entity: Entity,
    image_path: &str,
    background_color: Color,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    assets_loading: &mut Option<ResMut<AssetsLoading>>,
) {
    let radius = 0.085;
    let half_depth = crate::UNIT / 4.0;
    let quad_size = radius * 2.0;
    let quad_z = half_depth;
    let image_handle: Handle<Image> = asset_server.load(image_path.to_string());

    if let Some(loading) = assets_loading.as_deref_mut() {
        loading.0.push(image_handle.clone().untyped());
    }

    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Circle::new(radius))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: background_color,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, quad_z + 0.001),
        ));
        parent.spawn((
            Mesh3d(meshes.add(Rectangle::new(quad_size, quad_size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(image_handle),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, quad_z + 0.002),
        ));
    });
}

pub fn dominant_color_from_png(image_path: &str) -> Option<Color> {
    use image::GenericImageView;
    use std::collections::HashMap;

    let full_path = format!("assets/{}", image_path);
    let img = image::open(&full_path).ok()?;

    let mut counts: HashMap<(u8, u8, u8), u32> = HashMap::new();

    for (_, _, pixel) in img.pixels() {
        let [r, g, b, a] = pixel.0;
        let is_transparent = a < 128;
        let is_background_white = r > 200 && g > 200 && b > 200;
        let is_outline_black = r < 40 && g < 40 && b < 40;
        if is_transparent || is_background_white || is_outline_black {
            continue;
        }
        let quantized_to_32_cube = (r & 0xE0, g & 0xE0, b & 0xE0);
        *counts.entry(quantized_to_32_cube).or_insert(0) += 1;
    }

    let (r, g, b) = counts.into_iter().max_by_key(|(_, c)| *c)?.0;
    let cube_center = 16;
    Some(Color::srgb_u8(
        r.saturating_add(cube_center),
        g.saturating_add(cube_center),
        b.saturating_add(cube_center),
    ))
}
