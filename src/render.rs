pub use crate::texture::Texture2D;
pub use crate::gelem::*;
use crate::utils::DrawPos;

#[derive(Debug, Copy, Clone)]
/// A set of common render parameters, that every shader should take into account.
pub struct CommonRenderParams {
    pub draw_pos: DrawPos,
    /// (origin_x, origin_y, width, height)
    pub crop: Option<(i32, i32, u32, u32)>,
    pub is_source_grayscale: bool,
}

impl CommonRenderParams {
    pub fn new(draw_pos: DrawPos) -> CommonRenderParams {
        CommonRenderParams {
            draw_pos,
            crop: None,
            is_source_grayscale: false,
        }
    }
}

/// Render Parameters for some shader, containing a common part (position, crop, is_grayscale, ...) and a custom part
#[derive(Clone)]
pub struct RenderParams<R: Clone> {
    pub common: CommonRenderParams,
    pub custom: R,
}

impl<R: Clone + Copy> Copy for RenderParams<R> {}

impl<R: Default + Clone> RenderParams<R> {
    pub fn new(draw_pos: DrawPos) -> RenderParams<R> {
        RenderParams {
            common: CommonRenderParams::new(draw_pos),
            custom: Default::default(),
        }
    }
}

impl<R: std::fmt::Debug + Clone> std::fmt::Debug for RenderParams<R> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("RenderParams")
            .field("common", &self.common)
            .field("custom", &self.custom)
            .finish()
    }
}