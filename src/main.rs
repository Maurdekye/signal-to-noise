#![feature(associated_type_defaults)]
use std::{env, path::PathBuf};

use clap::{Parser, ValueEnum, crate_authors, crate_name};
use ggez::{
    ContextBuilder, GameResult,
    conf::{WindowMode, WindowSetup},
    event,
};
use ggez_no_re::{
    logger::{LogLevel, LoggerBuilder},
    sub_event_handler::SubEventHandler,
    util::ResultExtToGameError,
};
use noise::Distribution;
use scene_manager::SceneManager;

mod main_menu;
mod noise;
mod scene_manager;
mod shared;

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
    #[arg(short = 's', long, default_value_t = 0.05)]
    cell_spacing: f32,

    /// Width of the signal as a percentage of the window size.
    /// Bigger = harder. Reasonable values between 0.01 - 4.0.
    #[arg(short = 'w', long, default_value_t = 0.25)]
    signal_width: f32,

    /// Average value of the noise floor as a percentage of the total brightness of the screen.
    /// Bigger = harder. Reasonable values between 0.0 - 1.0.
    #[arg(short = 'l', long, default_value_t = 0.25)]
    noise_floor: f32,

    /// Distribution curve of the noise function.
    /// Use a high noise deviation of 20 or more with a pareto distribution.
    #[arg(short = 'd', long, value_enum, default_value_t = Distribution::Gaussian)]
    noise_distribution: Distribution,

    /// Alias `--alpha`. 
    /// The square-root of the `alpha` parameter passed to the pareto noise distribution. 
    /// Unused if another distribution besides pareto is chosen for the noise. 
    /// Higher values make the noise "spikey-er". 
    /// The `xm` parameter is fixed to 1.
    /// Reasonable values between 0 - 10.
    #[arg(alias = "alpha", long, default_value_t = 1.0)]
    noise_pareto_distribution_parameter: f32,

    /// Standard deviation of the noise from the noise floor.
    /// Bigger = harder. Reasonable values between 0.0 - 0.5.
    #[arg(short = 'v', long, default_value_t = 0.05)]
    noise_deviation: f32,

    /// Maximum number of standard deviations away from the noise floor that noise can generate.
    /// Bigger = harder. Reasonable values between 1.0 - 6.0.
    #[arg(short = 'c', long, default_value_t = 3.0)]
    noise_deviation_cap: f32,

    /// Length of time in seconds each "frame" of information is shown.
    /// Bigger = harder. Reasonable values between 0.016 - 5.0.
    #[arg(short = 'f', long, default_value_t = 0.1)]
    frame_time: f32,

    /// Approximate length of time until signal approaches full strength in seconds.
    /// Bigger = harder. Reasonable values 1.0 and above.
    #[arg(short = 'r', long, default_value_t = 180.0)]
    signal_ramp_duration: f32,

    /// Maximum strength of the signal once at peak strength.
    /// Smaller = harder. Reasonable values between 0.0 - 1.0.
    #[arg(short = 't', long, default_value_t = 1.0)]
    signal_max_strength: f32,

    /// Shape of the signal's distribution across space.
    #[arg(short = 'a', long, value_enum, default_value_t = Distribution::Gaussian)]
    signal_shape: Distribution,

    /// Starting scene.
    #[arg(short = 'e', long, value_enum, default_value_t = StartingScene::MainMenu)]
    starting_scene: StartingScene,

    /// Directory to record attempts in.
    #[arg(short = 'p', long, default_value = "records/")]
    record_path: PathBuf,
}

fn main() -> GameResult<()> {
    LoggerBuilder::new()
        .level(LogLevel::Trace)
        .prefix(module_path!())
        .install()
        .to_gameerror()?;
    let args = Args::parse();
    let (mut ctx, event_loop) = ContextBuilder::new(crate_name!(), crate_authors!())
        .window_mode(WindowMode::default().dimensions(800.0, 800.0))
        .window_setup(WindowSetup::default().title("Signal to Noise"))
        .build()?;
    let game = SceneManager::new(&mut ctx, args)?;
    event::run(ctx, event_loop, game.event_handler())
}
