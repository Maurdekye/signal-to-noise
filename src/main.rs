#![feature(associated_type_defaults)]
use std::{env, io::Write};

use clap::{ArgAction, Parser, ValueEnum, crate_authors, crate_name};
use ggez::{
    ContextBuilder, GameResult,
    conf::{WindowMode, WindowSetup},
    event,
};
use log::{Log, set_max_level};
use scene_manager::SceneManager;
use sub_event_handler::SubEventHandler;

mod noise_1d;
mod noise_2d;
mod shader_scene;
mod sub_event_handler;
mod ui_manager;
#[allow(unused)]
mod util;
mod shared {
    use crate::Args;

    #[derive(Clone)]
    pub struct Shared {
        pub args: Args,
    }

    impl Shared {
        pub fn new(args: Args) -> Shared {
            Shared { args }
        }
    }
}
mod main_menu;
mod scene_manager;

#[derive(Clone, ValueEnum)]
pub enum StartingScene {
    MainMenu,
    Noise1D,
    Noise2D,
}

/// Click on the signal location slowly emerging from the noise.
/// Press space to try again.
#[derive(Clone, Parser)]
pub struct Args {
    /// Size of individual cells in the grid as a percentage of the window size.
    /// Bigger = harder. Reasonable values between 0.001 - 0.25.
    #[arg(short, long, default_value_t = 0.05)]
    grid_spacing: f32,

    /// Width of the signal as a percentage of the window size.
    /// Bigger = harder. Reasonable values between 0.01 - 4.0.
    #[arg(short = 'w', long, default_value_t = 0.25)]
    signal_width: f32,

    /// Average value of the noise floor as a percentage of the total brightness of the screen.
    /// Bigger = harder. Reasonable values between 0.0 - 1.0.
    #[arg(short = 'l', long, default_value_t = 0.25)]
    noise_floor: f32,

    /// Standard deviation of the noise from the noise floor.
    /// Bigger = harder. Reasonable values between 0.0 - 0.5.
    #[arg(short = 'd', long, default_value_t = 0.05)]
    noise_deviation: f32,

    /// Maximum number of standard deviations away from the noise floor that noise can generate.
    /// Bigger = harder. Reasonable values between 1.0 - 6.0.
    #[arg(short = 'c', long, default_value_t = 3.0)]
    noise_deviation_cap: f32,

    /// Length of time in seconds each "frame" of information is shown.
    /// Bigger = harder. Reasonable values between 0.016 - 5.0.
    #[arg(short = 'f', long, default_value_t = 0.1)]
    frame_length: f32,

    /// Approximate length of time until signal approaches full strength in seconds.
    /// Bigger = harder. Reasonable values 1.0 and above.
    #[arg(short = 'r', long, default_value_t = 180.0)]
    signal_ramp_duration: f32,

    /// Maximum strength of the signal once at peak strength.
    /// Smaller = harder. Reasonable values between 0.0 - 1.0.
    #[arg(short = 't', long, default_value_t = 1.0)]
    signal_max_strength: f32,

    /// Starting scene.
    #[arg(short = 's', long, default_value = "main-menu")]
    starting_scene: StartingScene,

    /// Enable to exhibit the bug at https://github.com/ggez/ggez/issues/1310
    #[arg(short, long, action = ArgAction::SetTrue)]
    bug: bool,
}

struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.target().starts_with("signal_to_noise")
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!(
                "[{}:{}] {}",
                record.target(),
                record.line().unwrap(),
                record.args()
            );
        }
    }

    fn flush(&self) {
        std::io::stdout().flush().unwrap()
    }
}

fn main() -> GameResult<()> {
    let mut args = Args::parse();
    args.bug = true;
    // args.starting_scene = StartingScene::Noise2D;
    let _ = log::set_logger(&Logger).map(|_| set_max_level(log::LevelFilter::Trace));
    let (mut ctx, event_loop) = ContextBuilder::new(crate_name!(), crate_authors!())
        .window_mode(WindowMode::default().dimensions(800.0, 800.0))
        .window_setup(WindowSetup::default().title("Signal to Noise"))
        .build()?;
    let game = SceneManager::new(&mut ctx, args)?;
    event::run(ctx, event_loop, game.event_handler())
}
