use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::render::{create_shader, get_surface_y_ratio, gl};
use crate::render::gl::types::{GLint, GLsizei, GLsizeiptr, GLuint};
use crate::render::objects::{SQUAD_VERTEX_DATA, BoxProgram};
use crate::render::utils::position::FreePosition;

pub struct StartAnimation {
    gl: Arc<gl::Gl>,
    box_prog: BoxProgram,
    start: Instant,

    t_loc: GLint
}

impl StartAnimation {
    pub fn new(gl: Arc<gl::Gl>, pos: FreePosition) -> Self {
        unsafe {
            let squad = BoxProgram::new(gl.clone(), pos.get(), include_bytes!("start-animation-frag.glsl"));

            let t_loc = gl.GetUniformLocation(squad.program, b"t\0".as_ptr() as *const _);
            gl.Uniform1f(t_loc, 0.0);

            Self {
                gl,
                box_prog: squad,
                start: Instant::now(),

                t_loc
            }
        }
    }

    pub fn new_bg(gl: Arc<gl::Gl>) -> Self {
        Self::new(gl, FreePosition::new().width(1.0).height(get_surface_y_ratio()))
    }


    pub fn draw(&mut self, texture_id: GLuint) {
        let t = self.start.elapsed().as_secs_f32();

        self.box_prog.draw(texture_id, |gl| unsafe {
            gl.UseProgram(self.box_prog.program);
            gl.Uniform1f(self.t_loc, t);
        });
    }
}