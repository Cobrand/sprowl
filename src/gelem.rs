use crate::utils::Shape;

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