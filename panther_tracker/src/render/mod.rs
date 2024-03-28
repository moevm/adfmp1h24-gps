use std::ffi::CStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use glutin::display::Display;
use winit::dpi::PhysicalPosition;

pub mod screen;
pub mod main_screen;

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

#[derive(Debug)]
pub enum MyInputEvent {
    Back,
    TouchEvent(u64, PhysicalPosition<f64>, winit::event::TouchPhase),
}

pub struct AppState {
    screens: Vec<Box<dyn screen::ScreenTrait>>,
    exit_request: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(exit_request: Arc<AtomicBool>) -> Self {
        AppState {
            screens: Vec::new(),
            exit_request,
        }
    }

    // called once on resume
    pub fn ensure_renderer(&mut self, gl_display: &Display) {
        //nice place to create first screen
        if self.screens.len() == 0 {
            self.screens.push(Box::new(main_screen::Screen::new(gl_display, self.exit_request.clone())));
        }
    }

    // called repeatedly just before draw, to determine, should we draw
    pub fn renderer_ready(&self) -> bool {
        self.screens.len() > 0
    }

    pub fn get_input_screen(&mut self) -> Option<&mut Box<dyn screen::ScreenTrait>> {
        if self.screens.len() > 0 {
            let i = self.screens.len() - 1;
            Some(&mut self.screens[i])
        } else {
            None
        }
    }

    // called repeatedly from outside
    pub fn draw(&mut self) {
        for screen in self.screens.iter_mut() {
            screen.draw();
        }
    }
}