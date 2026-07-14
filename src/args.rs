use rapier_bevy::{SimulationMode, session_duration_secs};

use crate::game::background::palette::ColorPalette;

pub enum Command {
    Simulation(SimulationMode, u64, RosterSpec, ColorPalette, f32),
    BuildModules,
    PreprocessConcaveColliders,
}

pub enum RosterSpec {
    Default,
    Characters(Vec<String>),
    Slots(usize),
}

pub fn parse_command() -> Command {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--process-modules") {
        return Command::BuildModules;
    }
    if args.iter().any(|a| a == "--preprocess") {
        return Command::PreprocessConcaveColliders;
    }
    let mode = if args.iter().any(|a| a == "--sim-raw") {
        SimulationMode::Raw
    } else {
        SimulationMode::Precomputed
    };
    let palette = parse_palette(&args);
    let video_secs = session_duration_secs().map(|secs| secs as f32).unwrap_or(60.0);
    Command::Simulation(mode, parse_seed(&args), parse_roster_spec(&args), palette, video_secs)
}

fn parse_palette(args: &[String]) -> ColorPalette {
    if args.iter().any(|a| a == "--neon") {
        ColorPalette::neon()
    } else if args.iter().any(|a| a == "--rosa") {
        ColorPalette::rosa()
    } else {
        ColorPalette::azul()
    }
}

fn parse_characters(args: &[String]) -> Option<Vec<String>> {
    args.iter()
        .position(|a| a == "--characters")
        .and_then(|i| args.get(i + 1))
        .map(|list| list.split(',').map(|s| s.trim().to_string()).collect())
}

fn parse_roster_spec(args: &[String]) -> RosterSpec {
    let slots = args
        .iter()
        .position(|a| a == "--slots")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok());
    match (slots, parse_characters(args)) {
        (Some(_), Some(_)) => {
            eprintln!(
                "--slots y --characters son excluyentes: la timeline se escribe anónima \
                 con --slots y se viste con --characters al reproducirla (--play)."
            );
            std::process::exit(1);
        }
        (Some(n), None) => RosterSpec::Slots(n),
        (None, Some(names)) => RosterSpec::Characters(names),
        (None, None) => RosterSpec::Default,
    }
}

fn parse_seed(args: &[String]) -> u64 {
    args.iter()
        .position(|a| a == "--seed")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(random_seed)
}

fn random_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
