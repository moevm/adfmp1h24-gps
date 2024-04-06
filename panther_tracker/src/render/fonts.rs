use std::collections::BTreeMap;
use std::sync::OnceLock;
use ab_glyph::{Font, FontRef};
use image::math::Rect;
use log::error;
use crate::render::gl;
use crate::render::gl::Gles2;
use crate::render::gl::types::GLuint;

pub static QUEENSIDES_FONT: &[u8] = include_bytes!("../../resources/fonts/queensides.ttf");

#[derive(Clone)]
pub struct FontData {
    pub texture_id: GLuint,
    pub char_map: BTreeMap<char, Rect>
}

impl FontData {
    pub fn full_width(&self) -> u32 {
        (FONT_RASTER_SIZE * GRID_SIZE) as u32
    }

    pub fn single_width(&self) -> u32 {
        FONT_RASTER_SIZE as u32
    }

    pub fn full_height(&self) -> u32 {
        (FONT_RASTER_SIZE * GRID_SIZE) as u32
    }

    pub fn single_height(&self) -> u32 {
        FONT_RASTER_SIZE as u32
    }
}


pub struct FontLoader {
    fonts: BTreeMap<String, FontData>
}

const FONT_RASTER_SIZE: usize = 300;
const GRID_SIZE: usize = 9;

pub fn load_font(gl: &Gles2, font: &[u8]) -> FontData {
    let font = FontRef::try_from_slice(font).unwrap();

    let mut char_map = BTreeMap::new();

    // Load all characters into grid GRID_SIZE x GRID_SIZE
    let mut i = 0;
    let mut j = 0;
    let mut buf = vec![0u8; GRID_SIZE * GRID_SIZE * FONT_RASTER_SIZE * FONT_RASTER_SIZE];
    for c in ('A'..='Z').into_iter().chain('a'..='z').chain('0'..='9').chain([',', '.','!', '?', '*'].into_iter()) {
        let glyph = font.glyph_id(c)
            .with_scale_and_position(300.0f32,
                 ab_glyph::point(i as f32 * FONT_RASTER_SIZE as f32,
                                 j as f32 * FONT_RASTER_SIZE as f32));

        let outline_glyph = font.outline_glyph(glyph).unwrap();
        let bounds = outline_glyph.px_bounds();

        outline_glyph.draw(|x, y, v| {
            let x = x + i * FONT_RASTER_SIZE as u32;
            let y = y + j * FONT_RASTER_SIZE as u32;
            let idx = (y * GRID_SIZE as u32 * FONT_RASTER_SIZE as u32 + x) as usize;
            buf[idx] = (v * 255.0) as u8;
        });

        char_map.insert(c, Rect{
            x: i * FONT_RASTER_SIZE as u32,
            y: j * FONT_RASTER_SIZE as u32,
            width: bounds.width() as u32,
            height: bounds.height() as u32
        });

        j += 1;
        if j as usize == GRID_SIZE {
            j = 0;
            i += 1;
        }
        if i as usize >= GRID_SIZE {
            error!("Font grid too small, font loaded partially");
            break;
        }
    }

    let mut texture_id = 0;

    unsafe {
        gl.GenTextures(1, &mut texture_id);
        gl.BindTexture(gl::TEXTURE_2D, texture_id);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl.TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::R8 as i32,
            GRID_SIZE as i32 * FONT_RASTER_SIZE as i32,
            GRID_SIZE as i32 * FONT_RASTER_SIZE as i32,
            0,
            gl::RED,
            gl::UNSIGNED_BYTE,
            buf.as_ptr() as *const _,
        );
    }

    FontData {
        texture_id,
        char_map
    }
}

impl FontLoader {
    pub fn new(gl: &Gles2) -> Self {
        let mut fonts = BTreeMap::new();

        fonts.insert("queensides".to_string(),
                     load_font(&gl, QUEENSIDES_FONT));



        FontLoader {
            fonts
        }
    }

    pub fn get_font(&self, name: &str) -> Option<FontData> {
        self.fonts.get(name).cloned()
    }
}

static FONTS: OnceLock<FontLoader> = OnceLock::new();

pub fn load_fonts(gl: &Gles2) {
    FONTS.get_or_init(|| FontLoader::new(gl));
}
pub fn get_font(name: &str) -> Option<FontData> {
    FONTS.get().unwrap().get_font(name)
}