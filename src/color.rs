pub trait ColorType: ::std::fmt::Debug + Clone + Copy {
    const COLOR_MAX_VALUE: Self;
    const COLOR_MIN_VALUE: Self;
}

/// Color where the minimum is 0 and maximum is 255.
///
/// The industry standard
impl ColorType for u8 {
    const COLOR_MAX_VALUE: u8 = 255;
    const COLOR_MIN_VALUE: u8 = 0;
}

/// Color where the minimum is 0.0 and the maximum is 1.0
///
/// Mainly used for OpenGL
impl ColorType for f32 {
    const COLOR_MAX_VALUE: f32 = 1.0f32;
    const COLOR_MIN_VALUE: f32 = 0.0f32;
}

#[derive(Debug, Clone, Copy)]
pub struct Color<T: ColorType> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T
}

impl<T: ColorType> Color<T> {
    pub fn from_rgba(r: T, g: T, b: T, a: T) -> Color<T> {
        Color {r, g, b, a}
    }

    pub fn from_rgb(r: T, g: T, b: T) -> Color<T> {
        Color {r, g, b, a: T::COLOR_MAX_VALUE}
    }

    pub fn rgb(self) -> (T, T, T) {
        (self.r, self.g, self.b)
    }

    pub fn rgba(self) -> (T, T, T, T) {
        (self.r, self.g, self.b, self.a)
    }

    pub fn white() -> Color<T> {
        Color {r: T::COLOR_MAX_VALUE, g: T::COLOR_MAX_VALUE, b: T::COLOR_MAX_VALUE, a: T::COLOR_MAX_VALUE}
    }
    
    pub fn black() -> Color<T> {
        Color {r: T::COLOR_MIN_VALUE, g: T::COLOR_MIN_VALUE, b: T::COLOR_MIN_VALUE, a: T::COLOR_MAX_VALUE}
    }
}

impl Color<u8> {
    pub fn to_color_f32(self) -> Color<f32> {
        Color {
            r: (self.r as f32) / 255.0,
            g: (self.g as f32) / 255.0,
            b: (self.b as f32) / 255.0,
            a: (self.a as f32) / 255.0,
        }
    }
}