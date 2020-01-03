use gl;
use gl::types::*;
use std::os::raw::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Texture2DArrayRef {
    pub (crate) tex_id: GLuint,
    pub (crate) layer: GLuint,
}

#[derive(Debug)]
pub struct Texture2DArray {
    pub (crate) format: TextureFormat,
    pub (crate) id: GLuint,
    pub (crate) max_layers: GLuint,
    pub (crate) max_width: GLuint,
    pub (crate) max_height: GLuint,
    // stores the dimension of every texture.
    pub (crate) stats: Vec<(GLuint, GLuint)>,
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

    /// Returns the number of bytes to offset whenever we want to access the index `index`.
    fn offset(&self, index: u32) -> GLuint {
        index * (self.max_width * self.max_height)
    }

    pub fn new(width: GLuint, height: GLuint, max_layers: GLuint) -> Texture2DArray {
        let id = Self::gen_texture();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, id);
            // allocate the storage for the texture array
            gl::TexImage2D(
                gl::TEXTURE_2D_ARRAY,
                // only use 1 level for the mipmap (so value=0)
                0,
                gl::RGBA as GLint,
                width as GLint,
                height as GLint,
                // border must always be 0
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                // fill with void
                std::ptr::null()
            );

            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
        }
        Texture2DArray {
            id,
            max_layers,
            max_width: width,
            max_height: height,
            stats: Vec::with_capacity(32),
            format: TextureFormat::RGBA,
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
        }
    }

    pub fn add_texture(&mut self, bytes: &[u8], width: GLuint, height: GLuint) -> Texture2DArrayRef {
        debug_assert!(bytes.len() >= width as usize * height as usize * self.format.bytes());

        let next_layer = self.stats.len() as GLint;

        self.bind();
        unsafe {
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
        }
        self.unbind();

        self.stats.push((width, height));

        Texture2DArrayRef {
            layer: next_layer as GLuint,
            tex_id: self.id,
        }
    }

    pub fn update_texture(&mut self, tex_ref: &Texture2DArrayRef, bytes: &[u8], xoffset: GLint, yoffset: GLint, width: GLuint, height: GLuint) {
        assert_eq!(tex_ref.tex_id, self.id, "texture arrays ids are not the same: trying to update a texture to the wrong array");
        self.bind();
        unsafe {
            gl::TexSubImage3D(
                gl::TEXTURE_2D_ARRAY,
                0, // mipmap 0
                xoffset, // xoffset = 0
                yoffset, // yoffset = 0
                tex_ref.layer as GLint, // layer to update (create)
                width as GLint,
                height as GLint,
                1, // only one depth to update
                self.format.to_gl_format(),
                gl::UNSIGNED_BYTE, bytes.as_ptr() as *const c_void
            );
        }
        self.unbind();
    }
}

impl Drop for Texture2DArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id)
        }
    }
}

/// Represents an OpenGL 2D Texture.
///
/// Should be created via the Canvas, and rarely used manually.
#[derive(Debug)]
pub struct Texture2D {
    pub (crate) id: GLuint,
    width: GLuint,
    height: GLuint,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TextureFormat {
    RGBA,
    Greyscale,
}

impl TextureFormat {
    fn to_gl_format(self) -> gl::types::GLenum {
        match self {
            TextureFormat::Greyscale => gl::RED,
            TextureFormat::RGBA => gl::RGBA,
        }
    }

    fn bytes(self) -> usize {
        match self {
            TextureFormat::Greyscale => 1,
            TextureFormat::RGBA => 4,
        }
    }
}

impl Texture2D {
    fn gen_texture() -> GLuint {
        let mut id = std::mem::MaybeUninit::uninit();
        unsafe {
            gl::GenTextures(1, id.as_mut_ptr());
            id.assume_init()
        }
    }

    pub (crate) fn as_ref(&self) -> Texture2DRef {
        Texture2DRef {
            id: self.id,
            width: self.width,
            height: self.height,
        }
    }

    /// the bytes SHOULD be RGBA format.
    ///
    /// bytes = None will create a texture filled with 0s
    ///
    /// unexpected behavior if width and height don't match
    pub (crate) fn new(bytes: Option<&[u8]>, dims: (u32, u32), ) -> Texture2D {
        Self::from_bytes_with_format(bytes, dims, TextureFormat::RGBA)
    }

    pub (crate) fn from_bytes_with_format(bytes: Option<&[u8]>, dims: (u32, u32), format: TextureFormat) -> Texture2D {
        let (width, height) = (dims.0 as GLuint, dims.1 as GLuint);
        let texture_id = Self::gen_texture();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            if let Some(bytes) = bytes {
                debug_assert!(bytes.len() >= dims.0 as usize * dims.1 as usize * format.bytes());
                gl::TexImage2D(gl::TEXTURE_2D, 0, format.to_gl_format() as i32, width as i32, height as i32, 0, format.to_gl_format(), gl::UNSIGNED_BYTE, bytes.as_ptr() as *const c_void);
            } else {
                gl::TexImage2D(gl::TEXTURE_2D, 0, format.to_gl_format() as i32, width as i32, height as i32, 0, format.to_gl_format(), gl::UNSIGNED_BYTE, std::ptr::null());
            }

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::MIRRORED_REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::MIRRORED_REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        Texture2D {
            id: texture_id,
            width,
            height
        }
    }

    /// unexpected behavior if width and height don't match the bytes
    pub fn update(&self, bytes: &[u8], x: i32, y: i32, width: u32, height: u32, format: TextureFormat) {
        debug_assert!(bytes.len() >= width as usize * height as usize * format.bytes());
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::TexSubImage2D(gl::TEXTURE_2D, 0, x, y, width as i32, height as i32, format.to_gl_format(), gl::UNSIGNED_BYTE, bytes.as_ptr() as *const c_void);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn bind(&self, i: u8) {
        unsafe {
            let t = match i {
                0 => gl::TEXTURE0,
                1 => gl::TEXTURE1,
                2 => gl::TEXTURE2,
                3 => gl::TEXTURE3,
                4 => gl::TEXTURE4,
                5 => gl::TEXTURE5,
                6 => gl::TEXTURE6,
                7 => gl::TEXTURE7,
                8 => gl::TEXTURE8,
                _ => gl::TEXTURE9,
            };
            gl::ActiveTexture(t);
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

/// A reference to a texture, can be copied and dropped freely without affecting the original texture.
#[derive(Debug, Copy, Clone)]
pub struct Texture2DRef {
    pub (crate) id: GLuint,
    width: GLuint,
    height: GLuint,
}

impl Texture2DRef {
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn bind(&self, i: u8) {
        unsafe {
            let t = match i {
                0 => gl::TEXTURE0,
                1 => gl::TEXTURE1,
                2 => gl::TEXTURE2,
                3 => gl::TEXTURE3,
                4 => gl::TEXTURE4,
                5 => gl::TEXTURE5,
                6 => gl::TEXTURE6,
                7 => gl::TEXTURE7,
                8 => gl::TEXTURE8,
                _ => gl::TEXTURE9,
            };
            gl::ActiveTexture(t);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}