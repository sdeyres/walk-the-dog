use super::Point;

#[derive(Default)]
pub struct Rect {
    pub position: Point,
    pub width: i16,
    pub height: i16,
}

impl Rect {
    pub const fn new(position: Point, width: i16, height: i16) -> Self {
        Rect {
            position,
            width,
            height,
        }
    }

    pub const fn new_from_x_y(x: i16, y: i16, width: i16, height: i16) -> Self {
        let position = Point { x, y };
        Rect {
            position,
            width,
            height,
        }
    }

    pub fn set_x(&mut self, x: i16) {
        self.position.x = x;
    }

    pub fn x(&self) -> i16 {
        self.position.x
    }

    pub fn y(&self) -> i16 {
        self.position.y
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x() < other.right()
            && self.right() > other.x()
            && self.y() < other.bottom()
            && self.bottom() > other.y()
    }

    pub fn right(&self) -> i16 {
        self.x() + self.width
    }

    pub fn bottom(&self) -> i16 {
        self.y() + self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_rects_that_intersect_on_the_left() {
        let rect = Rect {
            position: Point { x: 10, y: 10 },
            width: 100,
            height: 100
        };

        let other = Rect {
            position: Point { x: 0, y: 10 },
            width: 100,
            height: 100,
        };

        assert!(rect.intersects(&other));
        assert!(other.intersects(&rect));
    }

    #[test]
    fn two_rects_that_do_not_intersect() {
        let rect = Rect {
            position: Point { x: 0, y: 0 },
            width: 5,
            height: 5,
        };

        let other = Rect {
            position: Point { x: 10, y: 10 },
            width: 5,
            height: 5,
        };

        assert!(!rect.intersects(&other));
        assert!(!other.intersects(&rect));
    }
}
