use std::time::Duration;

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

use crate::{
    Args, build_shader,
    shader_scene::ShaderScene,
    shared::Shared,
    sub_event_handler::SubEventHandler,
    util::{AnchorPoint, ContextExt, DrawableWihParamsExt, TextExt, inv_exp},
};

#[derive(AsStd140, Default)]
struct Uniforms {
    resolution: Vec2,
    column_spacing: f32,
    signal_center: f32,
    signal_strength: f32,
    signal_width: f32,
    noise_seed: f32,
    noise_floor: f32,
    noise_deviation: f32,
    noise_deviation_cap: f32,
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
    signal_center: f32,
    signal_ramp_duration: f32,
    signal_max_strength: f32,
    click_location: Option<ClickDecision>,
}

impl GameParams {
    fn reset(&mut self, ctx: &Context) {
        self.start_time = ctx.time.time_since_start();
        self.signal_center = rand::random::<f32>() * ctx.res().x;
        self.click_location = None;
    }
}

pub struct Noise1D {
    shader: ShaderScene<Uniforms>,
    params: GameParams,
    shared: Shared,
}

impl Noise1D {
    pub fn new(ctx: &mut Context, shared: Shared) -> GameResult<Noise1D> {
        let Args {
            grid_spacing,
            signal_width,
            noise_floor,
            noise_deviation,
            noise_deviation_cap,
            frame_length,
            signal_ramp_duration,
            signal_max_strength,
            ..
        } = shared.args;
        let uniforms = Uniforms {
            column_spacing: grid_spacing,
            signal_width,
            noise_floor,
            noise_deviation,
            noise_deviation_cap,
            ..Default::default()
        };
        let shader = build_shader!(ctx, "../resources/noise_1d.wgsl", uniforms)?;
        let mut params = GameParams {
            frame_length,
            signal_ramp_duration,
            signal_max_strength,
            ..Default::default()
        };
        params.reset(ctx);
        Ok(Noise1D {
            shader,
            params,
            shared,
        })
    }
}

impl SubEventHandler for Noise1D {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let res = ctx.res();
        let norm = res.x.min(res.y);
        let params = &mut self.params;
        let uniforms = &mut self.shader.uniforms;
        params.time = ctx.time.time_since_start() - params.start_time;
        uniforms.resolution = ctx.res();
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
                let distance = (params.signal_center - location.x).abs() / norm;
                let time = params.time;
                params.click_location = Some(ClickDecision {
                    location,
                    distance,
                    time,
                });

                uniforms.noise_floor = 0.0;
                uniforms.noise_deviation = 0.0;
            }
            uniforms.signal_center = params.signal_center;
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

            // draw lines indicating signal center vs click location
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
                &[
                    vec2(self.params.signal_center, 0.0),
                    vec2(self.params.signal_center, res.y),
                ],
                4.0,
                Color::RED,
            )?
            .draw(canvas);
            Mesh::new_line(
                ctx,
                &[location, vec2(self.params.signal_center, location.y)],
                4.0,
                Color::RED,
            )?
            .draw(canvas);

            Text::new(format!(
                "distance: {distance:.3}\ntime: {:.2}s\nbrightness: {:.1}%",
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
