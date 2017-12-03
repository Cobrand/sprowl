#[derive(Debug, Fail)]
pub enum SprowlError {
    #[fail(display = "texture with id {} was not found", _0)]
    MissingTextureID(u32),
    #[fail(display = "font with id {} was not found", _0)]
    MissingFontId(u32)
}