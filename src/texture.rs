use gl;
use gl::types::*;
use std::path::Path;
use std::os::raw::*;

use rusttype::{PositionedGlyph, Font, Scale as FontScale};

use image::{self, GenericImage, ImageBuffer, RgbaImage, Rgba, Pixel};

/// Represents an OpenGL 2D Texture
#[derive(Debug)]
pub struct Texture2D {
    id: GLuint,
    width: GLuint,
    height: GLuint,
}

impl Texture2D {
    fn gen_texture() -> GLuint {
        let mut id = unsafe {::std::mem::uninitialized()};
        unsafe {
            gl::GenTextures(1, &mut id);
        }
        id
    }

    /// the bytes SHOULD be RGBA format. For now.
    ///
    /// unexpected behavior if width and height don't match
    pub(crate) fn from_bytes(bytes: &[u8], dims: (u32, u32), ) -> Texture2D {
        debug_assert!(bytes.len() > dims.0 as usize * dims.1 as usize);
        let (width, height) = (dims.0 as GLuint, dims.1 as GLuint);
        let texture_id = Self::gen_texture();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width as i32, height as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, bytes.as_ptr() as *const c_void);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        Texture2D {
            id: texture_id,
            width: width,
            height: height
        }
    }

    // pub(crate) fn from_file<P: AsRef<Path>>(path: P) -> image::ImageResult<Texture2D> {
    //     let img = image::open(&path)?;
    //     let (img_w, img_h) = img.dimensions();

    //     let color_data: Vec<u8> = if let image::DynamicImage::ImageRgba8(rgba_data) = img {
    //         rgba_data.into_raw()
    //     } else {
    //         panic!("image {} is not RGBA", path.as_ref().display());
    //     };
    //     unsafe {
    //         Ok(Self::from_bytes(color_data.as_slice(), (img_w, img_h)))
    //     }
    // }

    // pub(crate) fn from_font(font: &Font, height: f32, text: &str) -> Texture2D {
    //     let pixel_height = height.ceil() as usize;
    //     let scale = FontScale::uniform(height);

    //     let v_metrics = font.v_metrics(scale);

    //     let offset = ::rusttype::point(0.0, v_metrics.ascent);
    //     let glyphs: Vec<PositionedGlyph> = font.layout(text, scale, offset).collect();
    //     let width = glyphs.iter().rev().map(|g| {
    //         g.position().x as f32 + g.unpositioned().h_metrics().advance_width
    //     }).next().unwrap_or(0.0).ceil();
    //     let mut rgba8_image = RgbaImage::new(width as u32, pixel_height as u32);
    //     for glyph in glyphs {
    //         if let Some(bb) = glyph.pixel_bounding_box() {
    //             glyph.draw(|x, y, v| {
    //                 let x = x as i32 + bb.min.x;
    //                 let y = y as i32 + bb.min.y;

    //                 // some fonts somehow have a value more than 1 sometimes...
    //                 // so we have to ceil at 1.0
    //                 let alpha = if v > 1.0 {
    //                     255
    //                 } else if v <= 0.0 {
    //                     0
    //                 } else {
    //                     (v * 255.0).ceil() as u8
    //                 };
    //                 if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
    //                     let x = x as u32;
    //                     let y = y as u32;
    //                     let r = ((4 * x) % 255) as u8;
    //                     let b = ((4 * y) % 255) as u8;
    //                     rgba8_image.put_pixel(x, y, Rgba::from_channels(r, b, 0, alpha));
    //                 }
    //             })
    //         }
    //     }
    //     unsafe {
    //         Self::from_bytes(&*rgba8_image, (width as u32, pixel_height as u32))
    //     }
    // }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub(crate) fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id)
        }
    }
}

impl PartialEq for Texture2D {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}