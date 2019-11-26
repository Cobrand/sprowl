
mod base_shader;
mod render_source;
mod shader_params;

pub use base_shader::*;
pub use render_source::*;
pub use shader_params::*;

pub use base_shader::*;

pub trait Shader {
    type D: ShaderDrawCall + 'static;
    type U: Uniform;

    /// Apply uniforms for this single draw call
    fn apply_draw_uniforms(&mut self, draw_call: Self::D);

    /// Apply uniforms for the current draw batch
    fn apply_global_uniforms(&mut self, window_size: (u32, u32));

    fn as_base_shader(&mut self) -> &mut BaseShader<Self::U>;

    fn init_all_uniform_locations(&mut self);
}