use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;

use super::BackdropSeed;

#[derive(Component)]
pub struct BackgroundStar {
    y_offset: f32,
}

#[derive(Component)]
pub struct StarTwinkle {
    phase: f32,      // offset de fase (0–2π)
    frequency: f32,  // Hz — cuántas veces parpadea por segundo
    base_alpha: f32, // opacidad base
    amplitude: f32,  // cuánto varía la opacidad
}

pub fn spawn_stars(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    seed: Res<BackdropSeed>,
) {
    let config = StarConfig::default();
    // Sub-semilla derivada del level seed para que el campo de estrellas sea
    // determinista por partida pero diferente al generador de módulos.
    let mut rng = SmallRng::seed_from_u64(seed.0.wrapping_mul(0x9e3779b97f4a7c15));

    let star_mesh = meshes.add(build_star_mesh(1.0));

    // Jitter sobre grid: el área se divide en celdas y cada una recibe exactamente
    // una estrella en posición aleatoria dentro de ella. Esto evita los cúmulos y
    // zonas vacías que produce la distribución uniforme pura.
    let (x_spread, y_spread) = (7.5_f32, 14.0_f32);
    let star_z = -15.0_f32;

    let cols = (config.count as f32).sqrt().round() as u32;
    let rows = (config.count + cols - 1) / cols;
    let cell_w = x_spread * 2.0 / cols as f32;
    let cell_h = y_spread * 2.0 / rows as f32;

    'outer: for row in 0..rows {
        for col in 0..cols {
            if row * cols + col >= config.count {
                break 'outer;
            }
            let cell_x = -x_spread + col as f32 * cell_w;
            let cell_y = -y_spread + row as f32 * cell_h;
            let x = cell_x + rng.gen_range(0.0..cell_w);
            let y_offset_in_cell = cell_y + rng.gen_range(0.0..cell_h);
            let size = rng.gen_range(config.min_size..config.max_size);
            let phase = rng.gen_range(0.0..std::f32::consts::TAU);
            let freq = rng.gen_range(config.twinkle_freq_min..config.twinkle_freq_max);
            let base_alpha = (config.base_alpha + rng.gen_range(-0.15..0.15)).clamp(0.2, 1.0);
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, base_alpha),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                ..default()
            });
            commands.spawn((
                Mesh3d(star_mesh.clone()),
                MeshMaterial3d(mat),
                Transform::from_xyz(x, y_offset_in_cell, star_z).with_scale(Vec3::splat(size)),
                BackgroundStar { y_offset: y_offset_in_cell },
                StarTwinkle {
                    phase,
                    frequency: freq,
                    base_alpha,
                    amplitude: config.twinkle_amplitude,
                },
            ));
        }
    }
}

pub fn stars_follow_camera(
    camera: Query<&Transform, With<Camera3d>>,
    mut stars: Query<(&BackgroundStar, &mut Transform), Without<Camera3d>>,
) {
    let Ok(camera_transform) = camera.single() else { return };
    let camera_y = camera_transform.translation.y;
    for (star, mut transform) in &mut stars {
        transform.translation.y = camera_y + star.y_offset;
    }
}

pub fn twinkle_stars(
    time: Res<Time>,
    stars: Query<(&StarTwinkle, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let elapsed = time.elapsed_secs();
    for (twinkle, material_handle) in &stars {
        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };
        let alpha = twinkle.base_alpha
            + twinkle.amplitude * (twinkle.phase + elapsed * twinkle.frequency * std::f32::consts::TAU).sin();
        material.base_color = Color::srgba(1.0, 1.0, 1.0, alpha.clamp(0.0, 1.0));
    }
}

// Parámetros editables del campo de estrellas — cambia aquí para tunar visualmente.
// count:      cuántas estrellas  (prueba 40–100)
// min/max_size: radio en unidades del mundo a Z=-15 (0.040≈2px, 0.090≈4px)
// base_alpha:   brillo base (0–1)
// twinkle_amplitude: variación de opacidad (0=sin parpadeo, 0.4=muy notable)
// twinkle_freq_min/max: rango de velocidad de parpadeo en Hz (0.8–2.0 = natural)
struct StarConfig {
    count: u32,
    min_size: f32,
    max_size: f32,
    base_alpha: f32,
    twinkle_amplitude: f32,
    twinkle_freq_min: f32,
    twinkle_freq_max: f32,
}

impl Default for StarConfig {
    fn default() -> Self {
        Self {
            count: 110,
            min_size: 0.015,
            max_size: 0.10,
            base_alpha: 0.25,
            twinkle_amplitude: 0.20,
            twinkle_freq_min: 0.7,
            twinkle_freq_max: 2.1,
        }
    }
}

// Mesh de estrella de 4 puntas en el plano XY (normal +Z hacia la cámara).
// outer_radius=1.0 → se escala por instancia con Transform::scale.
fn build_star_mesh(outer_radius: f32) -> Mesh {
    let inner_radius = outer_radius * 0.38;

    // Centro + 8 vértices alrededor (alternando punta/concavidad cada 45°)
    let mut positions: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]];
    for i in 0..8u32 {
        let angle = std::f32::consts::FRAC_PI_2 - i as f32 * std::f32::consts::FRAC_PI_4;
        let r = if i % 2 == 0 {
            outer_radius
        } else {
            inner_radius
        };
        positions.push([r * angle.cos(), r * angle.sin(), 0.0]);
    }

    // 8 triángulos en abanico desde el centro — orden CCW visto desde +Z (cámara)
    let indices: Vec<u32> = (0u32..8)
        .flat_map(|i| [0, (i + 1) % 8 + 1, i + 1])
        .collect();

    let normals = vec![[0.0_f32, 0.0, 1.0]; 9];

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}
