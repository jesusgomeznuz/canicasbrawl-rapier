mod args;
mod game;
mod process_modules;
mod production;
mod simulation;

use args::Command;

// Profundidad Z de canicas y plataformas — temporal mientras se calibran las físicas
pub const UNIT: f32 = 0.35;

fn main() {
    match args::parse_command() {
        Command::Simulation(seed, roster, palette, video_secs) => {
            simulation::run(seed, roster, palette, video_secs)
        }
        Command::BuildModules => process_modules::run(),
    }
}
