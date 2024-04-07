use std::mem;
use std::sync::{Arc, Mutex};
use image::{DynamicImage, GenericImageView};
use log::info;
use crate::render::{create_shader, get_surface_y_ratio, gl};
use crate::render::gl::{BLEND, ONE_MINUS_SRC_ALPHA, SRC_ALPHA};
use crate::render::gl::types::{GLsizei, GLsizeiptr, GLuint};
use crate::render::images::ImageData;
use crate::render::objects::{BoxProgram, SQUAD_VERTEX_DATA};
use crate::render::utils::position::FixedPosition;


pub struct Image {
    gl: Arc<gl::Gl>,
    box_prog: BoxProgram,

    img_texture: GLuint,
}

impl Image {
    pub fn new(gl: Arc<gl::Gl>, img: ImageData, pos: FixedPosition) -> Self {
        unsafe {
            let aspect_ratio = img.height as f64 / img.width as f64;
            let bounds = pos.get(aspect_ratio);

            let box_prog = BoxProgram::new(gl.clone(), bounds, include_bytes!("image-frag.glsl"));

            let tex_location = gl.GetUniformLocation(box_prog.program, b"tex\0".as_ptr() as *const _);
            gl.Uniform1i(tex_location, 1);
            // info!("[img] pos: {:?}", bounds);

            Self {
                gl,
                img_texture: img.texture_id,
                box_prog
            }
        }
    }

    pub fn new_bg(gl: Arc<gl::Gl>, img: ImageData) -> Self {
        Self::new(gl, img, FixedPosition::new().width(1.0))
    }

    pub fn draw(&mut self, texture_id: GLuint) {
        self.box_prog.draw(texture_id, |gl| unsafe {
            gl.ActiveTexture(gl::TEXTURE1);
            gl.BindTexture(gl::TEXTURE_2D, self.img_texture);
        });
    }
}