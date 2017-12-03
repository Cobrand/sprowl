use gl;
use gl::types::*;
use std::os::raw::*;
use fnv::FnvHashMap as HashMap;

use cgmath::{Matrix4, Vector3, Vector4};

use std::ffi::{CStr, CString};
use std::ptr;

static FRAGMENT_SHADER_SOURCE: &'static str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vanilla_fs.glsl"));
static VERTEX_SHADER_SOURCE: &'static str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/vanilla_vs.glsl"));

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum UniformName {
    Projection,
    Model,
    OutlineWidthX,
    OutlineWidthY,
    OutlineColor,
    ModelColorFilter,
    ModelColorBlend,
}

impl UniformName {
    fn as_str(&self) -> &'static str {
        use self::UniformName::*;
        match *self {
            Projection => "projection",
            Model => "model",
            OutlineColor => "outline_color",
            OutlineWidthX => "outline_width_x",
            OutlineWidthY => "outline_width_y",
            ModelColorFilter => "model_color_filter",
            ModelColorBlend => "model_color_blend",
        }
    }
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
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
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

pub struct Shader{
    id: GLuint,
    uniforms: HashMap<UniformName, GLint>,
}

impl Shader {
    fn init_all_uniform_locations(&mut self) {
        // Model and projection should be initialized and/or set everytime, no need to "init" them here
        self.init_uniform_location(UniformName::Model);
        self.init_uniform_location(UniformName::Projection);
        self.init_uniform_location(UniformName::ModelColorFilter);
        self.set_vector4(UniformName::ModelColorFilter, Vector4::<f32>::new(1.0, 1.0, 1.0, 1.0), false);
        self.init_uniform_location(UniformName::ModelColorBlend);
        self.set_vector4(UniformName::ModelColorBlend, Vector4::<f32>::new(1.0, 1.0, 1.0, 0.0), false);
        self.init_uniform_location(UniformName::OutlineWidthY);
        self.set_float(UniformName::OutlineWidthY, -1.0, false);
        self.init_uniform_location(UniformName::OutlineWidthX);
        self.set_float(UniformName::OutlineWidthX, -1.0, false);
        self.init_uniform_location(UniformName::OutlineColor);
        self.set_vector3(UniformName::OutlineColor, Vector3::<f32>::new(0.0, 0.0, 0.0), false);
    }

    fn init_uniform_location(&mut self, uniform: UniformName) {
        let name = CString::new(uniform.as_str()).unwrap();
        let uniform_location = unsafe {gl::GetUniformLocation(self.id, name.as_ptr())};
        if uniform_location < 0 {
            panic!("Error / Invalid location for {:?}: gl returned {}", uniform, uniform_location);
        };
        self.uniforms.insert(uniform, uniform_location);
    }

    /// creates a Vanilla Shader
    pub fn vanilla() -> Result<Shader, ShaderLoadError> {
        Shader::new(FRAGMENT_SHADER_SOURCE, VERTEX_SHADER_SOURCE)
    }

    /// Check that the build step "step" has been completed successfully, otherwise return an
    /// Error with the proper information
    fn check_build_step(object: GLuint, step: ShaderBuildStep) -> Result<(), ShaderLoadError> {
        unsafe {
            let mut compile_result = ::std::mem::uninitialized();
            let mut info_log_length = ::std::mem::uninitialized();
            match step {
                ShaderBuildStep::LinkProgram => {
                    gl::GetProgramiv(object, gl::LINK_STATUS, &mut compile_result);
                    if compile_result != i32::from(gl::TRUE) {
                        // retrieve the error
                        gl::GetProgramiv(object, gl::INFO_LOG_LENGTH, &mut info_log_length);
                        let mut error_message: Vec<c_char> = Vec::with_capacity(info_log_length as usize + 1);
                        gl::GetProgramInfoLog(object, info_log_length, ptr::null_mut(), error_message.as_mut_ptr());
                        let log_message = CStr::from_ptr(error_message.as_ptr());
                        return Err(ShaderLoadError::new(step.as_err_type(), format!("{}", log_message.to_string_lossy())))
                    }
                },
                _ => {
                    gl::GetShaderiv(object, gl::COMPILE_STATUS, &mut compile_result);
                    if compile_result != i32::from(gl::TRUE) {
                        // retrieve the error
                        gl::GetShaderiv(object, gl::INFO_LOG_LENGTH, &mut info_log_length);
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

    pub fn set_float(&mut self, name: UniformName, value: GLfloat, use_shader: bool) {
        unsafe {
            if use_shader {
                self.use_program();
            }
            gl::Uniform1f(self.uniforms.get(&name).cloned().unwrap(), value);
        }
    }

    pub fn set_vector4(&mut self, name: UniformName, value: Vector4<f32>, use_shader: bool) {
        unsafe {
            if use_shader {
                self.use_program();
            }
            gl::Uniform4f(self.uniforms.get(&name).cloned().unwrap(), value.x, value.y, value.z, value.w);
        }
    }
    
    pub fn set_vector3(&mut self, name: UniformName, value: Vector3<f32>, use_shader: bool) {
        unsafe {
            if use_shader {
                self.use_program();
            }
            gl::Uniform3f(self.uniforms.get(&name).cloned().unwrap(), value.x, value.y, value.z);
        }
    }

    pub fn set_matrix4(&mut self, name: UniformName, mat: &Matrix4<f32>, use_shader: bool) {
        unsafe {
            if use_shader {
                self.use_program();
            }
            gl::UniformMatrix4fv(self.uniforms.get(&name).cloned().unwrap(), 1, gl::FALSE, mat as *const _ as *const GLfloat)
        }
    } 

    pub fn new(fragment_source: &str, vertex_source: &str) -> Result<Shader, ShaderLoadError> {
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
            shader.init_all_uniform_locations();
            Ok(shader)
        }
    }

    pub fn use_program(&mut self) {
        unsafe { gl::UseProgram(self.id); }
    }
}