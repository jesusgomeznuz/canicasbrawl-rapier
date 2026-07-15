use bevy::prelude::*;

pub mod background;
pub mod camera;

pub use background::animate_the_backdrop;

/// Se prepara todo lo visual: la semilla del telón, la paleta, y el cielo,
/// las estrellas y las nubes en su lugar. (La cámara se COLOCA en
/// build_the_world — también es una entidad del mundo.)
pub fn prepare_the_scene(app: &mut App, palette: background::palette::ColorPalette, seed: u64) {
    app.insert_resource(background::BackdropSeed(seed))
        .insert_resource(ClearColor(palette.clear_color()))
        .insert_resource(palette)
        .add_systems(
            Startup,
            (
                background::sky::spawn_sky,
                background::stars::spawn_stars,
                background::clouds::spawn_clouds,
            ),
        );
}
