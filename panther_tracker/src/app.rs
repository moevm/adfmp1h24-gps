use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use glutin::prelude::*;

use glutin::config::{Config, ConfigSurfaceTypes, ConfigTemplate, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentContext};
use glutin::display::{Display, DisplayApiPreference, GlDisplay};
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use raw_window_handle::{HasRawWindowHandle, RawDisplayHandle, RawWindowHandle};
use winit::dpi::PhysicalPosition;
use winit::event_loop::EventLoopWindowTarget;
use crate::render::AppState;


struct SurfaceState {
    window: winit::window::Window,
    surface: glutin::surface::Surface<WindowSurface>,
}

pub enum TouchState {
    //start, press_time, send_move
    MovingStart(PhysicalPosition<f64>, Instant, bool), // moving less than 50ms
    Moving(PhysicalPosition<f64>, Instant, bool), // moving more than 50ms
}

pub struct App {
    winsys_display: RawDisplayHandle,
    glutin_display: Option<Display>,
    surface_state: Option<SurfaceState>,
    context: Option<glutin::context::PossiblyCurrentContext>,
    exit_request: Arc<AtomicBool>,

    app_state: AppState,

    touch_state: BTreeMap<u64, TouchState>,
}

impl App {
    pub fn new(winsys_display: RawDisplayHandle, exit_request: Arc<AtomicBool>) -> Self {
        Self {
            winsys_display,
            glutin_display: None,
            surface_state: None,
            context: None,
            app_state: AppState::new(exit_request.clone()),
            exit_request,
            touch_state: BTreeMap::new(),
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
            .expect("Can't ensure render without a Glutin Display connection");

        self.app_state.ensure_renderer(glutin_display);
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
                if self.app_state.renderer_ready() {
                    self.app_state.draw();
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
        if let Some(screen) = self.app_state.get_input_screen() {
            screen.back();
        }
    }

    pub fn handle_close_request(&mut self) {
        self.exit_request.store(true, Ordering::Relaxed);
    }

    pub fn handle_touch(&mut self, id: u64, location: PhysicalPosition<f64>, phase: winit::event::TouchPhase) {
        if let Some(screen) = self.app_state.get_input_screen() {
            match phase {
                winit::event::TouchPhase::Started => {
                    let should_send_move = screen.start_scroll((location.x, location.y));
                    self.touch_state.insert(id, TouchState::MovingStart(location, Instant::now(), should_send_move));
                }
                winit::event::TouchPhase::Moved => {
                    if let Some(touch_state) = self.touch_state.get_mut(&id) {
                        match *touch_state {
                            TouchState::MovingStart(prev_pos, start_time, should_send_move) => {
                                //trigger to switch to moving state
                                if should_send_move {
                                    let diff = (location.x - prev_pos.x, location.y - prev_pos.y);
                                    screen.scroll(diff);
                                }
                                if Instant::now().duration_since(start_time).as_millis() > 50 {
                                    *touch_state = TouchState::Moving(location, Instant::now(), should_send_move);
                                }
                                else {
                                    //just update location
                                    *touch_state = TouchState::MovingStart(location, start_time, should_send_move);
                                }
                            }
                            TouchState::Moving(prev_pos, _, should_send_move) => {
                                let diff = (location.x - prev_pos.x, location.y - prev_pos.y);
                                if should_send_move {
                                    screen.scroll(diff);
                                }
                                //just update location
                                *touch_state = TouchState::Moving(location, Instant::now(), should_send_move);
                            }
                        }
                    }
                }
                winit::event::TouchPhase::Ended => {
                    if let Some(touch_state) = self.touch_state.remove(&id) {
                        match touch_state {
                            TouchState::MovingStart(_, _, _) => {
                                screen.press((location.x, location.y));
                            }
                            _ => {}
                        }
                    }
                }
                winit::event::TouchPhase::Cancelled => {
                    self.touch_state.remove(&id); // just cancel
                }
            }
        }
        else {
            log::warn!("Touch event, but no screen to send it to");
        }
    }
}