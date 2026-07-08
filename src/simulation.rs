use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_rapier3d::plugin::PhysicsSet;
use rapier_bevy::{
    GameAppConfig, ReplayEvent, SimulationMode, bake_duration, physics_enabled,
    random_physics_game_app, record_duration,
};

use crate::args::RosterSpec;
use crate::game;
use crate::game::background::palette::ColorPalette;
use crate::game::roster::MarbleConfig;
use crate::production;

pub fn run(mode: SimulationMode, seed: u64, spec: RosterSpec, palette: ColorPalette) {
    let roster = resolve_roster(spec);
    println!("Level seed: {}", seed);

    let mut app = random_physics_game_app(
        mode,
        GameAppConfig {
            title: "CanicasBrawl",
            resolution: (540.0, 960.0),
        },
    );
    on_start(&mut app, seed, roster, palette);
    on_step(&mut app);
    on_frame_update(&mut app);
    after_frame_update(&mut app);
    on_exit(&mut app);

    app.run();
}

fn on_start(app: &mut App, seed: u64, roster: Vec<MarbleConfig>, palette: ColorPalette) {
    let clear = palette.clear_color();
    app.insert_resource(ClearColor(clear))
        .insert_resource(palette)
        .insert_resource(production::voice_tracker::VoiceTracker::default())
        .insert_resource(production::stall_detector::StallDetector::default())
        .insert_resource(game::finish::RaceResult::default())
        .insert_resource(game::finish::FinishLineY::default())
        .insert_resource(game::leader::RaceLeader::default())
        .insert_resource(game::roster::Roster(roster))
        .insert_resource(game::world::level_generation::LevelSeed(seed))
        .insert_resource(game::world::level_generation::FinishTarget(
            finish_target_secs(),
        ))
        .add_systems(
            Startup,
            (
                game::camera::spawn_camera_and_lights,
                game::leader::spawn_crown,
                game::world::setup::setup,
                game::world::setup::set_gravity,
                game::background::sky::spawn_sky,
                game::background::stars::spawn_stars,
                game::background::clouds::spawn_clouds,
                game::hud::spawn_hud,
            ),
        );
}

fn on_step(app: &mut App) {
    app.add_event::<game::baked_events::BakedEvent>();
    app.add_event::<ReplayEvent>();
    app.add_systems(
        FixedUpdate,
        (
            (
                game::finish::check_finish_crossing,
                game::leader::update_race_leader,
                production::voice_tracker::track_race_leader,
            )
                .chain(),
            (
                game::baked_events::reemit_baked_events,
                game::baked_events::record_baked_events,
                game::staging::stage_baked_events,
            )
                .chain(),
        )
            .after(PhysicsSet::Writeback),
    )
    .add_systems(
        FixedUpdate,
        (
            game::sensors::freeze::try_unfreeze,
            game::sensors::shrink::try_unshrink,
            game::sensors::swap::fade_swap_rings,
            game::sensors::icons::spin_icons,
            game::sensors::bouncy::animate_bounce_pulse,
            game::sensors::bouncy::tick_bounce_cooldown,
            game::camera::camera_follows_lowest_marble,
        )
            .after(PhysicsSet::Writeback),
    );

    if physics_enabled() {
        react_to_real_collisions(app);
    }
}

fn on_frame_update(app: &mut App) {
    app.add_systems(
        Update,
        (
            game::background::sky::update_sky_with_camera,
            game::background::stars::stars_follow_camera,
            game::background::stars::twinkle_stars,
            game::background::clouds::update_clouds,
            game::sensors::freeze::manage_freeze_badges,
            game::sensors::shrink::manage_shrink_badges,
        ),
    )
    .add_systems(Update, game::hud::update_hud)
    .add_systems(Update, production::stall_detector::detect_stall);
}

fn after_frame_update(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (
            game::marbles::update_marble_labels,
            game::leader::crown_follows_leader,
            game::sensors::badges::update_badges,
        )
            .after(TransformSystem::TransformPropagate),
    );
}

fn on_exit(app: &mut App) {
    app.add_systems(Last, production::voice_tracker::save_voice_tracker_on_exit);
}

fn react_to_real_collisions(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            game::world::level_generation::generate_level,
            game::world::level_generation::disable_modules_above_screen,
            game::sensors::freeze::on_freeze_contact,
            game::sensors::shrink::on_shrink_contact,
            game::sensors::swap::on_swap_contact,
            game::sensors::bouncy::trigger_bouncy_pulse,
        )
            .after(PhysicsSet::Writeback)
            .before(game::baked_events::reemit_baked_events),
    );
}

fn resolve_roster(spec: RosterSpec) -> Vec<MarbleConfig> {
    match spec {
        RosterSpec::Default => game::roster::build_roster(None),
        RosterSpec::Characters(names) => game::roster::build_roster(Some(names)),
        RosterSpec::Slots(n) => game::roster::slots_roster(n),
    }
    .unwrap_or_else(|err| {
        eprintln!("Error de roster: {err}");
        std::process::exit(1);
    })
}

fn finish_target_secs() -> f32 {
    let video_secs = record_duration()
        .or_else(bake_duration)
        .map(|d| d as f32)
        .unwrap_or(60.0);
    let tail_of_falling_marbles_secs = 12.0;
    video_secs - tail_of_falling_marbles_secs
}
