use std::{env, time::Duration};

use clap::{Parser, crate_authors, crate_name};
use crevice::std140::AsStd140;
use ggez::{
    conf::{WindowMode, WindowSetup}, event::{self, EventHandler}, glam::{vec2, Vec2}, graphics::{
        Canvas, Color, DrawMode, DrawParam, Drawable, Mesh, Rect, Shader, ShaderBuilder,
        ShaderParams, ShaderParamsBuilder, Text,
    }, winit::{
        event::MouseButton,
        keyboard::{Key, NamedKey},
    }, Context, ContextBuilder, GameError, GameResult
};
use util::{AnchorPoint, ContextExt, TextExt};

#[allow(unused)]
mod util;

fn inv_exp(x: f32) -> f32 {
    1.0 - (-x).exp()
}

#[derive(AsStd140, Default)]
struct Uniforms {
    resolution: Vec2,
    grid_spacing: f32,
    time: f32,
    signal_origin: Vec2,
    signal_strength: f32,
    signal_width: f32,
    noise_seed: f32,
    noise_floor: f32,
    noise_deviation: f32,
    noise_deviation_cap: f32,
}

impl Uniforms {
    pub fn update(&mut self, ctx: &Context, game_params: &GameParams) {
        self.resolution = ctx.res();
        self.time = game_params.time.as_secs_f32();
        self.signal_origin = game_params.signal_origin;
        self.noise_seed = game_params.noise_frame;
        self.signal_strength = game_params.signal_progression * game_params.signal_max_strength;
    }
}

struct ClickDecision {
    location: Vec2,
    distance: f32,
    time: Duration,
}

#[derive(Default)]
struct GameParams {
    start_time: Duration,
    time: Duration,
    noise_frame: f32,
    frame_length: f32,
    signal_progression: f32,
    signal_origin: Vec2,
    signal_ramp_speed: f32,
    signal_max_strength: f32,
    click_location: Option<ClickDecision>,
}

struct Game {
    uniforms: Uniforms,
    params: ShaderParams<Uniforms>,
    shader: Shader,
    game_params: GameParams,
    args: Args,
}

impl Game {
    pub fn new(ctx: &mut Context, args: Args) -> GameResult<Game> {
        let Args {
            grid_spacing,
            signal_width,
            noise_floor,
            noise_deviation,
            noise_deviation_cap,
            frame_length,
            signal_ramp_duration: signal_ramp_speed,
            signal_max_strength,
        } = args;
        let mut uniforms = Uniforms {
            grid_spacing,
            signal_width,
            noise_floor,
            noise_deviation,
            noise_deviation_cap,
            ..Default::default()
        };
        let game_params = GameParams {
            frame_length,
            start_time: ctx.time.time_since_start(),
            signal_origin: vec2(rand::random(), rand::random()) * ctx.res(),
            signal_ramp_speed,
            signal_max_strength,
            ..Default::default()
        };
        uniforms.update(ctx, &game_params);
        let params = ShaderParamsBuilder::new(&uniforms).build(ctx);
        let shader = ShaderBuilder::new()
            .fragment_code(include_str!("../resources/noise.wgsl"))
            .build(ctx)?;
        let this = Game {
            uniforms,
            params,
            shader,
            game_params,
            args,
        };

        Ok(this)
    }
}

impl EventHandler<Context> for Game {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let res = ctx.res();
        let norm = res.x.min(res.y);
        self.game_params.time = ctx.time.time_since_start() - self.game_params.start_time;
        if self.game_params.click_location.is_some() {
            if ctx
                .keyboard
                .is_logical_key_just_pressed(&Key::Named(NamedKey::Space))
                || ctx.mouse.button_just_pressed(MouseButton::Right)
            {
                self.uniforms.noise_floor = self.args.noise_floor;
                self.uniforms.noise_deviation = self.args.noise_deviation;
                self.game_params.signal_origin = vec2(rand::random(), rand::random()) * ctx.res();
                self.game_params.click_location = None;
                self.game_params.start_time = ctx.time.time_since_start();
            }
        } else {
            let new_noise_frame =
                (self.game_params.time.as_secs_f32() / self.game_params.frame_length).floor();
            if new_noise_frame != self.game_params.noise_frame {
                self.game_params.noise_frame = new_noise_frame;
                self.game_params.signal_progression = inv_exp(
                    (self.game_params.noise_frame * self.game_params.frame_length)
                        / (self.game_params.signal_ramp_speed
                            * self.game_params.signal_max_strength),
                );
            }
            if ctx.mouse.button_just_pressed(MouseButton::Left) {
                let location: Vec2 = ctx.mouse.position().into();
                let distance = location.distance(self.game_params.signal_origin) / norm;
                let time = self.game_params.time;
                self.uniforms.noise_floor = 0.0;
                self.uniforms.noise_deviation = 0.0;
                self.game_params.click_location = Some(ClickDecision {
                    location,
                    distance,
                    time,
                });
            }
            self.uniforms.update(ctx, &self.game_params);
        }
        self.params.set_uniforms(ctx, &self.uniforms);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

        {
            let canvas = &mut canvas;
            let res = ctx.res();
            canvas.set_shader(&self.shader);
            canvas.set_shader_params(&self.params);
            Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(0.0, 0.0, res.x, res.y),
                Color::WHITE,
            )?
            .draw(canvas, DrawParam::default());

            if let Some(ClickDecision {
                location,
                distance,
                time,
            }) = self.game_params.click_location
            {
                canvas.set_default_shader();
                Mesh::new_line(
                    ctx,
                    &[location, self.game_params.signal_origin],
                    4.0,
                    Color::RED,
                )?
                .draw(canvas, DrawParam::default());

                Text::new(format!(
                    "distance: {distance:.3}\ntime: {:.2}s",
                    time.as_secs_f32()
                ))
                .size(36.0)
                .anchored_by(ctx, vec2(0.5, 0.1) * res, AnchorPoint::NorthCenter)?
                .color(Color::BLUE)
                .draw(canvas);
            }
        }

        canvas.finish(ctx)
    }
}

/// Click on the signal location slowly emerging from the noise.
/// Press space to try again.
#[derive(Parser)]
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
    #[arg(short = 's', long, default_value_t = 1.0)]
    signal_max_strength: f32,
}

fn main() -> GameResult<()> {
    let args = Args::parse();
    let (mut ctx, event_loop) = ContextBuilder::new(crate_name!(), crate_authors!())
        .window_mode(WindowMode::default().dimensions(800.0, 800.0))
        .window_setup(WindowSetup::default().title("Signal to Noise"))
        .build()?;
    let game = Game::new(&mut ctx, args)?;
    event::run(ctx, event_loop, game)
}
