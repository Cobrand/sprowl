
use crate::texture::{Texture2D, Texture2DRef};
use crate::utils::Shape;

/// Represents something your shader would like to draw. Typically, a texture or a shape.
#[derive(Clone, Copy, Debug)]
pub enum RenderSource {
    Texture(Texture2DRef),
    Shape(Shape),
}

impl RenderSource {
    pub fn size(&self) -> (u32, u32) {
        match self {
            RenderSource::Texture(t) => t.size(),
            RenderSource::Shape(s) => s.max_size(),
        }
    }

    #[inline]
    pub fn compute_draw_vbo(&self, crop: Option<(i32, i32, u32, u32)>) -> [f32; 24] {
        if let Some((x, y, w, h)) = crop {
            let (t_w, t_h) = self.size();
            let (t_w, t_h) = (t_w as f32, t_h as f32);
            let (x, y, w, h) = (x as f32 / t_w, y as f32 / t_h, w as f32 / t_w, h as f32 / t_h);
            let left = x;
            let bottom = y + h;
            let right =  x + w;
            let top = y;
            [
                left, bottom, left, bottom,
                right, top, right, top,
                left, top, left, top,
                left, bottom, left, bottom,
                right, bottom, right, bottom,
                right, top, right, top,
            ]
        } else {
            [0.0, 1.0, 0.0, 1.0, // 0
            1.0, 0.0, 1.0, 0.0, // 1
            0.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 1.0,
            1.0, 1.0, 1.0, 1.0,
            1.0, 0.0, 1.0, 0.0]
        }
    }
}

impl<'a> From<&'a Texture2D> for RenderSource {
    fn from(t: &'a Texture2D) -> RenderSource {
        RenderSource::Texture(t.as_ref())
    }
}

impl<'a> From<&'a Shape> for RenderSource {
    fn from(s: &'a Shape) -> RenderSource {
        RenderSource::Shape(*s)
    }
}

/// Describes something to be drawn with a given shader.
///
/// The big 3 include: a texture, a shape, a text.
pub enum RenderStem<S: AsRef<str>> {
    Texture {
        /// The ID that was returned by add_texture_*
        id: u32,
    },
    Shape {
        shape: Shape,
    },
    Text {
        /// The ID that was returned by add_font_*
        font_id: u32,
        /// The font size, in pixels
        font_size: f32,
        /// The text that should be printed
        text: S,
    }
}

impl<S: AsRef<str> + std::fmt::Debug> std::fmt::Debug for RenderStem<S> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderStem::Texture { id } => {
                fmt.debug_struct("GraphicEntity::Texture").field("id", id).finish()
            }
            RenderStem::Shape { shape } => {
                fmt.debug_struct("GraphicEntity::Shape").field("shape", shape).finish()
            },
            RenderStem::Text { font_id, font_size, text } => {
                fmt.debug_struct("GraphicEntity::Text")
                    .field("font_id", font_id)
                    .field("font_size", font_size)
                    .field("text", text)
                    .finish()
            }
        }
    }
}

/// Represents a given entity (texture, text, shape) with set parameters ready for drawing.
///
/// The parameters depends on the shader you are using.
#[must_use]
pub struct GraphicElement<S: AsRef<str>, R: Clone> {
    pub render_stem: RenderStem<S>,
    pub render_params: R,
}

impl<S: AsRef<str> + std::fmt::Debug, R: std::fmt::Debug + Clone> std::fmt::Debug for GraphicElement<S, R> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("GraphicElement")
            .field("render_stem", &self.render_stem)
            .field("render_params", &self.render_params)
            .finish()
    }
}