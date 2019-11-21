use rusttype::gpu_cache::Cache as FontCacheData;
use rusttype::{Font, PositionedGlyph};

use crate::texture::{TextureFormat, Texture2D};

pub struct FontRenderer {
    pub (crate) font_cache_data: FontCacheData<'static>,
    pub (crate) tex: Texture2D,
    pub (crate) font: Font<'static>,
}

// courtesy of rusttype/examples/gpu_cache.rs
pub (crate) fn layout_paragraph<'a>(font: &'a Font, scale: rusttype::Scale, max_width: u32, text: &str) -> Vec<PositionedGlyph<'a>> {
    let mut result = Vec::new();
    let v_metrics = font.v_metrics(scale);
    let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
    let mut caret = rusttype::point(0.0, v_metrics.ascent);
    let mut last_glyph_id = None;
    for c in text.chars() {
        let base_glyph = font.glyph(c);
        if let Some(id) = last_glyph_id.take() {
            caret.x += font.pair_kerning(scale, id, base_glyph.id());
        }
        last_glyph_id = Some(base_glyph.id());
        let mut glyph = base_glyph.scaled(scale).positioned(caret);
        if let Some(bb) = glyph.pixel_bounding_box() {
            if bb.max.x > max_width as i32 {
                caret = rusttype::point(0.0, caret.y + advance_height);
                glyph.set_position(caret);
                last_glyph_id = None;
            }
        }
        caret.x += glyph.unpositioned().h_metrics().advance_width;
        result.push(glyph);
    }
    result
}

impl FontRenderer {
    pub fn new(font: Font<'static>) -> FontRenderer {
        let transparent_bytes = vec!(0u8; 2048 * 2048);
        FontRenderer {
            font_cache_data: FontCacheData::builder().dimensions(2048, 2048).build(),
            tex: Texture2D::from_bytes_with_format(&transparent_bytes, (2048, 2048), TextureFormat::Greyscale),
            font,
        }
    }
}