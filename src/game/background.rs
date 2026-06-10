use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;

use super::world::LevelSeed;

/// Paleta de color del juego — controla cielo, nubes, obstáculos y ClearColor.
/// Se inserta como recurso en `run_sim` según el flag `--neon` / `--rosa` / (default azul).
#[derive(Resource, Clone)]
pub struct ColorPalette {
    pub sky_stops: [(f32, [u8; 3]); 5],
    pub cloud_near: Color,
    pub cloud_mid: Color,
    pub cloud_far: Color,
    /// Color del cielo que se mezcla con el blanco de los obstáculos.
    pub obstacle_tint: Color,
    /// Qué tanto domina el tinte sobre el blanco base (0.0 = puro blanco, 1.0 = color puro).
    pub tint_strength: f32,
}

impl ColorPalette {
    pub fn azul() -> Self {
        Self {
            sky_stops: [
                (0.00, [0x07, 0x3B, 0x4C]),
                (0.40, [0x0E, 0x7C, 0x92]),
                (0.66, [0x13, 0xA0, 0xAE]),
                (0.84, [0x1E, 0xB6, 0xBE]),
                (1.00, [0x1E, 0xB6, 0xBE]),
            ],
            cloud_near: Color::srgb(0.92, 0.96, 1.00),
            cloud_mid: Color::srgb(0.60, 0.82, 0.90),
            cloud_far: Color::srgb(0.22, 0.60, 0.72),
            obstacle_tint: Color::srgb(0.027, 0.231, 0.298), // #073B4C top del cielo (oscuro)
            tint_strength: 0.5,
        }
    }

    // Crepúsculo neón: cielo índigo-morado → naranja-coral.
    // Nubes en violeta-púrpura (#8E7AC2 → #3A2C66).
    pub fn neon() -> Self {
        Self {
            sky_stops: [
                (0.00, [0x17, 0x15, 0x40]),
                (0.40, [0x46, 0x26, 0x6E]),
                (0.66, [0xC0, 0x41, 0x7E]),
                (0.84, [0xFF, 0x7E, 0x6B]),
                (1.00, [0xFF, 0x7E, 0x6B]),
            ],
            cloud_near: Color::srgb(0.56, 0.48, 0.76),
            cloud_mid: Color::srgb(0.39, 0.32, 0.58),
            cloud_far: Color::srgb(0.23, 0.17, 0.40),
            obstacle_tint: Color::srgb(0.27, 0.15, 0.43), // #46266E morado mid-sky
            tint_strength: 0.5,
        }
    }

    // Rosa: top muy oscuro fucsia → mid vibrante → horizonte durazno claro.
    // El salto de tono oscuro-frío a rosa-cálido da la profundidad que el original perdía.
    pub fn rosa() -> Self {
        Self {
            sky_stops: [
                (0.00, [0x5C, 0x08, 0x44]),
                (0.35, [0xB8, 0x20, 0x78]),
                (0.62, [0xEE, 0x55, 0xA4]),
                (0.82, [0xFF, 0x99, 0xCC]),
                (1.00, [0xFF, 0xBB, 0xDD]),
            ],
            cloud_near: Color::srgb(1.00, 1.00, 1.00),
            cloud_mid: Color::srgb(0.98, 0.82, 0.92),
            cloud_far: Color::srgb(0.93, 0.65, 0.83),
            obstacle_tint: Color::srgb(0.361, 0.031, 0.267), // #5C0844 top del cielo (oscuro)
            tint_strength: 0.5,
        }
    }

    pub fn clear_color(&self) -> Color {
        let [r, g, b] = self.sky_stops[0].1;
        Color::srgb_u8(r, g, b)
    }

    /// Blanco base de los obstáculos (`white_matte`) mezclado con el tinte del cielo.
    /// `tint_strength = 0` → puro blanco. `tint_strength = 0.10` → gota de color.
    pub fn obstacle_color(&self) -> Color {
        let s = self.tint_strength;
        let w = 1.0 - s;
        let Srgba {
            red, green, blue, ..
        } = self.obstacle_tint.to_srgba();
        // Base: (0.92, 0.92, 0.90) de white_matte
        Color::srgb(
            0.92f32.mul_add(w, red * s),
            0.92f32.mul_add(w, green * s),
            0.90f32.mul_add(w, blue * s),
        )
    }
}

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

#[derive(Component)]
pub struct BackgroundCloud {
    base_y: f32,
    base_x: f32,
    // Fracción de cam_y que la nube sigue. 0.60 = sube mucho (cercana). 0.92 = apenas se mueve (lejana).
    parallax_factor: f32,
    // Semirango visible en Y a la profundidad Z de esta capa — para el wrap.
    view_half_y: f32,
    float_phase: f32,
    float_amplitude: f32,
    float_frequency: f32,
}

struct CloudLayer {
    z: f32,
    count: u32,
    color: Color,
    scale_min: f32,
    scale_max: f32,
    x_spread: f32,
    float_amplitude: f32,
    // Fracción de cam_y que sigue la capa. 1.0 = pegada a la cámara (sin parallax).
    // <1.0 → se rezaga: cuanto más bajo, más sube en pantalla cuando la cámara baja.
    parallax_factor: f32,
}

fn cloud_layers(palette: &ColorPalette) -> [CloudLayer; 3] {
    [
        CloudLayer {
            z: -8.0,
            count: 4,
            color: palette.cloud_near,
            scale_min: 0.42,
            scale_max: 0.65,
            x_spread: 4.0,
            float_amplitude: 0.22,
            parallax_factor: 0.60,
        },
        CloudLayer {
            z: -12.0,
            count: 6,
            color: palette.cloud_mid,
            scale_min: 0.36,
            scale_max: 0.56,
            x_spread: 5.5,
            float_amplitude: 0.13,
            parallax_factor: 0.78,
        },
        CloudLayer {
            z: -14.0,
            count: 8,
            color: palette.cloud_far,
            scale_min: 0.28,
            scale_max: 0.44,
            x_spread: 6.5,
            float_amplitude: 0.06,
            parallax_factor: 0.93,
        },
    ]
}

pub fn spawn_clouds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    seed: Res<LevelSeed>,
    palette: Res<ColorPalette>,
) {
    let blob_mesh = meshes.add(Sphere::new(0.5));
    // Técnica cartoon cloud: base plana (z_scale pequeño) + bumps redondeados encima.
    // Tuple: (dx, dy, radius_scale, z_scale_factor)
    // Base: elipsoides anchos y planos → forman el "piso" de la nube
    // Bumps: esferas más esféricas sobre la base → puffs redondeados arriba
    let cloud_blob_offsets: [(f32, f32, f32, f32); 6] = [
        (-0.55, -0.05, 0.90, 0.18),
        (0.00, -0.05, 1.00, 0.18),
        (0.55, -0.05, 0.88, 0.18),
        (-0.28, 0.48, 0.65, 0.60),
        (0.00, 0.58, 0.72, 0.60),
        (0.28, 0.48, 0.60, 0.60),
    ];
    let mut rng = SmallRng::seed_from_u64(seed.0.wrapping_mul(0x6c62272e07bb0142));

    for layer in &cloud_layers(&palette) {
        let mat = materials.add(StandardMaterial {
            base_color: layer.color,
            unlit: true,
            ..default()
        });

        let cols = (layer.count as f32).sqrt().round() as u32;
        let rows = (layer.count + cols - 1) / cols;
        let (x_spread, y_spread) = (layer.x_spread, 7.0_f32);
        let cell_w = x_spread * 2.0 / cols as f32;
        let cell_h = y_spread * 2.0 / rows as f32;

        'outer: for row in 0..rows {
            for col in 0..cols {
                if row * cols + col >= layer.count {
                    break 'outer;
                }
                let cell_x = -x_spread + col as f32 * cell_w;
                let cell_y = -y_spread + row as f32 * cell_h;
                let base_x = cell_x + rng.gen_range(0.0..cell_w);
                // sesgo mínimo; el rango -7..+7 cubre todo el screen height con distribución uniforme
                let y_off = cell_y + rng.gen_range(0.0..cell_h);
                let scale = rng.gen_range(layer.scale_min..layer.scale_max);
                let phase = rng.gen_range(0.0..std::f32::consts::TAU);
                let freq = rng.gen_range(0.04..0.12);
                let amp = layer.float_amplitude * rng.gen_range(0.7..1.3);

                let bm = blob_mesh.clone();
                let m = mat.clone();

                // Semirango visible en Y a esta profundidad: distancia × tan(FOV/2).
                // Cámara Z=2.5, FOV vertical 60° → tan(30°)=0.5774.
                let view_half_y = (2.5 - layer.z) * 0.5774;

                commands
                    .spawn((
                        BackgroundCloud {
                            base_y: y_off,
                            base_x,
                            parallax_factor: layer.parallax_factor,
                            view_half_y,
                            float_phase: phase,
                            float_amplitude: amp,
                            float_frequency: freq,
                        },
                        Transform::from_xyz(base_x, y_off, layer.z),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        InheritedVisibility::VISIBLE,
                        ViewVisibility::default(),
                    ))
                    .with_children(|parent| {
                        for &(bx, by, br, bz) in &cloud_blob_offsets {
                            let s = scale * br;
                            parent.spawn((
                                Mesh3d(bm.clone()),
                                MeshMaterial3d(m.clone()),
                                Transform::from_xyz(bx * scale * 0.5, by * scale * 0.42, 0.0)
                                    .with_scale(Vec3::new(s, s * 0.85, s * bz)),
                            ));
                        }
                    });
            }
        }
    }
}

pub fn update_clouds(
    time: Res<Time>,
    camera: Query<&Transform, With<Camera3d>>,
    mut clouds: Query<
        (&BackgroundCloud, &mut Transform),
        (Without<Camera3d>, Without<SkyQuad>, Without<BackgroundStar>),
    >,
) {
    let Ok(cam) = camera.single() else { return };
    let cam_y = cam.translation.y;
    let t = time.elapsed_secs();

    for (cloud, mut transform) in &mut clouds {
        let float_x = cloud.float_amplitude
            * (cloud.float_phase + t * cloud.float_frequency * std::f32::consts::TAU).sin();

        // Parallax: la nube se rezaga respecto a la cámara según `parallax_factor`.
        // raw_rel_y es la posición relativa a la cámara acumulada desde el inicio.
        // Cuando cam_y baja, las nubes cercanas (factor bajo) se quedan "atrás" y
        // suben más en pantalla; las lejanas (factor alto) la siguen casi por completo.
        let raw_rel_y = cloud.base_y + cam_y * (cloud.parallax_factor - 1.0);

        // Wrap para que las nubes no desaparezcan en carreras largas. El wrap se
        // produce cuando la nube ya está fuera del encuadre (+2 unidades de margen).
        let wrap_range = (cloud.view_half_y + 2.0) * 2.0;
        let wrapped_rel_y =
            (raw_rel_y + wrap_range / 2.0).rem_euclid(wrap_range) - wrap_range / 2.0;

        transform.translation.x = cloud.base_x + float_x;
        transform.translation.y = cam_y + wrapped_rel_y;
    }
}

pub fn spawn_sky(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    palette: Res<ColorPalette>,
) {
    let texture = build_gradient_texture(&mut images, &palette);
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

fn build_gradient_texture(images: &mut Assets<Image>, palette: &ColorPalette) -> Handle<Image> {
    let height: u32 = 256;
    let mut data = Vec::with_capacity((height * 4) as usize);
    for row in 0..height {
        let t = row as f32 / (height - 1) as f32;
        let [r, g, b] = sample_sky_color(&palette.sky_stops, t);
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

pub fn sample_sky_color(stops: &[(f32, [u8; 3])], t: f32) -> [u8; 3] {
    for i in 0..(stops.len() - 1) {
        let (t0, c0) = stops[i];
        let (t1, c1) = stops[i + 1];
        if t <= t1 {
            let f = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
            return [
                lerp_channel(c0[0], c1[0], f),
                lerp_channel(c0[1], c1[1], f),
                lerp_channel(c0[2], c1[2], f),
            ];
        }
    }
    stops.last().unwrap().1
}

fn lerp_channel(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

pub fn update_sky_with_camera(
    camera: Query<&Transform, With<Camera3d>>,
    mut sky: Query<&mut Transform, (With<SkyQuad>, Without<Camera3d>, Without<BackgroundStar>)>,
    mut stars: Query<
        (&BackgroundStar, &mut Transform),
        (
            Without<SkyQuad>,
            Without<Camera3d>,
            Without<BackgroundCloud>,
        ),
    >,
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
