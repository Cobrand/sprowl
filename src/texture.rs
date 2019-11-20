use gl;
use gl::types::*;
use std::os::raw::*;

/// Represents an OpenGL 2D Texture.
///
/// Should be created via the Canvas, and rarely used manually.
#[derive(Debug)]
pub struct Texture2D {
    id: GLuint,
    width: GLuint,
    height: GLuint,
}

impl Texture2D {
    fn gen_texture() -> GLuint {
        let mut id = std::mem::MaybeUninit::uninit();
        unsafe {
            gl::GenTextures(1, id.as_mut_ptr());
            id.assume_init()
        }
    }

    /// the bytes SHOULD be RGBA format. For now.
    ///
    /// unexpected behavior if width and height don't match
    pub (crate) fn from_bytes(bytes: &[u8], dims: (u32, u32), ) -> Texture2D {
        debug_assert!(bytes.len() > dims.0 as usize * dims.1 as usize);
        let (width, height) = (dims.0 as GLuint, dims.1 as GLuint);
        let texture_id = Self::gen_texture();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width as i32, height as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, bytes.as_ptr() as *const c_void);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        Texture2D {
            id: texture_id,
            width: width,
            height: height
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub(crate) fn bind(&self) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
        }
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