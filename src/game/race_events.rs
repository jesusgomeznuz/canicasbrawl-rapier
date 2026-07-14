//! La partitura de canicasbrawl — todo lo que cruza de write-timeline a play:
//!   · movimiento:  poses por TimelineKey, capturadas solas por el engine (timeline.rs)
//!   · nivel:       eventos Module/Finish — receta: nombre + top + seed
//!   · utilería:    eventos Freeze/Shrink/Swap/Bouncy
//!   · identidades: NO viajan — la timeline se escribe anónima, el cast viste en play
//!   · liderazgo:   voice_tracker.json — para elegir carrera, no para reproducirla
//!
//! Este enum es la aduana de la pista de eventos: escribir (`payload`) y leer
//! (`parse`) viven juntos — una sola fuente de verdad del formato del sobre.
//!
//! También circula como evento vivo de Bevy: los contactos reales lo emiten en
//! física y `emit_race_events_from_timeline` lo re-emite en play — la
//! escenografía (staging.rs) lo consume igual en ambos mundos, y
//! `send_race_events_to_timeline` lo escribe a la partitura en --write-timeline
//! sin que ningún sensor arme strings a mano.

use bevy::prelude::*;
use rapier_bevy::{PlayEvent, TimelineEvents};

#[derive(Event, Clone)]
pub enum RaceEvent {
    Freeze { marble: usize, x: f32, y: f32, duration: f32 },
    Shrink { marble: usize, x: f32, y: f32, duration: f32 },
    Swap { marble_a: usize, marble_b: usize, x: f32, y: f32 },
    Bouncy { x: f32, y: f32, amplitude: f32 },
    Module { name: String, top: f32, seed: u64 },
    Finish { top: f32 },
}

impl RaceEvent {
    pub fn payload(&self) -> String {
        match self {
            RaceEvent::Freeze { marble, x, y, duration } => {
                format!("freeze {marble} {x:.3} {y:.3} {duration}")
            }
            RaceEvent::Shrink { marble, x, y, duration } => {
                format!("shrink {marble} {x:.3} {y:.3} {duration}")
            }
            RaceEvent::Swap { marble_a, marble_b, x, y } => {
                format!("swap {marble_a} {marble_b} {x:.3} {y:.3}")
            }
            RaceEvent::Bouncy { x, y, amplitude } => format!("bouncy {x} {y} {amplitude}"),
            RaceEvent::Module { name, top, seed } => format!("module {name} {top} {seed}"),
            RaceEvent::Finish { top } => format!("finish {top}"),
        }
    }

    pub fn parse(payload: &str) -> RaceEvent {
        RaceEvent::try_parse(payload).unwrap_or_else(|| {
            panic!("evento de carrera ilegible: '{payload}' — write-timeline y play hablan idiomas distintos")
        })
    }

    fn try_parse(payload: &str) -> Option<RaceEvent> {
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
            ["finish", top] => Some(RaceEvent::Finish { top: top.parse().ok()? }),
            _ => None,
        }
    }
}

pub fn send_race_events_to_timeline(
    mut events: EventReader<RaceEvent>,
    mut timeline: Option<ResMut<TimelineEvents>>,
) {
    let Some(timeline) = timeline.as_deref_mut() else { return };
    for event in events.read() {
        timeline.0.push(event.payload());
    }
}

pub fn emit_race_events_from_timeline(
    mut wire: EventReader<PlayEvent>,
    mut events: EventWriter<RaceEvent>,
) {
    for PlayEvent(payload) in wire.read() {
        events.write(RaceEvent::parse(payload));
    }
}

/// La banda de eventos — igual en todos los mundos: la actuación re-emite, la
/// física escribe, la escenografía monta. Encadenada en el mismo tick.
pub fn run_the_event_band(app: &mut App) {
    app.add_event::<RaceEvent>();
    app.add_event::<PlayEvent>();
    app.add_systems(
        FixedUpdate,
        (
            emit_race_events_from_timeline,
            send_race_events_to_timeline,
            super::staging::stage_race_events,
        )
            .chain()
            .after(bevy_rapier3d::plugin::PhysicsSet::Writeback),
    );
}
