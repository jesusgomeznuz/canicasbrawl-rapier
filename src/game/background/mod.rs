use bevy::prelude::*;

pub mod clouds;
pub mod palette;
pub mod sky;
pub mod stars;

/// El telón de fondo: la paleta, el cielo, las estrellas y las nubes en su lugar.
pub fn paint_the_backdrop(app: &mut App, palette: palette::ColorPalette) {
    app.insert_resource(ClearColor(palette.clear_color()))
        .insert_resource(palette)
        .add_systems(
            Startup,
            (sky::spawn_sky, stars::spawn_stars, clouds::spawn_clouds),
        );
}

/// El telón respira: el cielo sigue a la cámara, las estrellas titilan, las nubes derivan.
pub fn animate_the_backdrop(app: &mut App) {
    app.add_systems(
        Update,
        (
            sky::update_sky_with_camera,
            stars::stars_follow_camera,
            stars::twinkle_stars,
            clouds::update_clouds,
        ),
    );
}
