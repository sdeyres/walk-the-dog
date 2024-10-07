use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;
use web_sys::HtmlImageElement;

use crate::browser;

use super::{Point, Rect, Renderer};

pub struct SpriteSheet {
    sheet: Sheet,
    image: HtmlImageElement,
}

#[derive(Clone, Deserialize)]
struct SheetRect {
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

impl Cell {
    pub fn frame(&self) -> Rect {
        Rect::new_from_x_y(self.frame.x, self.frame.y, self.frame.w, self.frame.h)
    }

    pub fn destination(&self, position: &Point) -> Rect {
        Rect::new_from_x_y(
            position.x + self.sprite_source_size.x,
            position.y + self.sprite_source_size.y,
            self.frame.w,
            self.frame.h,
        )
    }
}

#[derive(Clone, Deserialize)]
struct Sheet {
    frames: HashMap<String, Cell>,
}

impl SpriteSheet {
    pub async fn new(json_resource: &str, image_resource: &str) -> Result<Self> {
        let sheet = serde_wasm_bindgen::from_value(browser::fetch_json(json_resource).await?)
            .expect(&format!("Could not load JSON resource {}", json_resource));
        let image = super::load_image(image_resource).await?;
        Ok(SpriteSheet { sheet, image })
    }

    pub fn cell(&self, name: &str) -> Option<&Cell> {
        self.sheet.frames.get(name)
    }

    pub fn draw(&self, renderer: &Renderer, source: &Rect, destination: &Rect) {
        renderer.draw_image(&self.image, source, destination);
    }
}
