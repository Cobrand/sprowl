use crate::texture::Texture2D;

use rusttype::{FontCollection, Scale as FontScale};
use image::{self, GenericImageView};

use crate::texture::TextureFormat;

use crate::font_renderer::FontRenderer;

use std::path::Path;
use hashbrown::HashMap;
use crate::color::*;
use crate::error::{SprowlError};
use crate::shader::Shader;
use crate::render::{RenderParams, Shape};
use crate::gelem::*;
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
            let vertices: Vertices24 = 
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

            // fill a VBO with the vertices, but allocate up to 4096 times the vertices.
            gl::BufferData(gl::ARRAY_BUFFER, size_of::<Vertices24>() as isize * 4096, &vertices as *const _ as *const c_void, gl::DYNAMIC_DRAW);
            gl::BindVertexArray(vao);

            // enable attribute 0 & 1
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
        
            // attribute 0 is a vec2 of floats (for position)
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 4 * size_of::<GLfloat>() as i32, 0 as *const _);
            // attribute 1 is a vec2 of float (for texture color)
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 4 * size_of::<GLfloat>() as i32, ptr::null::<GLfloat>().offset(2) as *const _);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            
            let mut canvas = Canvas {
                vao: vao,
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

    /// Load a texture from bytes: you must specify the correct width and height of the texture.
    ///
    /// # Panics
    ///
    /// * (debug only) if the size is incorrect (higher than the slice's)
    /// * (debug only) if the amount of textures  recorded is higher than u32::MAX_VALUE
    pub fn add_texture_from_raw_bytes(&mut self, bytes: &[u8], size: (u32, u32)) -> u32 {
        let texture = Texture2D::from_bytes(bytes, size);
        let _v = self.textures.insert(self.current_texture_id, texture);
        debug_assert_eq!(_v, None);
        let texture_id = self.current_texture_id;
        self.current_texture_id += 1;
        texture_id
    }

    /// Load a texture from some bytes. Preferably, the image should be PNG with an alpha layer.
    /// Returns a number representing the ID of the texture, which you can use later on in `draw_entities(..)`
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
    /// Returns a number representing the ID of the texture, which you can use later on in `draw_entities(..)`
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
    /// Returns a number representing the ID of the font, which you can use later on in `draw_entities(..)`
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
    pub fn set_size(&mut self, new_size: (u32, u32)) {
        self.size = new_size;
        self.set_viewport();
    }

    fn set_viewport(&self) {
        unsafe {
            gl::Viewport(0, 0, self.size.0 as i32, self.size.1 as i32);
        }
    }

    /// Default clear color is black, just like your soul.
    pub fn clear(&mut self, clear_color: Option<Color<u8>>) {
        let clear_color: Color<f32> = clear_color.unwrap_or(Color::<u8>::from_rgb(0, 0, 0)).to_color_f32();
        unsafe {
            gl::ClearColor(clear_color.r, clear_color.g, clear_color.b, 1.0f32);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    /// Given a shader and a `IntoIterator`, draw all the entities in the given order.
    /// 
    /// You may pass a simple iterator or anything that might transform into one, e.g. a &Vec
    ///
    /// You will need to sort all your entities from far to front. We CANNOT use the depth
    /// buffer and test method with the z-axis of opengl, because the depth buffer works extremely
    /// poorly with transparency. At the cost of sorting yourself all the textures you want,
    /// you are able to have transparent textures.
    pub fn draw<'a, T: AsRef<str> + 'a, S: Shader, I: IntoIterator<Item=&'a GraphicElement<T, S::R>>>(&mut self, shader: &mut S, graphic_elements: I) -> SprowlErrors {
        let mut errors = vec!();
        
        shader.as_base_shader().use_program();
        shader.apply_global_uniforms(self.size());

        for graphic_el in graphic_elements {
            if let Err(error) = self.draw_graphic_element(shader, graphic_el) {
                errors.push(error);
            };
        }
        errors
    }

    fn draw_texture<S: Shader>(&self, shader: &mut S, texture: &Texture2D, render_params: &RenderParams<S::R>) {
        texture.bind();
        self.draw_bound_texture(shader, texture, render_params)
    }

    #[inline]
    fn draw_bound_texture<S: Shader>(&self, shader: &mut S, texture: &Texture2D, render_params: &RenderParams<S::R>) {
        Self::draw_bound_texture_raw(shader, texture, render_params, self.vao, self.vbo)
    }

    fn draw_bound_texture_raw<S: Shader>(shader: &mut S, texture: &Texture2D, render_params: &RenderParams<S::R>, vao: GLuint, vbo: GLuint) {
        shader.apply_draw_uniforms(render_params, texture.into());
        let mut vert_n: GLsizei = 0;
        shader.set_draw_vbo(render_params, texture.into(), |vertices: &[f32], vertices_n| {
            let vertices_size = (size_of::<f32>() * vertices.len()) as isize;

            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                // replace data of the vbo by the given data
                gl::BufferSubData(gl::ARRAY_BUFFER, 0, vertices_size, vertices as *const _ as *const c_void);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            }

            vert_n = vertices_n as GLsizei;
        });

        unsafe {
            // TODO optimize and only make this call once across all draws?
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, vert_n);
            gl::BindVertexArray(0);
        }
    }

    // fn draw_cache_chunk<S: Shader>(&self, shader: &mut S, texture: &Texture2D, (x, y, w, h): (i32, i32, u32, u32), render_params: &S::R) {
    //     shader.apply_draw_uniforms(render_params, texture);
    //     let (tex_w, tex_h) = texture.size();
    //     let mut vert_n: GLsizei = 0;
    //     let (x, y, w, h) = (
    //         x as f32 / tex_w as f32,
    //         y as f32 / tex_h as f32, 
    //         w as f32 / tex_w as f32,
    //         h as f32 / tex_h as f32
    //     );
    //     shader.set_cache_extract_vbo(render_params, (x, y, w, h), |vertices: &[f32], vertices_n| {
    //         let vertices_size = (size_of::<f32>() * vertices.len()) as isize;

    //         unsafe {
    //             gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
    //             // replace data of the vbo by the given data
    //             gl::BufferSubData(gl::ARRAY_BUFFER, 0, vertices_size, vertices as *const _ as *const c_void);
    //             gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    //         }

    //         vert_n = vertices_n as GLsizei;
    //     });
        
    //     unsafe {
    //         // TODO optimize and only make this call once across all draws?
    //         gl::BindVertexArray(self.vao);
    //         gl::DrawArrays(gl::TRIANGLES, 0, vert_n);
    //         gl::BindVertexArray(0);
    //     }
    // }

    fn draw_shape<S: Shader>(&self, shader: &mut S, shape: &Shape, render_params: &RenderParams<S::R>) {
        shader.apply_draw_uniforms(render_params, shape.into());
        let mut vert_n: GLsizei = 0;
        shader.set_draw_vbo(render_params, shape.into(), |vertices: &[f32], vertices_n| {
            let vertices_size = (size_of::<f32>() * vertices.len()) as isize;

            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                // replace data of the vbo by the given data
                gl::BufferSubData(gl::ARRAY_BUFFER, 0, vertices_size, vertices as *const _ as *const c_void);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            }

            vert_n = vertices_n as GLsizei;
        });

        unsafe {
            // TODO optimize and only make this call once across all draws?
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, vert_n);
            gl::BindVertexArray(0);
        }
    }

    fn draw_text<S: Shader>(&mut self, shader: &mut S, font_id: u32, font_size: f32, text: &str, _max_width: Option<u32>, render_params: &RenderParams<S::R>) -> Result<(), SprowlError> {
        let font_renderer = self.fonts.get_mut(&font_id).ok_or(SprowlError::MissingTextureId(font_id))?;

        let scale = FontScale::uniform(font_size);

        let ascent = font_renderer.font.v_metrics(scale).ascent;

        // let glyphs = crate::font_renderer::layout_paragraph(&font_renderer.font, scale, max_width, text);
        let glyphs = font_renderer.font.layout(text, scale, rusttype::point(0.0f32, 0.0)).collect::<Vec<_>>();

        let tex = &mut font_renderer.tex;
        let r = font_renderer.font_cache.cache_glyphs(glyphs.iter(), |rect, data| {
            let rusttype::Point { x, y } = rect.min;
            let width = rect.width();
            let height = rect.height();
            tex.update(data, x as i32, y as i32, width, height, TextureFormat::Greyscale);
        });
        r.expect("failed to write to font gpu cache");
        
        let mut r = render_params.clone();
        r.common.is_source_grayscale = true;
        
        let (tex_w, tex_h) = tex.size();
        let (tex_w, tex_h) = (tex_w as f32, tex_h as f32);
        font_renderer.tex.bind();
        for glyph in &glyphs {
            if let Ok(Some((uv_rect, screen_rect))) = font_renderer.font_cache.rect_for(glyph) {
                let mut r = render_params.clone();
                r.common.is_source_grayscale = true;
                r.common.crop = Some((
                    (uv_rect.min.x * tex_w) as i32,
                    (uv_rect.min.y * tex_h) as i32,
                    (uv_rect.width() * tex_w) as u32,
                    (uv_rect.height() * tex_h) as u32
                ));
                r.common.draw_pos.offset(screen_rect.min.x, screen_rect.min.y + ascent as i32);
                Self::draw_bound_texture_raw(shader, &font_renderer.tex, &r, self.vao, self.vbo);
            }
        }
        Ok(())
    }

    fn draw_graphic_element<T: AsRef<str>, S: Shader>(&mut self, shader: &mut S, graphic_el: &GraphicElement<T, S::R>) -> Result<(), SprowlError> {
        match &graphic_el.render_stem {
            RenderStem::Texture {id} => {
                let texture = self.textures.get(&id).ok_or(SprowlError::MissingTextureId(*id))?;

                self.draw_texture(shader, &texture, &graphic_el.render_params);
            },
            RenderStem::Shape {shape} => {
                self.draw_shape(shader, &shape, &graphic_el.render_params);
            },
            RenderStem::Text {font_id, font_size, text, max_width} => {
                self.draw_text(
                    shader,
                    *font_id,
                    *font_size,
                    text.as_ref(),
                    *max_width,
                    &graphic_el.render_params
                )?;
            }
        };
        Ok(())
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