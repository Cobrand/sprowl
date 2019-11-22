use crate::shader::{Uniform, Shader, BaseShader, ShaderLoadError};
use crate::render::{RenderSource, RenderParams, DrawPos, Origin};

use cgmath::{Matrix4, Vector3};

static FRAGMENT_SHADER_SOURCE: &'static str = include_str!("vanilla_fs.glsl");
static VERTEX_SHADER_SOURCE: &'static str = include_str!("vanilla_vs.glsl");

pub struct VanillaShader(BaseShader<VanillaUniformName>);

impl AsRef<BaseShader<VanillaUniformName>> for VanillaShader {
    fn as_ref(&self) -> &BaseShader<VanillaUniformName> {
        &self.0
    }
}

impl AsMut<BaseShader<VanillaUniformName>> for VanillaShader {
    fn as_mut(&mut self) -> &mut BaseShader<VanillaUniformName> {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum VanillaUniformName {
    View,
    Model,
    IsGrayscale,
}

#[derive(Copy, Clone, Debug)]
pub struct RotateOptions {
    pub origin: Origin,
    pub angle: f32
}

#[derive(Copy, Clone, Debug, Default)]
pub struct VanillaRenderParams {
    pub rotate: Option<RotateOptions>,
}

impl Uniform for VanillaUniformName {
}

impl AsRef<str> for VanillaUniformName {
    fn as_ref(&self) -> &str {
        match self {
            VanillaUniformName::View => "view",
            VanillaUniformName::Model => "model",
            VanillaUniformName::IsGrayscale => "is_grayscale",
        }
    }
}

impl VanillaShader {
    pub fn new() -> Result<VanillaShader, ShaderLoadError> {
        let basic_shader = BaseShader::new(FRAGMENT_SHADER_SOURCE, VERTEX_SHADER_SOURCE)?;
        let mut vanilla_shader = VanillaShader(basic_shader);
        vanilla_shader.init_all_uniform_locations();
        Ok(vanilla_shader)
    }
}

impl VanillaShader {
    fn apply_common_uniforms(&mut self, render_params: &RenderParams<<Self as Shader>::R>, (width, height): (u32, u32)) {
        let (tex_width, tex_height) = (width, height);
        let DrawPos {origin, x, y} = render_params.common.draw_pos;
        let (crop_offset_x, crop_offset_y, sprite_w, sprite_h) = render_params.common.crop.unwrap_or((0, 0, tex_width, tex_height));
        let (translate_origin_x, translate_origin_y) = origin.compute(sprite_w, sprite_h);
        let mut model = Matrix4::from_nonuniform_scale(tex_width as f32, tex_height as f32, 1.0);

        if let Some(RotateOptions {angle, origin }) = render_params.custom.rotate {
            let (pivot_x, pivot_y) = origin.compute(sprite_w, sprite_h);
            let (pivot_x, pivot_y) = (pivot_x + crop_offset_x, pivot_y + crop_offset_y);
            model =
                // rotate around pivot center:
                // translate by (-width/2, -height/2)
                // then rotate,
                // then re-translate by (width/2, height/2)
                // YES this is the correct order, matrices multiplications should be read
                // from right to left!
                Matrix4::from_translation(Vector3::new(pivot_x as f32, pivot_y as f32, 0.0))
                * Matrix4::from_angle_z(cgmath::Deg(angle))
                * Matrix4::from_translation(Vector3::new(-pivot_x as f32, -pivot_y as f32, 0.0))
                * model
        }

        model = Matrix4::from_translation(Vector3::<f32>::new(
            x as f32 - (translate_origin_x + crop_offset_x) as f32,
            y as f32 - (translate_origin_y + crop_offset_y) as f32,
            0.0)
        ) * model;

        self.0.set_uint(VanillaUniformName::IsGrayscale, if render_params.common.is_source_grayscale { 1 } else { 0 });
        self.0.set_matrix4(VanillaUniformName::Model, &model);
    }
}

impl Shader for VanillaShader {
    type U = VanillaUniformName;
    type R = VanillaRenderParams;

    fn init_all_uniform_locations(&mut self) {
        // Model and view should be initialized and/or set everytime, no need to "init" them here
        self.0.init_uniform_location(VanillaUniformName::Model);
        self.0.init_uniform_location(VanillaUniformName::View);
        self.0.init_uniform_location(VanillaUniformName::IsGrayscale);
    }
    
    fn apply_draw_uniforms(&mut self, render_params: &RenderParams<Self::R>, source: RenderSource<'_>) {
        let (width, height) = source.size();
        self.apply_common_uniforms(render_params, (width, height))
    }

    fn apply_global_uniforms(&mut self, window_size: (u32, u32)) {
        let view_matrix = Matrix4::<f32>::from(cgmath::Ortho {
            left: 0.0,
            right: (window_size.0 as f32),
            bottom: (window_size.1 as f32),
            top: 0.0,
            near: -1.0,
            far: 1.0
        });
        self.0.set_matrix4(VanillaUniformName::View, &view_matrix);
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

    fn as_base_shader(&mut self) -> &mut BaseShader<Self::U> {
        &mut self.0
    }
}