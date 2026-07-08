//! La partitura de canicasbrawl — todo lo que cruza del bake al replay:
//!   · movimiento:  poses por BakeKey, capturadas solas por el engine (timeline.rs)
//!   · nivel:       eventos Module/Finish — receta: nombre + top + seed
//!   · utilería:    eventos Freeze/Shrink/Swap/Bouncy
//!   · identidades: NO viajan — el bake es anónimo, el cast viste en replay
//!   · liderazgo:   voice_tracker.json — para elegir carrera, no para reproducirla
//!
//! Este enum es la aduana de la pista de eventos: escribir (`payload`) y leer
//! (`parse`) viven juntos — una sola fuente de verdad del formato del sobre.

pub enum BakedEvent {
    Freeze { marble: usize, x: f32, y: f32, duration: f32 },
    Shrink { marble: usize, x: f32, y: f32, duration: f32 },
    Swap { marble_a: usize, marble_b: usize, x: f32, y: f32 },
    Bouncy { x: f32, y: f32, amplitude: f32 },
    Module { name: String, top: f32, seed: u64 },
    Finish { top: f32 },
}

impl BakedEvent {
    pub fn payload(&self) -> String {
        match self {
            BakedEvent::Freeze { marble, x, y, duration } => {
                format!("freeze {marble} {x:.3} {y:.3} {duration}")
            }
            BakedEvent::Shrink { marble, x, y, duration } => {
                format!("shrink {marble} {x:.3} {y:.3} {duration}")
            }
            BakedEvent::Swap { marble_a, marble_b, x, y } => {
                format!("swap {marble_a} {marble_b} {x:.3} {y:.3}")
            }
            BakedEvent::Bouncy { x, y, amplitude } => format!("bouncy {x} {y} {amplitude}"),
            BakedEvent::Module { name, top, seed } => format!("module {name} {top} {seed}"),
            BakedEvent::Finish { top } => format!("finish {top}"),
        }
    }

    pub fn parse(payload: &str) -> BakedEvent {
        BakedEvent::try_parse(payload).unwrap_or_else(|| {
            panic!("evento horneado ilegible: '{payload}' — bake y replay hablan idiomas distintos")
        })
    }

    fn try_parse(payload: &str) -> Option<BakedEvent> {
        let parts: Vec<&str> = payload.split_whitespace().collect();
        match parts.as_slice() {
            ["freeze", marble, x, y, duration] => Some(BakedEvent::Freeze {
                marble: marble.parse().ok()?,
                x: x.parse().ok()?,
                y: y.parse().ok()?,
                duration: duration.parse().ok()?,
            }),
            ["shrink", marble, x, y, duration] => Some(BakedEvent::Shrink {
                marble: marble.parse().ok()?,
                x: x.parse().ok()?,
                y: y.parse().ok()?,
                duration: duration.parse().ok()?,
            }),
            ["swap", marble_a, marble_b, x, y] => Some(BakedEvent::Swap {
                marble_a: marble_a.parse().ok()?,
                marble_b: marble_b.parse().ok()?,
                x: x.parse().ok()?,
                y: y.parse().ok()?,
            }),
            ["bouncy", x, y, amplitude] => Some(BakedEvent::Bouncy {
                x: x.parse().ok()?,
                y: y.parse().ok()?,
                amplitude: amplitude.parse().ok()?,
            }),
            ["module", name, top, seed] => Some(BakedEvent::Module {
                name: name.to_string(),
                top: top.parse().ok()?,
                seed: seed.parse().ok()?,
            }),
            ["finish", top] => Some(BakedEvent::Finish { top: top.parse().ok()? }),
            _ => None,
        }
    }
}
