//!# Simple and Painless Rust OpenGL Wrapper Library
//! 
//! The goal of this crate is to allow users to "plug"
//! this to an already existing OpenGL context, and
//! draw textures, text easily.
//!
//! You may have to implement your shader yourself, but there are examples
//! you can use in shaders as an inspiration.
//!
//! If you expected to have to write a single line of OpenGL, then sorry,
//! this is not for you. However if you are interested in OpenGL's features
//! but you think current libraries are too limiting and there is nothing
//! to make the process of drawing a texture easier, then this library may be useful to you.
//!
//! Checkout sdl2-simple example for a more basic example.

#[macro_use]
extern crate failure;

mod color;
mod texture;
mod canvas;
mod error;

/// Everything to use shaders and build your own.
pub mod shader;

/// A collection of shader samples. Everything from the simplest shader to more complex ones.
pub mod shaders;

pub use cgmath;

pub use self::color::*;
pub use self::canvas::*;
pub use self::error::{SprowlError as Error};