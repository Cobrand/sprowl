use rusttype::gpu_cache::Cache as FontCacheData;
use rusttype::Font;

use crate::texture::Texture2D;

pub struct FontRenderer {
    pub (crate) font_cache_data: FontCacheData<'static>,
    pub (crate) tex: Texture2D,
    pub (crate) font: Font<'static>,
}

impl FontRenderer {
    pub fn new(font: Font<'static>) -> FontRenderer {
        let transparent_bytes = vec!(0u8; 2048 * 2048 * 4 );
        FontRenderer {
            font_cache_data: FontCacheData::builder().dimensions(2048, 2048).build(),
            tex: Texture2D::from_bytes(&transparent_bytes, (2048, 2048)),
            font,
        }
    }
}