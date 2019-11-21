use crate::shader::{Uniform, BaseShader, Shader, ShaderLoadError};
use crate::render::{RenderParams, RenderSource, Texture2D, DrawPos, Origin, Shape};
use crate::color::Color;


use cgmath::{Matrix4, Vector2, Vector3, Vector4};

static FRAGMENT_SHADER_SOURCE: &'static str = include_str!("advanced_fs.glsl");
static VERTEX_SHADER_SOURCE: &'static str = include_str!("advanced_vs.glsl");

#[derive(Copy, Clone, Debug)]
pub struct AdvancedRenderParams {
    pub outline: Option<Color<u8>>,
    pub rotate: Option<(f32, Origin)>,
    pub scale: Option<f32>,
    pub effect: u32, // stub, glowing effect == 1 for now
    pub background_color: Option<Color<u8>>,
    pub t: f32,
}

pub struct AdvancedShader {
    shader: BaseShader<AdvancedUniformName>, 
    zoom_level: f32,
}

impl AdvancedShader {
    fn apply_common_uniforms(&mut self, render_params: &RenderParams<<Self as Shader>::R>, (width, height): (u32, u32)) {
        use AdvancedUniformName as UniName;

        let scale = render_params.custom.scale.unwrap_or(1.0);

        let (tex_width, tex_height) = (width, height);
        let DrawPos {origin, x, y} = render_params.common.draw_pos;
        let (crop_offset_x, crop_offset_y, sprite_w, sprite_h) = render_params.common.crop.unwrap_or((0, 0, tex_width, tex_height));
        let (translate_origin_x, translate_origin_y) = origin.compute(sprite_w, sprite_h);
        let mut model = Matrix4::from_nonuniform_scale((tex_width as f32) * scale, (tex_height as f32) * scale, 1.0);

        if let Some((angle, origin)) = render_params.custom.rotate {
            let (pivot_x, pivot_y) = origin.compute(sprite_w, sprite_h);
            let (pivot_x, pivot_y) = (pivot_x + crop_offset_x, pivot_y + crop_offset_y);
            model =
                // rotate around pivot center:
                // translate by (-width/2, -height/2)
                // then rotate,
                // then re-translate by (width/2, height/2)
                // YES this is the correct order, matrices multiplications should be read
                // from right to left!
                Matrix4::from_translation(Vector3::new(pivot_x as f32 * scale, pivot_y as f32 * scale, 0.0))
                * Matrix4::from_angle_z(cgmath::Deg(angle))
                * Matrix4::from_translation(Vector3::new(-pivot_x as f32 * scale, -pivot_y as f32 * scale, 0.0))
                * model
        }

        model = Matrix4::from_translation(Vector3::<f32>::new(
            x as f32 - (translate_origin_x + crop_offset_x) as f32 * scale,
            y as f32 - (translate_origin_y + crop_offset_y) as f32 * scale,
            0.0)
        ) * model;

        self.shader.set_vector2(UniName::OutlineThickness, &Vector2::from((1.0 / tex_width as f32 / scale, 1.0 / tex_height as f32 / scale)));
        if let Some(outline_color) = render_params.custom.outline {
            let color = Vector4::from(outline_color.to_color_f32().rgba());
            self.shader.set_vector4(UniName::OutlineColor, &color);
        } else {
            self.shader.set_vector4(UniName::OutlineColor, &Vector4::from((0f32, 0f32, 0f32, 0f32)));
        }
        self.shader.set_uint(UniName::Effect, render_params.custom.effect);
        let bg_color = render_params.custom.background_color.unwrap_or(Color::from_rgba(0u8, 0, 0, 0));
        self.shader.set_vector4(UniName::BackgroundColor, &Vector4::from(bg_color.to_color_f32().rgba()));
        self.shader.set_float(UniName::T, render_params.custom.t);
        self.shader.set_uint(UniName::IsGrayscale, if render_params.common.is_source_grayscale { 1 } else { 0 });
        self.shader.set_matrix4(UniName::Model, &model);
    }
}

impl Shader for AdvancedShader {
    type R = AdvancedRenderParams;
    type U = AdvancedUniformName;

    fn apply_global_uniforms(&mut self, (window_width, window_height): (u32, u32)) {
        let view_matrix = Matrix4::<f32>::from(cgmath::Ortho {
            left: 0.0,
            right: (window_width as f32) / self.zoom_level,
            bottom: (window_height as f32) / self.zoom_level,
            top: 0.0,
            near: -1.0,
            far: 1.0
        });
        self.shader.set_matrix4(AdvancedUniformName::View, &view_matrix);
    }

    fn as_base_shader(&mut self) -> &mut BaseShader<AdvancedUniformName> {
        &mut self.shader
    }

    fn apply_draw_uniforms(&mut self, render_params: &RenderParams<Self::R>, source: RenderSource<'_>) {
        let (w, h) = source.size();
        self.apply_common_uniforms(render_params, (w, h));
    }

    fn set_draw_vbo<F>(&mut self, render_params: &RenderParams<Self::R>, source: RenderSource<'_>, f: F) where F: FnOnce(&[f32], usize) {
        let vertices: [f32; 24] = match render_params.common.crop {
            Some((x, y, w, h)) => {
                let (texture_width, texture_height) = source.size();
                let f_x = (x as f32) / (texture_width as f32);
                let f_y = (y as f32) / (texture_height as f32);
                let f_w = (w as f32) / (texture_width as f32);
                let f_h = (h as f32) / (texture_height as f32);

                let (left, right) = (f_x, f_x + f_w);
                let (top, bottom) = (f_y, f_y + f_h); 
                [
                    left, bottom, left, bottom,
                    right, top, right, top,
                    left, top, left, top,
                    left, bottom, left, bottom,
                    right, bottom, right, bottom,
                    right, top, right, top,
                ]
            },
            None => 
                [0.0, 1.0, 0.0, 1.0, // 0
                1.0, 0.0, 1.0, 0.0, // 1
                0.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 1.0,
                1.0, 1.0, 1.0, 1.0,
                1.0, 0.0, 1.0, 0.0]
        };

        f(&vertices, 6);
    }

    fn init_all_uniform_locations(&mut self) {
        // Model and view should be initialized and/or set everytime, no need to "init" them here
        self.shader.init_uniform_location(AdvancedUniformName::Model);
        self.shader.init_uniform_location(AdvancedUniformName::View);
        self.shader.init_uniform_location(AdvancedUniformName::OutlineColor);
        self.shader.init_uniform_location(AdvancedUniformName::OutlineThickness);
        self.shader.init_uniform_location(AdvancedUniformName::Effect);
        self.shader.init_uniform_location(AdvancedUniformName::T);
        self.shader.init_uniform_location(AdvancedUniformName::IsGrayscale);
        self.shader.init_uniform_location(AdvancedUniformName::BackgroundColor);
    }
}

impl AdvancedShader {
    pub fn new() -> Result<AdvancedShader, ShaderLoadError> {
        let basic_shader = BaseShader::new(FRAGMENT_SHADER_SOURCE, VERTEX_SHADER_SOURCE)?;
        let mut advanced_shader = AdvancedShader { shader: basic_shader, zoom_level: 2.0 };
        advanced_shader.init_all_uniform_locations();
        Ok(advanced_shader)
    }
}

impl AsRef<BaseShader<AdvancedUniformName>> for AdvancedShader {
    fn as_ref(&self) -> &BaseShader<AdvancedUniformName> {
        &self.shader
    }
}

impl AsMut<BaseShader<AdvancedUniformName>> for AdvancedShader {
    fn as_mut(&mut self) -> &mut BaseShader<AdvancedUniformName> {
        &mut self.shader
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum AdvancedUniformName {
    View,
    Model,
    OutlineColor,
    OutlineThickness,
    BackgroundColor,
    Effect,
    IsGrayscale,
    T,
}

impl Uniform for AdvancedUniformName {}

impl AsRef<str> for AdvancedUniformName {
    fn as_ref(&self) -> &str {
        match self {
            AdvancedUniformName::View => "view",
            AdvancedUniformName::Model => "model",
            AdvancedUniformName::OutlineColor => "outline_color",
            AdvancedUniformName::OutlineThickness => "outline_thickness",
            AdvancedUniformName::Effect => "effect",
            AdvancedUniformName::BackgroundColor => "background_color",
            AdvancedUniformName::IsGrayscale => "is_grayscale",
            AdvancedUniformName::T => "t",
        }
    }
}
