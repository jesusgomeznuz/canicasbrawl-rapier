use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_rapier3d::plugin::PhysicsSet;
use rapier_bevy::{
    GameAppConfig, SimulationMode, bake_duration, random_physics_game_app, record_duration,
    replay_path,
};

use crate::args::RosterSpec;
use crate::game;
use crate::game::background::ColorPalette;
use crate::game::marbles::MarbleConfig;
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
    on_final_positions(&mut app);
    on_exit(&mut app);

    app.run();
}

fn resolve_roster(spec: RosterSpec) -> Vec<MarbleConfig> {
    match spec {
        RosterSpec::Default => game::marbles::build_roster(None),
        RosterSpec::Characters(names) => game::marbles::build_roster(Some(names)),
        RosterSpec::Slots(n) => game::marbles::slots_roster(n),
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

fn on_start(app: &mut App, seed: u64, roster: Vec<MarbleConfig>, palette: ColorPalette) {
    let clear = palette.clear_color();
    app.insert_resource(ClearColor(clear))
        .insert_resource(palette)
        .insert_resource(production::voice_tracker::VoiceTracker::default())
        .insert_resource(production::stall_detector::StallDetector::default())
        .insert_resource(game::finish::RaceResult::default())
        .insert_resource(game::finish::FinishLineY::default())
        .insert_resource(game::leader::RaceLeader::default())
        .insert_resource(game::marbles::Roster(roster))
        .insert_resource(game::world::LevelSeed(seed))
        .insert_resource(game::world::FinishTarget(finish_target_secs()))
        .add_systems(
            Startup,
            (
                game::camera::spawn_camera_and_lights,
                game::camera::spawn_leader_crown,
                game::world::setup,
                game::world::set_gravity,
                game::background::spawn_sky,
                game::background::spawn_stars,
                game::background::spawn_clouds,
                game::hud::spawn_hud,
            ),
        );
}

fn on_step(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            game::finish::check_finish_crossing,
            game::leader::update_race_leader,
            production::voice_tracker::track_race_leader,
        )
            .chain()
            .after(PhysicsSet::Writeback),
    )
    .add_systems(
        FixedUpdate,
        (
            game::effects::try_unfreeze,
            game::effects::try_unshrink,
            game::effects::fade_swap_rings,
            game::effects::spin_icons,
            game::bouncy::animate_bounce_pulse,
            game::bouncy::tick_bounce_cooldown,
            game::camera::camera_follows_lowest_marble,
        )
            .after(PhysicsSet::Writeback),
    );

    match replay_path() {
        Some(_) => register_replay_driven_systems(app),
        None => register_physics_contact_systems(app),
    }
}

fn on_frame_update(app: &mut App) {
    app.add_systems(
        Update,
        (
            game::background::update_sky_with_camera,
            game::background::twinkle_stars,
            game::background::update_clouds,
            game::effect_timers::manage_freeze_badges,
            game::effect_timers::manage_shrink_badges,
            game::effect_timers::update_badges,
        ),
    )
    .add_systems(Update, game::hud::update_hud)
    .add_systems(Update, production::stall_detector::detect_stall);
}

fn on_final_positions(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (
            game::camera::update_marble_labels,
            game::camera::update_leader_crown,
        )
            .after(TransformSystem::TransformPropagate),
    );
}

fn on_exit(app: &mut App) {
    app.add_systems(Last, production::voice_tracker::save_voice_tracker_on_exit);
}

fn register_physics_contact_systems(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            game::world::generate_level,
            game::world::disable_modules_above_screen,
            game::effects::on_freeze_contact,
            game::effects::on_shrink_contact,
            game::effects::on_swap_contact,
            game::bouncy::trigger_bouncy_pulse,
        )
            .after(PhysicsSet::Writeback),
    );
}

fn register_replay_driven_systems(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            game::replay_effects::apply_replay_effects,
            game::replay_effects::expire_replay_freezes,
        )
            .after(PhysicsSet::Writeback),
    );
}
