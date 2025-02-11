use std::time::Duration;

use crate::shader_scene::ShaderScene;
use crate::sub_event_handler::SubEventHandler;
use crate::util::{AnchorPoint, ContextExt, TextExt};
use crate::{Args, build_shader};
use crevice::std140::AsStd140;
use ggez::{
    Context, GameError, GameResult,
    glam::{Vec2, vec2},
    graphics::{Canvas, Color, DrawParam, Drawable, Mesh, Text},
    winit::{
        event::MouseButton,
        keyboard::{Key, NamedKey},
    },
};

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

pub struct Noise2D {
    shader: ShaderScene<Uniforms>,
    game_params: GameParams,
    args: Args,
}

impl Noise2D {
    pub fn new(ctx: &mut Context, args: Args) -> GameResult<Noise2D> {
        let Args {
            grid_spacing,
            signal_width,
            noise_floor,
            noise_deviation,
            noise_deviation_cap,
            frame_length,
            signal_ramp_duration,
            signal_max_strength,
        } = args;
        let uniforms = Uniforms {
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
            signal_ramp_duration,
            signal_max_strength,
            ..Default::default()
        };
        let shader = build_shader!(ctx, "../resources/noise.wgsl", uniforms)?;
        let this = Noise2D {
            shader,
            game_params,
            args,
        };

        Ok(this)
    }
}

impl SubEventHandler for Noise2D {
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
                self.shader.uniforms.noise_floor = self.args.noise_floor;
                self.shader.uniforms.noise_deviation = self.args.noise_deviation;
                self.game_params.signal_origin =
                    vec2(rand::random(), rand::random()) * ctx.res();
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
                        / (self.game_params.signal_ramp_duration
                            * self.game_params.signal_max_strength),
                );
            }
            if ctx.mouse.button_just_pressed(MouseButton::Left) {
                let location: Vec2 = ctx.mouse.position().into();
                let distance = location.distance(self.game_params.signal_origin) / norm;
                let time = self.game_params.time;
                self.shader.uniforms.noise_floor = 0.0;
                self.shader.uniforms.noise_deviation = 0.0;
                self.game_params.click_location = Some(ClickDecision {
                    location,
                    distance,
                    time,
                });
            }
            self.shader.uniforms.resolution = ctx.res();
            self.shader.uniforms.time = self.game_params.time.as_secs_f32();
            self.shader.uniforms.signal_origin = self.game_params.signal_origin;
            self.shader.uniforms.noise_seed = self.game_params.noise_frame;
            self.shader.uniforms.signal_strength =
                self.game_params.signal_progression * self.game_params.signal_max_strength;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        let res = ctx.res();
        self.shader.draw(ctx, canvas)?;

        if let Some(ClickDecision {
            location,
            distance,
            time,
        }) = self.game_params.click_location
        {
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

        Ok(())
    }
}
