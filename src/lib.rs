//!# Simple and Painless Rust OpenGL Wrapper Library
//! 
//! The goal of this crate is to allow users to "plug"
//! this to an already existing OpenGL context, and
//! draw textures, text easily.
//!
//! You will have to implement your shader yourself, but there are examples
//! you can use in shaders as an inspiration.
//!
//! This library is tailored for my own uses and is heavily unstable. Beware!
//!
//! Currently, the library is split under 2 big categories:
//! * A `render_storage` module, which allows you to store textures and fonts.
//! * As `renderer` and `shader` module, which allow you to print those textures and font.
//!
//! While speed is not the main focus of this crate, we still want every device to have a smooth
//! experience. As such, we use instanced rendering to only call glDrawArrays once, with
//! only two textures bound: one RGBA, for the usual textures, and one grayscale, for the text.
//!
//! Checkout sdl2-simple example for a basic example.

pub mod renderer;
pub mod render_storage;

pub mod gl_utils;

mod error;
pub use self::error::{SprowlError as Error};

mod color;
pub use self::color::*;

/// Everything to use shaders and build your own.
pub mod shader;

pub use rusttype;
pub use smallvec;
pub use cgmath;
pub use image;
