#![feature(associated_type_defaults)]
use std::{env, path::PathBuf};

use clap::{Parser, ValueEnum, crate_authors, crate_name};
use ggez::{
    ContextBuilder, GameResult,
    conf::{WindowMode, WindowSetup},
    event,
};
use logger::Logger;
use scene_manager::SceneManager;
use sub_event_handler::SubEventHandler;
use util::ResultExt;

mod noise_1d;
mod noise_2d;
mod shader_scene;
mod sub_event_handler;
#[allow(unused)]
mod ui_manager;
#[allow(unused)]
mod util;
mod shared;
mod logger;
mod main_menu;
mod recorder;
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
    #[arg(short = 's', long, value_enum, default_value_t = StartingScene::MainMenu)]
    starting_scene: StartingScene,

    /// Directory to record attempts in.
    #[arg(short = 'p', long, default_value = "records/")]
    record_path: PathBuf,
}

fn main() -> GameResult<()> {
    Logger::install(log::LevelFilter::Debug).to_gameerror()?;
    let args = Args::parse();
    let (mut ctx, event_loop) = ContextBuilder::new(crate_name!(), crate_authors!())
        .window_mode(WindowMode::default().dimensions(800.0, 800.0))
        .window_setup(WindowSetup::default().title("Signal to Noise"))
        .build()?;
    let game = SceneManager::new(&mut ctx, args)?;
    event::run(ctx, event_loop, game.event_handler())
}
