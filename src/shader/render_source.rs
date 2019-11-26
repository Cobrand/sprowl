use crate::texture::Texture2D;
use crate::utils::Shape;

/// Represents something your shader would like to draw. Typically, a texture or a shape.
pub enum RenderSource<'a> {
    Texture(&'a Texture2D),
    Shape(&'a Shape),
}

impl<'a> RenderSource<'a> {
    pub fn size(&self) -> (u32, u32) {
        match self {
            RenderSource::Texture(t) => t.size(),
            RenderSource::Shape(s) => s.max_size(),
        }
    }

    #[inline]
    pub fn compute_draw_vbo(&self, crop: Option<(i32, i32, u32, u32)>) -> [f32; 24] {
        if let Some((x, y, w, h)) = crop {
            let (t_w, t_h) = self.size();
            let (t_w, t_h) = (t_w as f32, t_h as f32);
            let (x, y, w, h) = (x as f32 / t_w, y as f32 / t_h, w as f32 / t_w, h as f32 / t_h);
            let left = x;
            let bottom = y + h;
            let right =  x + w;
            let top = y;
            [
                left, bottom, left, bottom,
                right, top, right, top,
                left, top, left, top,
                left, bottom, left, bottom,
                right, bottom, right, bottom,
                right, top, right, top,
            ]
        } else {
            [0.0, 1.0, 0.0, 1.0, // 0
            1.0, 0.0, 1.0, 0.0, // 1
            0.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 1.0,
            1.0, 1.0, 1.0, 1.0,
            1.0, 0.0, 1.0, 0.0]
        }
    }
}

impl<'a> From<&'a Texture2D> for RenderSource<'a> {
    fn from(t: &'a Texture2D) -> RenderSource<'a> {
        RenderSource::Texture(t)
    }
}

impl<'a> From<&'a Shape> for RenderSource<'a> {
    fn from(s: &'a Shape) -> RenderSource<'a> {
        RenderSource::Shape(s)
    }
}