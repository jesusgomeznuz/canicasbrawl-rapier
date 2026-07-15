use bevy::prelude::*;

/// La semilla visual del telón: estrellas y nubes deterministas en ambos
/// mundos (por eso --play también recibe --seed). No siembra el nivel — eso
/// es de los Dice del engine.
#[derive(Resource)]
pub struct BackdropSeed(pub u64);

pub mod clouds;
pub mod palette;
pub mod sky;
pub mod stars;

/// El telón de fondo: la paleta, el cielo, las estrellas y las nubes en su lugar.
pub fn paint_the_backdrop(app: &mut App, palette: palette::ColorPalette, seed: u64) {
    app.insert_resource(BackdropSeed(seed))
        .insert_resource(ClearColor(palette.clear_color()))
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
