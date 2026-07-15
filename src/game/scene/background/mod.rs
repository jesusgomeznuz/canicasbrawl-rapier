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
