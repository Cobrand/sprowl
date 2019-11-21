pub use crate::texture::Texture2D;

pub use crate::gelem::*;

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

    /// Computes the real origin position, width and height are of the entity you want to draw.
    pub fn compute(&self, width: u32, height: u32) -> (i32, i32) {
        match self {
            Origin::Center => ((width / 2) as i32, (height / 2) as i32),
            Origin::TopLeft(x, y) => (x.clone(), y.clone())
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DrawPos {
    pub origin: Origin,
    /// where on x the origin should be drawn on screen?
    pub x: i32,
    /// where on y the origin should be drawn on screen?
    pub y: i32,
}

#[derive(Debug)]
/// A set of common render parameters, that every shader should take into account.
pub struct CommonRenderParams {
    pub draw_pos: DrawPos,
    /// (origin_x, origin_y, width, height)
    pub crop: Option<(i32, i32, u32, u32)>,
    pub is_source_grayscale: bool,
}

impl CommonRenderParams {
    pub fn new(draw_pos: DrawPos) -> CommonRenderParams {
        CommonRenderParams {
            draw_pos,
            crop: None,
            is_source_grayscale: false,
        }
    }
}

/// Render Parameters for some shader, containing a common part (position, crop, is_grayscale, ...) and a custom part
pub struct RenderParams<R> {
    pub common: CommonRenderParams,
    pub custom: R,
}

impl<R: Default> RenderParams<R> {
    pub fn new(draw_pos: DrawPos) -> RenderParams<R> {
        RenderParams {
            common: CommonRenderParams::new(draw_pos),
            custom: Default::default(),
        }
    }
}

impl<R: std::fmt::Debug> std::fmt::Debug for RenderParams<R> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("RenderParams")
            .field("common", &self.common)
            .field("custom", &self.custom)
            .finish()
    }
}

/// Represents something your shader would like to draw. Typically, a texture or a shape.
pub enum RenderSource<'a> {
    Texture(&'a Texture2D),
    Shape(&'a Shape),
}

impl<'a> RenderSource<'a> {
    pub fn size(&self) -> (u32, u32) {
        match self {
            RenderSource::Texture(t) => t.size(),
            RenderSource::Shape(s) => s.max_size(),
        }
    }
}

impl<'a> From<&'a Texture2D> for RenderSource<'a> {
    fn from(t: &'a Texture2D) -> RenderSource<'a> {
        RenderSource::Texture(t)
    }
}

impl<'a> From<&'a Shape> for RenderSource<'a> {
    fn from(s: &'a Shape) -> RenderSource<'a> {
        RenderSource::Shape(s)
    }
}

/// Represents a simple shape: rect, circle, triangle, ect. Used when you want to draw without a texture.
#[derive(Debug, Clone, Copy)]
pub enum Shape {
    Rect(u32, u32),
}

impl Shape {
    pub fn max_size(&self) -> (u32, u32) {
        match self {
            Shape::Rect(w, h) => (*w, *h),
        }
    }
}