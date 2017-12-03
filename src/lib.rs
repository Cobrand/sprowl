//!# Simple and Painless Rust OpenGL Wrapper Library
//! 
//! The goal of this crate is to allow users to "plug"
//! this to an already existing OpenGL context, and
//! draw textures, text easily.
//!
//! Some useful OpenGL features are also built-in directly,
//! like outlining textures, changing their color, making them
//! transparent, scaling them in size, ...
//!
//! If you expected to have to write a single line of OpenGL, then sorry,
//! this is not for you. However if you are interested in OpenGL's features
//! but you think current libraries are too limiting and there is nothing
//! to make "draw a texture" easier, then this library may be useful to you.
//!
//! Checkout sdl2-simple example for a more basic example.
extern crate gl;
extern crate image;
extern crate cgmath;
extern crate rusttype;
extern crate fnv;
#[macro_use]
extern crate failure;

mod primitives;
mod color;
mod texture;
mod canvas;
mod error;

pub use self::texture::*;
pub use self::color::*;
pub use self::canvas::*;
pub use self::error::{SprowlError as Error};