//! La partitura de canicasbrawl — todo lo que cruza de write-timeline a play:
//!   · movimiento:  poses por TimelineKey, capturadas solas por el engine (timeline.rs)
//!   · nivel:       eventos Module/Finish — receta: nombre + top + seed
//!   · utilería:    eventos Freeze/Shrink/Swap/Bouncy
//!   · identidades: NO viajan — la timeline se escribe anónima, el cast viste en play
//!   · liderazgo:   voice_tracker.json — para elegir carrera, no para reproducirla
//!
//! Este enum es la aduana de la pista de eventos: escribir (`payload`) y leer
//! (`parse`) viven juntos — una sola fuente de verdad del formato del sobre.
//! La BANDA que lo transporta (actuación → buzón → escenografía) es estructura
//! y vive en el engine (`rapier_bevy::run_the_event_band`); aquí solo queda el
//! vocabulario, y la escenografía que lo consume vive en staging.rs.

use bevy::prelude::*;
use rapier_bevy::TimelineVocabulary;

#[derive(Event, Clone)]
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

impl TimelineVocabulary for RaceEvent {
    fn payload(&self) -> String {
        match self {
            RaceEvent::Freeze {
                marble,
                x,
                y,
                duration,
            } => {
                format!("freeze {marble} {x:.3} {y:.3} {duration}")
            }
            RaceEvent::Shrink {
                marble,
                x,
                y,
                duration,
            } => {
                format!("shrink {marble} {x:.3} {y:.3} {duration}")
            }
            RaceEvent::Swap {
                marble_a,
                marble_b,
                x,
                y,
            } => {
                format!("swap {marble_a} {marble_b} {x:.3} {y:.3}")
            }
            RaceEvent::Bouncy { x, y, amplitude } => format!("bouncy {x} {y} {amplitude}"),
            RaceEvent::Module { name, top, seed } => format!("module {name} {top} {seed}"),
            RaceEvent::Finish { top } => format!("finish {top}"),
        }
    }

    fn parse(payload: &str) -> Option<RaceEvent> {
        let parts: Vec<&str> = payload.split_whitespace().collect();
        match parts.as_slice() {
            ["freeze", marble, x, y, duration] => Some(RaceEvent::Freeze {
                marble: marble.parse().ok()?,
                x: x.parse().ok()?,
                y: y.parse().ok()?,
                duration: duration.parse().ok()?,
            }),
            ["shrink", marble, x, y, duration] => Some(RaceEvent::Shrink {
                marble: marble.parse().ok()?,
                x: x.parse().ok()?,
                y: y.parse().ok()?,
                duration: duration.parse().ok()?,
            }),
            ["swap", marble_a, marble_b, x, y] => Some(RaceEvent::Swap {
                marble_a: marble_a.parse().ok()?,
                marble_b: marble_b.parse().ok()?,
                x: x.parse().ok()?,
                y: y.parse().ok()?,
            }),
            ["bouncy", x, y, amplitude] => Some(RaceEvent::Bouncy {
                x: x.parse().ok()?,
                y: y.parse().ok()?,
                amplitude: amplitude.parse().ok()?,
            }),
            ["module", name, top, seed] => Some(RaceEvent::Module {
                name: name.to_string(),
                top: top.parse().ok()?,
                seed: seed.parse().ok()?,
            }),
            ["finish", top] => Some(RaceEvent::Finish {
                top: top.parse().ok()?,
            }),
            _ => None,
        }
    }
}
