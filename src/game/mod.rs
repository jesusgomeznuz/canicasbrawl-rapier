//! Los directores del juego — la responsabilidad vive en la estructura:
//!   race/  LA CARRERA: quiénes corren y quién gana (elenco, juez, reglas)
//!   world/ EL MUNDO: la realidad interactuable (pista, trampas, cuerpos)
//!   scene/ LA ESCENA: lo que se ve sin tocarse (telón, encuadre)
//! race_events.rs es la aduana; game.rs es la vida (run + las 3 fases).

mod game;
pub mod race;
pub mod race_events;
pub mod scene;
pub mod world;

pub use game::run;
