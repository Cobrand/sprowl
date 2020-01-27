use crate::shader::{Shader, Uniform};
use crate::color::Color;
use gl::types::*;
use std::{
    mem::{MaybeUninit, size_of},
    ptr,
    os::raw::c_void,
};
#[derive(Debug)]
pub struct RendererBuilder {
    pub (crate) vao: GLuint,
    pub (crate) quad_vbo: GLuint,
    pub (crate) instanced_vbo: GLuint,
    pub (crate) next_vertex_attrib: GLuint,
    /// vec of (attrib_index, width in 4bytes, type)
    ///
    /// type may be gl::FLOAT, gl::INT, gl::UNSIGNED_INT
    pub (crate) instanced_attribs: Vec<(GLuint, usize, GLenum)>,
    pub (crate) max_instances: usize,
}

const VERTICES_PER_ELEM: usize = 6;
impl RendererBuilder {
    /// Build a new renderer.
    ///
    /// `max_instances` is the number of instances you would lke to draw in one call.
    pub fn new(max_instances: usize) -> RendererBuilder {
        log::debug!("preparing renderer for max_instances={}", max_instances);
        let mut vao: MaybeUninit<GLuint> = MaybeUninit::uninit();
        let mut quad_vbo: MaybeUninit<GLuint> = MaybeUninit::uninit();

        let mut instanced_vbo: MaybeUninit<GLuint> = MaybeUninit::uninit();

        unsafe {
            gl::GenVertexArrays(1, vao.as_mut_ptr());
            gl::GenBuffers(1, quad_vbo.as_mut_ptr());
            gl::GenBuffers(1, instanced_vbo.as_mut_ptr());
        }

        let vao = unsafe { vao.assume_init() };
        let quad_vbo = unsafe { quad_vbo.assume_init() };
        let instanced_vbo = unsafe { instanced_vbo.assume_init() };

        RendererBuilder {
            vao,
            quad_vbo,
            instanced_vbo,
            next_vertex_attrib: 1,
            instanced_attribs: vec!(),
            max_instances,
        }
    }

    /// Add a vertex attrib
    ///
    /// `width` is the number of f32/u32/i32 in the attribute: 4 if vec4, 1 if uint, ect.
    ///
    /// `gl_type` is the type, typically `gl::FLOAT` or `gl::UNSIGNED_INT`.
    ///
    /// # Example
    ///
    /// `width` = 4, `gl_type` = `gl::FLOAT` will add a `vec4` attrib to the VAO, that you can
    /// use from your shader.
    ///
    /// Note that the first location is reserved for the vertex attrib, so the first instanced
    /// vertex atttrib should start from 1.
    pub fn with_instanced_vertex_attrib(mut self, width: usize, gl_type: GLenum) -> Self {
        self.instanced_attribs.push((self.next_vertex_attrib, width, gl_type));
        self.next_vertex_attrib += 1;
        self
    }

    pub fn build_with<U: Uniform>(self, shader: Shader<U>) -> Renderer<U> {
        // the total size of the vbo to cotnain "max_elements".
        let tot_width_quad_vbo: usize = 2;
        let tot_width_instanced_vbo: usize = self.instanced_attribs.iter().map(|(_, s, _)| s).sum();

        let all_elems_size_instanced_vbo = tot_width_instanced_vbo * self.max_instances * (size_of::<f32>());

        unsafe {
            // allocate both buffers
            const VERTICES12: [f32; 12] = 
            [
                0.0, 1.0,
                1.0, 0.0,
                0.0, 0.0,

                0.0, 1.0,
                1.0, 0.0,
                1.0, 1.0
            ];
            gl::BindBuffer(gl::ARRAY_BUFFER, self.quad_vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (tot_width_quad_vbo * VERTICES_PER_ELEM * size_of::<f32>()) as isize, &VERTICES12 as *const _ as *const c_void, gl::DYNAMIC_DRAW);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.instanced_vbo);
            gl::BufferData(gl::ARRAY_BUFFER, all_elems_size_instanced_vbo as isize, ptr::null(), gl::DYNAMIC_DRAW);
        }

        
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.quad_vbo);

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0, 2, gl::FLOAT, gl::FALSE,
                // f32, i32, u32 are all 4 bytes, and we have 2 vertices
                (2 * 4) as GLint,
                // f32, i32, u32 are all 4 bytes
                ptr::null::<c_void>()
            );


            let mut current_stride: usize = 0;
            gl::BindBuffer(gl::ARRAY_BUFFER, self.instanced_vbo);
            for (i, widthof_attrib, gl_type) in self.instanced_attribs {
                gl::EnableVertexAttribArray(i);
                log::debug!("enabled vertex attrib instanced i={} width={} gl_type={} current_stride={} tot_width_instanced_vbo={}",
                    i, widthof_attrib, gl_type, current_stride, tot_width_instanced_vbo);
                if gl_type != gl::FLOAT {
                    gl::VertexAttribIPointer(
                        i, widthof_attrib as GLint, gl_type,
                        // f32, i32, u32 are all 4 bytes
                        (tot_width_instanced_vbo * 4) as GLint,
                        // f32, i32, u32 are all 4 bytes
                        ptr::null::<c_void>().offset((current_stride * 4) as isize)
                    );
                } else {
                    gl::VertexAttribPointer(
                        i, widthof_attrib as GLint, gl_type, gl::FALSE,
                        // f32, i32, u32 are all 4 bytes
                        (tot_width_instanced_vbo * 4) as GLint,
                        // f32, i32, u32 are all 4 bytes
                        ptr::null::<c_void>().offset((current_stride * 4) as isize)
                    );
                }
                // only necessary for instanced arrays
                gl::VertexAttribDivisor(i, 1);
                current_stride += widthof_attrib;
            }
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        // general init for the renderer:
        unsafe {
            // enable alpha blending
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        Renderer {
            vao: self.vao,
            instanced_vbo: self.instanced_vbo,
            quad_vbo: self.quad_vbo,
            max_instances: self.max_instances,
            shader,

            instance_count: 0,
            temp_instanced_vb: Vec::with_capacity(all_elems_size_instanced_vbo as usize),
        }
    }
}

#[derive(Debug)]
pub struct Renderer<U: Uniform> {
    pub (crate) vao: GLuint,
    pub (crate) quad_vbo: GLuint,
    pub (crate) instanced_vbo: GLuint,
    pub (crate) max_instances: usize,
    pub shader: Shader<U>,

    // temp values, reset after every draw
    pub (crate) temp_instanced_vb: Vec<u8>,

    pub (crate) instance_count: usize,
}

impl<U: Uniform> Renderer<U> {
    /// Clear the screen with a solid color.
    /// 
    /// Default clear color is black, just like your soul.
    pub fn clear(&mut self, clear_color: Option<Color<u8>>) {
        let clear_color: Color<f32> = clear_color.unwrap_or_else(|| Color::<u8>::from_rgb(0, 0, 0)).to_color_f32();
        unsafe {
            gl::ClearColor(clear_color.r, clear_color.g, clear_color.b, 1.0f32);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    #[inline]
    pub fn set_viewport(&self, width: u32, height: u32) {
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }

    pub fn add_elem<E: AsVertexData>(&mut self, e: &E) {
        let added_instances = e.add_vertex_data(&mut self.temp_instanced_vb);
        self.instance_count += added_instances as usize;
    }

    pub fn draw(&mut self) {
        assert!(self.max_instances >= self.instance_count);
        unsafe {
            // fill instanced_vbo from temp
            gl::BindBuffer(gl::ARRAY_BUFFER, self.instanced_vbo);
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, self.temp_instanced_vb.len() as isize, self.temp_instanced_vb.as_ptr() as *const _);
            // note that temp VBs are used instead of copying 1 by 1, because we never know how long an opengl call might take,
            // every implementation might take a short or long time. Since we have to do this call several times (up to multiple thousands) per frame,
            // i found it best to regroup it into one single call, using a temporary buffer on the heap.
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            gl::BindVertexArray(self.vao);
            gl::DrawArraysInstanced(gl::TRIANGLES, 0, VERTICES_PER_ELEM as GLint, self.instance_count as GLint);
            gl::BindVertexArray(0);
        }
        self.instance_count = 0;
        self.temp_instanced_vb.clear();
    }
}

impl<U: Uniform> Drop for Renderer<U> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.quad_vbo);
            gl::DeleteBuffers(1, &self.instanced_vbo);
        }
    }
}

/// The main trait you need to implement for your entities you want to draw.
///
/// You might have to transmute or do some unsafe stuff in here, but hopefully you VertexAttribPointers
/// are correct!
pub trait AsVertexData {
    /// Given the instanced_vertex_buffer, you
    /// should add as many vertex as you want to this buffer, respecting of course
    /// your config. You should return the number of instances you have added.
    fn add_vertex_data(&self, instanced_vb: &mut Vec<u8>) -> u32;
}