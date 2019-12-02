use cgmath::Vector2;

pub type Pos = Vector2<i32>;

/// Represents where the anchor "origin" of something should be.
///
/// For instance, if you want to rotate something around its center, you would choose "Center".
#[derive(Copy, Clone, Debug)]
pub enum Origin {
    Center,
    TopLeft(i32, i32),
}

impl Origin {
    pub fn new() -> Origin {
        Origin::TopLeft(0, 0)
    }

    /// Computes the real origin position, relative to the topleft of the entity.
    pub fn compute_relative_origin(&self, Vector2 { x, y } : Vector2<u32>) -> Pos {
        match self {
            Origin::Center => Vector2::new((x / 2) as i32, (y / 2) as i32),
            Origin::TopLeft(x, y) => Vector2::new(*x, *y)
        }
    }
}

impl Default for Origin {
    fn default() -> Origin {
        Origin::new()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DrawPos {
    pub pos: Pos,
    pub origin: Origin,
}

impl DrawPos {
    pub fn new<I: Into<Pos>>(pos: I) -> DrawPos {
        DrawPos { pos: pos.into(), origin: Origin::new() }
    }
}

/// Represents a simple shape: rect, circle, triangle, ect. Used when you want to draw without a texture.
#[derive(Debug, Clone, Copy)]
pub enum Shape {
    Rect(u32, u32),
    Circle(u32),
}

impl Shape {
    pub fn max_size(self) -> (u32, u32) {
        match self {
            Shape::Rect(w, h) => (w, h),
            Shape::Circle(w) => (w, w),
        }
    }
}