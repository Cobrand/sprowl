//! A small list of helper functions related to OpenGL.
//!
//! Those functions are mostly used internally, but are still publicly available for convenience. 

use gl::types::{GLint, GLenum, GLfloat};
use std::{
    mem::MaybeUninit,
    ffi::CStr
};

pub fn gl_get_int(name: GLenum) -> GLint {
    let mut result = MaybeUninit::<GLint>::uninit();
    unsafe {
        gl::GetIntegerv(name, result.as_mut_ptr());
        result.assume_init()
    }
}

pub fn gl_get_float(name: GLenum) -> GLfloat {
    let mut result = MaybeUninit::<GLfloat>::uninit();
    unsafe {
        gl::GetFloatv(name, result.as_mut_ptr());
        result.assume_init()
    }
}

pub fn gl_get_string(name: GLenum) -> &'static CStr {
    unsafe {
        CStr::from_ptr(gl::GetString(name) as *const _)
    }
}

pub fn gl_get_error() -> Option<GLenum> {
    let r = unsafe { gl::GetError() };
    if r == 0 {
        None
    } else {
        Some(r)
    }
}