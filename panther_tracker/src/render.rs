use std::ffi::{CStr, CString};
use std::time::Instant;
use glutin::display::Display;
use glutin::prelude::*;

pub mod screen;

#[rustfmt::skip]
static VERTEX_DATA: [f32; 15] = [
    -0.5, -0.5,  1.0,  0.0,  0.0,
    0.0,  0.5,  0.0,  1.0,  0.0,
    0.5, -0.5,  0.0,  0.0,  1.0,
];

const VERTEX_SHADER_SOURCE: &[u8] = include_bytes!("render/shader-vert.glsl");
const FRAGMENT_SHADER_SOURCE: &[u8] = include_bytes!("render/shader-frag.glsl");

pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

    pub use Gles2 as Gl;
}

unsafe fn create_shader(
    gl: &gl::Gl,
    shader: gl::types::GLenum,
    source: &[u8],
) -> gl::types::GLuint {
    let shader = gl.CreateShader(shader);
    let len = source.len() as gl::types::GLint;
    gl.ShaderSource(shader, 1, [source.as_ptr().cast()].as_ptr(), &len);
    gl.CompileShader(shader);
    shader
}

pub fn get_gl_string(gl: &gl::Gl, variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl.GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

pub struct AppState {
    screen: Option<screen::Screen>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            screen: None,
        }
    }

    pub fn ensure_renderer(&mut self, gl_display: &Display) {
        self.screen.get_or_insert_with(|| screen::Screen::new(gl_display));
    }

    pub fn renderer_ready(&self) -> bool {
        self.screen.is_some()
    }

    pub fn draw(&mut self) {
        if let Some(screen) = &mut self.screen {
            screen.draw();
        }
    }
}