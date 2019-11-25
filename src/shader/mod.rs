
use cgmath::Vector2;

mod base_shader;
mod render_source;
mod shader_params;

pub use base_shader::*;
pub use render_source::*;
pub use shader_params::*;

use render_source::RenderSource;
use crate::render::RenderStem;

pub use base_shader::*;

// use crate::render::{RenderParams, RenderSource};

/// 
pub struct ShaderDrawCall<'a, C> {
    pub source: RenderSource<'a>,
    pub params: ShaderDrawParams<C>,
}

pub trait Shader {
    type RenderParams: AsShaderDrawCall<RenderStem>;
    type U: Uniform;

    /// Apply uniforms for this single draw call
    fn apply_draw_uniforms(&mut self, draw_elem: ShaderDrawCall<'_, <Self::RenderParams as AsShaderDrawCall<RenderStem>>::CustomShaderDrawParams >);

    /// Apply uniforms for the current draw batch
    fn apply_global_uniforms(&mut self, window_size: (u32, u32));

    fn as_base_shader(&mut self) -> &mut BaseShader<Self::U>;

    fn init_all_uniform_locations(&mut self);
}