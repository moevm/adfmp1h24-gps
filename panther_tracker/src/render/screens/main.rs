use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use crate::render::gl;
use crate::render::gl::types::GLint;
use crate::render::objects::squad::Squad;
use crate::render::screens::{ScreenManagementCmd, ScreenTrait};
use crate::render::utils::circle_animation::CircleAnimation;


pub struct MainScreen {
    gl_mtx: Arc<Mutex<gl::Gl>>,
    squad_test: Squad,

    circle: GLint,
    circ_anim: CircleAnimation,

    exit_request: Arc<AtomicBool>,
    start: Instant,
}

impl MainScreen {
    pub fn new(gl_mtx: Arc<Mutex<gl::Gl>>, exit_request: Arc<AtomicBool>) -> Self {
        let squad = Squad::new(gl_mtx.clone());


        let circ_anim = CircleAnimation::new(1.0, [(0.5, 0.5, 0.5), (-0.5, -0.2, 0.0), (0.0, 2.0, 3.0)]);
        MainScreen {
            gl_mtx,
            squad_test: squad,
            circle: 0,
            circ_anim,
            exit_request,
            start: Instant::now(),
        }
    }
}

impl ScreenTrait for MainScreen {
    fn press(&mut self, pos: (f64, f64)) -> ScreenManagementCmd {
        // ScreenManagementCmd::PushScreen(Box::new(StatsScreen::new(self.gl_mtx.clone(), self.exit_request.clone())))
        ScreenManagementCmd::None
    }
    fn back(&mut self) -> ScreenManagementCmd {
        self.exit_request.store(true, Ordering::Relaxed);
        ScreenManagementCmd::None
    }
    fn draw(&mut self) {
        self.squad_test.draw();
    }
    fn is_expanded(&self) -> bool {
        Instant::now().duration_since(self.start).as_secs_f32() > 0.5
    }
}