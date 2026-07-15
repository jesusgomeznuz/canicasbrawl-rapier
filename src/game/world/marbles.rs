use bevy::prelude::*;
use rapier_bevy::{
    AssetsLoading, BodyType, ColliderShape, LockedAxes, ObjectDef, TimelineKey, spawn_object,
};

use crate::game::race::faces::{attach_marble_face, dominant_color_from_png};
use crate::game::race::labels::spawn_marble_label;
use crate::game::race::roster::MarbleConfig;

#[derive(Component)]
pub struct Marble;

#[derive(Component)]
pub struct MarbleName(pub String);

#[derive(Component)]
pub struct MarbleIndex(pub usize);

/// El mundo le pone cuerpo al elenco de la carrera: nacen las canicas, cada
/// una con su identidad (índice + TimelineKey), su etiqueta y su cara.
pub fn spawn_marbles(
    mut commands: Commands,
    roster: Res<crate::game::race::roster::Roster>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut assets_loading: Option<ResMut<AssetsLoading>>,
) {
    let spawn_cx = 0.0;
    let spawn_cy = 0.0;
    let grid = spawn_grid(spawn_cx, spawn_cy);
    for (i, (config, position)) in roster.0.iter().zip(grid.iter()).enumerate() {
        let color = config.image.as_deref()
            .and_then(dominant_color_from_png)
            .unwrap_or(Color::WHITE);
        let entity = spawn_marble_body(
            &mut commands,
            &asset_server,
            &mut meshes,
            &mut materials,
            config,
            *position,
            color,
        );
        commands.entity(entity).insert((MarbleIndex(i), TimelineKey(i as u64)));
        spawn_marble_label(&mut commands, &asset_server, entity, &config.nickname);
        if let Some(image) = &config.image {
            attach_marble_face(
                &mut commands,
                entity,
                image,
                color,
                &asset_server,
                &mut meshes,
                &mut materials,
                &mut assets_loading,
            );
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
