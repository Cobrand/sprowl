use rusttype::Font;
use crate::font_cache::Cache as FontCache;

use crate::texture::{TextureFormat, Texture2D};

pub struct FontRenderer {
    pub (crate) font_cache: FontCache,
    pub (crate) tex: Texture2D,
    pub (crate) font: Font<'static>,
}

impl FontRenderer {
    pub fn new(font: Font<'static>) -> FontRenderer {
        const CACHE_WIDTH: usize = 1024;
        FontRenderer {
            font_cache: FontCache::builder()
                .dimensions(CACHE_WIDTH as u32, CACHE_WIDTH as u32)
                .pad_glyphs(false)
                .align_4x4(true)
                .build(),
            tex: Texture2D::from_bytes_with_format(None, (CACHE_WIDTH as u32, CACHE_WIDTH as u32), TextureFormat::Greyscale),
            font,
        }
    }
}