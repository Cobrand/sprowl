pub mod texture;
pub mod font;

use font::FontRenderer;
use texture::{Texture2DArray, TextureFormat, TextureArrayLayer, TextureArrayLayerRef, TextureLayerStats};

use rusttype::FontCollection;
use image::GenericImageView;

use hashbrown::HashMap;

pub type FontId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureKind {
    Grayscale,
    RGBA,
}

pub struct RenderStorage {
    current_font_id: FontId,
    pub fonts: HashMap<FontId, FontRenderer>,
    // array grayscale holds textures of 2048/2048 in grayscale, and is made for fonts.
    pub texture_array_grayscale: Texture2DArray,
    // array rgba is made for "normal" pixelperfect textures,
    pub texture_array_rgba: Texture2DArray,
}

impl RenderStorage {
    pub fn new() -> RenderStorage {
        let mut texture_array_grayscale = Texture2DArray::new(2048, 2048, 16, TextureFormat::Greyscale);
        texture_array_grayscale.set_linear(true);
        let texture_array_rgba = Texture2DArray::new(1024, 1024, 32, TextureFormat::RGBA);

        let mut render_storage = RenderStorage {
            current_font_id: 0,
            fonts: Default::default(),
            texture_array_grayscale,
            texture_array_rgba,
        };
        render_storage.set_active();
        render_storage
    }

    /// Load a font from *static* bytes. There is currently no way to dynamically load a font, for convenience only.
    ///
    /// Returns a number representing the ID of the font, which you can use later on in `draw(..)`
    ///
    /// # Panics
    ///
    /// Panics if there's more than one font
    pub fn add_font_from_bytes(&mut self, bytes: &'static [u8]) -> FontId {
        let collection = FontCollection::from_bytes(bytes).expect("wrong font added from static bytes");
        let font = collection.into_font().expect("fatal: collection consists of more than one font"); // only succeeds if collection consists of one font

        let grayscale_layer = self.texture_array_grayscale.add_empty_texture(2048, 2048);

        let _v = self.fonts.insert(self.current_font_id, FontRenderer::new(font, grayscale_layer));
        debug_assert!(_v.is_none());
        let font_id = self.current_font_id;
        self.current_font_id += 1;
        font_id
    }

    /// Load a texture from bytes: you must specify the correct width and height of the texture.
    ///
    /// # Panics
    ///
    /// * (debug only) if the size is incorrect (higher than the slice's)
    /// * (debug only) if the amount of textures  recorded is higher than u32::MAX_VALUE
    pub fn add_texture_from_raw_bytes(&mut self, bytes: &[u8], size: (u32, u32)) -> TextureArrayLayer {
        self.texture_array_rgba.add_texture(bytes, size.0, size.1)
    }

    /// Load a texture from some bytes. Preferably, the image should be PNG with an alpha layer.
    /// Returns a number representing the ID of the texture, which you can use later on in `draw(..)`
    ///
    /// # Panics
    /// 
    /// Panics if the image is not RGBA (the texture has no alpha layer)
    pub fn add_texture_from_image_bytes(&mut self, bytes: &[u8], image_format: Option<image::ImageFormat>) -> image::ImageResult<u32> {
        let opened_image = match image_format {
            Some(image_format) => image::load_from_memory_with_format(bytes, image_format),
            None => image::load_from_memory(bytes)
        }?;
        let img_w = opened_image.width();
        let img_h = opened_image.height();

        let color_data: Vec<u8> = if let image::DynamicImage::ImageRgba8(rgba_data) = opened_image {
            rgba_data.into_raw()
        } else {
            panic!("image loaded from memory is not RGBA");
        };
        Ok(self.add_texture_from_raw_bytes(color_data.as_slice(), (img_w, img_h)))
    }

    pub fn get_font(&mut self, font_id: FontId) -> Option<&mut FontRenderer> {
        self.fonts.get_mut(&font_id)
    }

    pub fn get_font_with_texture<'a>(&'a mut self, font_id: FontId) -> Option<(&'a mut FontRenderer, TextureArrayLayerRef<'a>)> {
        let texture_2d_array_ref = &mut self.texture_array_grayscale;
        self.fonts.get_mut(&font_id).map(move |font_renderer| {
            let texture_layer = font_renderer.texture_layer;
            (
                font_renderer,
                TextureArrayLayerRef::new(texture_2d_array_ref, texture_layer)
            )
        })
    }

    pub fn set_active(&mut self) {
        self.texture_array_rgba.set_active(0);
        self.texture_array_grayscale.set_active(1);
    }

    pub fn get_stats(&self, layer: TextureArrayLayer) -> TextureLayerStats {
        self.texture_array_rgba.stats[layer as usize]
    }

    pub fn get_max_dims(&self, texture_kind: TextureKind) -> (u32, u32) {
        let t = match texture_kind {
            TextureKind::Grayscale => &self.texture_array_grayscale,
            TextureKind::RGBA => &self.texture_array_rgba,
        };
        (t.max_width, t.max_height)
    }
}