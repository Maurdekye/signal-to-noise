use std::sync::mpsc::Sender;

use ggez::{
    Context, GameError, GameResult,
    graphics::{Canvas, Rect, Text},
};

use crate::{scene_manager::SceneManagerEvent, shared::Shared};

use ggez_no_re::{
    sub_event_handler::SubEventHandler,
    ui_manager::{Bounds, Button, UIElement, UIManager},
};

pub struct MainMenu {
    ui: UIManager<SceneManagerEvent>,
    _shared: Shared,
}

impl MainMenu {
    pub fn new(parent_channel: Sender<SceneManagerEvent>, shared: Shared) -> GameResult<MainMenu> {
        Ok(MainMenu {
            ui: UIManager::new(parent_channel, [
                UIElement::Button(Button::new(
                    Bounds {
                        relative: Rect::new(0.5, 0.5, 0.0, 0.0),
                        absolute: Rect::new(-80.0, -50.0, 160.0, 40.0),
                    },
                    Text::new("2D Noise"),
                    SceneManagerEvent::Noise2D,
                )),
                UIElement::Button(Button::new(
                    Bounds {
                        relative: Rect::new(0.5, 0.5, 0.0, 0.0),
                        absolute: Rect::new(-80.0, 10.0, 160.0, 40.0),
                    },
                    Text::new("1D Noise"),
                    SceneManagerEvent::Noise1D,
                )),
            ]),
            _shared: shared,
        })
    }
}

impl SubEventHandler for MainMenu {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.ui.update(ctx)
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        self.ui.draw(ctx, canvas)
    }
}
