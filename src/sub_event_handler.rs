use std::error::Error;

use ggez::{
    Context, GameError,
    context::{ContextFields, Has, HasMut},
    event::EventHandler,
    graphics::{Canvas, Color, GraphicsContext},
    input::mouse::MouseContext,
};

pub trait SubEventHandler<C = Context, E = GameError>: Sized {
    fn update(&mut self, ctx: &mut C) -> Result<(), E>;
    fn draw(&mut self, ctx: &mut C, canvas: &mut Canvas) -> Result<(), E>;
    fn event_handler(self) -> EventHandlerWrapper<Self> {
        EventHandlerWrapper(self)
    }
}

pub struct EventHandlerWrapper<H>(H);

impl<S, C, E> EventHandler<C, E> for EventHandlerWrapper<S>
where
    C: Has<GraphicsContext>
        + HasMut<GraphicsContext>
        + HasMut<MouseContext>
        + HasMut<ContextFields>,
    S: SubEventHandler<C, E>,
    E: Error + From<GameError>,
{
    fn update(&mut self, ctx: &mut C) -> Result<(), E> {
        self.0.update(ctx)
    }

    fn draw(&mut self, ctx: &mut C) -> Result<(), E> {
        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);
        self.0.draw(ctx, &mut canvas)?;
        Ok(canvas.finish(ctx)?)
    }
}
