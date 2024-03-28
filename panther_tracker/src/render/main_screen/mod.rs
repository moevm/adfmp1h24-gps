use std::ffi::CString;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;
use glutin::display::Display;
use glutin::prelude::*;
use crate::render::{create_shader, get_gl_string};
use crate::render::screen::ScreenTrait;

#[rustfmt::skip]
static VERTEX_DATA: [f32; 15] = [
    -0.5, -0.5,  1.0,  0.0,  0.0,
    0.0,  0.5,  0.0,  1.0,  0.0,
    0.5, -0.5,  0.0,  0.0,  1.0,
];

const VERTEX_SHADER_SOURCE: &[u8] = include_bytes!("shader-vert.glsl");
const FRAGMENT_SHADER_SOURCE: &[u8] = include_bytes!("shader-frag.glsl");

pub use super::gl;

pub struct Screen {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    gl: gl::Gl,
    vertex_data: Vec<f32>,

    exit_request: Arc<AtomicBool>,
    start: Instant,
}

impl Screen {
    pub fn new(gl_display: &Display, exit_request: Arc<AtomicBool>) -> Self {
        unsafe {
            let gl = gl::Gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            if let Some(renderer) = get_gl_string(&gl, gl::RENDERER) {
                println!("Running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = get_gl_string(&gl, gl::VERSION) {
                println!("OpenGL Version {}", version.to_string_lossy());
            }

            if let Some(shaders_version) = get_gl_string(&gl, gl::SHADING_LANGUAGE_VERSION) {
                println!("Shaders version on {}", shaders_version.to_string_lossy());
            }

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

            Self {
                program,
                vao,
                vbo,
                gl,
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

            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            self.gl.BufferData(
                gl::ARRAY_BUFFER,
                (self.vertex_data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.vertex_data.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}

impl ScreenTrait for Screen {
    // fn process_input(&mut self, input: MyInputEvent) {
    //     info!("Recv input event: {:?}", input);
    //     match input {
    //         MyInputEvent::Back => {
    //             self.exit_request.store(true, std::sync::atomic::Ordering::Relaxed);
    //         }
    //         MyInputEvent::TouchEvent(id, location, phase) => {
    //             info!("Touch event: id: {}, location: {:?}, phase: {:?}", id, location, phase);
    //         }
    //     }
    // }

    fn draw(&mut self) {
        unsafe {
            self.gl.UseProgram(self.program);

            self.gl.BindVertexArray(self.vao);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            self.update_vertex_data();

            self.gl.ClearColor(0.1, 0.1, 0.1, 0.9);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
            self.gl.DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.program);
            self.gl.DeleteBuffers(1, &self.vbo);
            self.gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}
