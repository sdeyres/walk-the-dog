mod barrier;
mod obstacle;
mod platform;
mod redhatboy;
mod walk_the_dog;

use std::rc::Rc;

use anyhow::Result;
pub use barrier::Barrier;
pub use obstacle::Obstacle;
pub use platform::Platform;
use rand::{prelude::*, Rng};
pub use redhatboy::RedHatBoy;
pub use walk_the_dog::WalkTheDog;
use web_sys::HtmlImageElement;

use crate::{
    engine::{Image, Renderer, SpriteSheet},
    segment::{platform_and_stone, stone_and_platform},
};

const WIDTH: i16 = 600;
const HEIGHT: i16 = 600;
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
    fn draw(&self, renderer: &Renderer) -> Result<()> {
        self.backgrounds
            .iter()
            .try_for_each(|background| -> Result<()> {
                background.draw(renderer)
            })?;
        self.boy.draw(renderer)?;
        self.obstacles
            .iter()
            .try_for_each(|obstacle| -> Result<()> { obstacle.draw(renderer) })
    }

    fn knocked_out(&self) -> bool {
        self.boy.knocked_out()
    }

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

    fn reset(walk: Self) -> Self {
        let starting_obstacles =
            stone_and_platform(walk.stone.clone(), walk.obstacle_sheet.clone(), 0);
        let timeline = rightmost(&starting_obstacles);

        Walk {
            backgrounds: walk.backgrounds,
            boy: RedHatBoy::reset(walk.boy),
            obstacle_sheet: walk.obstacle_sheet,
            obstacles: starting_obstacles,
            stone: walk.stone,
            timeline,
        }
    }
}

const TIMELINE_MINIMUM: i16 = 1000;

fn rightmost(obstacle_list: &[Box<dyn Obstacle>]) -> i16 {
    obstacle_list
        .iter()
        .map(|obstacle| obstacle.right())
        .max()
        .unwrap_or(0)
}
