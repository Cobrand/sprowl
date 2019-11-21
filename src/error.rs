
/// Describes an error that might happen when drawing something.
#[derive(Debug)]
pub enum SprowlError {
    MissingTextureId(u32),
    MissingFontId(u32)
}

impl std::fmt::Display for SprowlError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SprowlError::MissingTextureId(id) => write!(f, "texture with id {} was not found", id),
            SprowlError::MissingFontId(id) => write!(f, "font with id {} was not found", id),
        }
    }
}

impl std::error::Error for SprowlError {}