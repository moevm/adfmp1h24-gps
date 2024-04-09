use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use crate::render::{gl, SURFACE_HEIGHT, SURFACE_WIDTH};
use crate::render::fonts::get_font;
use crate::render::images::{get_gif, get_image};
use crate::render::objects::animated_image::AnimatedImage;
use crate::render::objects::image::Image;
use crate::render::objects::r#box::Squad;
use crate::render::objects::tab::Tab;
use crate::render::objects::textbox::TextBox;
use crate::render::screens::{ScreenManagementCmd, ScreenRendering, ScreenTrait};


use crate::render::utils::circle_animation::CircleAnimation;
use crate::render::utils::position::{FixedPosition, FreePosition};


pub struct ActiveTrainingScreen {
    gl: Arc<gl::Gl>,
    bg_squad: Squad,

    screen_rendering: ScreenRendering,

    play: Image,
    walking_gif: AnimatedImage,

    exit_request: Arc<AtomicBool>,
    start: Instant,

    tab1: Tab,
    tab2: Tab,
    tab3: Tab,

    tab_label_1: TextBox,
    tab_label_2: TextBox,
    tab_label_3: TextBox,

    //total
    total_time_val: TextBox,
    total_time_units: TextBox,

    total_dist_val: TextBox,
    total_dist_units: TextBox,


    is_pause: bool
}

impl ActiveTrainingScreen {
    pub fn new(gl: Arc<gl::Gl>, exit_request: Arc<AtomicBool>) -> Self {
        let squad = Squad::new_bg(gl.clone(), (0.4, 0.3, 0.5));

        let dims = (SURFACE_WIDTH.load(Ordering::Relaxed), SURFACE_HEIGHT.load(Ordering::Relaxed));

        let circ_anim = CircleAnimation::new(1.0, [(0.5, 0.5, 0.5), (-0.5, -0.2, 0.0), (0.0, 2.0, 3.0)]);
        let screen_rendering = ScreenRendering::new(gl.clone(), dims, circ_anim);

        let sparky_stones = get_font("sparky-stones").unwrap();
        let queensides = get_font("queensides").unwrap();

        let mut pos = FreePosition::new().bottom(-0.5).left(0.0).width(1.0)
            .height(1.8);
        let tab1 = Tab::new(gl.clone(), (0.05, 0.2, 0.3), pos, 0.2);

        pos = pos.height(1.45);
        let tab2 = Tab::new(gl.clone(), (0.15, 0.1, 0.3), pos, 0.4);

        pos = pos.height(1.1);
        let tab3 = Tab::new(gl.clone(), (0.3, 0.05, 0.3), pos, 0.6);

        let tab_label_1 = TextBox::new(gl.clone(), sparky_stones.clone(), "total".to_string(), (0.25, 1.21), 0.5, 0);
        let tab_label_2 = TextBox::new(gl.clone(), sparky_stones.clone(), "cur".to_string(), (0.47, 0.86), 0.5, 0);
        let tab_label_3 = TextBox::new(gl.clone(), sparky_stones.clone(), "avg".to_string(), (0.67, 0.51), 0.5, 0);

        let play = Image::new(gl.clone(), get_image("play").unwrap(),
                              FixedPosition::new().bottom(1.7).width(0.25).left(0.15), Some((0.1, 0.9, 0.3)));
        let walking_gif = AnimatedImage::new(gl.clone(), get_gif("walking").unwrap(),
                                             FixedPosition::new().bottom(1.7).width(0.55).left(0.45), 0.08);

        let total_time_val = TextBox::new(gl.clone(), queensides.clone(), "00:00".to_string(), (0.1, 1.05), 1.0, 0);
        let total_time_units = TextBox::new(gl.clone(), queensides.clone(), "min:sec".to_string(), (0.1, 0.95), 1.0, 0);

        let total_dist_val = TextBox::new(gl.clone(), queensides.clone(), "0.0".to_string(), (0.75, 1.05), 1.0, 0);
        let total_dist_units = TextBox::new(gl.clone(), queensides.clone(), "km".to_string(), (0.76, 0.95), 1.0, 0);

        ActiveTrainingScreen {
            gl,
            bg_squad: squad,

            exit_request,
            start: Instant::now(),
            screen_rendering,

            tab1,
            tab2,
            tab3,

            tab_label_1,
            tab_label_2,
            tab_label_3,

            play,
            walking_gif,

            total_time_val,
            total_time_units,
            total_dist_val,
            total_dist_units,

            is_pause: false
        }
    }

    fn paused(&mut self) {
        self.is_pause = !self.is_pause;
    }
}

impl ScreenTrait for ActiveTrainingScreen {
    fn press(&mut self, _pos: (f64, f64)) -> ScreenManagementCmd {
        self.paused();
        ScreenManagementCmd::None
    }
    fn back(&mut self) -> ScreenManagementCmd {
        self.paused();
        ScreenManagementCmd::None
    }
    fn draw(&mut self) {
        let texture_id = self.screen_rendering.texture_id();
        self.screen_rendering.clear_texture();

        self.bg_squad.draw(texture_id);

        self.play.draw(texture_id);
        self.walking_gif.draw(texture_id);

        self.tab1.draw(texture_id);
        self.tab2.draw(texture_id);
        self.tab3.draw(texture_id);

        self.tab_label_1.draw(texture_id);
        self.tab_label_2.draw(texture_id);
        self.tab_label_3.draw(texture_id);

        self.total_time_val.draw(texture_id);
        self.total_time_units.draw(texture_id);
        self.total_dist_val.draw(texture_id);
        self.total_dist_units.draw(texture_id);

        self.screen_rendering.present();
    }
    fn scroll(&mut self, _pos: (f64, f64)) {

    }
    fn is_expanded(&self) -> bool {
        Instant::now().duration_since(self.start).as_secs_f32() > 1.0
    }
}