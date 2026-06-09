use bevy::prelude::*;

use super::leader::RaceLeader;
use super::marbles::{Marble, MarbleName};

// Colores del diseño — paleta "azul"
const ACCENT: Color = Color::srgb(1.0, 0.824, 0.369); // #FFD25E
const GLASS_BG: Color = Color::srgba(1.0, 1.0, 1.0, 0.16);
const GLASS_BORDER: Color = Color::srgba(1.0, 1.0, 1.0, 0.38);
const TRACK_BG: Color = Color::srgba(1.0, 1.0, 1.0, 0.28);
const DOT_MUTED: Color = Color::srgba(1.0, 1.0, 1.0, 0.50);
const DOT_BORDER_MUTED: Color = Color::srgba(1.0, 1.0, 1.0, 0.30);

const TRACK_HEIGHT: f32 = 300.0;
const DOT_SIZE: f32 = 16.0;

#[derive(Component)]
pub struct ProgressTrack;

#[derive(Component)]
pub struct ProgressDot(pub Entity);

#[derive(Component)]
pub struct LeaderBadge;

pub fn spawn_hud(mut commands: Commands) {
    // Raíz del HUD — cubre toda la pantalla, invisible
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .with_children(|root| {
            spawn_progress_track(root);
            spawn_leader_badge(root);
        });
}

fn spawn_progress_track(root: &mut ChildSpawnerCommands) {
    // La barra lateral derecha — viewport fijo 960px, centro=480, barra=300 → top=330
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(22.0),
            top: Val::Px(330.0),
            width: Val::Px(8.0),
            height: Val::Px(TRACK_HEIGHT),
            ..default()
        },
        BackgroundColor(TRACK_BG),
        BorderRadius::all(Val::Px(8.0)),
        ProgressTrack,
    ));
}

fn spawn_leader_badge(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(18.0),
            top: Val::Px(24.0),
            padding: UiRect { left: Val::Px(12.0), right: Val::Px(12.0), top: Val::Px(7.0), bottom: Val::Px(7.0) },
            border: UiRect::all(Val::Px(1.0)),
            align_items: AlignItems::Center,
            column_gap: Val::Px(7.0),
            ..default()
        },
        BackgroundColor(GLASS_BG),
        BorderColor(GLASS_BORDER),
        BorderRadius::all(Val::Px(16.0)),
    ))
    .with_children(|badge| {
        badge.spawn((
            Text::new("👑  —"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
            LeaderBadge,
        ));
    });
}

// Añade un punto al track por cada canica — corre una sola vez el primer frame
// en que los marbles ya existen.
pub fn init_marble_dots(
    mut commands: Commands,
    track: Query<Entity, With<ProgressTrack>>,
    marbles: Query<Entity, With<Marble>>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    let marble_list: Vec<Entity> = marbles.iter().collect();
    if marble_list.is_empty() {
        return;
    }
    let Ok(track_entity) = track.single() else {
        return;
    };

    for marble_entity in marble_list {
        commands.entity(track_entity).with_children(|track| {
            track.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(-(DOT_SIZE - 8.0) / 2.0),
                    top: Val::Px(TRACK_HEIGHT / 2.0 - DOT_SIZE / 2.0),
                    width: Val::Px(DOT_SIZE),
                    height: Val::Px(DOT_SIZE),
                    border: UiRect::all(Val::Px(2.5)),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                BorderColor(ACCENT),
                BorderRadius::all(Val::Percent(50.0)),
                ProgressDot(marble_entity),
            ));
        });
    }
    *done = true;
}

pub fn update_hud(
    marbles: Query<(Entity, &Transform), With<Marble>>,
    names: Query<&MarbleName, With<Marble>>,
    leader: Res<RaceLeader>,
    mut dots: Query<(&ProgressDot, &mut Node, &mut BackgroundColor, &mut BorderColor)>,
    mut badge: Query<&mut Text, With<LeaderBadge>>,
) {
    update_progress_dots(&marbles, &leader, &mut dots);
    update_leader_text(&names, &leader, &mut badge);
}

fn update_progress_dots(
    marbles: &Query<(Entity, &Transform), With<Marble>>,
    leader: &Res<RaceLeader>,
    dots: &mut Query<(&ProgressDot, &mut Node, &mut BackgroundColor, &mut BorderColor)>,
) {
    let positions: Vec<(Entity, f32)> = marbles.iter().map(|(e, t)| (e, t.translation.y)).collect();
    if positions.is_empty() {
        return;
    }

    let best_y = positions.iter().map(|(_, y)| *y).fold(f32::INFINITY, f32::min);
    let worst_y = positions.iter().map(|(_, y)| *y).fold(f32::NEG_INFINITY, f32::max);
    let spread = worst_y - best_y;

    for (ProgressDot(marble_e), mut node, mut bg, mut border) in dots {
        let Some((_, marble_y)) = positions.iter().find(|(e, _)| *e == *marble_e) else {
            continue;
        };

        let progress = if spread < 0.001 {
            0.5
        } else {
            (marble_y - worst_y) / (best_y - worst_y) // 0=último, 1=líder
        };

        // progress=1 → top=0 (cima del track), progress=0 → top fondo
        node.top = Val::Px((1.0 - progress) * (TRACK_HEIGHT - DOT_SIZE));

        let is_leader = leader.marble == Some(*marble_e);
        if is_leader {
            bg.0 = Color::WHITE;
            border.0 = ACCENT;
        } else {
            bg.0 = DOT_MUTED;
            border.0 = DOT_BORDER_MUTED;
        }
    }
}

fn update_leader_text(
    names: &Query<&MarbleName, With<Marble>>,
    leader: &Res<RaceLeader>,
    badge: &mut Query<&mut Text, With<LeaderBadge>>,
) {
    let Ok(mut text) = badge.single_mut() else {
        return;
    };
    let label = match leader.marble.and_then(|e| names.get(e).ok()) {
        Some(MarbleName(name)) => format!("👑  {}", name),
        None => "👑  —".to_string(),
    };
    **text = label;
}
