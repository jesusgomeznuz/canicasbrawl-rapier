use bevy::prelude::*;
use rapier_bevy::{
    AssetsLoading, BakeKey, BodyType, ColliderShape, LockedAxes, ObjectDef, SimulationMode, spawn_object,
};

#[derive(Component)]
pub struct Marble;

#[derive(Component)]
pub struct MarbleName(pub String);

/// Posición de la canica en el roster (= índice de slot en el flujo de casting).
/// Identidad estable entre bake y replay: los nombres cambian con el cast, el
/// índice no. Los eventos horneados refieren canicas por este índice.
#[derive(Component)]
pub struct MarbleIndex(pub usize);

#[derive(Component)]
pub struct MarbleLabel(pub Entity);

#[derive(Component)]
pub struct MarbleLabelOutline {
    pub marble: Entity,
    pub offset: Vec2,
}

pub struct MarbleConfig {
    pub nickname: String,
    /// `None` = canica anónima (modo `--slots`): sin cara ni color de personaje.
    /// La física no depende de la identidad, así que el reparto se decide después.
    pub image: Option<String>,
}

/// Qué personajes corren esta carrera. Lo arma `build_roster` desde el CLI
/// (`--characters`) y lo lee `world::setup` al spawnar las canicas.
#[derive(Resource)]
pub struct Roster(pub Vec<MarbleConfig>);

/// Construye el roster a partir de los nombres pedidos por CLI (CamelCase canónico,
/// mismos strings que se emiten como `leader` en voice_tracker.json). Sin nombres,
/// devuelve el roster por defecto. Falla nombrando al personaje si le falta imagen.
pub fn build_roster(characters: Option<Vec<String>>) -> Result<Vec<MarbleConfig>, String> {
    let Some(names) = characters else {
        return Ok(default_roster());
    };
    if names.len() > 9 {
        return Err(format!(
            "Pediste {} personajes pero el grid de salida es 3×3 (máximo 9).",
            names.len(),
        ));
    }
    names.iter().map(|name| character_config(name)).collect()
}

/// Roster anónimo para el casting: N canicas `slot_0..slot_{N-1}` sin identidad.
/// El bake corre la física con slots y el voice_tracker reporta qué slot lidera;
/// al renderizar la timeline elegida, `--characters` viste los slots por posición
/// (el nombre i-ésimo es slot_i). Mismo N y mismo orden de spawn en ambas fases.
pub fn slots_roster(n: usize) -> Result<Vec<MarbleConfig>, String> {
    if n == 0 || n > 9 {
        return Err(format!("--slots {n}: el grid de salida es 3×3 (entre 1 y 9)."));
    }
    Ok((0..n)
        .map(|i| MarbleConfig { nickname: format!("slot_{i}"), image: None })
        .collect())
}

fn character_config(name: &str) -> Result<MarbleConfig, String> {
    let image = format!("characters/{}.png", name.to_lowercase());
    if !std::path::Path::new("assets").join(&image).exists() {
        return Err(format!(
            "Personaje '{name}' sin imagen: falta assets/{image}. \
             Revisa el nombre (CamelCase canónico) o agrega el asset.",
        ));
    }
    Ok(MarbleConfig {
        nickname: name.to_string(),
        image: Some(image),
    })
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
    for (i, (cfg, pos)) in roster.iter().zip(grid.iter()).enumerate() {
        let color = cfg.image.as_deref()
            .and_then(dominant_color_from_png)
            .unwrap_or(Color::WHITE);
        let entity = spawn_marble_body(
            commands,
            mode,
            asset_server,
            meshes,
            materials,
            cfg,
            *pos,
            color,
        );
        commands.entity(entity).insert((MarbleIndex(i), BakeKey(i as u64)));
        spawn_marble_label(commands, asset_server, entity, &cfg.nickname);
        if let Some(image) = &cfg.image {
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

fn default_roster() -> Vec<MarbleConfig> {
    [
        "Marceline",
        "Perla",
        "Steven",
        "Wendy",
        "Naruto",
        "Ben10",
        "Patricio",
        "Finn",
        "Bart",
    ]
    .iter()
    .map(|name| character_config(name).expect("default roster asset missing"))
    .collect()
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
    cfg: &MarbleConfig,
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
            collision_groups: Some(super::effects::marble_groups()),
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
        MarbleName(cfg.nickname.clone()),
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

    // (pr, pz, nr, nz) por ring. Misma posición con distinta normal crea un seam (filo nítido).
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

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn spawn_marble_label(
    commands: &mut Commands,
    asset_server: &AssetServer,
    marble_entity: Entity,
    nickname: &str,
) {
    // La fuente default de Bevy es un subset ASCII: nombres con acentos ("pacífica")
    // renderizaban tofu (□). DM Sans cubre Latín extendido.
    let font = asset_server.load("fonts/DMSans-Medium.ttf");
    // 4 copias oscuras desplazadas = outline fake (N/S/E/O a 1.5 px, z=-1 para quedar detrás)
    for &(ox, oy) in &[(-1.5f32, 0.0f32), (1.5, 0.0), (0.0, -1.5), (0.0, 1.5)] {
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
                offset: Vec2::new(ox, oy),
            },
        ));
    }
    // Texto principal blanco encima
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
    bg_color: Color,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    assets_loading: &mut Option<ResMut<AssetsLoading>>,
) {
    let radius = 0.085;
    let half_depth = crate::UNIT / 4.0;
    let quad_size = radius * 2.0;
    let quad_z = half_depth;
    let img_handle: Handle<Image> = asset_server.load(image_path.to_string());

    if let Some(al) = assets_loading.as_deref_mut() {
        al.0.push(img_handle.clone().untyped());
    }

    commands.entity(entity).with_children(|parent| {
        // Disco sólido sin textura → sin píxeles antialiased que causan franja de color
        parent.spawn((
            Mesh3d(meshes.add(Circle::new(radius))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: bg_color,
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

/// Lee el PNG desde `assets/<path>` y devuelve el color más frecuente ignorando:
/// - píxeles transparentes (alpha < 128)
/// - píxeles demasiado claros (todos los canales > 200) para no capturar fondos blancos
/// - píxeles demasiado oscuros (todos < 40) para evitar contornos negros
/// Los colores se cuantifican en cubos de 32 unidades por canal para agrupar tonos similares.
fn dominant_color_from_png(image_path: &str) -> Option<Color> {
    use image::GenericImageView;
    use std::collections::HashMap;

    let full_path = format!("assets/{}", image_path);
    let img = image::open(&full_path).ok()?;

    let mut counts: HashMap<(u8, u8, u8), u32> = HashMap::new();

    for (_, _, pixel) in img.pixels() {
        let [r, g, b, a] = pixel.0;
        if a < 128 {
            continue;
        }
        if r > 200 && g > 200 && b > 200 {
            continue;
        } // blanco / muy claro
        if r < 40 && g < 40 && b < 40 {
            continue;
        } // negro / muy oscuro
        // Cuantizar a cubos de 32 para agrupar tonos similares
        let key = (r & 0xE0, g & 0xE0, b & 0xE0);
        *counts.entry(key).or_insert(0) += 1;
    }

    let (r, g, b) = counts.into_iter().max_by_key(|(_, c)| *c)?.0;
    // Usar el centro del cubo (+16) para un color más representativo
    Some(Color::srgb_u8(
        r.saturating_add(16),
        g.saturating_add(16),
        b.saturating_add(16),
    ))
}
