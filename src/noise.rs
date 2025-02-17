use std::time::Duration;

use crate::{Args, shared::Shared};
use clap::ValueEnum;
use crevice::std140::AsStd140;
use ggez::{
    Context, GameError, GameResult,
    glam::{Vec2, vec2},
    graphics::{Canvas, Color, Mesh, Text},
    winit::{
        event::MouseButton,
        keyboard::{Key, NamedKey},
    },
};
use ggez_no_re::build_shader;
use ggez_no_re::shader_scene::ShaderScene;
use ggez_no_re::sub_event_handler::SubEventHandler;
use ggez_no_re::util::{AnchorPoint, ContextExt, DrawableWihParamsExt, TextExt};
use serde::Serialize;

pub fn inv_exp(x: f32) -> f32 {
    1.0 - (-x).exp()
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
#[repr(u32)]
pub enum Distribution {
    #[default]
    Gaussian = 0,
    Pareto,
    Triangle,
    Uniform,
}

impl std::fmt::Display for Distribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_possible_value().unwrap().get_name())
    }
}

#[derive(AsStd140, Default)]
struct Uniforms {
    resolution: Vec2,
    cell_spacing: f32,
    signal_origin: Vec2,
    signal_strength: f32,
    signal_width: f32,
    signal_shape: u32,
    noise_seed: f32,
    noise_floor: f32,
    noise_deviation: f32,
    noise_deviation_cap: f32,
    noise_distribution: u32,
    noise_pareto_distribution_parameter: f32,
    dimensions: u32,
}

#[derive(Serialize)]
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
    signal_ramp_duration: f32,
    signal_max_strength: f32,
    click_location: Option<ClickDecision>,
}

impl GameParams {
    fn reset(&mut self, ctx: &Context) {
        self.start_time = ctx.time.time_since_start();
        self.signal_origin = vec2(rand::random(), rand::random());
        self.click_location = None;
    }
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NoiseMode {
    OneDimensional = 1,
    TwoDimensional = 2,
}

pub struct Noise {
    shader: ShaderScene<Uniforms>,
    params: GameParams,
    shared: Shared,
    mode: NoiseMode,
}

impl Noise {
    pub fn new(ctx: &mut Context, shared: Shared, mode: NoiseMode) -> GameResult<Noise> {
        let Args {
            cell_spacing,
            signal_width,
            noise_floor,
            noise_distribution,
            noise_pareto_distribution_parameter,
            noise_deviation,
            noise_deviation_cap,
            frame_time: frame_length,
            signal_ramp_duration,
            signal_max_strength,
            signal_shape,
            ..
        } = shared.args;
        let uniforms = Uniforms {
            cell_spacing,
            signal_width,
            signal_shape: signal_shape as u32,
            noise_floor,
            noise_distribution: noise_distribution as u32,
            noise_pareto_distribution_parameter,
            noise_deviation,
            noise_deviation_cap,
            dimensions: mode as u32,
            ..Default::default()
        };
        let mut params = GameParams {
            frame_length,
            signal_ramp_duration,
            signal_max_strength,
            ..Default::default()
        };
        params.reset(ctx);
        let shader = build_shader!(ctx, "../resources/noise.wgsl", uniforms)?;
        Ok(Noise {
            shader,
            params,
            shared,
            mode,
        })
    }
}

impl SubEventHandler for Noise {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let res = ctx.res();
        let params = &mut self.params;
        let uniforms = &mut self.shader.uniforms;
        params.time = ctx.time.time_since_start() - params.start_time;
        uniforms.resolution = res;
        if params.click_location.is_some() {
            if ctx
                .keyboard
                .is_logical_key_just_pressed(&Key::Named(NamedKey::Space))
                || ctx.mouse.button_just_pressed(MouseButton::Right)
            {
                params.reset(ctx);

                uniforms.noise_floor = self.shared.args.noise_floor;
                uniforms.noise_deviation = self.shared.args.noise_deviation;
            }
        } else {
            let new_noise_frame = (params.time.as_secs_f32() / params.frame_length).floor();
            if new_noise_frame != params.noise_frame {
                params.noise_frame = new_noise_frame;
                params.signal_progression = if params.signal_ramp_duration > 0.0 {
                    inv_exp(
                        (params.noise_frame * params.frame_length)
                            / (params.signal_ramp_duration * params.signal_max_strength),
                    )
                } else {
                    1.0
                };
            }
            if ctx.mouse.button_just_pressed(MouseButton::Left) {
                let location: Vec2 = ctx.mouse.position().into();
                let location = location / res;
                let distance = params.signal_origin.distance(location);
                let time = params.time;

                params.click_location = Some(ClickDecision {
                    location,
                    distance,
                    time,
                });

                uniforms.noise_floor = 0.0;
                uniforms.noise_deviation = 0.0;

                #[derive(Serialize)]
                struct Record {
                    distance: f32,
                    time: f32,
                    strength: f32,
                }

                self.shared.recorder.record(
                    format!(
                        "{}/{}-{}-{}-{}-{}-{}-{}-{}-{}-{}",
                        match self.mode {
                            NoiseMode::OneDimensional => "noise_1d",
                            NoiseMode::TwoDimensional => "noise_2d",
                        },
                        self.shared.args.cell_spacing,
                        self.shared.args.signal_width,
                        self.shared.args.noise_floor,
                        match self.shared.args.noise_distribution {
                            Distribution::Pareto => format!(
                                "pareto({})",
                                self.shared.args.noise_pareto_distribution_parameter
                            ),
                            distribution => format!("{distribution}"),
                        },
                        self.shared.args.noise_deviation,
                        self.shared.args.noise_deviation_cap,
                        self.shared.args.frame_time,
                        self.shared.args.signal_ramp_duration,
                        self.shared.args.signal_max_strength,
                        self.shared.args.signal_shape
                    ),
                    Record {
                        distance,
                        time: time.as_secs_f32(),
                        strength: params.signal_progression * params.signal_max_strength,
                    },
                );
            }
            uniforms.signal_origin = params.signal_origin;
            uniforms.noise_seed = params.noise_frame;
            uniforms.signal_strength = params.signal_progression * params.signal_max_strength;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        self.shader.draw(ctx, canvas)?;

        if let Some(ClickDecision {
            location,
            distance,
            time,
        }) = self.params.click_location
        {
            let res = ctx.res();
            if self.mode == NoiseMode::OneDimensional {
                let location = location * res;
                let signal_center = self.params.signal_origin.x * res.x;
                let height = vec2(0.0, res.y * 0.1) / 2.0;
                Mesh::new_line(
                    ctx,
                    &[location - height, location + height],
                    4.0,
                    Color::RED,
                )?
                .draw(canvas);
                Mesh::new_line(
                    ctx,
                    &[vec2(signal_center, 0.0), vec2(signal_center, res.y)],
                    4.0,
                    Color::RED,
                )?
                .draw(canvas);
                Mesh::new_line(
                    ctx,
                    &[location, vec2(signal_center, location.y)],
                    4.0,
                    Color::RED,
                )?
                .draw(canvas);
            } else {
                Mesh::new_line(
                    ctx,
                    &[location * res, self.params.signal_origin * res],
                    4.0,
                    Color::RED,
                )?
                .draw(canvas);
            }

            Text::new(format!(
                "\
distance: {distance:.3}
time: {:.2}s
strength: {:.1}%",
                time.as_secs_f32(),
                self.params.signal_progression * 100.0
            ))
            .size(24.0)
            .anchored_by(ctx, vec2(20.0, 20.0), AnchorPoint::NorthWest)?
            .color(Color::BLUE)
            .draw(canvas);
        }

        Ok(())
    }
}
