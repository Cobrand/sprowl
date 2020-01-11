use gl::types::*;
use std::os::raw::c_void;

pub type TextureArrayLayer = u32;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TextureFormat {
    RGBA,
    Greyscale,
}

impl TextureFormat {
    pub (crate) fn to_gl_format(self) -> gl::types::GLenum {
        match self {
            TextureFormat::Greyscale => gl::RED,
            TextureFormat::RGBA => gl::RGBA,
        }
    }

    pub (crate) fn bytes(self) -> usize {
        match self {
            TextureFormat::Greyscale => 1,
            TextureFormat::RGBA => 4,
        }
    }
}

#[derive(Debug)]
pub struct TextureArrayLayerRef<'a> {
    pub (crate) texture_array: &'a mut Texture2DArray,
    pub (crate) layer: TextureArrayLayer,
}

impl<'a> TextureArrayLayerRef<'a> {
    pub fn new(texture_array: &'a mut Texture2DArray, layer: TextureArrayLayer) -> TextureArrayLayerRef<'a> {
        TextureArrayLayerRef {
            texture_array,
            layer
        }
    }

    pub fn update(&mut self, bytes: &[u8], x_offset: i32, y_offset: i32, width: u32, height: u32) {
        self.texture_array.update_texture(self.layer, bytes, x_offset, y_offset, width, height);
    }

    pub fn stats(&self) -> TextureLayerStats {
        self.texture_array.stats[self.layer as usize]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextureLayerStats {
    pub width: GLuint,
    pub height: GLuint,
}

impl TextureLayerStats {
    pub fn new(width: GLuint, height: GLuint) -> TextureLayerStats {
        TextureLayerStats {
            width,
            height,
        }
    }

    pub fn size(&self) -> (GLuint, GLuint) {
        (self.width, self.height)
    }
}

#[derive(Debug)]
pub struct Texture2DArray {
    pub (crate) format: TextureFormat,
    pub (crate) id: GLuint,
    pub (crate) max_layers: GLuint,
    pub (crate) max_width: GLuint,
    pub (crate) max_height: GLuint,
    // stores the dimension of every texture.
    pub (crate) stats: Vec<TextureLayerStats>,
}

/// Represents an array of RGBA textures.
impl Texture2DArray {
    fn gen_texture() -> GLuint {
        let mut id = std::mem::MaybeUninit::uninit();
        unsafe {
            gl::GenTextures(1, id.as_mut_ptr());
            id.assume_init()
        }
    }

    pub fn new(width: GLuint, height: GLuint, max_layers: GLuint, format: TextureFormat) -> Texture2DArray {
        let id = Self::gen_texture();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, id);
            // allocate the storage for the texture array
            gl::TexImage3D(
                gl::TEXTURE_2D_ARRAY,
                // only use 1 level for the mipmap (so value=0)
                0,
                format.to_gl_format() as GLint,
                width as GLint,
                height as GLint,
                max_layers as GLint,
                // border must always be 0
                0,
                format.to_gl_format(),
                gl::UNSIGNED_BYTE,
                // fill with void
                std::ptr::null()
            );

            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::MIRRORED_REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::MIRRORED_REPEAT as GLint);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
        }
        Texture2DArray {
            id,
            max_layers,
            max_width: width,
            max_height: height,
            stats: Vec::with_capacity(max_layers as usize),
            format,
        }
    }

    /// Set the MIN and MAG filter to linear instead of NEAREST
    pub fn set_linear(&mut self, flag: bool) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.id);
            let flag = if flag { gl::LINEAR as GLint } else { gl::NEAREST as GLint };
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, flag);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER, flag);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
        }
    }

    pub fn set_active(&self, index: GLuint) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + index);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.id);
        }
    }

    pub fn add_texture(&mut self, bytes: &[u8], width: GLuint, height: GLuint) -> TextureArrayLayer {
        debug_assert!(bytes.len() >= width as usize * height as usize * self.format.bytes());

        let next_layer = self.stats.len() as GLint;

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.id);
            gl::TexSubImage3D(
                gl::TEXTURE_2D_ARRAY,
                0, // mipmap 0
                0, // xoffset = 0
                0, // yoffset = 0
                next_layer, // layer to update (create)
                width as GLint,
                height as GLint,
                1, // only one depth to update
                self.format.to_gl_format(),
                gl::UNSIGNED_BYTE, bytes.as_ptr() as *const c_void
            );
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
        }

        self.stats.push(TextureLayerStats::new(width, height));

        next_layer as u32
    }

    pub fn add_empty_texture(&mut self, width: GLuint, height: GLuint) -> TextureArrayLayer {
        let next_layer = self.stats.len() as GLint;
        self.stats.push(TextureLayerStats::new(width, height));
        next_layer as u32
    }

    pub fn update_texture(&mut self, layer: TextureArrayLayer, bytes: &[u8], xoffset: GLint, yoffset: GLint, width: GLuint, height: GLuint) {
        debug_assert!(bytes.len() >= width as usize * height as usize * self.format.bytes());
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.id);
            gl::TexSubImage3D(
                gl::TEXTURE_2D_ARRAY,
                0, // mipmap 0
                xoffset, // xoffset
                yoffset, // yoffset
                layer as GLint, // layer to update
                width as GLint,
                height as GLint,
                1, // only one depth to update
                self.format.to_gl_format(),
                gl::UNSIGNED_BYTE, bytes.as_ptr() as *const c_void
            );
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
        }
    }
}

impl Drop for Texture2DArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id)
        }
    }
}