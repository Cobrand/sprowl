use crate::texture::Texture2D;
use smallvec::SmallVec;

use rusttype::FontCollection;
use image::{self, GenericImageView};

use crate::font::FontRenderer;

use crate::render::{RenderSource, GraphicElement};
use std::path::Path;
use hashbrown::HashMap;
use crate::shader::ShaderDrawCall;
use crate::color::*;
use crate::error::{SprowlError};
use crate::shader::Shader;
use gl;
use gl::types::*;
use std::os::raw::*;
use std::mem::{size_of, MaybeUninit};
use std::ptr;

#[must_use]
type SprowlErrors = Vec<SprowlError>;

/// The representation of the screen and its associated assets.
///
/// Currently, a Canvas can hold textures and fonts, and print them dynamically.
///
/// You first have to register textures via the `add_*` methods here,
/// A Canvas doesn't do anything by itself, it MUST be linked to an OpenGL context;
/// see the sdl2-simple example.
pub struct Canvas {
    vao: GLuint,
    vbo: GLuint,
    current_texture_id: u32,
    textures: HashMap<u32, Texture2D>,
    current_font_id: u32,
    fonts: HashMap<u32, FontRenderer>,
    size: (u32, u32),
}

impl Canvas {
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    fn inner_init(&mut self) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
        self.set_viewport();
    }

    /// Creates a new Canvas with the given dimensions.
    ///
    /// The camera should have the format `(center_x, center_y, width, height)`
    ///
    /// # Failures
    ///
    /// May return an error if the shader has not been compiled correctly. This may
    /// happen when your version OpenGL is too old to support fragment and vertex shaders.
    pub fn new(size: (u32, u32)) -> Canvas {
        type Vertices24 = [GLfloat; 24];
        unsafe {

            let mut vao: MaybeUninit<u32> = MaybeUninit::uninit();
            let mut vbo: MaybeUninit<GLuint> = MaybeUninit::uninit();

            // these vertices will be put in the VBO
            const VERTICES: Vertices24 = 
                // Position / Texture
                [0.0, 1.0, 0.0, 1.0,
                1.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 1.0,
                1.0, 1.0, 1.0, 1.0,
                1.0, 0.0, 1.0, 0.0];
            gl::GenVertexArrays(1, vao.as_mut_ptr());
            gl::GenBuffers(1, vbo.as_mut_ptr());

            let vao = vao.assume_init();
            let vbo = vbo.assume_init();

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // fill a VBO with the vertices, but allocate up to 1024 times the vertices.
            gl::BufferData(gl::ARRAY_BUFFER, size_of::<Vertices24>() as isize * 1024, &VERTICES as *const _ as *const c_void, gl::DYNAMIC_DRAW);
            gl::BindVertexArray(vao);

            // enable attribute 0 & 1
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
        
            // attribute 0 is a vec2 of floats (for position)
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 4 * size_of::<GLfloat>() as i32, ptr::null());
            // attribute 1 is a vec2 of float (for texture color)
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 4 * size_of::<GLfloat>() as i32, ptr::null::<GLfloat>().offset(2) as *const _);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::Enable(gl::TEXTURE_2D);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::Enable(gl::TEXTURE_2D);
            
            let mut canvas = Canvas {
                vao,
                vbo,
                current_texture_id: 0,
                textures: Default::default(),
                current_font_id: 0,
                fonts: Default::default(),
                size,
            };
            canvas.inner_init();
            canvas
        }
    }

    #[inline]
    pub fn get_font(&self, id: u32) -> Option<&FontRenderer> {
        self.fonts.get(&id)
    }

    #[inline]
    pub fn get_font_mut(&mut self, id: u32) -> Option<&mut FontRenderer> {
        self.fonts.get_mut(&id)
    }

    #[inline]
    pub fn get_texture(&self, id: u32) -> Option<&Texture2D> {
        self.textures.get(&id)
    }

    #[inline]
    pub fn get_texture_mut(&mut self, id: u32) -> Option<&mut Texture2D> {
        self.textures.get_mut(&id)
    }

    /// Load a texture from bytes: you must specify the correct width and height of the texture.
    ///
    /// # Panics
    ///
    /// * (debug only) if the size is incorrect (higher than the slice's)
    /// * (debug only) if the amount of textures  recorded is higher than u32::MAX_VALUE
    pub fn add_texture_from_raw_bytes(&mut self, bytes: &[u8], size: (u32, u32)) -> u32 {
        let texture = Texture2D::new(Some(bytes), size);
        let _v = self.textures.insert(self.current_texture_id, texture);
        debug_assert_eq!(_v, None);
        let texture_id = self.current_texture_id;
        self.current_texture_id += 1;
        texture_id
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

    /// Load a texture from a file given a path.
    /// Returns a number representing the ID of the texture, which you can use later on in `draw(..)`
    ///
    /// # Panics
    /// 
    /// Panics if the image is not RGBA (the texture has no alpha layer)
    pub fn add_texture_from_image_path<P: AsRef<Path>>(&mut self, path: P) -> image::ImageResult<u32> {
        let img = image::open(&path)?;
        let (img_w, img_h) = img.dimensions();

        let color_data: Vec<u8> = if let image::DynamicImage::ImageRgba8(rgba_data) = img {
            rgba_data.into_raw()
        } else {
            panic!("image {} is not RGBA", path.as_ref().display());
        };
        Ok(self.add_texture_from_raw_bytes(color_data.as_slice(), (img_w, img_h)))
    }

    /// Load a font from *static* bytes. There is currently no way to dynamically load a font, for convenience only.
    ///
    /// Returns a number representing the ID of the font, which you can use later on in `draw(..)`
    ///
    /// # Panics
    ///
    /// Panics if there's more than one font
    pub fn add_font_from_bytes(&mut self, bytes: &'static [u8]) -> u32 {
        let collection = FontCollection::from_bytes(bytes).expect("wrong font added from static bytes");
        let font = collection.into_font().expect("fatal: collection consists of more than one font"); // only succeeds if collection consists of one font
        let _v = self.fonts.insert(self.current_font_id, FontRenderer::new(font));
        debug_assert!(_v.is_none());
        let font_id = self.current_font_id;
        self.current_font_id += 1;
        font_id
    }

    /// Sets the Canvas' size in pixels.
    ///
    /// If you resize your window without calling this function, your `Canvas` and `Shader`
    /// will not be able to know that the window was resized, and thus you might render to only a
    /// part of the window.
    pub fn set_size(&mut self, new_size: (u32, u32)) {
        self.size = new_size;
        self.set_viewport();
    }

    #[inline]
    fn set_viewport(&self) {
        unsafe {
            gl::Viewport(0, 0, self.size.0 as i32, self.size.1 as i32);
        }
    }

    /// Clear the screen with a solid color.
    /// 
    /// Default clear color is black, just like your soul.
    pub fn clear(&mut self, clear_color: Option<Color<u8>>) {
        let clear_color: Color<f32> = clear_color.unwrap_or_else(|| Color::<u8>::from_rgb(0, 0, 0)).to_color_f32();
        unsafe {
            gl::ClearColor(clear_color.r, clear_color.g, clear_color.b, 1.0f32);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    /// Given a shader and a `IntoIterator`, draw all the entities in the given order.
    /// 
    /// You may pass a simple iterator or anything that might transform into one, e.g. a `&Vec`
    ///
    /// You will need to sort all your entities from far to front. We CANNOT use the depth
    /// buffer and test method with the z-axis of opengl, because the depth buffer works extremely
    /// poorly with transparency. At the cost of sorting yourself all the textures you want,
    /// you are able to have transparent textures.
    pub fn draw<'a, T: AsRef<str> + 'a, S: Shader, I>(&mut self, shader: &mut S, graphic_elements: I) -> SprowlErrors
    where I: IntoIterator<Item=&'a GraphicElement<T, <S::D as ShaderDrawCall>::RenderParams>> {
        let mut errors = vec!();
        
        shader.as_base_shader().use_program();
        shader.apply_global_uniforms(self.size());

        for graphic_el in graphic_elements {
            match self.gelem_as_draw_call::<T, S::D>(graphic_el) {
                Err(error) => {
                    errors.push(error);
                },
                Ok(draw_calls) => {
                    for draw_call in draw_calls {
                        self.draw_elem(shader, draw_call);
                    }
                }
            }
        }

        if ! errors.is_empty() {
            log::warn!("drawing routine had {} errors", errors.len());
        }
        errors
    }

    fn gelem_as_draw_call<T: AsRef<str>, D: ShaderDrawCall>(
        &mut self,
        graphic_el: &GraphicElement<T, <D as ShaderDrawCall>::RenderParams>
    ) -> Result<SmallVec<[D; 2]>, SprowlError> {
        D::from_graphic_elem(graphic_el, self)
    }

    fn draw_elem<S: Shader>(&mut self, shader: &mut S, draw_call: S::D) {
        if let RenderSource::Texture(texture) = draw_call.render_source() {
            texture.bind(0);
        }
        let pad = draw_call.common_params().pad;
        let crop = draw_call.common_params().crop.map(|(x, y, w, h)| {
            // if pad is not None, increase the crop by `pad` pixels.
            if let Some(pad) = pad {
                (x - pad, y - pad, w + pad as u32 * 2, h + pad as u32 * 2)
            } else {
                (x, y, w, h)
            }
        });
        let vertices = draw_call.render_source().compute_draw_vbo(crop);
        shader.apply_draw_uniforms(draw_call);

        let vert_n: GLsizei = 6;

        let vertices_size = (size_of::<f32>() * vertices.len()) as isize;

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        }
        unsafe{
            // replace data of the vbo by the given data
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, vertices_size, &vertices as *const _ as *const c_void);
        }

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        unsafe {
            // TODO optimize and only make this call once across all draws?
            gl::BindVertexArray(self.vao);
        }
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, vert_n);
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}