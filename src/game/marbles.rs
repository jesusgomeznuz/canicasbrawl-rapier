use bevy::prelude::*;
use rapier_bevy::{
    AssetsLoading, BodyType, ColliderShape, LockedAxes, ObjectDef, SimulationMode, TimelineKey, spawn_object,
};

use super::roster::MarbleConfig;

#[derive(Component)]
pub struct Marble;

#[derive(Component)]
pub struct MarbleName(pub String);

#[derive(Component)]
pub struct MarbleIndex(pub usize);

#[derive(Component)]
pub struct MarbleLabel(pub Entity);

#[derive(Component)]
pub struct MarbleLabelOutline {
    pub marble: Entity,
    pub offset: Vec2,
}

pub fn spawn_marbles(
    commands: &mut Commands,
    mode: &SimulationMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    roster: &[MarbleConfig],
    spawn_cx: f32,
    spawn_cy: f32,
    assets_loading: &mut Option<ResMut<AssetsLoading>>,
) {
    let grid = spawn_grid(spawn_cx, spawn_cy);
    for (i, (config, position)) in roster.iter().zip(grid.iter()).enumerate() {
        let color = config.image.as_deref()
            .and_then(dominant_color_from_png)
            .unwrap_or(Color::WHITE);
        let entity = spawn_marble_body(
            commands,
            mode,
            asset_server,
            meshes,
            materials,
            config,
            *position,
            color,
        );
        commands.entity(entity).insert((MarbleIndex(i), TimelineKey(i as u64)));
        spawn_marble_label(commands, asset_server, entity, &config.nickname);
        if let Some(image) = &config.image {
            attach_marble_face(
                commands,
                entity,
                image,
                color,
                asset_server,
                meshes,
                materials,
                assets_loading,
            );
        }
    }
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

fn spawn_grid(cx: f32, cy: f32) -> [(f32, f32); 9] {
    let dx = 0.25;
    let dy = 0.30;
    [
        (cx - dx, cy + dy),
        (cx, cy + dy),
        (cx + dx, cy + dy),
        (cx - dx, cy),
        (cx, cy),
        (cx + dx, cy),
        (cx - dx, cy - dy),
        (cx, cy - dy),
        (cx + dx, cy - dy),
    ]
}

fn spawn_marble_body(
    commands: &mut Commands,
    mode: &SimulationMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    config: &MarbleConfig,
    (x, y): (f32, f32),
    body_color: Color,
) -> Entity {
    let radius = 0.085;
    let half_depth = crate::UNIT / 4.0;
    let border = 0.02;
    let entity = spawn_object(
        commands,
        ObjectDef {
            shape: ColliderShape::Cylinder {
                half_height: half_depth,
                radius,
                axis: Vec3::Z,
            },
            position: Vec3::new(x, y, 0.0),
            body: BodyType::Dynamic,
            restitution: Some(0.6),
            friction: Some(0.3),
            linear_damping: Some(0.15),
            angular_damping: Some(0.9),
            ccd: true,
            locked_axes: Some(
                LockedAxes::TRANSLATION_LOCKED_Z
                    | LockedAxes::ROTATION_LOCKED_X
                    | LockedAxes::ROTATION_LOCKED_Y,
            ),
            visual: None,
            collision_groups: Some(super::sensors::freeze::marble_groups()),
            ..Default::default()
        },
        mode,
        asset_server,
        meshes,
        materials,
    );
    let body_mesh = meshes.add(build_marble_mesh(half_depth, radius, border));
    let body_material = materials.add(StandardMaterial {
        base_color: body_color,
        perceptual_roughness: 0.8,
        metallic: 0.0,
        ..default()
    });
    commands.entity(entity).insert((
        Mesh3d(body_mesh),
        MeshMaterial3d(body_material),
        Marble,
        MarbleName(config.nickname.clone()),
    ));
    entity
}

fn build_marble_mesh(half_depth: f32, radius: f32, border: f32) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::mesh::{Indices, PrimitiveTopology};

    let n_radial = 48;
    let n_arc = 8;
    let z_back_inner = -half_depth + border;
    let r_inner = (radius - border).max(0.0);

    let mut rings: Vec<(f32, f32, f32, f32)> = Vec::new();
    rings.push((0.0, half_depth, 0.0, 1.0));
    rings.push((radius, half_depth, 0.0, 1.0));
    rings.push((radius, half_depth, 1.0, 0.0));
    rings.push((radius, z_back_inner, 1.0, 0.0));
    for i in 1..n_arc {
        let theta = std::f32::consts::FRAC_PI_2 * (i as f32) / (n_arc as f32);
        let (s, c) = theta.sin_cos();
        rings.push((r_inner + border * c, z_back_inner - border * s, c, -s));
    }
    rings.push((r_inner, -half_depth, 0.0, -1.0));
    rings.push((0.0, -half_depth, 0.0, -1.0));

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    for &(pr, pz, nr, nz) in &rings {
        for j in 0..n_radial {
            let phi = std::f32::consts::TAU * (j as f32) / (n_radial as f32);
            let (s, c) = phi.sin_cos();
            positions.push([pr * c, pr * s, pz]);
            normals.push([nr * c, nr * s, nz]);
        }
    }

    let mut indices: Vec<u32> = Vec::new();
    for i in 0..(rings.len() - 1) {
        let (pr_a, pz_a, _, _) = rings[i];
        let (pr_b, pz_b, _, _) = rings[i + 1];
        if (pr_a - pr_b).abs() < 1e-6 && (pz_a - pz_b).abs() < 1e-6 {
            continue;
        }
        for j in 0..n_radial as u32 {
            let jn = (j + 1) % n_radial as u32;
            let a = i as u32 * n_radial as u32 + j;
            let b = (i as u32 + 1) * n_radial as u32 + j;
            let c = (i as u32 + 1) * n_radial as u32 + jn;
            let d = i as u32 * n_radial as u32 + jn;
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}

fn spawn_marble_label(
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

fn attach_marble_face(
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

fn dominant_color_from_png(image_path: &str) -> Option<Color> {
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
