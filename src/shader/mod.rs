use gl::{self, types::*};
use cgmath::{Matrix4, Vector2, Vector3, Vector4};
use hashbrown::HashMap;
use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    ptr,
    os::raw::*,
};

/// Trait defining a uniform, typically an enum.
pub trait Uniform: std::fmt::Debug + Clone + Copy + std::hash::Hash + PartialEq + Eq {
    /// Return the name of the uniform, as exactly wirtten in the vertex/fragment shader.
    fn name(&self) -> &str;

    /// Executes the function for all variants of the enum.
    fn for_each<F: FnMut(Self)>(f: F);
}

#[derive(Debug)]
/// Represents a shader: a vertex shader, a fragment shader, a list of uniforms.
pub struct Shader<U: Uniform> {
    id: GLuint,
    uniforms: HashMap<U, GLint>,
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
    fn as_err_type(self) -> &'static str {
        match self {
            ShaderBuildStep::CompileVertexShader => "COMPILE_VERTEX",
            ShaderBuildStep::CompileFragmentShader => "COMPILE_FRAGMENT",
            // ShaderBuildStep::CompileGeometryShader => "COMPILE_GEOMETRY",
            ShaderBuildStep::LinkProgram => "LINK_PROGRAM"
        }
    }
}

impl<U: Uniform> Shader<U> {
    /// Init a uniform location. If you forget to do this for some uniform, your
    /// program will crash at runtime (opengl compile time)
    fn init_uniform_location(&mut self, uniform: U) {
        let name = CString::new(uniform.name()).unwrap();
        let uniform_location = unsafe {gl::GetUniformLocation(self.id, name.as_ptr())};
        if uniform_location < 0 {
            panic!("Error / Invalid location for {:?}: gl returned {}", uniform, uniform_location);
        };
        self.uniforms.insert(uniform, uniform_location);
    }

    /// Check that the build step "step" has been completed successfully, otherwise return an
    /// Error with the proper information
    fn check_build_step(object: GLuint, step: ShaderBuildStep) -> Result<(), ShaderError> {
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
                        return Err(ShaderError::new(step.as_err_type(), format!("{}", log_message.to_string_lossy())))
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
                        return Err(ShaderError::new(step.as_err_type(), format!("{}", log_message.to_string_lossy())))
                    }
                }
            }
        }
        Ok(())
    }

    /// Give a uniform a new uint value.
    pub fn set_uint(&mut self, name: U, value: GLuint) {
        unsafe {
            gl::Uniform1ui(self.uniforms.get(&name).cloned().expect("uniform location was not initialized"), value);
        }
    }
    
    /// Give a uniform a new int value.
    pub fn set_int(&mut self, name: U, value: GLint) {
        unsafe {
            gl::Uniform1i(self.uniforms.get(&name).cloned().expect("uniform location was not initialized"), value);
        }
    }

    /// Give a uniform a new float value.
    pub fn set_float(&mut self, name: U, value: GLfloat) {
        unsafe {
            gl::Uniform1f(self.uniforms.get(&name).cloned().expect("uniform location was not initialized"), value);
        }
    }

    /// Give a uniform a new vector4 value.
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

    /// Create a base fragment shader from a fragment source as raw text (not a path), and a base
    /// vertex shader as raw text as well.
    ///
    /// `texture_units` are the names of the texture units in your shader.
    pub fn new(
        fragment_source: &str,
        vertex_source: &str,
        texture_units: &[&str],
    ) -> Result<Shader<U>, ShaderError> {
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

            let mut shader = Shader {
                id: program_id,
                uniforms: HashMap::default()
            };
            shader.use_program();

            // make sure the names of the texture units match TEXTURE0, TEXTURE1 and so on
            shader.use_texture_units(texture_units);

            // initialize the cache for the glUniformLocation of all the uniforms.
            U::for_each(|uniform| { shader.init_uniform_location(uniform) });

            Ok(shader)
        }
    }

    /// Initialize the texture units.
    fn use_texture_units(&mut self, names: &[&str]) {
        for (i, name) in names.iter().enumerate() {
            self.use_texture_unit(i as GLint, name);
        }
    }

    fn use_texture_unit(&mut self, index: GLint, name: &str) {
        let cname = CString::new(name).unwrap();
        let texture_unit_location = unsafe {gl::GetUniformLocation(self.id, cname.as_ptr())};
        if texture_unit_location < 0 {
            panic!("Error / Invalid location for texture_unit \"{}\": gl returned {}", name, texture_unit_location);
        };
        unsafe {
            // set so that the "name" inside the shader matches TEXTURE`index`. 
            gl::Uniform1i(texture_unit_location, index);
            log::debug!("\"{}\" uniform initialized with value {} (p={})", name, index, texture_unit_location);
        }
    }

    /// Use this program (shader).
    pub fn use_program(&mut self) {
        unsafe { gl::UseProgram(self.id); }
    }
}

impl<U: Uniform> Drop for Shader<U> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

#[derive(Debug)]
pub struct ShaderError {
    err_type: &'static str,
    error_message: String,
}

impl ::std::error::Error for ShaderError {
}

impl ShaderError {
    fn new(err_type: &'static str, error_message: String) -> ShaderError {
        ShaderError {
            err_type,
            error_message,
        }
    } 
}

impl ::std::fmt::Display for ShaderError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "error of type {} while loading shader: {}", self.err_type, self.error_message)
    }
}