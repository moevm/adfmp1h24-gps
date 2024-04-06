use std::mem;
use std::sync::{Arc, Mutex};
use ab_glyph::ScaleFont;
use image::{DynamicImage, GenericImageView};
use log::info;
use crate::render::{create_shader, get_surface_y_ratio, gl};
use crate::render::fonts::FontData;
use crate::render::gl::{BLEND, Gles2, ONE_MINUS_SRC_ALPHA, SRC_ALPHA};
use crate::render::gl::types::{GLsizei, GLsizeiptr, GLuint};

const VERTEX_SHADER_SOURCE: &[u8] = include_bytes!("textbox-vert.glsl");
const FRAGMENT_SHADER_SOURCE: &[u8] = include_bytes!("textbox-frag.glsl");

pub struct TextBox {
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    fbo: GLuint,
    gl: Arc<gl::Gl>,
    font_table: FontData,

    pos: (f32, f32),
    scale: f32,
    triangle_cnt: usize
}

fn build_vertex_buffer(gl: &Gles2, pos: &(f32, f32), scale: f32, vbo: GLuint, font_table: &FontData, string: String) -> usize {

    let mut temp_buf = vec![];

    let full_width = font_table.full_width();
    let full_height = font_table.full_height();

    let single_width = font_table.single_width();
    let single_height = font_table.single_height();

    let mut cursor_pos_x = pos.0;
    let mut cursor_pos_y = pos.1;
    for c in string.chars() {
        let rect = font_table.char_map.get(&c).unwrap();
        let x = pos.0 + rect.x as f32;
        let y = pos.1 + rect.y as f32;
        let w = single_width as f32;
        let h = single_height as f32;

        let x_ratio = x / full_width as f32;
        let y_ratio = y / full_height as f32;
        let w_ratio = w / full_width as f32;
        let h_ratio = h / full_height as f32;


        let cell_sz = scale * (w_ratio);

        let glyph_w = scale * (rect.width as f32 / full_width as f32);
        info!("current glyph width: {}", rect.width);
        // cursor_pos_y += pos_sz;

        temp_buf.extend_from_slice(&[
            cursor_pos_x + cell_sz, cursor_pos_y, x_ratio + w_ratio, y_ratio + h_ratio,
            cursor_pos_x + cell_sz, cursor_pos_y + cell_sz, x_ratio + w_ratio, y_ratio,
            cursor_pos_x, cursor_pos_y + cell_sz, x_ratio, y_ratio,

            cursor_pos_x + cell_sz, cursor_pos_y, x_ratio + w_ratio, y_ratio + h_ratio,
            cursor_pos_x, cursor_pos_y + cell_sz, x_ratio, y_ratio,
            cursor_pos_x, cursor_pos_y, x_ratio, y_ratio + h_ratio,
        ]);

        cursor_pos_x += glyph_w + 0.02;
        info!("new cursor x: {}", cursor_pos_x);
    }

    unsafe {
        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl.BufferData(
            gl::ARRAY_BUFFER,
            (temp_buf.len() * mem::size_of::<f32>()) as GLsizeiptr,
            temp_buf.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
    }

    temp_buf.len() / 4
}

impl TextBox {
    pub fn new(gl: Arc<gl::Gl>, font: FontData, string: String, pos: (f32, f32), scale: f32) -> Self {
        unsafe {
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
            let triangle_cnt = build_vertex_buffer(&gl, &pos, scale, vbo, &font, string);


            let ratio_location = gl.GetUniformLocation(program, b"y_ratio\0".as_ptr() as *const _);
            let ratio = get_surface_y_ratio();
            gl.Uniform1f(ratio_location, ratio as f32);

            let pos_attrib = gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            gl.VertexAttribPointer(
                pos_attrib as GLuint,
                2,
                gl::FLOAT,
                0,
                4 * mem::size_of::<f32>() as GLsizei,
                std::ptr::null(),
            );
            gl.EnableVertexAttribArray(pos_attrib as GLuint);

            let texcoord_attrib = gl.GetAttribLocation(program, b"texcoord\0".as_ptr() as *const _);
            gl.VertexAttribPointer(
                texcoord_attrib as GLuint,
                2,
                gl::FLOAT,
                0,
                4 * mem::size_of::<f32>() as GLsizei,
                (2 * mem::size_of::<f32>()) as *const _,
            );
            gl.EnableVertexAttribArray(texcoord_attrib as GLuint);

            let tex_location = gl.GetUniformLocation(program, b"tex\0".as_ptr() as *const _);
            gl.Uniform1i(tex_location, 1);

            Self {
                program,
                vao,
                vbo,
                gl,
                fbo,
                font_table: font,
                pos,
                scale,
                triangle_cnt
            }
        }
    }

    pub fn set_text(&mut self, string: String) {
        self.triangle_cnt = build_vertex_buffer(&self.gl, &self.pos, self.scale, self.vbo, &self.font_table, string);
    }

    pub fn draw(&mut self, texture_id: GLuint) {
        let gl = &self.gl;

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
            gl.BindTexture(gl::TEXTURE_2D, self.font_table.texture_id);

            // let params = self.circ_anim.cur();
            // gl.Uniform3f(self.circle, params.0, params.1, params.2);

            gl.DrawArrays(gl::TRIANGLES, 0, self.triangle_cnt as GLsizei);
        }
    }
}


impl Drop for TextBox {
    fn drop(&mut self) {
        let gl = &self.gl;
        unsafe {
            gl.DeleteProgram(self.program);
            gl.DeleteVertexArrays(1, &self.vao);
            gl.DeleteBuffers(1, &self.vbo);
            gl.DeleteFramebuffers(1, &self.fbo);
        }
    }
}
