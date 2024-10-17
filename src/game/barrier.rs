use anyhow::Result;

use crate::engine::Image;

use super::obstacle::Obstacle;

pub struct Barrier {
    image: Image,
}

impl Barrier {
    pub fn new(image: Image) -> Self {
        Barrier { image }
    }
}

impl Obstacle for Barrier {
    fn check_intersection(&self, boy: &mut super::redhatboy::RedHatBoy) {
        if boy.bounding_box().intersects(self.image.bounding_box()) {
            boy.knock_out();
        }
    }

    fn draw(&self, renderer: &crate::engine::Renderer) -> Result<()> {
        self.image.draw(renderer)
    }

    fn move_horizontally(&mut self, dx: i16) {
        self.image.move_horizontally(dx);
    }

    fn right(&self) -> i16 {
        self.image.bounding_box().right()
    }
}
