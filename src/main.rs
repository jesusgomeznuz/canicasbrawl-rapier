mod content;
mod game;
mod production;

use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_rapier3d::plugin::PhysicsSet;
use rapier_bevy::{GameAppConfig, SimMode, bake_duration, game_app, preprocess_assets, record_duration, replay_path};

// Profundidad Z de canicas y plataformas — temporal mientras se calibran las físicas
pub(crate) const UNIT: f32 = 0.35;

enum Command {
    ProcessModules,
    Preprocess,
    Sim(SimMode, u64, RosterSpec, game::background::ColorPalette),
}

/// Quién corre la carrera. `Slots` es el modo casting: física anónima
/// (slot_0..slot_{N-1}) cuyo reparto se decide al renderizar — la posición i
/// de `--characters` viste al slot_i de la timeline horneada.
enum RosterSpec {
    Default,
    Characters(Vec<String>),
    Slots(usize),
}

fn parse_command() -> Command {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--process-modules") {
        return Command::ProcessModules;
    }
    if args.iter().any(|a| a == "--preprocess") {
        return Command::Preprocess;
    }
    let mode = if args.iter().any(|a| a == "--sim-raw") { SimMode::Raw } else { SimMode::Precomputed };
    let palette = parse_palette(&args);
    Command::Sim(mode, parse_seed(&args), parse_roster_spec(&args), palette)
}

fn parse_palette(args: &[String]) -> game::background::ColorPalette {
    if args.iter().any(|a| a == "--neon") {
        game::background::ColorPalette::neon()
    } else if args.iter().any(|a| a == "--rosa") {
        game::background::ColorPalette::rosa()
    } else {
        game::background::ColorPalette::azul()
    }
}

/// `--characters Name1,Name2,...` → lista de nombres canónicos (CamelCase). Sin el
/// flag devuelve None y el juego usa el roster por defecto.
fn parse_characters(args: &[String]) -> Option<Vec<String>> {
    args.iter().position(|a| a == "--characters")
        .and_then(|i| args.get(i + 1))
        .map(|list| list.split(',').map(|s| s.trim().to_string()).collect())
}

fn parse_roster_spec(args: &[String]) -> RosterSpec {
    let slots = args.iter().position(|a| a == "--slots")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok());
    match (slots, parse_characters(args)) {
        (Some(_), Some(_)) => {
            eprintln!("--slots y --characters son excluyentes: la física se hornea anónima \
                       con --slots y se viste con --characters al renderizar la timeline.");
            std::process::exit(1);
        }
        (Some(n), None)      => RosterSpec::Slots(n),
        (None, Some(names))  => RosterSpec::Characters(names),
        (None, None)         => RosterSpec::Default,
    }
}

fn parse_seed(args: &[String]) -> u64 {
    args.iter().position(|a| a == "--seed")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(random_seed)
}

fn random_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64
}

fn main() {
    match parse_command() {
        Command::ProcessModules => content::process_modules::run(),
        Command::Preprocess => preprocess_assets(),
        Command::Sim(mode, seed, characters, palette) => run_sim(mode, seed, characters, palette),
    }
}

fn run_sim(mode: SimMode, seed: u64, spec: RosterSpec, palette: game::background::ColorPalette) {
    let roster = match spec {
        RosterSpec::Default            => game::marbles::build_roster(None),
        RosterSpec::Characters(names)  => game::marbles::build_roster(Some(names)),
        RosterSpec::Slots(n)           => game::marbles::slots_roster(n),
    }
    .unwrap_or_else(|err| {
        eprintln!("Error de roster: {err}");
        std::process::exit(1);
    });

    // La meta se cierra unos segundos antes del final del video para dejar cola de
    // canicas cayendo. En ventana (sin --record) se asume un minuto.
    let video_secs = record_duration().or_else(bake_duration).map(|d| d as f32).unwrap_or(60.0);
    let finish_margin_secs = 12.0;
    let finish_target_secs = video_secs - finish_margin_secs;

    // En replay no hay física → CollisionEvent nunca se inicializa; los sistemas que
    // lo leen hacen panic si se registran. Se gatean aquí en lugar de en cada sistema
    // para no dispersar la condición por todo el codebase.
    let is_replay = replay_path().is_some();

    println!("Level seed: {}", seed);
    let clear = palette.clear_color();
    let mut app = game_app(
        mode,
        GameAppConfig {
            title: "CanicasBrawl",
            resolution: (540.0, 960.0),
        },
    );
    app
    .insert_resource(ClearColor(clear))
    .insert_resource(palette)
    .insert_resource(production::voice_tracker::VoiceTracker::default())
    .insert_resource(production::stall_detector::StallDetector::default())
    .insert_resource(game::finish::RaceResult::default())
    .insert_resource(game::finish::FinishLineY::default())
    .insert_resource(game::leader::RaceLeader::default())
    .insert_resource(game::marbles::Roster(roster))
    .insert_resource(game::world::LevelSeed(seed))
    .insert_resource(game::world::FinishTarget(finish_target_secs))
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
    )
    // FixedUpdate corre a 60 steps/s junto con la física. Toda la lógica que mide
    // tiempo de juego cuenta estos steps, no el wall-clock (que --record acelera 50×).
    // Corren after Writeback para ver colisiones y transforms del step actual.
    //
    // Cadena de liderazgo: detectar cruce de meta → decidir líder/ganador → registrar voz.
    .add_systems(
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
    )
    .add_systems(
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
    .add_systems(
        Update,
        game::hud::update_hud,
    )
    // Update corre por frame de render en wall-clock — solo el watchdog que cierra
    // la app cuando el solver se atasca (mide tiempo real, no tiempo de simulación).
    .add_systems(Update, production::stall_detector::detect_stall)
    .add_systems(
        PostUpdate,
        (
            game::camera::update_marble_labels,
            game::camera::update_leader_crown,
        )
            .after(TransformSystem::TransformPropagate),
    )
    .add_systems(Last, production::voice_tracker::save_voice_tracker_on_exit);

    // Sistemas que leen CollisionEvent: solo disponible cuando corre física (bake/sim),
    // no en replay donde la timeline dicta los transforms sin colisiones.
    if !is_replay {
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
    } else {
        // En replay la utilería de los efectos (sensor consumido, hielo, anillos)
        // se re-actúa desde los eventos horneados en la timeline.
        app.add_systems(
            FixedUpdate,
            (
                game::replay_effects::apply_replay_effects,
                game::replay_effects::expire_replay_freezes,
            )
                .after(PhysicsSet::Writeback),
        );
    }

    app.run();
}
