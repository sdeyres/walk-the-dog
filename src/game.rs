mod barrier;
mod obstacle;
mod platform;
mod redhatboy;

use std::{collections::HashMap, rc::Rc};

use anyhow::{anyhow, Result};
pub use barrier::Barrier;
pub use obstacle::Obstacle;
pub use platform::Platform;
use rand::{prelude::*, Rng};
use redhatboy::RedHatBoy;
use serde::Deserialize;
use web_sys::HtmlImageElement;

use crate::{
    browser,
    engine::{self, Game, Image, KeyState, Point, Rect, Renderer, SpriteSheet},
    segment::{platform_and_stone, stone_and_platform},
};

const WIDTH: i16 = 600;
const HEIGHT: i16 = 600;

#[derive(Clone, Deserialize)]
pub struct SheetRect {
    x: i16,
    y: i16,
    w: i16,
    h: i16,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
    frame: SheetRect,
    sprite_source_size: SheetRect,
}

#[derive(Clone, Deserialize)]
pub struct Sheet {
    pub frames: HashMap<String, Cell>,
}

const OBSTACLE_BUFFER: i16 = 20;

pub struct Walk {
    backgrounds: [Image; 2],
    boy: RedHatBoy,
    obstacle_sheet: Rc<SpriteSheet>,
    obstacles: Vec<Box<dyn Obstacle>>,
    stone: HtmlImageElement,
    timeline: i16,
}

impl Walk {
    fn velocity(&self) -> i16 {
        -self.boy.walking_speed()
    }

    fn generate_next_segment(&mut self) {
        let mut rng = thread_rng();
        let next_segment = rng.gen_range(0..2);

        let mut next_obstacles = match next_segment {
            0 => stone_and_platform(
                self.stone.clone(),
                self.obstacle_sheet.clone(),
                self.timeline + OBSTACLE_BUFFER,
            ),
            1 => platform_and_stone(
                self.stone.clone(),
                self.obstacle_sheet.clone(),
                self.timeline + OBSTACLE_BUFFER,
            ),
            _ => vec![],
        };

        self.timeline += rightmost(&next_obstacles);
        self.obstacles.append(&mut next_obstacles);
    }
}

pub enum WalkTheDog {
    Loading,
    Loaded(Walk),
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog::Loading
    }
}

const TIMELINE_MINIMUM: i16 = 1000;

impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<impl Game + 'static>> {
        match self {
            WalkTheDog::Loading => {
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

                let sheet = serde_wasm_bindgen::from_value(
                    browser::fetch_json("assets/sprite_sheets/rhb_trimmed.json").await?,
                )
                .expect("Could not load rhb.json");
                let image = engine::load_image("assets/sprite_sheets/rhb_trimmed.png").await?;
                let boy = RedHatBoy::new(sheet, image);

                let tiles = serde_wasm_bindgen::from_value(
                    browser::fetch_json("assets/sprite_sheets/tiles.json").await?,
                )
                .expect("Could not load tiles.json");
                let image = engine::load_image("assets/sprite_sheets/tiles.png").await?;
                let obstacle_sheet = Rc::new(SpriteSheet::new(tiles, image));

                let stone =
                    engine::load_image("assets/original/freetileset/png/Object/Stone.png").await?;

                let starting_obstacles =
                    stone_and_platform(stone.clone(), obstacle_sheet.clone(), 0);
                let timeline = rightmost(&starting_obstacles);

                Ok(Box::new(WalkTheDog::Loaded(Walk {
                    backgrounds,
                    boy,
                    obstacles: starting_obstacles,
                    obstacle_sheet,
                    stone,
                    timeline,
                })))
            }
            WalkTheDog::Loaded(_) => Err(anyhow!("Error: Game is already initialized")),
        }
    }

    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(walk) = self {
            if keystate.is_pressed("ArrowRight") {
                walk.boy.run();
            }
            if keystate.is_pressed("ArrowDown") {
                walk.boy.slide();
            }
            if keystate.is_pressed("Space") {
                walk.boy.jump();
            }

            walk.boy.update();

            let velocity = walk.velocity();

            let [first_background, second_background] = &mut walk.backgrounds;
            first_background.move_horizontally(velocity);
            second_background.move_horizontally(velocity);

            if first_background.right() < 0 {
                first_background.set_x(second_background.right());
            }
            if second_background.right() < 0 {
                second_background.set_x(first_background.right());
            }

            walk.obstacles.retain(|obstacle| obstacle.right() > 0);

            walk.obstacles.iter_mut().for_each(|obstacle| {
                obstacle.move_horizontally(velocity);
                obstacle.check_intersection(&mut walk.boy);
            });

            if walk.timeline < TIMELINE_MINIMUM {
                walk.generate_next_segment();
            } else {
                walk.timeline += velocity;
            }
        }
    }

    fn draw(&self, renderer: &Renderer) {
        renderer.clear(&Rect::new_from_x_y(0, 0, WIDTH, HEIGHT));

        if let WalkTheDog::Loaded(walk) = self {
            walk.backgrounds
                .iter()
                .for_each(|background| background.draw(renderer));
            walk.boy.draw(renderer);
            walk.obstacles
                .iter()
                .for_each(|obstacle| obstacle.draw(renderer));
        }
    }
}

fn rightmost(obstacle_list: &Vec<Box<dyn Obstacle>>) -> i16 {
    obstacle_list
        .iter()
        .map(|obstacle| obstacle.right())
        .max()
        .unwrap_or(0)
}
