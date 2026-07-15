//! La partitura de canicasbrawl — todo lo que cruza de write-timeline a play:
//!   · movimiento:  poses por TimelineKey, capturadas solas por el engine (timeline.rs)
//!   · nivel:       eventos Module/Finish — receta: nombre + top + seed
//!   · utilería:    eventos Freeze/Shrink/Swap/Bouncy
//!   · identidades: NO viajan — la timeline se escribe anónima, el cast viste en play
//!   · liderazgo:   voice_tracker.json — para elegir carrera, no para reproducirla
//!
//! Este enum ES la aduana de la pista de eventos: el derive de serde genera
//! la ida (evento → renglón JSON) y la vuelta (renglón → evento) desde la
//! estructura misma — no hay formato a mano que pueda desalinearse. La BANDA
//! que lo transporta (replay → record → escenografía) es estructura y vive en
//! el engine (`rapier_bevy::run_the_event_band`); la escenografía que lo
//! consume vive en staging.rs.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Event, Clone, Serialize, Deserialize)]
pub enum RaceEvent {
    Freeze {
        marble: usize,
        x: f32,
        y: f32,
        duration: f32,
    },
    Shrink {
        marble: usize,
        x: f32,
        y: f32,
        duration: f32,
    },
    Swap {
        marble_a: usize,
        marble_b: usize,
        x: f32,
        y: f32,
    },
    Bouncy {
        x: f32,
        y: f32,
        amplitude: f32,
    },
    Module {
        name: String,
        top: f32,
        seed: u64,
    },
    Finish {
        top: f32,
    },
}
