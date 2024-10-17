use std::{cell::RefCell, rc::Rc};

use anyhow::{anyhow, Result};

use crate::browser::{self, LoopClosure};

use super::{prepare_input, process_input, KeyState, Renderer, FRAME_SIZE};

pub trait Game {
    async fn initialize(&self) -> Result<Box<impl Game + 'static>>;
    fn update(&mut self, keystate: &KeyState);
    fn draw(&self, renderer: &Renderer) -> Result<()>;
}

pub struct GameLoop {
    last_frame: f64,
    accumulated_delta: f32,
}

type SharedLoopClosure = Rc<RefCell<Option<LoopClosure>>>;

impl GameLoop {
    pub async fn start(game: impl Game + 'static) -> Result<()> {
        let mut keyevent_receiver = prepare_input()?;
        let mut keystate = super::KeyState::new();
        let mut game = game.initialize().await?;
        let mut game_loop = GameLoop {
            last_frame: browser::now()?,
            accumulated_delta: 0.0,
        };

        let renderer = Renderer::new(browser::context()?);

        let f: SharedLoopClosure = Rc::new(RefCell::new(None));
        let g = Rc::clone(&f);
        *g.borrow_mut() = Some(browser::create_raf_closure(move |perf| {
            process_input(&mut keystate, &mut keyevent_receiver);
            game_loop.accumulated_delta += (perf - game_loop.last_frame) as f32;
            while game_loop.accumulated_delta > FRAME_SIZE {
                game.update(&keystate);
                game_loop.accumulated_delta -= FRAME_SIZE;
            }
            game_loop.last_frame = perf;
            if let Err(err) = game.draw(&renderer) {
                error!("Error while drawing the game: {:#?}", err);
            }

            if let Err(err) = browser::request_animation_frame(f.borrow().as_ref().unwrap()) {
                error!("Unable to request animation frame: {:#?}", err);
            }
        }));

        browser::request_animation_frame(
            g.borrow()
                .as_ref()
                .ok_or_else(|| anyhow!("GameLoop: Loop is None"))?,
        )?;
        Ok(())
    }
}
