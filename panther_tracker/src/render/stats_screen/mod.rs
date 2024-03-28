use std::ffi::CString;
use std::mem;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::time::Instant;
use glutin::display::Display;
use glutin::prelude::*;
use crate::render::{create_shader, get_gl_string};
use crate::render::gl::Gles2;
use crate::render::screen::{ScreenManagementCmd, ScreenTrait};

#[rustfmt::skip]
static VERTEX_DATA: [f32; 15] = [
    -0.5, -0.5,  1.0,  0.0,  0.0,
    0.0,  0.5,  0.0,  1.0,  0.0,
    0.5, -0.5,  0.0,  0.0,  1.0,
];

const VERTEX_SHADER_SOURCE: &[u8] = include_bytes!("shader-vert.glsl");
const FRAGMENT_SHADER_SOURCE: &[u8] = include_bytes!("shader-frag.glsl");

pub use super::gl;

pub struct StatsScreen {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    gl_mtx: Arc<Mutex<gl::Gl>>,
    vertex_data: Vec<f32>,

    exit_request: Arc<AtomicBool>,
    start: Instant,
}

impl StatsScreen {
    pub fn new(gl_mtx: Arc<Mutex<gl::Gl>>, exit_request: Arc<AtomicBool>) -> Self {
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

            let mut vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (VERTEX_DATA.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                VERTEX_DATA.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let pos_attrib = gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            let color_attrib = gl.GetAttribLocation(program, b"color\0".as_ptr() as *const _);
            gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl.VertexAttribPointer(
                color_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (2 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl.EnableVertexAttribArray(color_attrib as gl::types::GLuint);

            mem::drop(gl);
            Self {
                program,
                vao,
                vbo,
                gl_mtx,
                vertex_data: VERTEX_DATA.to_vec(),

                exit_request,
                start: Instant::now(),
            }
        }
    }

    pub fn update_vertex_data(&mut self) {
        unsafe {
            let time1 = self.start.elapsed().as_secs_f32() * 2.0 * std::f32::consts::PI * 0.4;
            let time2 = self.start.elapsed().as_secs_f32() * 2.0 * std::f32::consts::PI * 0.3 + 2.0;
            let time3 = self.start.elapsed().as_secs_f32() * 2.0 * std::f32::consts::PI * 0.2 + 4.0;

            self.vertex_data[4] = time1.sin() / 4.0 + 0.5;
            self.vertex_data[7] = time2.sin() / 4.0 + 0.5;
            self.vertex_data[13] = time3.sin() / 3.0 + 0.5;

            let gl = self.gl_mtx.lock().unwrap();
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (self.vertex_data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.vertex_data.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
        }
    }
}

impl ScreenTrait for StatsScreen {
    fn back(&mut self) -> ScreenManagementCmd {
        ScreenManagementCmd::PopScreen
    }

    fn draw(&mut self) {
        let gl = self.gl_mtx.lock().unwrap();
        unsafe {
            gl.UseProgram(self.program);

            gl.BindVertexArray(self.vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            mem::drop(gl);
            self.update_vertex_data();
            let gl = self.gl_mtx.lock().unwrap();

            gl.ClearColor(0.7, 0.1, 0.9, 0.9);
            gl.Clear(gl::COLOR_BUFFER_BIT);
            gl.DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }
}

impl Drop for StatsScreen {
    fn drop(&mut self) {
        let gl = self.gl_mtx.lock().unwrap();
        unsafe {
            gl.DeleteProgram(self.program);
            gl.DeleteBuffers(1, &self.vbo);
            gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}
