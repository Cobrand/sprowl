use rusttype::{Font, Scale as FontScale};
use cgmath::Vector2;
use crate::font_cache::Cache as FontCache;

use crate::texture::{TextureFormat, Texture2D};

pub struct FontRenderer {
    pub (crate) font_cache: FontCache,
    pub (crate) tex: Texture2D,
    pub (crate) font: Font<'static>,
}

pub struct FontStemDrawCall<'a> {
    pub source_crop: (i32, i32, u32, u32),
    pub dest_origin: Vector2<i32>,
    pub texture: &'a Texture2D,
    pub character_index: usize,
}

impl FontRenderer {
    pub fn new(font: Font<'static>) -> FontRenderer {
        const CACHE_WIDTH: usize = 1024;
        FontRenderer {
            font_cache: FontCache::builder()
                .dimensions(CACHE_WIDTH as u32, CACHE_WIDTH as u32)
                .pad_glyphs(true)
                .align_4x4(true)
                .build(),
            tex: Texture2D::from_bytes_with_format(None, (CACHE_WIDTH as u32, CACHE_WIDTH as u32), TextureFormat::Greyscale),
            font,
        }
    }

    #[inline]
    pub fn font(&self) -> &Font<'static> {
        &self.font
    }

    pub fn texture(&self) -> &Texture2D {
        &self.tex
    }

    pub fn word_to_draw_call<'a, 'b>(&'a mut self, text: &'b str, font_size: f32, origin: Vector2<i32>) -> Vec<FontStemDrawCall<'a>> {
        let scale = FontScale::uniform(font_size);

        let advance = self.font().v_metrics(scale).ascent.round() as i32;
        let glyphs = self.font.layout(text, scale, rusttype::point(origin.x as f32, origin.y as f32)).enumerate().collect::<Vec<_>>();

        let tex = &mut self.tex;
        let r = self.font_cache.cache_glyphs(glyphs.iter().map(|(_, c)| c), |rect, data| {
            let rusttype::Point { x, y } = rect.min;
            let width = rect.width();
            let height = rect.height();
            tex.update(data, x as i32, y as i32, width, height, TextureFormat::Greyscale);
        });
        r.expect("failed to write to font gpu cache");

        let (tex_w, tex_h) = self.tex.size();
        let (tex_w, tex_h) = (tex_w as f32, tex_h as f32);

        let mut results: Vec<FontStemDrawCall<'a>> = Vec::with_capacity(glyphs.len());
        for (i, glyph) in &glyphs {
            if let Ok(Some((uv_rect, screen_rect))) = self.font_cache.rect_for(glyph) {
                let source_crop: (i32, i32, u32, u32) = (
                    (uv_rect.min.x * tex_w) as i32,
                    (uv_rect.min.y * tex_h) as i32,
                    (uv_rect.width() * tex_w) as u32,
                    (uv_rect.height() * tex_h) as u32, 
                );
                results.push(FontStemDrawCall {
                    source_crop,
                    dest_origin: Vector2::new(screen_rect.min.x, screen_rect.min.y + advance),
                    texture: &self.tex,
                    character_index: *i,
                });
            }
        }
        results
    }
}