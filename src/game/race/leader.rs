use bevy::prelude::*;
use bevy::text::TextLayoutInfo;

use super::finish::RaceResult;
use super::labels::MarbleLabel;
use crate::game::world::marbles::Marble;

#[derive(Resource, Default)]
pub struct RaceLeader {
    pub marble: Option<Entity>,
    locked: bool,
}

#[derive(Component)]
pub struct LeaderCrown;

pub fn update_race_leader(
    marbles: Query<(Entity, &Transform), With<Marble>>,
    result: Res<RaceResult>,
    mut leader: ResMut<RaceLeader>,
) {
    if leader.locked {
        return;
    }

    if let Some((winner, _)) = result.finishers.first() {
        leader.marble = Some(*winner);
        leader.locked = true;
        return;
    }

    if let Some((front_runner, _)) = lowest_marble(&marbles) {
        leader.marble = Some(front_runner);
    }
}

pub fn spawn_crown(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut assets_loading: Option<ResMut<rapier_bevy::AssetsLoading>>,
) {
    let handle: Handle<Image> = asset_server.load("img/crown.png");
    if let Some(al) = assets_loading.as_deref_mut() {
        al.0.push(handle.clone().untyped());
    }
    commands.spawn((
        Sprite {
            image: handle,
            custom_size: Some(Vec2::splat(28.0)),
            ..default()
        },
        Transform::default(),
        Visibility::Hidden,
        LeaderCrown,
    ));
}

pub fn crown_follows_leader(
    leader: Res<RaceLeader>,
    labels: Query<(&Transform, &TextLayoutInfo, &MarbleLabel), Without<LeaderCrown>>,
    mut crown: Query<(&mut Transform, &mut Visibility), (With<LeaderCrown>, Without<MarbleLabel>)>,
) {
    let Ok((mut crown_transform, mut crown_visibility)) = crown.single_mut() else {
        return;
    };
    let Some(leader_marble) = leader.marble else {
        *crown_visibility = Visibility::Hidden;
        return;
    };
    let crown_size = 28.0_f32;
    let gap = 6.0_f32;
    for (label_transform, layout, MarbleLabel(marble_entity)) in &labels {
        if *marble_entity == leader_marble {
            crown_transform.translation.x =
                label_transform.translation.x - layout.size.x / 2.0 - crown_size / 2.0 - gap;
            crown_transform.translation.y = label_transform.translation.y;
            crown_transform.translation.z = label_transform.translation.z;
            *crown_visibility = Visibility::Visible;
            return;
        }
    }
    *crown_visibility = Visibility::Hidden;
}

fn lowest_marble(marbles: &Query<(Entity, &Transform), With<Marble>>) -> Option<(Entity, f32)> {
    marbles
        .iter()
        .map(|(e, t)| (e, t.translation.y))
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
}
