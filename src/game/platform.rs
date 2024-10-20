use std::rc::Rc;

use anyhow::Result;

use crate::engine::{Cell, Point, Rect, Renderer, SpriteSheet};

use super::{Obstacle, RedHatBoy};

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
    fn check_intersection(&self, boy: &mut RedHatBoy) {
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

    fn draw(&self, renderer: &Renderer) -> Result<()> {
        let mut dx = 0;

        self.sprites.iter().try_for_each(|sprite| -> Result<()> {
            self.sheet.draw(
                renderer,
                &sprite.frame(),
                &sprite.destination(&Point {
                    x: self.position.x + dx,
                    y: self.position.y,
                }),
            )?;
            dx += sprite.frame().width;
            Ok(())
        })?;

        if cfg!(debug_assertions) {
            for rect in self.bounding_boxes() {
                renderer.draw_rect(rect);
            }
        }

        Ok(())
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
