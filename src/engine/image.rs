use web_sys::HtmlImageElement;

use super::{Point, Rect, Renderer};

pub struct Image {
    element: HtmlImageElement,
    bounding_box: Rect,
}

impl Image {
    pub fn new(element: HtmlImageElement, position: Point) -> Self {
        let bounding_box = Rect::new(position, element.width() as i16, element.height() as i16);
        Image {
            element,
            bounding_box,
        }
    }

    pub fn draw(&self, renderer: &Renderer) {
        renderer.draw_entire_image(&self.element, &self.bounding_box.position);
        renderer.draw_rect(&self.bounding_box);
    }

    pub fn bounding_box(&self) -> &Rect {
        &self.bounding_box
    }

    pub fn right(&self) -> i16 {
        self.bounding_box.right()
    }

    pub fn move_horizontally(&mut self, dx: i16) {
        self.bounding_box.set_x(self.bounding_box.x() + dx);
    }

    pub fn set_x(&mut self, x: i16) {
        self.bounding_box.set_x(x);
    }
}
