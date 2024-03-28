use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use glutin::display::{Display, GlDisplay};
use log::info;
use winit::dpi::PhysicalPosition;

pub mod screen;
pub mod main_screen;
pub mod stats_screen;

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
    gl: Option<Arc<Mutex<gl::Gl>>>,
}

impl AppState {
    pub fn new(exit_request: Arc<AtomicBool>) -> Self {
        AppState {
            screens: Vec::new(),
            exit_request,
            gl: None
        }
    }

    // called once on resume
    pub fn ensure_renderer(&mut self, gl_display: &Display) {
        let gl = self.gl.get_or_insert_with(|| {
            log::info!("[AppState] Initializing GL...");

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

            Arc::new(Mutex::new(gl))
        });

        //nice place to create first screen
        if self.screens.len() == 0 {
            self.screens.push(Box::new(main_screen::MainScreen::new(gl.clone(), self.exit_request.clone())));
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
        let mut screens_len = self.screens.len();
        let mut i = 0;
        while i < screens_len {
            self.screens[i].draw();
            if self.screens[i].is_expanded() {
                if i > 0 {
                    info!("[ScreenStack]Screen {} is expanded, dropping back screens...", i);
                }
                let new_screens = self.screens.split_off(i);
                self.screens = new_screens;
                screens_len = self.screens.len();
                i = 0;
            }
            else {
                i += 1;
            }
        }
    }

    pub fn pop_screen(&mut self) {
        log::info!("[ScreenStack] Popping top screen");
        self.screens.pop();
        if self.screens.len() == 0 {
            self.exit_request.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub fn push_screen(&mut self, screen: Box<dyn screen::ScreenTrait>) {
        log::info!("[ScreenStack] Pushing new screen");
        self.screens.push(screen);
    }
}