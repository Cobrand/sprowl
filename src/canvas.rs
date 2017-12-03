use super::primitives::{Shader, UniformName};
use super::texture::Texture2D;
use cgmath::{Matrix4, Vector3, Vector4, Ortho, Deg};

use rusttype::{PositionedGlyph, FontCollection, Font, Scale as FontScale};
use image::{self, GenericImage, RgbaImage, Rgba, Pixel};

use std::path::Path;
use fnv::FnvHashMap as HashMap;
use super::color::*;
use super::error::{SprowlError};
use gl;
use gl::types::*;
use std::os::raw::*;
use std::mem::size_of;

pub use primitives::ShaderLoadError;

#[must_use]
type SprowlErrors = Vec<SprowlError>;

/// Flip "around" the `flip` axis.
#[derive(Debug, Clone, Copy)]
pub enum Flip {
    None,
    Vertical,
    Horizontal,
    Both
}

impl Default for Flip {
    fn default() -> Flip {
        Flip::None
    }
}

/// The representation of the world's camera and its associated assets.
///
/// Currently, a Canvas can hold textures and fonts, and print them dynamically.
///
/// You first have to register textures via the `add_*` methods here,
/// A Canvas doesn't do anything by itself, it MUST be linked to an OpenGL context;
/// see the sdl2-simple example.
pub struct Canvas {
    shader: Shader,
    quad_vao: GLuint,
    current_texture_id: u32,
    textures: HashMap<u32, Texture2D>,
    current_font_id: u32,
    fonts: HashMap<u32, Font<'static>>,
    // canvas_bounds: x CENTER, y CENTER, w, h
    camera_bounds: (i32, i32, u32, u32),
    zoom_level: f32,
}

/// format: (x center, y center, width, height)
// `zoom_level` should be higher than 0
fn compute_projection_matrix(x: i32, y: i32, w: u32, h: u32, zoom_level: f32) -> Matrix4<f32> {
    debug_assert!(zoom_level > 0.0);
    let x = x as f32;
    let y = y as f32;
    let camera_half_w = w as f32 / 2.0;
    let camera_half_h = h as f32 / 2.0;
    Matrix4::<f32>::from(Ortho {
        left: (x - camera_half_w / zoom_level),
        right: (x + camera_half_w / zoom_level),
        bottom: (y + camera_half_h / zoom_level),
        top: (y - camera_half_h / zoom_level),
        near: -1.0,
        far: 1.0
    })
}

#[derive(Debug, Clone, Copy)]
pub enum CameraRelativePosition<T: ::std::fmt::Debug + Clone + Copy> {
    FromTopLeft(T, T),
    FromTopRight(T, T),
    FromBottomLeft(T, T),
    FromBottomRight(T, T),
}

/// Represents how should an element be displayed in our world:
///
/// `CameraRelative` will keep the same aspect and position ratio no matter
/// what the zoom level and the camera position is; while `WorldAbsolute`
/// positions will only be displayed if they are within the camera's field of view.
///
/// Simply put, `CameraRelative` should be used for UIs, while `WorldAbsolute`
/// should be used for "In Game" stuff.
#[derive(Debug, Clone, Copy)]
pub enum Graphic2DRepresentation<T: ::std::fmt::Debug + Clone + Copy> {
    CameraRelative {
        position: CameraRelativePosition<T>,
    },
    WorldAbsolute {
        x: T,
        y: T,
    }
}

pub enum GraphicEntity<'a> {
    Texture {
        /// The ID that was returned by add_texture_*
        id: u32,
        repr: Graphic2DRepresentation<i32>,
        render_options: RenderOptions,
        scale: Option<f32>
    },
    Text {
        /// The ID that was returned by add_font_*
        font_id: u32,
        // The font size, in pixels
        font_size: f32,
        // The text that should be printed
        text: &'a str,
        // The color that should be used for this text. Default is white.
        color: Option<Color<u8>>,
        repr: Graphic2DRepresentation<i32>,
        render_options: RenderOptions
    }
}

#[derive(Debug, Default)]
pub struct RenderOptions {
    /// Filter color is useful to make a texture more transparent
    /// and map the white parts of the texture to another color.
    ///
    /// If your texture is red and you apply a filter "Green",
    /// your texture will become black. If your base texture is white,
    /// your texture will become green
    pub filter_color: Option<Color<u8>>,
    /// Blend color will "blend" the origin texture with that color,
    /// and the "mix" value is actually the alpha value of this color.
    /// 
    /// For textures, alpha values between 0.1 and 0.6 are recommended.
    ///
    /// This can be used to simulate poison on a caracter (green-ish tint),
    /// or a "enemy has been hit by your attack" kind of effect with red stuff.
    pub blend_color: Option<Color<u8>>,
    /// Draws an outline of `0` pixels with the color `1`. The outline may
    /// have an alpha value.
    /// 
    /// Please note that the outline is applied *after* any scaling, but this is not
    /// absolute: if the zoom level is lower, the outline will be less pronounced,
    /// while if the zoom level is higher than 1, the outline may look more "solid"
    /// and not antiliased.
    pub outline: Option<(f32, Color<u8>)>,
    /// Flip the texture around an axis
    ///
    /// There is no correction to do, the texture will keep its origin and its size,
    /// but the output texture will be flipped in whatever way you want.
    pub flip: Flip,
}

impl Canvas {
    fn inner_init(&mut self) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }

    /// Creates a new Canvas with the given dimensions.
    ///
    /// The camera should have the format `(center_x, center_y, width, height)`
    ///
    /// # Failures
    ///
    /// May return an error if the shader has not been compiled correctly. This may
    /// happen when your version OpenGL is too old to support fragment and vertex shaders.
    pub fn new(camera_bounds: (i32, i32, u32, u32)) -> Result<Canvas, ShaderLoadError> {
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
            gl::BufferData(gl::ARRAY_BUFFER, size_of::<Vertices24>() as isize, &vertices as *const _ as *const c_void, gl::STATIC_DRAW);
            gl::BindVertexArray(quad_vao);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, 4 * size_of::<GLfloat>() as i32, 0 as *const _);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            
            let mut canvas = Canvas {
                shader: Shader::vanilla()?,
                quad_vao: quad_vao,
                current_texture_id: 0,
                textures: Default::default(),
                current_font_id: 0,
                fonts: Default::default(),
                camera_bounds: camera_bounds,
                zoom_level: 1.0,
            };
            canvas.apply_projection();
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
        let (img_w, img_h) = opened_image.dimensions();

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
        let collection = FontCollection::from_bytes(bytes);
        let font = collection.into_font().unwrap(); // only succeeds if collection consists of one font
        let _v = self.fonts.insert(self.current_font_id, font);
        debug_assert!(_v.is_none());
        let font_id = self.current_font_id;
        self.current_font_id += 1;
        font_id
    }

    fn apply_projection(&mut self) {
        let projection_matrix = compute_projection_matrix(
            self.camera_bounds.0,
            self.camera_bounds.1,
            self.camera_bounds.2,
            self.camera_bounds.3,
            self.zoom_level
        );
        println!("new bounds: {:?} zoom {}", self.camera_bounds, self.zoom_level);
        self.shader.set_matrix4(UniformName::Projection, &projection_matrix, true);
    }

    /// This SHOULD be called when the screen resizes, but NOT when zooming/de-zoming;
    /// use set_zoom_level for that.
    pub fn set_camera_size(&mut self, new_size: (u32, u32)) {
        let (x, y, _, _) = self.camera_bounds;
        self.set_camera((x, y, new_size.0, new_size.1));
    }

    /// Sets the camera's center position.
    pub fn set_camera_position(&mut self, new_position: (i32, i32)) {
        let (_, _, w, h) = self.camera_bounds;
        self.set_camera((new_position.0, new_position.1, w, h));
    }

    /// Sets the camera bounds as the format (center_x, center_y, width, height)
    pub fn set_camera(&mut self, new_bounds: (i32, i32, u32, u32)) {
        self.camera_bounds = new_bounds;
        self.apply_projection();
    }

    /// Sets the zoom level of the canvas
    ///
    /// # Panics
    ///
    /// Panics if zoom_level is equal or less than 0
    pub fn set_zoom_level(&mut self, zoom_level: f32) {
        if self.zoom_level <= 0.0 {
            panic!("zoom_level cannot be less than 0, received {}", self.zoom_level);
        }
        self.zoom_level = zoom_level;
        self.apply_projection();
    }

    /// Returns the current zoom level. Note that a zoom level of 2
    /// will zoom on the center of the camera, and make the textures
    /// and elements of the world appear larger. A level below 1
    pub fn zoom_level(&mut self) -> f32 {
        self.zoom_level
    }

    /// Returns the bounds of the camera as (center_x, center_y, width, height)
    pub fn camera_bounds(&self) -> (i32, i32, u32, u32) {
        self.camera_bounds
    }

    /// Default clear color is black, just like your soul.
    pub fn clear(&mut self, clear_color: Option<Color<u8>>) {
        let clear_color: Color<f32> = clear_color.unwrap_or(Color::<u8>::from_rgb(0, 0, 0)).to_color_f32();
        unsafe {
            gl::ClearColor(clear_color.r, clear_color.g, clear_color.b, 1.0f32);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    /// Draw entities in the given order.
    ///
    /// The first entities of this iterator will be on the "bottom" of the layer while others will be on
    /// "top" of the layer.
    ///
    /// Note that while you can pass a slice to this method, it can also accept iterators. This may allow
    /// you to actually print all of your world without making extra allocations.
    ///
    /// # Notes
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let entities: Vec<GraphicEntity> = vec!();
    /// canvas.draw(&entities);
    /// ```
    pub fn draw<'a, 'b: 'a, I: IntoIterator<Item=&'a GraphicEntity<'b>>>(&mut self, graphic_entities: I) -> SprowlErrors {
        let mut errors = vec!();
        self.shader.use_program();
        for graphic_entity in graphic_entities {
            if let Err(error) = self.draw_graphic_entity(graphic_entity) {
                errors.push(error);
            };
        }
        errors
    }

    fn compute_model_matrix(origin_x: f32, origin_y: f32, width: f32, height: f32, flip: Flip) -> Matrix4<f32> {
        let m = match flip {
            Flip::None => Matrix4::from_translation(Vector3::<f32>::new(origin_x, origin_y, 0.0)),
            Flip::Vertical =>
                Matrix4::from_translation(Vector3::<f32>::new(origin_x + width, origin_y, 0.0)) *
                Matrix4::from_angle_y(Deg(180.0)),
            Flip::Horizontal =>
                Matrix4::from_translation(Vector3::<f32>::new(origin_x, origin_y + height, 0.0)) *
                Matrix4::from_angle_x(Deg(180.0)),
            Flip::Both =>
                Matrix4::from_translation(Vector3::<f32>::new(origin_x + width, origin_y + height, 0.0)) *
                Matrix4::from_angle_x(Deg(180.0)) * Matrix4::from_angle_y(Deg(180.0)),
        };
        m * Matrix4::from_nonuniform_scale(width, height, 1.0)
    }

    fn compute_model_matrix_from_2d_repr(&self, pos: &Graphic2DRepresentation<i32>, element_dims: (u32, u32), scale: Option<f32>, flip: Flip) -> Matrix4<f32> {
        use CameraRelativePosition::*;
        use Graphic2DRepresentation::*;
        let elt_w = element_dims.0 as f32 * scale.unwrap_or(1.0);
        let elt_h = element_dims.1 as f32 * scale.unwrap_or(1.0);
        let cam_center_x = self.camera_bounds.0 as f32;
        let cam_center_y = self.camera_bounds.1 as f32;
        let cam_w = self.camera_bounds.2 as f32 / self.zoom_level;
        let cam_h = self.camera_bounds.3 as f32 / self.zoom_level;
        match *pos {
            WorldAbsolute {x, y} =>
                Self::compute_model_matrix(x as f32, y as f32, elt_w, elt_h, flip),
            CameraRelative {ref position} => {
                let (x, y) = match *position {
                    FromTopLeft(x, y) =>
                        ((cam_center_x - cam_w / 2.0) + (x as f32) / self.zoom_level,
                        (cam_center_y - cam_h / 2.0) + (y as f32) / self.zoom_level),
                    FromBottomLeft(x, y) =>
                        ((cam_center_x - cam_w / 2.0) + x as f32 / self.zoom_level,
                        (cam_center_y + cam_h / 2.0) - (y as f32 + elt_h) / self.zoom_level),
                    FromBottomRight(x, y) =>
                        ((cam_center_x + cam_w / 2.0) - (x as f32 + elt_w) / self.zoom_level,
                        (cam_center_y + cam_h / 2.0) - (y as f32 + elt_h) / self.zoom_level),
                    FromTopRight(x, y) =>
                        ((cam_center_x + cam_w / 2.0) - (x as f32 + elt_w) / self.zoom_level,
                        (cam_center_y - cam_h / 2.0) + y as f32 / self.zoom_level),
                };
                Self::compute_model_matrix(x, y, elt_w / self.zoom_level, elt_h / self.zoom_level, flip)
            }
        }
    }

    fn draw_bound_texture(&mut self, texture_dims: (u32, u32), model: &Matrix4<f32>, render_options: &RenderOptions) {
        self.shader.set_matrix4(UniformName::Model, model, false);
        if let Some((outline_thickn, color)) = render_options.outline {
            // relative to the texture in the OpenGL sense where 1.0 is max and 0.0 is min,
            // how big is the outline in those coordinates?
            let outline_x = outline_thickn as f32 / texture_dims.0 as f32;
            let outline_y = outline_thickn as f32 / texture_dims.1 as f32;
            self.shader.set_float(UniformName::OutlineWidthX, outline_x, false);
            self.shader.set_float(UniformName::OutlineWidthY, outline_y, false);
            let (r, g, b) = color.rgb();
            self.shader.set_vector3(UniformName::OutlineColor, Vector3::new(f32::from(r) / 255.0, f32::from(g) / 255.0, f32::from(b) / 255.0), false);
        }
        if let Some(color_u8) = render_options.filter_color {
            let (r, g, b, a) = color_u8.to_color_f32().rgba();
            self.shader.set_vector4(UniformName::ModelColorFilter, Vector4::new(r, g, b, a), false);
        }
        if let Some(color_u8) = render_options.blend_color {
            let (r, g, b, mix_value) = color_u8.to_color_f32().rgba();
            self.shader.set_vector4(UniformName::ModelColorBlend, Vector4::new(r, g, b, mix_value), false);
        }
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
        }

        unsafe {
            // TODO optimize and only make this call once across all draws?
            gl::BindVertexArray(self.quad_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
        }
        if render_options.outline.is_some() {
            // if outline was enabled, disable it
            self.shader.set_float(UniformName::OutlineWidthX, -1.0, false);
            self.shader.set_float(UniformName::OutlineWidthY, -1.0, false);
        }
        if render_options.filter_color.is_some() {
            self.shader.set_vector4(UniformName::ModelColorFilter, Vector4::new(1.0, 1.0, 1.0, 1.0), false);
        }
        if render_options.blend_color.is_some() {
            self.shader.set_vector4(UniformName::ModelColorBlend, Vector4::new(1.0, 1.0, 1.0, 0.0), false);
        }
    }

    fn draw_text(&mut self, font_id: u32, font_size: f32, text: &str, font_color: Option<Color<u8>>, repr: &Graphic2DRepresentation<i32>, options: &RenderOptions) -> Result<(), SprowlError> {
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
            let glyphs: Vec<PositionedGlyph> = font.layout(text, scale, offset).collect();
            let width = glyphs.iter().rev().map(|g| {
                g.position().x as f32 + g.unpositioned().h_metrics().advance_width
            }).next().unwrap_or(0.0).ceil();
            let mut rgba8_image = RgbaImage::new(width as u32, pixel_height as u32);
            for glyph in glyphs {
                if let Some(bb) = glyph.pixel_bounding_box() {
                    glyph.draw(|x, y, v| {
                        let x = x as i32 + bb.min.x;
                        let y = y as i32 + bb.min.y;

                        // some fonts somehow have a value more than 1 sometimes...
                        // so we have to ceil at 1.0
                        let alpha = if v > 1.0 {
                            255
                        } else if v <= 0.0 {
                            0
                        } else {
                            (v * 255.0).ceil() as u8
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
        let model = self.compute_model_matrix_from_2d_repr(repr, (width as u32, height as u32), None, options.flip);
        texture.bind();
        self.draw_bound_texture((width as u32, height as u32), &model, options);
        Ok(())
    }

    fn draw_graphic_entity<'a>(&mut self, graphic_entity: &GraphicEntity<'a>) -> Result<(), SprowlError> {
        match *graphic_entity {
            GraphicEntity::Texture {id, ref repr, ref render_options, scale} => {
                let (model, dims) = {
                    let texture = match self.textures.get(&id) {
                        None => return Err(SprowlError::MissingTextureID(id)),
                        Some(texture) => texture,
                    };
                    texture.bind();
                    let texture_dims = texture.size();
                    (self.compute_model_matrix_from_2d_repr(repr, texture_dims, scale, render_options.flip), texture_dims)
                };
                self.draw_bound_texture(dims, &model, render_options);
            },
            GraphicEntity::Text {font_id, font_size, text, color, ref repr, ref render_options} => {
                self.draw_text(font_id, font_size, text, color, repr, render_options)?;
            }
        };
        Ok(())
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.quad_vao);
        }
    }
}