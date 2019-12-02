
mod base_shader;
mod shader_params;

pub use base_shader::*;
pub use shader_params::*;

pub use base_shader::*;

/// The trait defining your shader.
///
/// Implement this for your shader, it must have a base_shader member that you must return here.
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