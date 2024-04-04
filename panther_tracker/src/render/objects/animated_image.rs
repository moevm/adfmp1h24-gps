use std::mem;
use std::sync::{Arc, Mutex};
use image::{DynamicImage, GenericImageView};
use log::info;
use crate::render::{create_shader, get_surface_y_ratio, gl};
use crate::render::gl::{BLEND, ONE_MINUS_SRC_ALPHA, SRC_ALPHA};
use crate::render::gl::types::{GLint, GLsizei, GLsizeiptr, GLuint};
use crate::render::images::ImageData;
use crate::render::objects::SQUAD_VERTEX_DATA;
use crate::render::utils::position::FixedPosition;

const VERTEX_SHADER_SOURCE: &[u8] = include_bytes!("animated-image-vert.glsl");
const FRAGMENT_SHADER_SOURCE: &[u8] = include_bytes!("animated-image-frag.glsl");

pub struct AnimatedImage {
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    fbo: GLuint,
    gl_mtx: Arc<Mutex<gl::Gl>>,

    img_textures: Vec<GLuint>,
    pub img_count: usize,

    u_texture_loc: GLint,
    dims: (u32, u32)
}

impl AnimatedImage {
    pub fn new(gl_mtx: Arc<Mutex<gl::Gl>>, imgs: Vec<ImageData>, pos: FixedPosition) -> Self {
        unsafe {
            let gl = gl_mtx.lock().unwrap();

            let vertex_shader = create_shader(&gl, gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE);
            let fragment_shader = create_shader(&gl, gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);

            let program = gl.CreateProgram();

            gl.AttachShader(program, vertex_shader);
            gl.AttachShader(program, fragment_shader);

            gl.LinkProgram(program);

            gl.UseProgram(program);

            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            gl.Enable(BLEND);
            gl.BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);

            let mut fbo = 0;
            gl.GenFramebuffers(1, &mut fbo);

            let mut vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (SQUAD_VERTEX_DATA.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                SQUAD_VERTEX_DATA.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );


            let ratio_location = gl.GetUniformLocation(program, b"y_ratio\0".as_ptr() as *const _);
            let ratio = get_surface_y_ratio();
            gl.Uniform1f(ratio_location, ratio as f32);

            let pos_attrib = gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            gl.VertexAttribPointer(
                pos_attrib as GLuint,
                2,
                gl::FLOAT,
                0,
                2 * mem::size_of::<f32>() as GLsizei,
                std::ptr::null(),
            );
            gl.EnableVertexAttribArray(pos_attrib as GLuint);

            let bounds_location = gl.GetUniformLocation(program, b"bounds\0".as_ptr() as *const _);

            let u_texture_loc = gl.GetUniformLocation(program, b"tex\0".as_ptr() as *const _);
            gl.Uniform1i(u_texture_loc, 1);

            let dims = (imgs[0].width, imgs[1].height);
            let aspect_ratio = imgs[0].height as f64 / imgs[0].width as f64;
            let pos = pos.get(aspect_ratio);
            info!("[img] pos: {:?}", pos);
            gl.Uniform4f(bounds_location, pos.0 as f32, pos.1 as f32, pos.2 as f32, pos.3 as f32);

            mem::drop(gl);

            let img_textures: Vec<_> = imgs.into_iter().map(|i| i.texture_id).collect();
            let img_count = img_textures.len();
            Self {
                program,
                vao,
                vbo,
                gl_mtx,
                fbo,
                img_textures,
                u_texture_loc,
                dims,
                img_count,
            }
        }
    }

    pub fn new_bg(gl_mtx: Arc<Mutex<gl::Gl>>, imgs: Vec<ImageData>) -> Self {
        Self::new(gl_mtx, imgs, FixedPosition::new().width(1.0))
    }

    pub fn draw(&mut self, texture_id: GLuint, frame: usize) {

        let gl = self.gl_mtx.lock().unwrap();


        // Check if the framebuffer is complete
        // let status = unsafe { gl.CheckFramebufferStatus(gl::FRAMEBUFFER) };
        // if status != gl::FRAMEBUFFER_COMPLETE {
        //     panic!("Failed to create framebuffer");
        // }

        unsafe {
            gl.UseProgram(self.program);

            gl.BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
            gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture_id, 0);

            gl.BindVertexArray(self.vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl.ActiveTexture(gl::TEXTURE1);
            gl.BindTexture(gl::TEXTURE_2D, self.img_textures[frame]);

            // let params = self.circ_anim.cur();
            // gl.Uniform3f(self.circle, params.0, params.1, params.2);

            gl.DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }
}


impl Drop for AnimatedImage {
    fn drop(&mut self) {
        let gl = self.gl_mtx.lock().unwrap();
        unsafe {
            gl.DeleteProgram(self.program);
            gl.DeleteVertexArrays(1, &self.vao);
            gl.DeleteBuffers(1, &self.vbo);
            gl.DeleteFramebuffers(1, &self.fbo);
        }
    }
}
