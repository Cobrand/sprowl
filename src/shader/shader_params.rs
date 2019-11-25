use crate::utils::{Origin, DrawPos};
use super::ShaderDrawCall;
use smallvec::SmallVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flip {
    None,
    Horizontal,
    Vertical,
    Both,
}

impl Default for Flip {
    fn default() -> Self {
        Flip::None
    }
}

pub struct CommonShaderDrawParams {
    pub crop: Option<(i32, i32, u32, u32)>,
    /// angle is degrees
    pub rotate: Option<(f32, Origin)>,
    pub flip: Flip,
    pub draw_pos: DrawPos,
}

pub struct ShaderDrawParams<C> {
    pub common: CommonShaderDrawParams,
    pub custom: C,
}

pub trait AsShaderDrawCall {
    type CustomShaderDrawParams;

    fn as_shader_params<'a>(&self, canvas: &'a ()) -> SmallVec<[ ShaderDrawCall<'a, Self::CustomShaderDrawParams>; 2]>;
}