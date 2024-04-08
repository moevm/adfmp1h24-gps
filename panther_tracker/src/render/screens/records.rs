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


pub struct RecordsScreen {
    gl: Arc<gl::Gl>,
    bg_squad: Squad,

    screen_rendering: ScreenRendering,

    logo: Image,

    bottom_home_text: TextBox,
    bottom_records_text: TextBox,
    bottom_stats_text: TextBox,

    home_icon: Image,
    records_icon: Image,
    stats_icon: Image,

    exit_request: Arc<AtomicBool>,
    start: Instant,
}

impl RecordsScreen {
    pub fn new(gl: Arc<gl::Gl>, exit_request: Arc<AtomicBool>) -> Self {
        let squad = Squad::new_bg(gl.clone(), (0.6, 0.8, 0.2));

        let dims = (SURFACE_WIDTH.load(Ordering::Relaxed), SURFACE_HEIGHT.load(Ordering::Relaxed));

        let circ_anim = CircleAnimation::new(1.0, [(0.5, 0.5, 0.5), (-0.5, -0.2, 0.0), (0.0, 2.0, 3.0)]);
        let screen_rendering = ScreenRendering::new(gl.clone(), dims, circ_anim);

        let font = get_font("queensides").unwrap();

        let logo = Image::new(gl.clone(), get_image("panther_logo").unwrap(),
                                                                         FixedPosition::new().bottom(1.75).width(0.25).left(0.65), Some((0.05, 0.06, 0.1)));

        let bottom_home_text = TextBox::new(gl.clone(), font.clone(), "Home".to_string(), (0.2, 0.068), 0.45, 1);
        let bottom_records_text = TextBox::new(gl.clone(), font.clone(), "Records".to_string(), (0.44, 0.068), 0.45, 1);
        let bottom_stats_text = TextBox::new(gl.clone(), font.clone(), "Stats".to_string(), (0.72, 0.068), 0.45, 1);

        let home_icon = Image::new(gl.clone(), get_image("home").unwrap(),
                                   FixedPosition::new().bottom(0.12).height(0.08).left(0.2), Some((0.4, 0.2, 0.6)));
        let records_icon = Image::new(gl.clone(), get_image("records").unwrap(),
                                      FixedPosition::new().bottom(0.12).height(0.08).left(0.45), Some((1.0, 0.9, 1.0)));
        let stats_icon = Image::new(gl.clone(), get_image("stats").unwrap(),
                                    FixedPosition::new().bottom(0.12).height(0.08).left(0.73), Some((0.4, 0.5, 0.9)));

        RecordsScreen {
            gl,
            bg_squad: squad,

            exit_request,
            start: Instant::now(),
            screen_rendering,

            logo,

            bottom_home_text,
            bottom_records_text,
            bottom_stats_text,

            home_icon,
            records_icon,
            stats_icon
        }
    }
}

impl ScreenTrait for RecordsScreen {
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

        self.logo.draw(texture_id);

        self.bottom_home_text.draw(texture_id);
        self.bottom_records_text.draw(texture_id);
        self.bottom_stats_text.draw(texture_id);

        self.home_icon.draw(texture_id);
        self.records_icon.draw(texture_id);
        self.stats_icon.draw(texture_id);

        self.screen_rendering.present();
    }
    fn scroll(&mut self, pos: (f64, f64)) {

    }
    fn is_expanded(&self) -> bool {
        Instant::now().duration_since(self.start).as_secs_f32() > 1.0
    }
}