use std::{env, path::PathBuf};

use clap::{crate_authors, crate_name};
use crevice::std140::AsStd140;
use ggez::{
    Context, ContextBuilder, GameError, GameResult,
    conf::WindowMode,
    event::{self, EventHandler},
    glam::Vec2,
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

#[derive(AsStd140)]
struct Uniforms {
    resolution: Vec2,
    mouse: Vec2,
    grid_spacing: f32,
    shift_duration: f32,
    time: f32,
}

impl Uniforms {
    pub fn update(&mut self, ctx: &Context) {
        self.resolution = ctx.res();
        self.time = ctx.time.time_since_start().as_secs_f32();
        self.mouse = ctx.mouse.position().into();
    }
}

struct Game {
    uniforms: Uniforms,
    params: ShaderParams<Uniforms>,
    shader: Shader,
}

impl Game {
    pub fn new(ctx: &mut Context) -> GameResult<Game> {
        let mut uniforms = Uniforms {
            resolution: Vec2::ZERO,
            mouse: Vec2::ZERO,
            time: 0.0,
            grid_spacing: 0.005,
            shift_duration: 0.05,
        };
        uniforms.update(ctx);
        let params = ShaderParamsBuilder::new(&uniforms).build(ctx);
        let shader = ShaderBuilder::new()
            .fragment_path("/noise.wgsl")
            .build(ctx)?;
        let this = Game {
            uniforms,
            params,
            shader,
        };

        Ok(this)
    }
}

impl EventHandler<Context> for Game {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.uniforms.update(ctx);
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
