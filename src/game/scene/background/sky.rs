use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::palette::ColorPalette;

#[derive(Component)]
pub struct SkyQuad;

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

pub fn update_sky_with_camera(
    camera: Query<&Transform, With<Camera3d>>,
    mut sky: Query<&mut Transform, (With<SkyQuad>, Without<Camera3d>)>,
) {
    let Ok(camera_transform) = camera.single() else { return };
    if let Ok(mut sky_transform) = sky.single_mut() {
        sky_transform.translation.y = camera_transform.translation.y;
    }
}

fn build_gradient_texture(images: &mut Assets<Image>, palette: &ColorPalette) -> Handle<Image> {
    let height: u32 = 256;
    let data: Vec<u8> = (0..height)
        .flat_map(|row| {
            let t = row as f32 / (height - 1) as f32;
            let [r, g, b] = sample_sky_color(&palette.sky_stops, t);
            [r, g, b, 255]
        })
        .collect();
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

fn sample_sky_color(stops: &[(f32, [u8; 3])], t: f32) -> [u8; 3] {
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
