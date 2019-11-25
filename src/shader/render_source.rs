use crate::texture::Texture2D;
use crate::utils::Shape;

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