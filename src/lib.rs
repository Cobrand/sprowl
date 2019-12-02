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

/// Font caching, advanced layouting & rendering
pub mod font;

// Utility structs and enums. 
pub mod utils;
mod texture;
pub use self::texture::*;

mod canvas;
pub use self::canvas::*;

mod error;
pub use self::error::{SprowlError as Error};

mod color;
pub use self::color::*;

/// Everything to pass to the canvas to draw stuff.
pub mod render;

/// Everything to use shaders and build your own.
pub mod shader;

// /// A collection of shader samples. Everything from the simplest shader to more complex ones.
// pub mod shaders;

pub use rusttype;
pub use smallvec;
pub use cgmath;
pub use image;
