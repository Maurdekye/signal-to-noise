use crevice::std140::AsStd140;
use ggez::{Context, GameError, GameResult, graphics::Canvas};

use crate::{
    build_shader, shader_scene::ShaderScene, shared::Shared, sub_event_handler::SubEventHandler,
};

#[derive(AsStd140, Default)]
struct Uniforms {
    x: f32,
}

pub struct Noise1D {
    shader: ShaderScene<Uniforms>,
    _shared: Shared,
}

impl Noise1D {
    pub fn new(ctx: &mut Context, shared: Shared) -> GameResult<Noise1D> {
        let shader = build_shader!(ctx, "../resources/noise_1d.wgsl")?;
        Ok(Noise1D { shader, _shared: shared })
    }
}

impl SubEventHandler for Noise1D {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.shader.uniforms.x = ctx.time.time_since_start().as_secs_f32().sin().abs();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        self.shader.draw(ctx, canvas)
    }
}
