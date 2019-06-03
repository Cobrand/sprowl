use crate::shader::{ShaderLoadError};
use crate::texture::Texture2D;

use rusttype::{PositionedGlyph, FontCollection, Font, Scale as FontScale};
use image::{self, RgbaImage, Rgba, Pixel, GenericImageView};

use std::path::Path;
use fnv::FnvHashMap as HashMap;
use crate::color::*;
use crate::error::{SprowlError};
use crate::shader::Shader;
use gl;
use gl::types::*;
use std::os::raw::*;
use std::mem::size_of;
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
    quad_vao: GLuint,
    vbo: GLuint,
    current_texture_id: u32,
    textures: HashMap<u32, Texture2D>,
    current_font_id: u32,
    fonts: HashMap<u32, Font<'static>>,
    size: (u32, u32),
}

/// Describes something to be drawn with a given shader.
///
/// The big 3 include: a texture, 
pub enum GraphicEntity<S: AsRef<str>> {
    Texture {
        /// The ID that was returned by add_texture_*
        id: u32,
    },
    Shape {
        shape: Shape,
    },
    Text {
        /// The ID that was returned by add_font_*
        font_id: u32,
        // The font size, in pixels
        font_size: f32,
        // The text that should be printed
        text: S,
        // The color that should be used for this text. Default is white.
        color: Option<Color<u8>>,
    }
}

/// Represents a simple shape: rect, circle, triangle, ect. Used when you want to draw without a texture.
///
#[derive(Debug, Clone, Copy)]
pub enum Shape {
    Rect(u32, u32),
}

/// Represents a given entity (texture, text, shape) with set parameters ready for drawing.
///
/// The parameters depends on the shader you are using.
pub struct GraphicElement<S: AsRef<str>, R> {
    pub graphic_entity: GraphicEntity<S>,
    pub render_params: R,
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
    pub fn new(size: (u32, u32)) -> Result<Canvas, ShaderLoadError> {
        type Vertices24 = [GLfloat; 24];
        unsafe {

            let mut quad_vao = ::std::mem::uninitialized();
            let mut vbo: GLuint = ::std::mem::uninitialized();
            let vertices: Vertices24 = 
                // Position / Texture
                [0.0, 1.0, 0.0, 1.0,
                1.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 1.0,
                1.0, 1.0, 1.0, 1.0,
                1.0, 0.0, 1.0, 0.0];
            gl::GenVertexArrays(1, &mut quad_vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // fill a VBO with the vertices, but allocate up to 4096 times the vertices.
            gl::BufferData(gl::ARRAY_BUFFER, size_of::<Vertices24>() as isize * 4096, &vertices as *const _ as *const c_void, gl::DYNAMIC_DRAW);
            gl::BindVertexArray(quad_vao);

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
                quad_vao: quad_vao,
                vbo,
                current_texture_id: 0,
                textures: Default::default(),
                current_font_id: 0,
                fonts: Default::default(),
                size,
            };
            canvas.inner_init();
            Ok(canvas)
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
        let _v = self.fonts.insert(self.current_font_id, font);
        debug_assert!(_v.is_none());
        let font_id = self.current_font_id;
        self.current_font_id += 1;
        font_id
    }

    // fn apply_view_matrix<R: RenderParams>(&mut self, shader: &mut BaseShader<R>) {
    //     let view_matrix = view_matrix(
    //         0,
    //         0,
    //         self.size.0,
    //         self.size.1,
    //     );
    //     shader.set_matrix4(R::U::view(), &view_matrix);
    // }

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
    pub fn draw<'a, T: AsRef<str> + 'static, S: Shader, I: IntoIterator<Item=&'a GraphicElement<T, S::R>>>(&mut self, shader: &mut S, graphic_elements: I) -> SprowlErrors {
        let mut errors = vec!();
        // we'll use a program here for now, but it may be wiser to use it less often than 1 per draw()
        // call, even though "draw" will draw multiple at once.
        shader.as_base_shader().use_program();

        shader.apply_uniforms(self.size());
        for graphic_el in graphic_elements {
            if let Err(error) = self.draw_graphic_element(shader, graphic_el) {
                errors.push(error);
            };
        }
        errors
    }

    fn draw_texture<S: Shader>(&self, shader: &mut S, texture: &Texture2D, render_params: &S::R) {
        texture.bind();
        self.draw_bound_texture(shader, texture, render_params)
    }

    fn draw_bound_texture<S: Shader>(&self, shader: &mut S, texture: &Texture2D, render_params: &S::R) {
        shader.apply_texture_uniforms(render_params, texture);
        let mut vert_n: GLsizei = 0;
        shader.set_texture_vbo(render_params, texture, |vertices: &[f32], vertices_n| {
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
            gl::BindVertexArray(self.quad_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, vert_n);
            gl::BindVertexArray(0);
        }
    }

    fn draw_shape<S: Shader>(&self, shader: &mut S, shape: &Shape, render_params: &S::R) {
        shader.apply_shape_uniforms(render_params, shape);
        let mut vert_n: GLsizei = 0;
        shader.set_shape_vbo(render_params, shape, |vertices: &[f32], vertices_n| {
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
            gl::BindVertexArray(self.quad_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, vert_n);
            gl::BindVertexArray(0);
        }
    }

    fn draw_text<S: Shader>(&mut self, shader: &mut S, font_id: u32, font_size: f32, text: &str, font_color: Option<Color<u8>>, render_params: &S::R) -> Result<(), SprowlError> {
        let (rgba8_image, width, height) = {
            let font = match self.fonts.get(&font_id) {
                None => return Err(SprowlError::MissingTextureID(font_id)),
                Some(font) => font
            };
            let pixel_height = font_size.ceil() as usize;
            let scale = FontScale::uniform(font_size);

            let font_color: Color<u8> = font_color.unwrap_or_else(Color::white);

            let v_metrics = font.v_metrics(scale);

            let offset = ::rusttype::point(0.0, v_metrics.ascent);
            let glyphs: Vec<PositionedGlyph<'_>> = font.layout(text, scale, offset).collect();
            let width = glyphs.iter().rev().map(|g| {
                g.position().x as f32 + g.unpositioned().h_metrics().advance_width
            }).next().unwrap_or(0.0).ceil();
            let mut rgba8_image = RgbaImage::new(width as u32, pixel_height as u32);
            for glyph in glyphs {
                if let Some(bb) = glyph.pixel_bounding_box() {
                    // TODO: instead of drawing every frame, use gpu_cache from rusttype.
                    glyph.draw(|x, y, v| {
                        let x = x as i32 + bb.min.x;
                        let y = y as i32 + bb.min.y;

                        // some fonts somehow have a value more than 1 sometimes...
                        // so we have to ceil at 1.0
                        let alpha = if v >= 0.5 {
                            255
                        } else {
                            0
                        };
                        if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
                            let x = x as u32;
                            let y = y as u32;
                            rgba8_image.put_pixel(x, y, Rgba::from_channels(font_color.r, font_color.g, font_color.b, alpha));
                        }
                    })
                }
            };
            (rgba8_image, width, pixel_height)
        };
        let texture = Texture2D::from_bytes(&*rgba8_image, (width as u32, height as u32));
        texture.bind();
        self.draw_bound_texture(shader, &texture, render_params);
        Ok(())
    }

    fn draw_graphic_element<T: AsRef<str>, S: Shader>(&mut self, shader: &mut S, graphic_el: &GraphicElement<T, S::R>) -> Result<(), SprowlError> {
        match &graphic_el.graphic_entity {
            GraphicEntity::Texture {id} => {
                let texture = self.textures.get(&id).ok_or(SprowlError::MissingTextureID(*id))?;

                self.draw_texture(shader, &texture, &graphic_el.render_params);
            },
            GraphicEntity::Shape {shape} => {
                self.draw_shape(shader, &shape, &graphic_el.render_params);
            },
            GraphicEntity::Text {font_id, font_size, text, color} => {
                self.draw_text(shader, *font_id, *font_size, text.as_ref(), *color, &graphic_el.render_params)?;
            }
        };
        Ok(())
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.quad_vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}