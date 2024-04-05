use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use ab_glyph::{Font, FontRef};
use crate::render::{check_gl_errors, gl, SURFACE_HEIGHT, SURFACE_WIDTH};
use crate::render::images::ImageData;
use crate::render::objects::r#box::Squad;
use crate::render::objects::textbox::TextBox;
use crate::render::screens::{ScreenManagementCmd, ScreenRendering, ScreenTrait};
use crate::render::screens::stats::StatsScreen;
use crate::render::utils::circle_animation::CircleAnimation;
use crate::render::utils::position::FixedPosition;


pub struct MainScreen {
    gl: Arc<gl::Gl>,
    bg_squad: Squad,
    screen_rendering: ScreenRendering,

    text: TextBox,

    exit_request: Arc<AtomicBool>,
    start: Instant,
}

impl MainScreen {
    pub fn new(gl: Arc<gl::Gl>, exit_request: Arc<AtomicBool>) -> Self {
        let squad = Squad::new_bg(gl.clone(), (0.4, 0.5, 0.9));

        let dims = (SURFACE_WIDTH.load(Ordering::Relaxed), SURFACE_HEIGHT.load(Ordering::Relaxed));

        let text = "PANTHER";
        let font = include_bytes!("../../../resources/fonts/queensides.ttf");
        let font = FontRef::try_from_slice(font).unwrap();

        let glyph = font.glyph_id('P').with_scale(400.0);
        let g = font.outline_glyph(glyph).unwrap();

        let bounds = g.px_bounds();
        let mut buf = vec![0u8; (bounds.height() * bounds.width()) as usize * 4];
        g.draw(|x, y, v| {
            buf[(y * bounds.width() as u32 + x) as usize * 4] = (150.0) as u8;
            buf[(y * bounds.width() as u32 + x) as usize * 4 + 1] = (35.0) as u8;
            buf[(y * bounds.width() as u32 + x) as usize * 4 + 2] = (255.0) as u8;
            buf[(y * bounds.width() as u32 + x) as usize * 4 + 3] = (v * 255.0) as u8;
        });


        let mut texture_id = 0;
        unsafe {
            gl.GenTextures(1, &mut texture_id);
            gl.BindTexture(gl::TEXTURE_2D, texture_id);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);


            gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                bounds.width() as i32,
                bounds.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                buf.as_ptr() as *const _,
            );
        }

        let text = TextBox::new(gl.clone(), ImageData {
            texture_id,
            width: bounds.width() as u32,
            height: bounds.height() as u32
        }, FixedPosition::new().width(0.2).left(0.1).bottom(0.9));

        let circ_anim = CircleAnimation::new(1.0, [(0.5, 0.5, 0.5), (-0.5, -0.2, 0.0), (0.0, 2.0, 3.0)]);
        let screen_rendering = ScreenRendering::new(gl.clone(), dims, circ_anim);

        MainScreen {
            gl,
            bg_squad: squad,
            exit_request,
            start: Instant::now(),
            screen_rendering,
            text
        }
    }
}

impl ScreenTrait for MainScreen {
    fn press(&mut self, pos: (f64, f64)) -> ScreenManagementCmd {
        ScreenManagementCmd::PushScreen(Box::new(StatsScreen::new(self.gl.clone(), self.exit_request.clone())))
    }
    fn back(&mut self) -> ScreenManagementCmd {
        self.exit_request.store(true, Ordering::Relaxed);
        ScreenManagementCmd::None
    }
    fn draw(&mut self) {
        let texture_id = self.screen_rendering.texture_id();
        self.screen_rendering.clear_texture();

        self.bg_squad.draw(texture_id);
        self.text.draw(texture_id);

        self.screen_rendering.present();
    }
    fn is_expanded(&self) -> bool {
        Instant::now().duration_since(self.start).as_secs_f32() > 0.5
    }
}