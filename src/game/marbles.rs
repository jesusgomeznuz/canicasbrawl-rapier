use bevy::prelude::*;
use rapier_bevy::{
    AssetsLoading, BodyType, ColliderShape, LockedAxes, ObjectDef, SimMode, VisualDef, spawn_object,
};

#[derive(Component)]
pub struct Marble;

#[derive(Component)]
pub struct MarbleName(pub &'static str);

#[derive(Component)]
pub struct MarbleLabel(pub Entity);

pub struct MarbleConfig {
    pub nickname: &'static str,
    pub image: &'static str,
}

pub fn spawn_marbles(
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    spawn_cx: f32,
    spawn_cy: f32,
    assets_loading: &mut Option<ResMut<AssetsLoading>>,
) {
    let roster = marble_roster();
    let grid = spawn_grid(spawn_cx, spawn_cy);
    for (cfg, pos) in roster.iter().zip(grid.iter()) {
        let entity = spawn_marble_body(commands, mode, asset_server, meshes, materials, cfg, *pos);
        spawn_marble_label(commands, entity, cfg.nickname);
        attach_marble_face(
            commands,
            entity,
            cfg.image,
            asset_server,
            meshes,
            materials,
            assets_loading,
        );
    }
}

fn marble_roster() -> Vec<MarbleConfig> {
    vec![
        MarbleConfig {
            nickname: "Marceline",
            image: "characters/marceline.png",
        },
        MarbleConfig {
            nickname: "Perla",
            image: "characters/perla.png",
        },
        MarbleConfig {
            nickname: "Steven",
            image: "characters/steven.png",
        },
        MarbleConfig {
            nickname: "Wendy",
            image: "characters/wendy.png",
        },
        MarbleConfig {
            nickname: "Naruto",
            image: "characters/naruto.png",
        },
        MarbleConfig {
            nickname: "Ben10",
            image: "characters/ben10.png",
        },
        MarbleConfig {
            nickname: "Patricio",
            image: "characters/patricio.png",
        },
        MarbleConfig {
            nickname: "Finn",
            image: "characters/finn.png",
        },
        MarbleConfig {
            nickname: "Bart",
            image: "characters/bart.png",
        },
    ]
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
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    cfg: &MarbleConfig,
    (x, y): (f32, f32),
) -> Entity {
    let radius = 0.09;
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
            friction: Some(0.4),
            linear_damping: Some(0.15),
            angular_damping: Some(0.4),
            ccd: true,
            locked_axes: Some(
                LockedAxes::TRANSLATION_LOCKED_Z
                    | LockedAxes::ROTATION_LOCKED_X
                    | LockedAxes::ROTATION_LOCKED_Y,
            ),
            visual: None,
            ..Default::default()
        },
        mode,
        asset_server,
        meshes,
        materials,
    );
    let body_mesh = meshes.add(build_marble_mesh(half_depth, radius, border));
    let body_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.8,
        metallic: 0.0,
        ..default()
    });
    commands.entity(entity).insert((
        Mesh3d(body_mesh),
        MeshMaterial3d(body_material),
        Marble,
        MarbleName(cfg.nickname),
    ));
    entity
}

fn build_marble_mesh(half_depth: f32, radius: f32, border: f32) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::mesh::{Indices, PrimitiveTopology};

    let n_radial = 48;
    let n_arc    = 8;
    let z_back_inner = -half_depth + border;
    let r_inner = (radius - border).max(0.0);

    let mut profile: Vec<(f32, f32)> = Vec::new();
    profile.push((0.0, half_depth));
    profile.push((radius, half_depth));
    profile.push((radius, z_back_inner));
    for i in 1..n_arc {
        let theta = std::f32::consts::FRAC_PI_2 * (i as f32) / (n_arc as f32);
        let (s, c) = theta.sin_cos();
        profile.push((r_inner + border * c, z_back_inner - border * s));
    }
    profile.push((r_inner, -half_depth));
    profile.push((0.0, -half_depth));

    let mut positions: Vec<[f32; 3]> = Vec::new();
    for &(pr, pz) in &profile {
        for j in 0..n_radial {
            let phi = std::f32::consts::TAU * (j as f32) / (n_radial as f32);
            let (s, c) = phi.sin_cos();
            positions.push([pr * c, pr * s, pz]);
        }
    }

    let mut indices: Vec<u32> = Vec::new();
    for i in 0..(profile.len() - 1) as u32 {
        for j in 0..n_radial as u32 {
            let jn = (j + 1) % n_radial as u32;
            let a = i       * n_radial as u32 + j;
            let b = (i + 1) * n_radial as u32 + j;
            let c = (i + 1) * n_radial as u32 + jn;
            let d = i       * n_radial as u32 + jn;
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_normals();
    mesh
}

fn spawn_marble_label(commands: &mut Commands, marble_entity: Entity, nickname: &'static str) {
    commands.spawn((
        Text2d::new(nickname),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::BLACK),
        TextLayout::new_with_justify(JustifyText::Center),
        MarbleLabel(marble_entity),
    ));
}

fn attach_marble_face(
    commands: &mut Commands,
    entity: Entity,
    image_path: &'static str,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    assets_loading: &mut Option<ResMut<AssetsLoading>>,
) {
    let radius = 0.09;
    let half_depth = crate::UNIT / 4.0;
    let quad_size = radius * 2.0;
    let quad_z = half_depth;

    let bg_handle: Handle<Image> = asset_server.load("characters/circle_white.png");
    let img_handle: Handle<Image> = asset_server.load(image_path);

    if let Some(al) = assets_loading.as_deref_mut() {
        al.0.push(bg_handle.clone().untyped());
        al.0.push(img_handle.clone().untyped());
    }

    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Rectangle::new(quad_size, quad_size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(bg_handle),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, quad_z + 0.001),
        ));
        parent.spawn((
            Mesh3d(meshes.add(Rectangle::new(quad_size, quad_size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(img_handle),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, quad_z + 0.002),
        ));
    });
}
