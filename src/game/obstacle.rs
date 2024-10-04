use crate::engine::Renderer;

use super::redhatboy::RedHatBoy;

pub trait Obstacle {
    fn check_intersection(&self, boy: &mut RedHatBoy);
    fn draw(&self, renderer: &Renderer);
    fn move_horizontally(&mut self, dx: i16);
    fn right(&self) -> i16;
}