mod args;
mod game;
mod figma_to_modules;
mod production;

use args::Command;

// Profundidad Z de canicas y plataformas — temporal mientras se calibran las físicas
pub const UNIT: f32 = 0.35;

fn main() {
    match args::parse_command() {
        Command::Play(seed, roster, palette, video_secs) => {
            game::run(seed, roster, palette, video_secs)
        }
        Command::BuildModules => figma_to_modules::run(),
    }
}
