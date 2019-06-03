use crate::shader::{Uniform, BaseShader, ShaderLoadError};
use crate::texture::Texture2D;
use crate::shader::Shader;
use crate::Shape;

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
}

#[derive(Copy, Clone, Debug)]
pub enum Origin {
    Center,
    TopLeft(i32, i32),
}

impl Origin {
    fn compute(&self, width: u32, height: u32) -> (i32, i32) {
        match self {
            Origin::Center => ((width / 2) as i32, (height / 2) as i32),
            Origin::TopLeft(x, y) => (x.clone(), y.clone())
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RotateOptions {
    pub origin: Origin,
    pub angle: f32
}

#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub origin: Origin,
    pub pos_x: i32,
    pub pos_y: i32,
}

#[derive(Copy, Clone, Debug)]
pub struct VanillaRenderParams {
    pub position: Position,
    pub rotate: Option<RotateOptions>,
}

impl Uniform for VanillaUniformName {
}

impl AsRef<str> for VanillaUniformName {
    fn as_ref(&self) -> &str {
        match self {
            VanillaUniformName::View => "view",
            VanillaUniformName::Model => "model",
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
    fn apply_common_uniforms(&mut self, render_params: &<Self as Shader>::R, (width, height): (u32, u32)) {
        let Position {origin, pos_x, pos_y} = render_params.position;
        let (translate_origin_x, translate_origin_y) = origin.compute(width, height);
        let mut model = Matrix4::from_nonuniform_scale(width as f32, height as f32, 1.0);
        // model = translate * rot * scale, but multiplications are applied from right to left
        // (scale, then rotate, then translate) with matrices
        if let Some(RotateOptions { angle, origin }) = render_params.rotate {
            let (pivot_x, pivot_y) = origin.compute(width, height);
            model =
                // rotate around pivot center:
                // translate by (-width/2, -height/2)
                // then rotate,
                // then re-translate by (width/2, height/2)
                // YES this is the correct order, matrices multiplications should be read
                // from right to left!
                Matrix4::from_translation(Vector3::new(pivot_x as f32 , pivot_y as f32, 0.0))
                * Matrix4::from_angle_z(cgmath::Deg(angle))
                * Matrix4::from_translation(Vector3::new(-pivot_x as f32, -pivot_y as f32, 0.0))
                * model
        }
        model = Matrix4::from_translation(Vector3::<f32>::new((pos_x - translate_origin_x)  as f32, (pos_y - translate_origin_y) as f32, 0.0)) * model;
        
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
    }
    
    fn apply_shape_uniforms(&mut self, render_params: &Self::R, shape: &Shape) {
        let (width, height) = match shape {
            Shape::Rect(width, height) => (width.clone(), height.clone()),
        };
        self.apply_common_uniforms(render_params, (width, height))
    }

    fn apply_texture_uniforms(&mut self, render_params: &Self::R, texture: &Texture2D) {
        let (width, height) = texture.size();
        self.apply_common_uniforms(render_params, (width, height))
    }

    fn apply_uniforms(&mut self, window_size: (u32, u32)) {
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

    fn as_base_shader(&mut self) -> &mut BaseShader<Self::U> {
        &mut self.0
    }
}