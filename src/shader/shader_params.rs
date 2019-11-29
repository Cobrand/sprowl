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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Scaling {
    None,
    Raw(f32, f32),
    Stretch(u32, u32),
}

impl Default for Scaling {
    fn default() -> Scaling {
        Scaling::None
    }
}

impl Scaling {
    pub fn new(s: f32) -> Scaling {
        Scaling::Raw(s, s)
    }

    #[inline]
    pub fn compute_scale(self, width: u32, height: u32) -> (f32, f32) {
        match self {
            Scaling::None => (1.0, 1.0),
            Scaling::Raw(s_x, s_y) => (s_x, s_y),
            Scaling::Stretch(new_width, new_height) => 
            (new_width as f32 / width as f32,
             new_height as f32 / height as f32),
        }
    }
}

#[derive(Debug)]
pub struct CommonShaderDrawParams {
    pub crop: Option<(i32, i32, u32, u32)>,
    /// "garbage" pixels to add on all sides; for shader purposes
    pub pad: Option<i32>,
    /// angle is degrees
    pub rotate: Option<(f32, Origin)>,
    pub flip: Flip,
    pub scaling: Scaling,
    /// `is_source_grayscale` is necessary to know if the source texture is GL_RED or not.
    /// Mostly used by font rendering.
    pub is_source_grayscale: bool,
    pub draw_pos: DrawPos,
}

impl CommonShaderDrawParams {
    pub fn new(draw_pos: DrawPos) -> CommonShaderDrawParams {
        CommonShaderDrawParams {
            draw_pos,
            crop: Default::default(),
            pad: Default::default(),
            rotate: Default::default(),
            flip: Default::default(),
            scaling: Default::default(),
            is_source_grayscale: false,
        }
    }
}

pub trait ShaderDrawCall: Sized {
    type RenderParams: Clone;

    fn render_source(&self) -> RenderSource;

    fn common_params(&self) -> &CommonShaderDrawParams;

    fn from_graphic_elem<S: AsRef<str>>(graphic_elem: &GraphicElement<S, Self::RenderParams>, canvas: &mut Canvas) -> Result<SmallVec<[ Self; 2]>, SprowlError>;
}