use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use crate::render::{gl, SURFACE_HEIGHT, SURFACE_WIDTH};
use crate::render::fonts::get_font;
use crate::render::images::{get_gif, get_image, PANTHER_HD};
use crate::render::objects::animated_image::AnimatedImage;
use crate::render::objects::image::Image;
use crate::render::objects::r#box::Squad;
use crate::render::objects::textbox::TextBox;
use crate::render::screens::{ScreenManagementCmd, ScreenRendering, ScreenTrait};
use crate::render::screens::main::MainScreen;
use crate::render::screens::stats::StatsScreen;
use crate::render::utils::circle_animation::CircleAnimation;
use crate::render::utils::position::FixedPosition;


pub struct ActiveTrainingScreen {
    gl: Arc<gl::Gl>,
    bg_squad: Squad,

    screen_rendering: ScreenRendering,

    exit_request: Arc<AtomicBool>,
    start: Instant,

    test_text: TextBox,
}

impl ActiveTrainingScreen {
    pub fn new(gl: Arc<gl::Gl>, exit_request: Arc<AtomicBool>) -> Self {
        let squad = Squad::new_bg(gl.clone(), (0.6, 0.8, 0.2));

        let dims = (SURFACE_WIDTH.load(Ordering::Relaxed), SURFACE_HEIGHT.load(Ordering::Relaxed));

        let circ_anim = CircleAnimation::new(1.0, [(0.5, 0.5, 0.5), (-0.5, -0.2, 0.0), (0.0, 2.0, 3.0)]);
        let screen_rendering = ScreenRendering::new(gl.clone(), dims, circ_anim);

        let font = get_font("queensides").unwrap();
        let test_text = TextBox::new(gl.clone(), font, "doing some kind of\nrunning or idk".to_string(), (0.2, 0.9), 1.0, 1);

        ActiveTrainingScreen {
            gl,
            bg_squad: squad,

            exit_request,
            start: Instant::now(),
            screen_rendering,

            test_text,
        }
    }
}

impl ScreenTrait for ActiveTrainingScreen {
    fn press(&mut self, pos: (f64, f64)) -> ScreenManagementCmd {
        if pos.1 < 0.25 {
            match pos.0 {
                x if x < 0.33 => {
                    ScreenManagementCmd::PushScreen(Box::new(MainScreen::new(self.gl.clone(), self.exit_request.clone())))
                }
                x if x < 0.66 => {
                    // ScreenManagementCmd::PushScreen(Box::new(RecordsScreen::new(self.gl.clone(), self.exit_request.clone())))
                    ScreenManagementCmd::None
                }
                _ => {
                    ScreenManagementCmd::PushScreen(Box::new(StatsScreen::new(self.gl.clone(), self.exit_request.clone())))
                }

            }
        }
        else {
            ScreenManagementCmd::None
        }
    }
    fn back(&mut self) -> ScreenManagementCmd {
        // self.exit_request.store(true, Ordering::Relaxed);
        ScreenManagementCmd::PushScreen(Box::new(MainScreen::new(self.gl.clone(), self.exit_request.clone())))
    }
    fn draw(&mut self) {
        let texture_id = self.screen_rendering.texture_id();
        self.screen_rendering.clear_texture();

        self.bg_squad.draw(texture_id);

        self.test_text.draw(texture_id);

        self.screen_rendering.present();
    }
    fn scroll(&mut self, pos: (f64, f64)) {

    }
    fn is_expanded(&self) -> bool {
        Instant::now().duration_since(self.start).as_secs_f32() > 1.0
    }
}