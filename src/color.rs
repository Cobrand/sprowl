/// `u8` or `f32`, represents the max and min values for a valid color representation.
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

/// Represents a Color holding values of f32 (0 to 1) or u8 (0 to 255)
impl<T: ColorType> Color<T> {
    pub fn from_rgba(r: T, g: T, b: T, a: T) -> Color<T> {
        Color {r, g, b, a}
    }

    pub fn from_rgb(r: T, g: T, b: T) -> Color<T> {
        Color {r, g, b, a: T::COLOR_MAX_VALUE}
    }

    pub fn with_alpha(mut self, a: T) -> Color<T> {
        self.a = a;
        self
    }

    pub fn with_opaque(mut self) -> Color<T> {
        self.a = T::COLOR_MAX_VALUE;
        self
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

    pub fn to_vec3(self) -> cgmath::Vector3<T> {
        cgmath::Vector3::new(self.r, self.g, self.b)
    }

    pub fn to_vec4(self) -> cgmath::Vector4<T> {
        cgmath::Vector4::new(self.r, self.g, self.b, self.a)
    }
}

impl<T: ColorType> From<(T, T, T)> for Color<T> {
    fn from((r, g, b): (T, T, T)) -> Self {
        Color {
            r,
            g,
            b,
            a: T::COLOR_MAX_VALUE,
        }
    }
}

impl<T: ColorType> From<(T, T, T, T)> for Color<T> {
    fn from((r, g, b, a): (T, T, T, T)) -> Self {
        Color {
            r,
            g,
            b,
            a,
        }
    }
}

impl Color<u8> {
    pub fn to_color_f32(self) -> Color<f32> {
        Color {
            r: f32::from(self.r) / 255.0,
            g: f32::from(self.g) / 255.0,
            b: f32::from(self.b) / 255.0,
            a: f32::from(self.a) / 255.0,
        }
    }
}