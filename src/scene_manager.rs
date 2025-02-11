use std::sync::mpsc::{Receiver, Sender, channel};

use ggez::{
    Context, GameError, GameResult,
    input::mouse::{CursorIcon, set_cursor_type},
    winit::keyboard::{Key, NamedKey},
};

use crate::{
    Args, StartingScene,
    main_menu::MainMenu,
    noise_1d::Noise1D,
    noise_2d::Noise2D,
    shared::Shared,
    sub_event_handler::{EventReceiver, SubEventHandler},
    util::ReceiverExt,
};

#[derive(Clone)]
pub enum SceneManagerEvent {
    MainMenu,
    Noise2D,
    Noise1D,
}

pub struct SceneManager {
    scene: Box<dyn SubEventHandler>,
    shared: Shared,
    event_sender: Sender<SceneManagerEvent>,
    event_receiver: Receiver<SceneManagerEvent>,
}

impl SceneManager {
    pub fn new(ctx: &mut Context, args: Args) -> GameResult<SceneManager> {
        let shared = Shared::new(args);
        let (event_sender, event_receiver) = channel();
        let scene: Box<dyn SubEventHandler> = match shared.args.starting_scene {
            StartingScene::MainMenu => {
                Box::new(MainMenu::new(event_sender.clone(), shared.clone())?)
            }
            StartingScene::Noise2D => Box::new(Noise2D::new(ctx, shared.clone())?),
            StartingScene::Noise1D => Box::new(Noise1D::new(ctx, shared.clone())?),
        };
        Ok(SceneManager {
            scene,
            shared,
            event_sender,
            event_receiver,
        })
    }
}

impl SubEventHandler for SceneManager {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        set_cursor_type(ctx, CursorIcon::Default);
        self.scene.update(ctx)?;
        if ctx
            .keyboard
            .is_logical_key_just_pressed(&Key::Named(NamedKey::Escape))
        {
            self.event_sender.send(SceneManagerEvent::MainMenu).unwrap();
        }
        self.handle_events(ctx)?;
        Ok(())
    }

    fn draw(
        &mut self,
        ctx: &mut Context,
        canvas: &mut ggez::graphics::Canvas,
    ) -> Result<(), GameError> {
        self.scene.draw(ctx, canvas)
    }

    fn quit_event(&mut self, ctx: &mut Context) -> Result<bool, GameError> {
        if ctx
            .keyboard
            .is_logical_key_pressed(&Key::Named(NamedKey::Escape))
        {
            Ok(true)
        } else {
            self.scene.quit_event(ctx)
        }
    }
}

impl EventReceiver for SceneManager {
    type Event = SceneManagerEvent;

    fn handle_event(&mut self, ctx: &mut Context, event: Self::Event) -> Result<(), GameError> {
        match event {
            SceneManagerEvent::MainMenu => {
                self.scene = Box::new(MainMenu::new(
                    self.event_sender.clone(),
                    self.shared.clone(),
                )?);
            }
            SceneManagerEvent::Noise2D => {
                self.scene = Box::new(Noise2D::new(ctx, self.shared.clone())?);
            }
            SceneManagerEvent::Noise1D => {
                self.scene = Box::new(Noise1D::new(ctx, self.shared.clone())?);
            }
        };
        Ok(())
    }

    fn poll_event(&mut self) -> Result<Option<Self::Event>, GameError> {
        self.event_receiver.poll_event()
    }
}
