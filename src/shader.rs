use gl;
use gl::types::*;
use std::os::raw::*;
use fnv::FnvHashMap as HashMap;

pub use crate::texture::Texture2D;
pub use crate::Shape;

use cgmath::{Matrix4, Vector2, Vector3, Vector4};

use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::ptr;


pub trait Uniform: AsRef<str> + ::std::fmt::Debug + Clone + Copy + ::std::hash::Hash + PartialEq + Eq {
}

#[derive(Debug)]
pub struct ShaderLoadError {
    err_type: &'static str,
    error_message: String,
}

impl ::std::error::Error for ShaderLoadError {
    fn description(&self) -> &str {
        "error while loading shader"
    }
}

impl ShaderLoadError {
    fn new(err_type: &'static str, error_message: String) -> ShaderLoadError {
        ShaderLoadError {
            err_type,
            error_message
        }
    } 
}

impl ::std::fmt::Display for ShaderLoadError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "error of type {} while loading shader: {}", self.err_type, self.error_message)
    }
}


#[derive(Clone, Copy, Debug)]
enum ShaderBuildStep {
    CompileVertexShader,
    CompileFragmentShader,
    // // Will come soon...
    // CompileGeometryShader,
    LinkProgram
}

impl ShaderBuildStep {
    fn as_err_type(&self) -> &'static str {
        match *self {
            ShaderBuildStep::CompileVertexShader => "COMPILE_VERTEX",
            ShaderBuildStep::CompileFragmentShader => "COMPILE_FRAGMENT",
            // ShaderBuildStep::CompileGeometryShader => "COMPILE_GEOMETRY",
            ShaderBuildStep::LinkProgram => "LINK_PROGRAM"
        }
    }
}

pub trait Shader {
    type R: 'static;
    type U: Uniform;

    fn apply_texture_uniforms(&mut self, render_params: &Self::R, texture: &Texture2D);

    fn apply_shape_uniforms(&mut self, render_params: &Self::R, shape: &Shape);

    fn apply_uniforms(&mut self, window_size: (u32, u32));

    fn set_texture_vbo<F>(&mut self, _render_params: &Self::R, _texture: &Texture2D, mut f: F) where F: FnMut(&[f32], usize) {
        static DEFAULT_VERTICES: [f32; 24] =
            [0.0, 1.0, 0.0, 1.0, // 0
            1.0, 0.0, 1.0, 0.0, // 1
            0.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 1.0,
            1.0, 1.0, 1.0, 1.0,
            1.0, 0.0, 1.0, 0.0];
        f(&DEFAULT_VERTICES, 6);
    }
    
    fn set_shape_vbo<F>(&mut self, _render_params: &Self::R, _shape: &Shape, mut f: F) where F: FnMut(&[f32], usize) {
        static DEFAULT_VERTICES: [f32; 24] =
            [0.0, 1.0, 0.0, 1.0, // 0
            1.0, 0.0, 1.0, 0.0, // 1
            0.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 1.0,
            1.0, 1.0, 1.0, 1.0,
            1.0, 0.0, 1.0, 0.0];
        f(&DEFAULT_VERTICES, 6);
    }

    fn as_base_shader(&mut self) -> &mut BaseShader<Self::U>;

    fn init_all_uniform_locations(&mut self);
}

pub struct BaseShader<U: Uniform> {
    id: GLuint,
    uniforms: HashMap<U, GLint>,
}

impl<U: Uniform> BaseShader<U> {

    pub fn init_uniform_location(&mut self, uniform: U) {
        let name = CString::new(uniform.as_ref()).unwrap();
        let uniform_location = unsafe {gl::GetUniformLocation(self.id, name.as_ptr())};
        if uniform_location < 0 {
            panic!("Error / Invalid location for {:?}: gl returned {}", uniform, uniform_location);
        };
        self.uniforms.insert(uniform, uniform_location);
    }

    /// Check that the build step "step" has been completed successfully, otherwise return an
    /// Error with the proper information
    fn check_build_step(object: GLuint, step: ShaderBuildStep) -> Result<(), ShaderLoadError> {
        unsafe {
            let mut compile_result: MaybeUninit<GLint> = MaybeUninit::uninit();
            let mut info_log_length: MaybeUninit<GLint> = MaybeUninit::uninit();
            match step {
                ShaderBuildStep::LinkProgram => {
                    gl::GetProgramiv(object, gl::LINK_STATUS, compile_result.as_mut_ptr());
                    let compile_result = compile_result.assume_init();
                    if compile_result != i32::from(gl::TRUE) {
                        // retrieve the error
                        gl::GetProgramiv(object, gl::INFO_LOG_LENGTH, info_log_length.as_mut_ptr());
                        let info_log_length = info_log_length.assume_init();
                        let mut error_message: Vec<c_char> = Vec::with_capacity(info_log_length as usize + 1);
                        gl::GetProgramInfoLog(object, info_log_length, ptr::null_mut(), error_message.as_mut_ptr());
                        let log_message = CStr::from_ptr(error_message.as_ptr());
                        return Err(ShaderLoadError::new(step.as_err_type(), format!("{}", log_message.to_string_lossy())))
                    }
                },
                _ => {
                    gl::GetShaderiv(object, gl::COMPILE_STATUS, compile_result.as_mut_ptr());
                    let compile_result = compile_result.assume_init();
                    if compile_result != i32::from(gl::TRUE) {
                        // retrieve the error
                        gl::GetShaderiv(object, gl::INFO_LOG_LENGTH, info_log_length.as_mut_ptr());
                        let info_log_length = info_log_length.assume_init();
                        let mut error_message: Vec<c_char> = Vec::with_capacity(info_log_length as usize + 1);
                        gl::GetShaderInfoLog(object, info_log_length, ptr::null_mut(), error_message.as_mut_ptr());
                        let log_message = CStr::from_ptr(error_message.as_ptr());
                        return Err(ShaderLoadError::new(step.as_err_type(), format!("{}", log_message.to_string_lossy())))
                    }
                }
            }
        }
        Ok(())
    }

    pub fn set_uint(&mut self, name: U, value: GLuint) {
        unsafe {
            gl::Uniform1ui(self.uniforms.get(&name).cloned().expect("uniform location was not initialized"), value);
        }
    }
    
    pub fn set_int(&mut self, name: U, value: GLint) {
        unsafe {
            gl::Uniform1i(self.uniforms.get(&name).cloned().expect("uniform location was not initialized"), value);
        }
    }

    pub fn set_float(&mut self, name: U, value: GLfloat) {
        unsafe {
            gl::Uniform1f(self.uniforms.get(&name).cloned().expect("uniform location was not initialized"), value);
        }
    }

    pub fn set_vector4(&mut self, name: U, value: &Vector4<f32>) {
        unsafe {
            gl::Uniform4f(self.uniforms.get(&name).cloned().expect("uniform location was not initialized"), value.x, value.y, value.z, value.w);
        }
    }
    
    pub fn set_vector3(&mut self, name: U, value: &Vector3<f32>) {
        unsafe {
            gl::Uniform3f(self.uniforms.get(&name).cloned().expect("uniform location was not initialized"), value.x, value.y, value.z);
        }
    }

    pub fn set_vector2(&mut self, name: U, value: &Vector2<f32>) {
        unsafe {
            gl::Uniform2f(self.uniforms.get(&name).cloned().expect("uniform location was not initialized"), value.x, value.y);
        }
    }

    pub fn set_matrix4(&mut self, name: U, mat: &Matrix4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(self.uniforms.get(&name).cloned().expect("uniform location was not initialized"), 1, gl::FALSE, mat as *const _ as *const GLfloat)
        }
    }

    pub fn new(fragment_source: &str, vertex_source: &str) -> Result<BaseShader<U>, ShaderLoadError> {
        unsafe {
            let vertex_shader_id = gl::CreateShader(gl::VERTEX_SHADER);
            let fragment_shader_id = gl::CreateShader(gl::FRAGMENT_SHADER);

            let fragment_shader = CString::new(fragment_source).unwrap();
            let vertex_shader = CString::new(vertex_source).unwrap();
            
            gl::ShaderSource(vertex_shader_id, 1, &vertex_shader.as_c_str().as_ptr(), ::std::ptr::null());
            gl::CompileShader(vertex_shader_id);
            Self::check_build_step(vertex_shader_id, ShaderBuildStep::CompileVertexShader)?;

            gl::ShaderSource(fragment_shader_id, 1, &fragment_shader.as_c_str().as_ptr(), ::std::ptr::null());
            gl::CompileShader(fragment_shader_id);
            Self::check_build_step(fragment_shader_id, ShaderBuildStep::CompileFragmentShader)?;

            let program_id = gl::CreateProgram();
            gl::AttachShader(program_id, vertex_shader_id);
            gl::AttachShader(program_id, fragment_shader_id);
            gl::LinkProgram(program_id);
            Self::check_build_step(program_id, ShaderBuildStep::LinkProgram)?;

            gl::DetachShader(program_id, vertex_shader_id);
            gl::DetachShader(program_id, fragment_shader_id);

            gl::DeleteShader(vertex_shader_id);
            gl::DeleteShader(fragment_shader_id);

            let mut shader = BaseShader {
                id: program_id,
                uniforms: HashMap::default()
            };
            shader.use_program();
            Ok(shader)
        }
    }

    pub fn use_program(&mut self) {
        unsafe { gl::UseProgram(self.id); }
    }
}