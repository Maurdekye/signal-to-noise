use std::{env, path::PathBuf};

use clap::{crate_authors, crate_name};
use crevice::std140::AsStd140;
use ggez::{
    Context, ContextBuilder, GameError, GameResult,
    conf::WindowMode,
    event::{self, EventHandler},
    glam::{Vec2, vec2},
    graphics::{
        Canvas, Color, DrawMode, DrawParam, Drawable, Mesh, Rect, Shader, ShaderBuilder,
        ShaderParams, ShaderParamsBuilder,
    },
};
use util::ContextExt;

mod util {
    use ggez::{Context, glam::Vec2};

    pub trait ContextExt {
        fn res(&self) -> Vec2;
    }
    impl ContextExt for Context {
        fn res(&self) -> Vec2 {
            self.gfx.drawable_size().into()
        }
    }
}

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
    pub fn update(&mut self, ctx: &Context, params: &GameParams) {
        self.resolution = ctx.res();
        self.time = params.time;
        self.signal_origin = params.signal_origin;
        self.noise_seed = params.noise_frame;
        self.signal_strength = params.signal_progression * params.signal_max_strength;
    }
}

#[derive(Default)]
struct GameParams {
    time: f32,
    noise_frame: f32,
    signal_progression: f32,
    signal_origin: Vec2,
    signal_ramp_speed: f32,
    signal_max_strength: f32,
}

struct Game {
    uniforms: Uniforms,
    params: ShaderParams<Uniforms>,
    shader: Shader,
    game_params: GameParams,
}

impl Game {
    pub fn new(ctx: &mut Context) -> GameResult<Game> {
        let mut uniforms = Uniforms {
            grid_spacing: 0.1,
            signal_strength: 0.0,
            signal_width: 2.0,
            noise_floor: 0.25,
            noise_deviation: 0.1,
            noise_deviation_cap: 3.0,
            ..Default::default()
        };
        let game_params = GameParams {
            signal_origin: vec2(rand::random(), rand::random()) * ctx.res(),
            signal_ramp_speed: 180.0,
            signal_max_strength: 1.0,
            ..Default::default()
        };
        dbg!(&game_params.signal_origin);
        uniforms.update(ctx, &game_params);
        let params = ShaderParamsBuilder::new(&uniforms).build(ctx);
        let shader = ShaderBuilder::new()
            .fragment_path("/noise.wgsl")
            .build(ctx)?;
        let this = Game {
            uniforms,
            params,
            shader,
            game_params,
        };

        Ok(this)
    }
}

impl EventHandler<Context> for Game {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.game_params.time = ctx.time.time_since_start().as_secs_f32();
        let new_noise_frame = (self.game_params.time / 0.5).floor();
        let new_frame = new_noise_frame != self.game_params.noise_frame;
        self.game_params.noise_frame = new_noise_frame;
        self.game_params.signal_progression = inv_exp(
            self.game_params.noise_frame
                / (self.game_params.signal_ramp_speed * self.game_params.signal_max_strength),
        );
        if new_frame {
            dbg!(self.game_params.noise_frame);
            dbg!(self.game_params.signal_progression);
        }
        self.uniforms.update(ctx, &self.game_params);
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
        }

        canvas.finish(ctx)
    }
}

fn main() -> GameResult<()> {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        PathBuf::from("./resources")
    };
    let (mut ctx, event_loop) = ContextBuilder::new(crate_name!(), crate_authors!())
        .window_mode(WindowMode::default().dimensions(800.0, 800.0))
        .add_resource_path(resource_dir)
        .build()?;
    let game = Game::new(&mut ctx)?;
    event::run(ctx, event_loop, game)
}
