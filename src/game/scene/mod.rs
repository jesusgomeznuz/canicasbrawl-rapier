use bevy::prelude::*;

pub mod background;
pub mod camera;

/// Se prepara todo lo visual: la semilla del telón, la paleta, y el cielo,
/// las estrellas y las nubes en su lugar. (La cámara se COLOCA en
/// build_world — también es una entidad del mundo.)
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

/// La escena se actualiza: el cielo sigue a la cámara, las estrellas titilan,
/// las nubes derivan — y las etiquetas de las canicas se re-proyectan a
/// pantalla cuando las posiciones ya quedaron firmes (utilería de la carrera
/// mantenida por la escena: el acto cruza oficios).
pub fn update_scene(app: &mut App) {
    app.add_systems(
        Update,
        (
            background::sky::update_sky_with_camera,
            background::stars::stars_follow_camera,
            background::stars::twinkle_stars,
            background::clouds::update_clouds,
        ),
    );
    app.add_systems(
        PostUpdate,
        crate::game::race::labels::update_marble_labels
            .after(bevy::transform::TransformSystem::TransformPropagate),
    );
}
