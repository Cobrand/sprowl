use crate::utils::{Origin, DrawPos};
use smallvec::SmallVec;
use crate::gelem::GraphicElement;
use crate::canvas::Canvas;
use crate::error::SprowlError;
use super::render_source::RenderSource;

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

#[derive(Debug)]
pub struct CommonShaderDrawParams {
    pub crop: Option<(i32, i32, u32, u32)>,
    /// angle is degrees
    pub rotate: Option<(f32, Origin)>,
    pub flip: Flip,
    pub draw_pos: DrawPos,
}

pub trait ShaderDrawCall: Sized {
    type RenderParams: Clone;

    fn render_source<'a>(&'a self) -> RenderSource<'a>;

    fn common_params(&self) -> &CommonShaderDrawParams;

    fn from_graphic_elem<'a, S: AsRef<str>>(graphic_elem: &GraphicElement<S, Self::RenderParams>, canvas: &'a mut Canvas) -> Result<SmallVec<[ Self; 2]>, SprowlError>;
}