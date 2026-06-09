use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;

use super::world::LevelSeed;

// Paleta "azul" — stops del cielo del diseño (#073B4C → #0E7C92 → #13A0AE → #1EB6BE)
pub const SKY_STOPS: [(f32, [u8; 3]); 5] = [
    (0.00, [0x07, 0x3B, 0x4C]),
    (0.40, [0x0E, 0x7C, 0x92]),
    (0.66, [0x13, 0xA0, 0xAE]),
    (0.84, [0x1E, 0xB6, 0xBE]),
    (1.00, [0x1E, 0xB6, 0xBE]),
];

#[derive(Component)]
pub struct SkyQuad;

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

pub fn spawn_sky(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let texture = build_gradient_texture(&mut images);
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(100.0, 200.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, -50.0),
        SkyQuad,
    ));
}

pub fn spawn_stars(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    seed: Res<LevelSeed>,
) {
    let cfg = StarConfig::default();
    // Sub-semilla derivada del level seed para que el campo de estrellas sea
    // determinista por partida pero diferente al generador de módulos.
    let mut rng = SmallRng::seed_from_u64(seed.0.wrapping_mul(0x9e3779b97f4a7c15));

    let star_mesh = meshes.add(build_star_mesh(1.0));

    // Jitter sobre grid: el área se divide en celdas y cada una recibe exactamente
    // una estrella en posición aleatoria dentro de ella. Esto evita los cúmulos y
    // zonas vacías que produce la distribución uniforme pura.
    let (x_spread, y_spread) = (7.5_f32, 14.0_f32);
    let star_z = -15.0_f32;

    let cols = (cfg.count as f32).sqrt().round() as u32;
    let rows = (cfg.count + cols - 1) / cols;
    let cell_w = x_spread * 2.0 / cols as f32;
    let cell_h = y_spread * 2.0 / rows as f32;

    'outer: for row in 0..rows {
        for col in 0..cols {
            if row * cols + col >= cfg.count {
                break 'outer;
            }
            let cell_x = -x_spread + col as f32 * cell_w;
            let cell_y = -y_spread + row as f32 * cell_h;
            let x = cell_x + rng.gen_range(0.0..cell_w);
            let y_off = cell_y + rng.gen_range(0.0..cell_h);
            let size = rng.gen_range(cfg.min_size..cfg.max_size);
            let phase = rng.gen_range(0.0..std::f32::consts::TAU);
            let freq = rng.gen_range(cfg.twinkle_freq_min..cfg.twinkle_freq_max);
            let base_alpha = (cfg.base_alpha + rng.gen_range(-0.15..0.15)).clamp(0.2, 1.0);
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
                Transform::from_xyz(x, y_off, star_z).with_scale(Vec3::splat(size)),
                BackgroundStar { y_offset: y_off },
                StarTwinkle {
                    phase,
                    frequency: freq,
                    base_alpha,
                    amplitude: cfg.twinkle_amplitude,
                },
            ));
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

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn build_gradient_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let height: u32 = 256;
    let mut data = Vec::with_capacity((height * 4) as usize);
    for row in 0..height {
        let t = row as f32 / (height - 1) as f32;
        let [r, g, b] = sample_sky_color(t);
        data.extend_from_slice(&[r, g, b, 255]);
    }
    images.add(Image::new(
        Extent3d {
            width: 1,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ))
}

pub fn sample_sky_color(t: f32) -> [u8; 3] {
    for i in 0..(SKY_STOPS.len() - 1) {
        let (t0, c0) = SKY_STOPS[i];
        let (t1, c1) = SKY_STOPS[i + 1];
        if t <= t1 {
            let f = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
            return [
                lerp_channel(c0[0], c1[0], f),
                lerp_channel(c0[1], c1[1], f),
                lerp_channel(c0[2], c1[2], f),
            ];
        }
    }
    SKY_STOPS.last().unwrap().1
}

fn lerp_channel(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

pub fn update_sky_with_camera(
    camera: Query<&Transform, With<Camera3d>>,
    mut sky: Query<&mut Transform, (With<SkyQuad>, Without<Camera3d>, Without<BackgroundStar>)>,
    mut stars: Query<(&BackgroundStar, &mut Transform), (Without<SkyQuad>, Without<Camera3d>)>,
) {
    let Ok(cam) = camera.single() else { return };
    let cam_y = cam.translation.y;

    if let Ok(mut sky_t) = sky.single_mut() {
        sky_t.translation.y = cam_y;
    }
    for (star, mut t) in &mut stars {
        t.translation.y = cam_y + star.y_offset;
    }
}

pub fn twinkle_stars(
    time: Res<Time>,
    stars: Query<(&StarTwinkle, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let elapsed = time.elapsed_secs();
    for (tw, mat_handle) in &stars {
        let Some(mat) = materials.get_mut(&mat_handle.0) else {
            continue;
        };
        let alpha = tw.base_alpha
            + tw.amplitude * (tw.phase + elapsed * tw.frequency * std::f32::consts::TAU).sin();
        mat.base_color = Color::srgba(1.0, 1.0, 1.0, alpha.clamp(0.0, 1.0));
    }
}
