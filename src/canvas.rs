//! A 2D canvas: batched colored and textured rectangles in window pixels,
//! drawn over whatever is already on screen (HUDs, GUIs, editor panels).
//!
//! Queue shapes with [`Canvas2d::rect`] / [`Canvas2d::image`], then
//! [`Canvas2d::flush`] once per frame. Consecutive shapes using the same
//! texture are drawn in one call; order is preserved, so later shapes paint
//! over earlier ones.
//!
//! Coordinates are pixels from the window's top-left corner, y pointing
//! down. Colors are straight (non-premultiplied) RGBA and alpha-blend.

use std::marker::PhantomData;

use crate::shader;
use crate::texture::{Filter, Texture};

/// Floats per vertex: position (2), UV (2), color (4).
const VERTEX_FLOATS: usize = 8;

struct Batch {
    texture: gl::types::GLuint,
    first_vertex: usize,
    vertex_count: usize,
}

/// See the [module docs](self). `!Send`: it must stay on the GL thread.
pub struct Canvas2d {
    program: gl::types::GLuint,
    screen_loc: gl::types::GLint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    white: Texture,
    vertices: Vec<f32>,
    batches: Vec<Batch>,
    _gl_thread: PhantomData<*const ()>,
}

impl Canvas2d {
    /// Compiles the canvas shader and allocates its buffers.
    ///
    /// # Safety
    /// Requires a current GL context.
    pub unsafe fn new() -> Self {
        unsafe {
            let program = shader::link_program(
                include_str!("../assets/shaders/canvas.vert"),
                include_str!("../assets/shaders/canvas.frag"),
            )
            .expect("canvas shader failed to compile/link");

            let (mut vao, mut vbo) = (0, 0);
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            let stride = (VERTEX_FLOATS * size_of::<f32>()) as i32;
            for (location, size, offset) in [(0, 2, 0), (1, 2, 2), (2, 4, 4)] {
                gl::VertexAttribPointer(
                    location,
                    size,
                    gl::FLOAT,
                    gl::FALSE,
                    stride,
                    (offset * size_of::<f32>()) as *const _,
                );
                gl::EnableVertexAttribArray(location);
            }
            gl::BindVertexArray(0);

            Self {
                program,
                screen_loc: shader::uniform_location(program, "screen"),
                vao,
                vbo,
                white: Texture::from_rgba8(1, 1, &[255; 4], Filter::Nearest),
                vertices: Vec::new(),
                batches: Vec::new(),
                _gl_thread: PhantomData,
            }
        }
    }

    /// Queues a solid rectangle.
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        let white = self.white.id();
        self.push_quad(white, [x, y, w, h], [0.0, 0.0, 1.0, 1.0], color);
    }

    /// Queues the `uv` region (`[u0, v0, u1, v1]`, 0..1, v down) of
    /// `texture`, stretched over `dst` (`[x, y, w, h]`) and multiplied by
    /// `color`.
    pub fn image(&mut self, texture: &Texture, dst: [f32; 4], uv: [f32; 4], color: [f32; 4]) {
        self.push_quad(texture.id(), dst, uv, color);
    }

    fn push_quad(&mut self, texture: gl::types::GLuint, dst: [f32; 4], uv: [f32; 4], c: [f32; 4]) {
        let [x, y, w, h] = dst;
        let [u0, v0, u1, v1] = uv;
        let corner = |x: f32, y: f32, u: f32, v: f32| [x, y, u, v, c[0], c[1], c[2], c[3]];
        let (tl, tr) = (corner(x, y, u0, v0), corner(x + w, y, u1, v0));
        let (bl, br) = (corner(x, y + h, u0, v1), corner(x + w, y + h, u1, v1));
        for v in [tl, bl, br, tl, br, tr] {
            self.vertices.extend_from_slice(&v);
        }
        match self.batches.last_mut() {
            Some(last) if last.texture == texture => last.vertex_count += 6,
            _ => self.batches.push(Batch {
                texture,
                first_vertex: self.vertices.len() / VERTEX_FLOATS - 6,
                vertex_count: 6,
            }),
        }
    }

    /// Draws everything queued, in order, on a `width`×`height` pixel
    /// window, then empties the queue. Leaves depth testing as it found it.
    ///
    /// # Safety
    /// Requires a current GL context.
    pub unsafe fn flush(&mut self, width: f32, height: f32) {
        if self.batches.is_empty() {
            return;
        }
        unsafe {
            let depth_was_on = gl::IsEnabled(gl::DEPTH_TEST) == gl::TRUE;
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::UseProgram(self.program);
            gl::Uniform2f(self.screen_loc, width, height);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * size_of::<f32>()) as isize,
                self.vertices.as_ptr().cast(),
                gl::STREAM_DRAW,
            );
            for batch in &self.batches {
                gl::BindTexture(gl::TEXTURE_2D, batch.texture);
                gl::DrawArrays(
                    gl::TRIANGLES,
                    batch.first_vertex as i32,
                    batch.vertex_count as i32,
                );
            }
            gl::BindVertexArray(0);
            if depth_was_on {
                gl::Enable(gl::DEPTH_TEST);
            }
        }
        self.vertices.clear();
        self.batches.clear();
    }
}

impl Drop for Canvas2d {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteProgram(self.program);
        }
    }
}
