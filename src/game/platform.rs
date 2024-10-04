use std::rc::Rc;

use crate::engine::{Point, Rect, Renderer, SpriteSheet};

use super::{obstacle::Obstacle, Cell};

pub struct Platform {
    bounding_boxes: Vec<Rect>,
    position: Point,
    sheet: Rc<SpriteSheet>,
    sprites: Vec<Cell>,
}

impl Platform {
    pub fn new(
        bounding_boxes: &[Rect],
        position: Point,
        sheet: Rc<SpriteSheet>,
        sprite_names: &[&str],
    ) -> Self {
        let bounding_boxes = bounding_boxes
            .iter()
            .map(|bounding_box| {
                Rect::new_from_x_y(
                    bounding_box.x() + position.x,
                    bounding_box.y() + position.y,
                    bounding_box.width,
                    bounding_box.height,
                )
            })
            .collect();
        let sprites = sprite_names
            .iter()
            .filter_map(|sprite_name| sheet.cell(sprite_name).cloned())
            .collect();
        Platform {
            bounding_boxes,
            position,
            sheet,
            sprites,
        }
    }

    fn bounding_boxes(&self) -> &Vec<Rect> {
        &self.bounding_boxes
    }
}

impl Obstacle for Platform {
    fn check_intersection(&self, boy: &mut super::redhatboy::RedHatBoy) {
        if let Some(box_to_land_on) = self
            .bounding_boxes()
            .iter()
            .find(|&bounding_box| boy.bounding_box().intersects(bounding_box))
        {
            if boy.velocity_y() > 0 && boy.pos_y() < self.position.y {
                boy.land_on(box_to_land_on.y());
            } else {
                boy.knock_out();
            }
        }
    }

    fn draw(&self, renderer: &Renderer) {
        let mut dx = 0;

        self.sprites.iter().for_each(|sprite| {
            self.sheet.draw(
                renderer,
                &Rect::new_from_x_y(
                    sprite.frame.x,
                    sprite.frame.y,
                    sprite.frame.w,
                    sprite.frame.h,
                ),
                &Rect::new_from_x_y(
                    self.position.x + dx,
                    self.position.y,
                    sprite.frame.w,
                    sprite.frame.h,
                ),
            );
            dx += sprite.frame.w;
        });

        for rect in self.bounding_boxes() {
            renderer.draw_rect(rect);
        }
    }

    fn move_horizontally(&mut self, dx: i16) {
        self.position.x += dx;
        self.bounding_boxes.iter_mut().for_each(|bounding_box| {
            bounding_box.set_x(bounding_box.x() + dx);
        });
    }

    fn right(&self) -> i16 {
        self.bounding_boxes()
            .last()
            .unwrap_or(&Rect::default())
            .right()
    }
}
