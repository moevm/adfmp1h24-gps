use std::ffi::{CStr, CString};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use glutin::prelude::*;

use glutin::config::{Config, ConfigSurfaceTypes, ConfigTemplate, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentContext};
use glutin::display::{Display, DisplayApiPreference, GlDisplay};
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use log::info;
use raw_window_handle::{HasRawWindowHandle, RawDisplayHandle, RawWindowHandle};
use winit::dpi::PhysicalPosition;
use winit::event_loop::{ControlFlow, EventLoopWindowTarget};


#[rustfmt::skip]
static VERTEX_DATA: [f32; 15] = [
    -0.5, -0.5,  1.0,  0.0,  0.0,
    0.0,  0.5,  0.0,  1.0,  0.0,
    0.5, -0.5,  0.0,  0.0,  1.0,
];

const VERTEX_SHADER_SOURCE: &[u8] = include_bytes!("shader-vert.glsl");
const FRAGMENT_SHADER_SOURCE: &[u8] = include_bytes!("shader-frag.glsl");

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

fn get_gl_string(gl: &gl::Gl, variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl.GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

pub struct Renderer {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    gl: gl::Gl,
    vertex_data: Vec<f32>,
    start: Instant,
}

impl Renderer {

    pub fn new(gl_display: &Display) -> Self {
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

    pub fn draw(&mut self) {
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

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.program);
            self.gl.DeleteBuffers(1, &self.vbo);
            self.gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}

struct SurfaceState {
    window: winit::window::Window,
    surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

pub struct App {
    winsys_display: RawDisplayHandle,
    glutin_display: Option<Display>,
    surface_state: Option<SurfaceState>,
    context: Option<glutin::context::PossiblyCurrentContext>,
    render_state: Option<Renderer>,
    exit_request: Arc<std::sync::atomic::AtomicBool>,
}

impl App {
    pub fn new(winsys_display: RawDisplayHandle, exit_request: Arc<AtomicBool>) -> Self {
        Self {
            winsys_display,
            glutin_display: None,
            surface_state: None,
            context: None,
            render_state: None,
            exit_request
        }
    }
}

impl App {
    #[allow(unused_variables)]
    fn create_display(
        raw_display: RawDisplayHandle,
        raw_window_handle: RawWindowHandle,
    ) -> Display {

        let preference = DisplayApiPreference::Egl;

        // Create connection to underlying OpenGL client Api.
        unsafe { Display::new(raw_display, preference).unwrap() }
    }

    fn ensure_glutin_display(&mut self, window: &winit::window::Window) {
        if self.glutin_display.is_none() {
            let raw_window_handle = window.raw_window_handle();
            self.glutin_display =
                Some(Self::create_display(self.winsys_display, raw_window_handle));
        }
    }

    fn create_compatible_gl_context(
        glutin_display: &Display,
        raw_window_handle: RawWindowHandle,
        config: &Config,
    ) -> NotCurrentContext {
        let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));

        // Since glutin by default tries to create OpenGL core context, which may not be
        // present we should try gles.
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(Some(raw_window_handle));
        unsafe {
            glutin_display
                .create_context(&config, &context_attributes)
                .unwrap_or_else(|_| {
                    glutin_display
                        .create_context(config, &fallback_context_attributes)
                        .expect("failed to create context")
                })
        }
    }

    /// Create template to find OpenGL config.
    fn config_template(raw_window_handle: RawWindowHandle) -> ConfigTemplate {
        let builder = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .compatible_with_native_window(raw_window_handle)
            .with_surface_type(ConfigSurfaceTypes::WINDOW);


        builder.build()
    }

    fn ensure_surface_and_context<T>(&mut self, event_loop: &EventLoopWindowTarget<T>) {
        let window = winit::window::Window::new(&event_loop).unwrap();
        let raw_window_handle = window.raw_window_handle();

        // Lazily initialize, egl, wgl, glx etc
        self.ensure_glutin_display(&window);
        let glutin_display = self
            .glutin_display
            .as_ref()
            .expect("Can't ensure surface + context without a Glutin Display connection");

        let template = Self::config_template(raw_window_handle);
        let config = unsafe {
            glutin_display
                .find_configs(template)
                .unwrap()
                .reduce(|accum, config| {
                    // Find the config with the maximum number of samples.
                    //
                    // In general if you're not sure what you want in template you can request or
                    // don't want to require multisampling for example, you can search for a
                    // specific option you want afterwards.
                    //
                    // XXX however on macOS you can request only one config, so you should do
                    // a search with the help of `find_configs` and adjusting your template.
                    if config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        };
        println!("Picked a config with {} samples", config.num_samples());

        // XXX: Winit is missing a window.surface_size() API and the inner_size may be the wrong
        // size to use on some platforms!
        let (width, height): (u32, u32) = window.inner_size().into();
        let raw_window_handle = window.raw_window_handle();
        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );
        let surface = unsafe {
            glutin_display
                .create_window_surface(&config, &attrs)
                .unwrap()
        };
        let surface_state = SurfaceState { window, surface };

        let prev_ctx = self.context.take();
        match prev_ctx {
            Some(ctx) => {
                let not_current_context = ctx
                    .make_not_current()
                    .expect("Failed to make GL context not current");
                self.context = Some(
                    not_current_context
                        .make_current(&surface_state.surface)
                        .expect("Failed to make GL context current"),
                );
            }
            None => {
                let not_current_context =
                    Self::create_compatible_gl_context(glutin_display, raw_window_handle, &config);
                self.context = Some(
                    not_current_context
                        .make_current(&surface_state.surface)
                        .expect("Failed to make GL context current"),
                );
            }
        }

        self.surface_state = Some(surface_state);
    }

    fn ensure_renderer(&mut self) {
        let glutin_display = self
            .glutin_display
            .as_ref()
            .expect("Can't ensure renderer without a Glutin Display connection");
        self.render_state
            .get_or_insert_with(|| Renderer::new(glutin_display));
    }

    pub fn queue_redraw(&self) {
        if let Some(surface_state) = &self.surface_state {
            log::debug!("Making Redraw Request");
            surface_state.window.request_redraw();
        }
    }

    pub fn resume<T>(&mut self, event_loop: &EventLoopWindowTarget<T>) {
        log::debug!("Resumed, creating render state...");
        self.ensure_surface_and_context(event_loop);
        self.ensure_renderer();
        self.queue_redraw();
    }

    pub fn handle_redraw_request(&mut self) {
        if let Some(ref surface_state) = self.surface_state {
            if let Some(ctx) = &self.context {
                if let Some(ref mut renderer) = self.render_state {
                    renderer.draw();
                    if let Err(err) = surface_state.surface.swap_buffers(ctx) {
                        log::error!("Failed to swap buffers after render: {}", err);
                    }
                }
                self.queue_redraw();
            }
        }
    }

    pub fn handle_suspend(&mut self) {
        self.surface_state = None;
    }

    /// can potentially call exit
    pub fn handle_back_button(&mut self) {
        log::debug!("Back button pressed, exiting...");
        self.exit_request.store(true, Ordering::Relaxed);
    }

    pub fn handle_close_request(&mut self) {
        self.exit_request.store(true, Ordering::Relaxed);
    }

    pub fn handle_touch(&mut self, id: u64, location: PhysicalPosition<f64>, phase: winit::event::TouchPhase) {
        log::debug!("Touch event: id: {}, location: {:?}, phase: {:?}", id, location, phase);
    }
}