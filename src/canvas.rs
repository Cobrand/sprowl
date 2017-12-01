use super::primitives::{Shader, UniformName};
use super::texture::Texture2D;
use cgmath::prelude::*;
use cgmath::{Matrix4, Vector2, Vector3, Vector4, Ortho};

use rusttype::{PositionedGlyph, FontCollection, Font, Scale as FontScale};
use image::{self, GenericImage, ImageBuffer, RgbaImage, Rgba, Pixel};

use std::path::Path;
use fnv::FnvHashMap as HashMap;
use super::texture::*;
use super::color::*;
use gl;
use gl::types::*;
use std::os::raw::*;
use std::mem::size_of;
use std::cmp::{max, min};

/// A Canvas doesn't do anything by itself, it MUST be linked to an OpenGL context
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

/// x center, y center, zoom_level higher than 0 plz
fn compute_projection_matrix(x: i32, y: i32, w: u32, h: u32, zoom_level: f32) -> Matrix4<f32> {
    debug_assert!(zoom_level > 0.0);
    let x = x as f32;
    let y = y as f32;
    let camera_half_w = (w as f32) / 2.0;
    let camera_half_h = (h as f32) / 2.0;
    Matrix4::<f32>::from(Ortho {
        left: (x - camera_half_w / zoom_level),
        right: (x + camera_half_w / zoom_level),
        bottom: (y + camera_half_h / zoom_level),
        top: (y - camera_half_h / zoom_level),
        near: -1.0,
        far: 1.0
    })
}

#[derive(Debug)]
pub enum CameraRelativePosition<T: ::std::fmt::Debug + Clone + Copy> {
    FromTopLeft(T, T),
    FromTopRight(T, T),
    FromBottomLeft(T, T),
    FromBottomRight(T, T),
}

#[derive(Debug)]
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
        font_id: u32,
        font_size: f32,
        text: &'a str,
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
    /// Can be used to simulate poison on a caracter (green-ish tint),
    /// or a "enemy has been hit by your attack" kind of effect with white stuff.
    pub blend_color: Option<Color<u8>>,
    pub outline: Option<(f32, Color<u8>)>
}

impl Canvas {
    fn inner_init(&mut self) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }

    // pub(crate) fn restore_context(&mut self) {
    //     // TODO
    // }


    pub fn new(camera_bounds: (i32, i32, u32, u32)) -> Canvas {
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
                shader: Shader::vanilla().unwrap(),
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
            canvas
        }
    }

    /// # Panics
    ///
    /// (debug only) if the size is incorrect (higher than the slice's)
    pub fn add_texture_from_raw_bytes(&mut self, bytes: &[u8], size: (u32, u32)) -> u32 {
        let texture = Texture2D::from_bytes(bytes, size);
        let _v = self.textures.insert(self.current_texture_id, texture);
        debug_assert_eq!(_v, None);
        let texture_id = self.current_texture_id;
        self.current_texture_id += 1;
        texture_id
    }

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

    /// this SHOULD be called when the screen resizes, but NOT when zooming/de-zoming;
    /// use set_zoom_level for that.
    pub fn set_camera_size(&mut self, new_size: (u32, u32)) {
        let (x, y, _, _) = self.camera_bounds.clone();
        self.set_camera((x, y, new_size.0, new_size.1));
    }

    pub fn set_camera_position(&mut self, new_position: (i32, i32)) {
        let (_, _, w, h) = self.camera_bounds.clone();
        self.set_camera((new_position.0, new_position.1, w, h));
    }

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
        self.zoom_level = zoom_level;
        self.apply_projection();
    }

    pub fn zoom_level(&mut self) -> f32 {
        self.zoom_level
    }

    /// CENTER x, CENTER y, width, height
    pub fn camera_bounds(&self) -> (i32, i32, u32, u32) {
        self.camera_bounds
    }

    /// Default is black, like your soul.
    pub fn clear(&mut self, clear_color: Option<Color<u8>>) {
        let clear_color: Color<f32> = clear_color.unwrap_or(Color::<u8>::from_rgb(0, 0, 0)).to_color_f32();
        unsafe {
            gl::ClearColor(clear_color.r, clear_color.g, clear_color.b, 1.0f32);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    // Entities will be drawn from first to last; if you want ORDERED
    // drawing you must use the `ordered_draw` method 
    pub fn draw<'a, 'b: 'a, I: IntoIterator<Item=&'a GraphicEntity<'b>>>(&mut self, graphic_entities: I) {
        self.shader.use_program();
        for graphic_entity in graphic_entities {
            self.draw_graphic_entity(graphic_entity);
        }
    }

    // pub fn ordered_draw<'a, I: IntoIterator<Item=&'a (GraphicEntity, i32)>>(&mut self, graphic_entities_with_z: I) {
    // }

    fn compute_model_matrix(origin_x: f32, origin_y: f32, width: f32, height: f32) -> Matrix4<f32> {
        Matrix4::from_translation(Vector3::<f32>::new(origin_x, origin_y, 0.0)) *
        Matrix4::from_nonuniform_scale(width, height, 1.0)
    }

    fn compute_model_matrix_from_2d_repr(&self, pos: &Graphic2DRepresentation<i32>, element_dims: (u32, u32), scale: Option<f32>) -> Matrix4<f32> {
        use CameraRelativePosition::*;
        use Graphic2DRepresentation::*;
        let elt_w = element_dims.0 as f32 * scale.unwrap_or(1.0);
        let elt_h = element_dims.1 as f32 * scale.unwrap_or(1.0);
        let cam_center_x = self.camera_bounds.0 as f32;
        let cam_center_y = self.camera_bounds.1 as f32;
        let cam_w = self.camera_bounds.2 as f32 / self.zoom_level;
        let cam_h = self.camera_bounds.3 as f32 / self.zoom_level;
        match pos {
            &WorldAbsolute {x, y} =>
                Self::compute_model_matrix(x as f32, y as f32, elt_w, elt_h),
            &CameraRelative {ref position} => {
                match position {
                    &FromTopLeft(x, y) =>
                        Self::compute_model_matrix(
                            ((cam_center_x - cam_w / 2.0) + (x as f32) / self.zoom_level),
                            ((cam_center_y - cam_h / 2.0) + (y as f32) / self.zoom_level),
                            elt_w / self.zoom_level,
                            elt_h / self.zoom_level,
                        ),
                    &FromBottomLeft(x, y) =>
                        Self::compute_model_matrix(
                            ((cam_center_x - cam_w / 2.0) + x as f32 / self.zoom_level),
                            ((cam_center_y + cam_h / 2.0) - (y as f32 + elt_h) / self.zoom_level),
                            elt_w / self.zoom_level,
                            elt_h / self.zoom_level,
                        ),
                    &FromBottomRight(x, y) =>
                        Self::compute_model_matrix(
                            ((cam_center_x + cam_w / 2.0) - (x as f32 + elt_w) / self.zoom_level),
                            ((cam_center_y + cam_h / 2.0) - (y as f32 + elt_h) / self.zoom_level),
                            elt_w / self.zoom_level,
                            elt_h / self.zoom_level,
                        ),
                    &FromTopRight(x, y) =>
                        Self::compute_model_matrix(
                            ((cam_center_x + cam_w / 2.0) - (x as f32 + elt_w) / self.zoom_level),
                            ((cam_center_y - cam_h / 2.0) + y as f32 / self.zoom_level),
                            elt_w / self.zoom_level,
                            elt_h / self.zoom_level,
                        ),
                }
            }
        }
    }

    fn draw_bound_texture(&mut self, texture_dims: (u32, u32), model: &Matrix4<f32>, render_options: &RenderOptions, scale: Option<f32>) {
        self.shader.set_matrix4(UniformName::Model, &model, false);
        if let Some((outline_thickn, color)) = render_options.outline {
            // relative to the texture in the OpenGL sense where 1.0 is max and 0.0 is min,
            // how big is the outline in those coordinates?
            let outline_x = outline_thickn as f32 / texture_dims.0 as f32;
            let outline_y = outline_thickn as f32 / texture_dims.1 as f32;
            self.shader.set_float(UniformName::OutlineWidthX, outline_x, false);
            self.shader.set_float(UniformName::OutlineWidthY, outline_y, false);
            let (r, g, b) = color.rgb();
            self.shader.set_vector3(UniformName::OutlineColor, Vector3::new((r as f32) / 255.0, (g as f32) / 255.0, (b as f32) / 255.0), false);
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

    fn draw_text(&mut self, font_id: u32, font_size: f32, text: &str, font_color: Option<Color<u8>>, repr: &Graphic2DRepresentation<i32>, options: &RenderOptions) {
        let (rgba8_image, width, height) = {
            let font = &self.fonts[&font_id];
            let pixel_height = font_size.ceil() as usize;
            let scale = FontScale::uniform(font_size);

            let font_color: Color<u8> = font_color.unwrap_or(Color::white());

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
        let model = self.compute_model_matrix_from_2d_repr(repr, (width as u32, height as u32), None);
        texture.bind();
        self.draw_bound_texture((width as u32, height as u32), &model, options, None);
    }

    fn draw_graphic_entity<'a>(&mut self, graphic_entity: &GraphicEntity<'a>) {
        match graphic_entity {
            &GraphicEntity::Texture {id, ref repr, ref render_options, scale} => {
                let (model, dims) = {
                    let texture = &self.textures[&id];
                    texture.bind();
                    let texture_dims = texture.size();
                    (self.compute_model_matrix_from_2d_repr(repr, texture_dims, scale), texture_dims)
                };
                self.draw_bound_texture(dims, &model, render_options, scale);
            },
            &GraphicEntity::Text {font_id, font_size, text, color, ref repr, ref render_options} => {
                self.draw_text(font_id, font_size, text, color, repr, render_options);
            }
        }
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.quad_vao);
        }
    }
}