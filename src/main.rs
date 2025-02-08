use clap::{crate_authors, crate_name};
use ggez::{
    Context, ContextBuilder, GameError, GameResult,
    conf::WindowMode,
    event::{self, EventHandler},
    glam::ivec2,
    graphics::{Canvas, Color, DrawMode, DrawParam, Drawable, Mesh, Rect},
};
use rand::{Rng, SeedableRng, rngs::StdRng};
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

struct Game {
    grid_density: f32,
    shift_duration: u128,
    grid_pixel: Mesh,
}

impl Game {
    pub fn new(ctx: &Context) -> GameResult<Game> {
        let grid_density = 0.1;
        let shift_duration = 500;
        let (width, height) = ctx.gfx.drawable_size();
        let grid_pixel = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0.0, 0.0, width / grid_density, height / grid_density),
            Color::WHITE,
        )?;
        let this = Game {
            grid_density,
            shift_duration,
            grid_pixel,
        };

        Ok(this)
    }
}

impl EventHandler<Context> for Game {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

        {
            let canvas = &mut canvas;
            let res = ctx.res();

            let grid_size = (res * self.grid_density).as_u64vec2();
            let ires = res.as_ivec2();

            let time = ctx.time.time_since_start().as_millis();
            let rng_frame = time / self.shift_duration;
            let mut rng = StdRng::seed_from_u64(rng_frame.try_into().unwrap_or_default());

            for x in (0..ires.x).step_by(grid_size.x as usize) {
                for y in (0..ires.y).step_by(grid_size.y as usize) {
                    let pos = ivec2(x, y).as_vec2();
                    let shade = rng.random();
                    let color = Color {
                        r: shade,
                        b: shade,
                        g: shade,
                        a: 1.0,
                    };
                    self.grid_pixel
                        .draw(canvas, DrawParam::default().dest(pos).color(color));
                }
            }
        }

        canvas.finish(ctx)
    }
}

fn main() -> GameResult<()> {
    let (ctx, event_loop) = ContextBuilder::new(crate_name!(), crate_authors!())
        .window_mode(WindowMode::default().dimensions(800.0, 800.0))
        .build()?;
    let game = Game::new(&ctx)?;
    event::run(ctx, event_loop, game)
}
