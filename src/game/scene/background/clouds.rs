use bevy::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;

use super::palette::ColorPalette;
use super::BackdropSeed;

#[derive(Component)]
pub struct BackgroundCloud {
    base_y: f32,
    base_x: f32,
    // Fracción de camera_y que la nube sigue. 0.60 = sube mucho (cercana). 0.92 = apenas se mueve (lejana).
    parallax_factor: f32,
    // Semirango visible en Y a la profundidad Z de esta capa — para el wrap.
    view_half_y: f32,
    float_phase: f32,
    float_amplitude: f32,
    float_frequency: f32,
}

pub fn spawn_clouds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    seed: Res<BackdropSeed>,
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
        let layer_material = materials.add(StandardMaterial {
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
                let y_offset_in_cell = cell_y + rng.gen_range(0.0..cell_h);
                let scale = rng.gen_range(layer.scale_min..layer.scale_max);
                let phase = rng.gen_range(0.0..std::f32::consts::TAU);
                let freq = rng.gen_range(0.04..0.12);
                let amp = layer.float_amplitude * rng.gen_range(0.7..1.3);

                let blob_mesh_for_cloud = blob_mesh.clone();
                let material_for_cloud = layer_material.clone();

                // Semirango visible en Y a esta profundidad: distancia × tan(FOV/2).
                // Cámara Z=2.5, FOV vertical 60° → tan(30°)=0.5774.
                let view_half_y = (2.5 - layer.z) * 0.5774;

                commands
                    .spawn((
                        BackgroundCloud {
                            base_y: y_offset_in_cell,
                            base_x,
                            parallax_factor: layer.parallax_factor,
                            view_half_y,
                            float_phase: phase,
                            float_amplitude: amp,
                            float_frequency: freq,
                        },
                        Transform::from_xyz(base_x, y_offset_in_cell, layer.z),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        InheritedVisibility::VISIBLE,
                        ViewVisibility::default(),
                    ))
                    .with_children(|parent| {
                        for &(bx, by, br, bz) in &cloud_blob_offsets {
                            let s = scale * br;
                            parent.spawn((
                                Mesh3d(blob_mesh_for_cloud.clone()),
                                MeshMaterial3d(material_for_cloud.clone()),
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
    mut clouds: Query<(&BackgroundCloud, &mut Transform), Without<Camera3d>>,
) {
    let Ok(camera_transform) = camera.single() else { return };
    let camera_y = camera_transform.translation.y;
    let elapsed = time.elapsed_secs();

    for (cloud, mut transform) in &mut clouds {
        let float_x = cloud.float_amplitude
            * (cloud.float_phase + elapsed * cloud.float_frequency * std::f32::consts::TAU).sin();

        // Parallax: la nube se rezaga respecto a la cámara según `parallax_factor`.
        // raw_rel_y es la posición relativa a la cámara acumulada desde el inicio.
        // Cuando camera_y baja, las nubes cercanas (factor bajo) se quedan "atrás" y
        // suben más en pantalla; las lejanas (factor alto) la siguen casi por completo.
        let raw_rel_y = cloud.base_y + camera_y * (cloud.parallax_factor - 1.0);

        // Wrap para que las nubes no desaparezcan en carreras largas. El wrap se
        // produce cuando la nube ya está fuera del encuadre (+2 unidades de margen).
        let wrap_range = (cloud.view_half_y + 2.0) * 2.0;
        let wrapped_rel_y =
            (raw_rel_y + wrap_range / 2.0).rem_euclid(wrap_range) - wrap_range / 2.0;

        transform.translation.x = cloud.base_x + float_x;
        transform.translation.y = camera_y + wrapped_rel_y;
    }
}

struct CloudLayer {
    z: f32,
    count: u32,
    color: Color,
    scale_min: f32,
    scale_max: f32,
    x_spread: f32,
    float_amplitude: f32,
    // Fracción de camera_y que sigue la capa. 1.0 = pegada a la cámara (sin parallax).
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
