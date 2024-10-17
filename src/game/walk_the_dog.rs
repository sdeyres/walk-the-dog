use std::rc::Rc;

use anyhow::{anyhow, Result};
use futures::channel::mpsc::UnboundedReceiver;

use crate::{
    browser,
    engine::{self, audio::Audio, Game, Image, KeyState, Point, Rect, Renderer, SpriteSheet},
    segment::stone_and_platform,
};

use super::{redhatboy::RedHatBoy, rightmost, Walk, HEIGHT, TIMELINE_MINIMUM, WIDTH};

pub struct WalkTheDog {
    machine: Option<WalkTheDogStateMachine>,
}

enum WalkTheDogStateMachine {
    Ready(WalkTheDogState<Ready>),
    Walking(WalkTheDogState<Walking>),
    GameOver(WalkTheDogState<GameOver>),
}

struct WalkTheDogState<T> {
    walk: Walk,
    _state: T,
}

struct Ready;

struct Walking;

struct GameOver {
    new_game_event: UnboundedReceiver<()>,
}

enum ReadyEndState {
    Complete(WalkTheDogState<Walking>),
    Continue(WalkTheDogState<Ready>),
}

enum WalkingEndState {
    Complete(WalkTheDogState<GameOver>),
    Continue(WalkTheDogState<Walking>),
}

enum GameOverEndState {
    Complete(WalkTheDogState<Ready>),
    Continue(WalkTheDogState<GameOver>),
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog { machine: None }
    }
}

impl Game for WalkTheDog {
    async fn initialize(&self) -> anyhow::Result<Box<impl Game + 'static>> {
        match self.machine {
            None => {
                // Background
                let background =
                    engine::load_image("assets/original/freetileset/png/BG/BG.png").await?;
                let background_width = background.width() as i16;
                let backgrounds = [
                    Image::new(background.clone(), Point { x: 0, y: 0 }),
                    Image::new(
                        background,
                        Point {
                            x: background_width,
                            y: 0,
                        },
                    ),
                ];

                // Red hat boy
                let sprite_sheet = SpriteSheet::new(
                    "assets/sprite_sheets/rhb_trimmed.json",
                    "assets/sprite_sheets/rhb_trimmed.png",
                )
                .await?;
                let audio = Audio::new()?;
                let jump_sound = audio.load_sound("assets/sounds/SFX_Jump_23.mp3").await?;
                let background_music = audio
                    .load_sound("assets/sounds/background_song.mp3")
                    .await?;
                if let Err(err) = audio.play_loop(&background_music) {
                    error!("Error starting the audio loop {:#?}", err);
                }
                let boy = RedHatBoy::new(sprite_sheet, audio, jump_sound);

                // Platform sprite sheet
                let obstacle_sheet = Rc::new(
                    SpriteSheet::new(
                        "assets/sprite_sheets/tiles.json",
                        "assets/sprite_sheets/tiles.png",
                    )
                    .await?,
                );

                // Stone
                let stone =
                    engine::load_image("assets/original/freetileset/png/Object/Stone.png").await?;

                // Starting obstacles
                let starting_obstacles =
                    stone_and_platform(stone.clone(), obstacle_sheet.clone(), 0);
                let timeline = rightmost(&starting_obstacles);

                // State machine
                let machine = WalkTheDogStateMachine::new(Walk {
                    backgrounds,
                    boy,
                    obstacles: starting_obstacles,
                    obstacle_sheet,
                    stone,
                    timeline,
                });

                Ok(Box::new(WalkTheDog {
                    machine: Some(machine),
                }))
            }
            Some(_) => Err(anyhow!("Error: Game is already initialized!")),
        }
    }

    fn update(&mut self, keystate: &engine::KeyState) {
        if let Some(machine) = self.machine.take() {
            self.machine.replace(machine.update(keystate));
        }
        assert!(self.machine.is_some());
    }

    fn draw(&self, renderer: &engine::Renderer) -> Result<()> {
        renderer.clear(&Rect::new(Point { x: 0, y: 0 }, WIDTH, HEIGHT));

        if let Some(machine) = &self.machine {
            machine.draw(renderer)?;
        }

        Ok(())
    }
}

impl WalkTheDogStateMachine {
    fn new(walk: Walk) -> WalkTheDogStateMachine {
        WalkTheDogStateMachine::Ready(WalkTheDogState::new(walk))
    }

    fn update(self, keystate: &KeyState) -> Self {
        match self {
            WalkTheDogStateMachine::Ready(state) => state.update(keystate).into(),
            WalkTheDogStateMachine::Walking(state) => state.update(keystate).into(),
            WalkTheDogStateMachine::GameOver(state) => state.update().into(),
        }
    }

    fn draw(&self, renderer: &Renderer) -> Result<()> {
        match self {
            WalkTheDogStateMachine::Ready(state) => state.draw(renderer),
            WalkTheDogStateMachine::Walking(state) => state.draw(renderer),
            WalkTheDogStateMachine::GameOver(state) => state.draw(renderer),
        }
    }
}

impl WalkTheDogState<Ready> {
    fn new(walk: Walk) -> WalkTheDogState<Ready> {
        WalkTheDogState {
            walk,
            _state: Ready,
        }
    }

    fn update(mut self, keystate: &KeyState) -> ReadyEndState {
        self.walk.boy.update();
        if keystate.is_pressed("ArrowRight") {
            ReadyEndState::Complete(self.start_running())
        } else {
            ReadyEndState::Continue(self)
        }
    }

    fn draw(&self, renderer: &Renderer) -> Result<()> {
        self.walk.draw(renderer)
    }

    fn start_running(mut self) -> WalkTheDogState<Walking> {
        self.run();
        WalkTheDogState {
            walk: self.walk,
            _state: Walking,
        }
    }

    fn run(&mut self) {
        self.walk.boy.run();
    }
}

impl WalkTheDogState<Walking> {
    fn update(mut self, keystate: &KeyState) -> WalkingEndState {
        if keystate.is_pressed("Space") {
            self.walk.boy.jump();
        }
        if keystate.is_pressed("ArrowDown") {
            self.walk.boy.slide();
        }
        self.walk.boy.update();

        let walking_speed = self.walk.velocity();

        // Backgrounds
        let [first_background, second_background] = &mut self.walk.backgrounds;
        first_background.move_horizontally(walking_speed);
        second_background.move_horizontally(walking_speed);
        if first_background.right() < 0 {
            first_background.set_x(second_background.right());
        }
        if second_background.right() < 0 {
            second_background.set_x(first_background.right());
        }

        // Obstacles
        self.walk.obstacles.retain(|obstacle| obstacle.right() > 0);
        self.walk.obstacles.iter_mut().for_each(|obstacle| {
            obstacle.move_horizontally(walking_speed);
            obstacle.check_intersection(&mut self.walk.boy);
        });

        // Timeline
        if self.walk.timeline < TIMELINE_MINIMUM {
            self.walk.generate_next_segment();
        } else {
            self.walk.timeline += walking_speed;
        }

        if self.walk.knocked_out() {
            WalkingEndState::Complete(self.end_game())
        } else {
            WalkingEndState::Continue(self)
        }
    }

    fn draw(&self, renderer: &Renderer) -> Result<()> {
        self.walk.draw(renderer)
    }

    fn end_game(self) -> WalkTheDogState<GameOver> {
        let receiver = browser::draw_ui("<button id=\"new_game\">New game</button>")
            .and_then(|_unit| browser::find_html_element_by_id("new_game"))
            .map(engine::add_click_handler)
            .unwrap();
        WalkTheDogState {
            walk: self.walk,
            _state: GameOver {
                new_game_event: receiver,
            },
        }
    }
}

impl WalkTheDogState<GameOver> {
    fn update(mut self) -> GameOverEndState {
        if self._state.new_game_pressed() {
            GameOverEndState::Complete(self.new_game())
        } else {
            GameOverEndState::Continue(self)
        }
    }

    fn draw(&self, renderer: &Renderer) -> Result<()> {
        self.walk.draw(renderer)
    }

    fn new_game(self) -> WalkTheDogState<Ready> {
        if let Err(err) = browser::hide_ui() {
            error!("Error hiding the browser {:#?}", err);
        }
        WalkTheDogState {
            walk: Walk::reset(self.walk),
            _state: Ready,
        }
    }
}

impl GameOver {
    fn new_game_pressed(&mut self) -> bool {
        matches!(self.new_game_event.try_next(), Ok(Some(())))
    }
}

impl From<WalkTheDogState<Ready>> for WalkTheDogStateMachine {
    fn from(state: WalkTheDogState<Ready>) -> Self {
        WalkTheDogStateMachine::Ready(state)
    }
}

impl From<WalkTheDogState<Walking>> for WalkTheDogStateMachine {
    fn from(state: WalkTheDogState<Walking>) -> Self {
        WalkTheDogStateMachine::Walking(state)
    }
}

impl From<WalkTheDogState<GameOver>> for WalkTheDogStateMachine {
    fn from(state: WalkTheDogState<GameOver>) -> Self {
        WalkTheDogStateMachine::GameOver(state)
    }
}

impl From<ReadyEndState> for WalkTheDogStateMachine {
    fn from(end_state: ReadyEndState) -> Self {
        match end_state {
            ReadyEndState::Complete(walking_state) => walking_state.into(),
            ReadyEndState::Continue(ready_state) => ready_state.into(),
        }
    }
}

impl From<WalkingEndState> for WalkTheDogStateMachine {
    fn from(end_state: WalkingEndState) -> Self {
        match end_state {
            WalkingEndState::Complete(game_over_state) => game_over_state.into(),
            WalkingEndState::Continue(walking_state) => walking_state.into(),
        }
    }
}

impl From<GameOverEndState> for WalkTheDogStateMachine {
    fn from(end_state: GameOverEndState) -> Self {
        match end_state {
            GameOverEndState::Complete(ready_state) => ready_state.into(),
            GameOverEndState::Continue(game_over_state) => game_over_state.into(),
        }
    }
}
