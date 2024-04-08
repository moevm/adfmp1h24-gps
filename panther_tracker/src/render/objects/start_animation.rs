use std::sync::{Arc, Mutex};
use crate::render::{create_shader, get_surface_y_ratio, gl};
use crate::render::gl::types::{GLint, GLsizei, GLsizeiptr, GLuint};
use crate::render::objects::{SQUAD_VERTEX_DATA, BoxProgram};
use crate::render::utils::position::FreePosition;

pub struct StartAnimation {
    gl: Arc<gl::Gl>,
    box_prog: BoxProgram,

    u_color_loc: GLint,
    color: (f32, f32, f32)
}

impl StartAnimation {
    pub fn new(gl: Arc<gl::Gl>, color: (f32, f32, f32), pos: FreePosition) -> Self {
        unsafe {
            let squad = BoxProgram::new(gl.clone(), pos.get(), include_bytes!("box-frag.glsl"));

            let u_color_loc = gl.GetUniformLocation(squad.program, b"color\0".as_ptr() as *const _);
            gl.Uniform3f(u_color_loc, color.0, color.1, color.2);

            Self {
                gl,
                box_prog: squad,

                u_color_loc,
                color
            }
        }
    }

    pub fn new_bg(gl: Arc<gl::Gl>, color: (f32, f32, f32)) -> Self {
        Self::new(gl, color, FreePosition::new().width(1.0).height(get_surface_y_ratio()))
    }

    pub fn set_color(&mut self, color: (f32, f32, f32)) {
        let gl = &self.gl;
        unsafe {
            gl.UseProgram(self.box_prog.program);
            gl.Uniform3f(self.u_color_loc, color.0, color.1, color.2);
        }
    }

    pub fn draw(&mut self, texture_id: GLuint) {
        self.box_prog.draw(texture_id, |_| {});
    }
}