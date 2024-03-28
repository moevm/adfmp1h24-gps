use std::mem;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use crate::render::{create_shader, gl, SURFACE_HEIGHT, SURFACE_WIDTH};
use crate::render::gl::{BLEND, ONE_MINUS_SRC_ALPHA, SRC_ALPHA};
use crate::render::gl::types::{GLsizei, GLsizeiptr, GLuint};

#[rustfmt::skip]
static VERTEX_DATA: [f32; 12] = [
    -1.0, -1.0,
    1.0,  1.0,
    1.0, -1.0,
    -1.0, -1.0,
    -1.0,  1.0,
    1.0,  1.0,
];

const VERTEX_SHADER_SOURCE: &[u8] = include_bytes!("squad-vert.glsl");
const FRAGMENT_SHADER_SOURCE: &[u8] = include_bytes!("squad-frag.glsl");

pub struct Squad {
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    gl_mtx: Arc<Mutex<gl::Gl>>,
}


impl Squad {
    pub fn new(gl_mtx: Arc<Mutex<gl::Gl>>) -> Self {
        unsafe {
            let gl = gl_mtx.lock().unwrap();

            let vertex_shader = create_shader(&gl, gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE);
            let fragment_shader = create_shader(&gl, gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);

            let program = gl.CreateProgram();

            gl.AttachShader(program, vertex_shader);
            gl.AttachShader(program, fragment_shader);

            gl.LinkProgram(program);

            gl.UseProgram(program);

            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            gl.Enable(BLEND);
            gl.BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);

            let mut vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (VERTEX_DATA.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                VERTEX_DATA.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let ratio_location = gl.GetUniformLocation(program, b"y_ratio\0".as_ptr() as *const _);

            let dims = (SURFACE_WIDTH.load(Ordering::Relaxed) as f32, SURFACE_HEIGHT.load(Ordering::Relaxed) as f32);
            gl.Uniform1f(ratio_location, dims.1 / dims.0);

            let pos_attrib = gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            gl.VertexAttribPointer(
                pos_attrib as GLuint,
                2,
                gl::FLOAT,
                0,
                2 * mem::size_of::<f32>() as GLsizei,
                std::ptr::null(),
            );
            gl.EnableVertexAttribArray(pos_attrib as GLuint);

            mem::drop(gl);
            Self {
                program,
                vao,
                vbo,
                gl_mtx,
                // circle: circle_location,
                // circ_anim
            }
        }
    }

    pub fn draw(&mut self) {
        let gl = self.gl_mtx.lock().unwrap();

        unsafe {
            gl.UseProgram(self.program);

            gl.BindVertexArray(self.vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            // let params = self.circ_anim.cur();
            // gl.Uniform3f(self.circle, params.0, params.1, params.2);

            gl.DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }
}


impl Drop for Squad {
    fn drop(&mut self) {
        let gl = self.gl_mtx.lock().unwrap();
        unsafe {
            gl.DeleteProgram(self.program);
            gl.DeleteBuffers(1, &self.vbo);
            gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}
