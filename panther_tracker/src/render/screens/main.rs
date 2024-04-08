use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use crate::render::{check_gl_errors, gl, SURFACE_HEIGHT, SURFACE_WIDTH};
use crate::render::fonts::get_font;
use crate::render::objects::r#box::Squad;
use crate::render::objects::start_animation::StartAnimation;
use crate::render::objects::textbox::TextBox;
use crate::render::screens::{ScreenManagementCmd, ScreenRendering, ScreenTrait};
use crate::render::screens::records::RecordsScreen;
use crate::render::screens::stats::StatsScreen;
use crate::render::utils::circle_animation::CircleAnimation;
use crate::render::utils::position::FreePosition;


pub struct MainScreen {
    gl: Arc<gl::Gl>,
    bg_squad: Squad,
    screen_rendering: ScreenRendering,

    panther_text: TextBox,

    start_animation: StartAnimation,
    start_text: TextBox,

    bottom_home_text: TextBox,
    bottom_records_text: TextBox,
    bottom_stats_text: TextBox,

    exit_request: Arc<AtomicBool>,
    start: Instant,
}

impl MainScreen {
    pub fn new(gl: Arc<gl::Gl>, exit_request: Arc<AtomicBool>) -> Self {
        let squad = Squad::new_bg(gl.clone(), (0.05, 0.06, 0.1));

        let font = get_font("queensides").unwrap();
        let panther_text = TextBox::new(gl.clone(), font.clone(), "Panther\ntracker".to_string(), (0.1, 1.9), 2.0, 0);

        let start_text = TextBox::new(gl.clone(), font.clone(), "Start".to_string(), (0.32, 1.1), 2.2, 0);
        let start_animation = StartAnimation::new(gl.clone(),
                                                  FreePosition::new().left(0.1).width(0.8).bottom(0.7).height(0.8));

        let bottom_home_text = TextBox::new(gl.clone(), font.clone(), "Home".to_string(), (0.1, 0.1), 0.4, 1);
        let bottom_records_text = TextBox::new(gl.clone(), font.clone(), "Records".to_string(), (0.44, 0.1), 0.4, 1);
        let bottom_stats_text = TextBox::new(gl.clone(), font.clone(), "Stats".to_string(), (0.82, 0.1), 0.4, 1);

        let circ_anim = CircleAnimation::new(1.0, [(0.5, 0.5, 0.5), (-0.5, -0.2, 0.0), (0.0, 2.0, 3.0)]);

        let dims = (SURFACE_WIDTH.load(Ordering::Relaxed), SURFACE_HEIGHT.load(Ordering::Relaxed));
        let screen_rendering = ScreenRendering::new(gl.clone(), dims, circ_anim);

        MainScreen {
            gl,
            bg_squad: squad,
            exit_request,
            start: Instant::now(),
            screen_rendering,
            panther_text,

            start_text,
            start_animation,

            bottom_home_text,
            bottom_records_text,
            bottom_stats_text,
        }
    }

    fn start_pressed(&mut self) {
        self.start_animation.launch();
    }
}

impl ScreenTrait for MainScreen {
    fn press(&mut self, pos: (f64, f64)) -> ScreenManagementCmd {
        if pos.1 < 0.25 {
            match pos.0 {
                x if x < 0.33 => {
                    // ScreenManagementCmd::PushScreen(Box::new(HomeScreen::new(self.gl.clone(), self.exit_request.clone())))
                    ScreenManagementCmd::None
                }
                x if x < 0.66 => {
                    ScreenManagementCmd::PushScreen(Box::new(RecordsScreen::new(self.gl.clone(), self.exit_request.clone())))
                }
                _ => {
                    ScreenManagementCmd::PushScreen(Box::new(StatsScreen::new(self.gl.clone(), self.exit_request.clone())))
                }

            }
        }
        else if pos.0 > 0.3 && pos.0 < 0.7 && pos.1 > 1.1 && pos.1 < 1.4 {
            self.start_pressed();
            ScreenManagementCmd::None
        }
        else {
            ScreenManagementCmd::None
        }
    }

    fn back(&mut self) -> ScreenManagementCmd {
        self.exit_request.store(true, Ordering::Relaxed);
        ScreenManagementCmd::None
    }
    fn draw(&mut self) {
        let texture_id = self.screen_rendering.texture_id();
        self.screen_rendering.clear_texture();

        self.bg_squad.draw(texture_id);
        self.panther_text.draw(texture_id);

        self.start_text.draw(texture_id);
        self.start_animation.draw(texture_id);

        self.bottom_home_text.draw(texture_id);
        self.bottom_records_text.draw(texture_id);
        self.bottom_stats_text.draw(texture_id);

        self.screen_rendering.present();
    }
    fn is_expanded(&self) -> bool {
        Instant::now().duration_since(self.start).as_secs_f32() > 1.0
    }
}