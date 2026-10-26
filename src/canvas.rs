//! A 2D canvas: batched shapes in window pixels, drawn over whatever is
//! already on screen (HUDs, GUIs, editor panels).
//!
//! Queue shapes with [`Canvas2d::rect`], [`Canvas2d::rounded_rect`],
//! [`Canvas2d::shadow`] and [`Canvas2d::image`], optionally restricted to a
//! clip rectangle with [`Canvas2d::set_clip`], then [`Canvas2d::flush`] once
//! per frame. Order is preserved: later shapes paint over earlier ones.
//! Consecutive shapes sharing a texture and clip go out in one draw call.
//!
//! Rounded corners, borders and shadows are computed per pixel in the
//! fragment shader from a signed distance field, so their edges stay
//! anti-aliased at any size.
//!
//! Coordinates are pixels from the window's top-left corner, y pointing
//! down. Colors are straight (non-premultiplied) RGBA and alpha-blend.

use std::marker::PhantomData;

use crate::shader;
use crate::texture::{Filter, Texture};

/// Floats per vertex: position 2, UV 2, color 4, shape-local position 2,
/// shape half size 2, radius/border/softness 3, border color 4.
const VERTEX_FLOATS: usize = 19;

/// A rectangle as `[x, y, width, height]` in pixels.
pub type Rect = [f32; 4];

/// The rounded-rect part of a quad. `half[0] <= 0` turns shaping off.
#[derive(Clone, Copy)]
struct Shape {
    center: [f32; 2],
    half: [f32; 2],
    radius: f32,
    border: f32,
    softness: f32,
    border_color: [f32; 4],
}

const NO_SHAPE: Shape = Shape {
    center: [0.0; 2],
    half: [0.0; 2],
    radius: 0.0,
    border: 0.0,
    softness: 0.0,
    border_color: [0.0; 4],
};

#[derive(PartialEq)]
struct Batch {
    texture: gl::types::GLuint,
    clip: Option<[i32; 4]>,
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
    clip: Option<[i32; 4]>,
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
            let layout = [
                (0, 2, 0),
                (1, 2, 2),
                (2, 4, 4),
                (3, 2, 8),
                (4, 2, 10),
                (5, 3, 12),
                (6, 4, 15),
            ];
            for (location, size, offset) in layout {
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
                clip: None,
                vertices: Vec::new(),
                batches: Vec::new(),
                _gl_thread: PhantomData,
            }
        }
    }

    /// Restricts the shapes queued after this call to `clip` (pixels), or
    /// lifts the restriction with `None`.
    pub fn set_clip(&mut self, clip: Option<Rect>) {
        self.clip = clip.map(|[x, y, w, h]| {
            [
                x.floor() as i32,
                y.floor() as i32,
                w.ceil().max(0.0) as i32,
                h.ceil().max(0.0) as i32,
            ]
        });
    }

    /// Queues a solid rectangle with square corners.
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        let white = self.white.id();
        self.push_quad(white, [x, y, w, h], [0.0, 0.0, 1.0, 1.0], color, NO_SHAPE);
    }

    /// Queues a rectangle with corners rounded by `radius` pixels, filled
    /// with `fill` and outlined inside its edge by a `border`-pixel band of
    /// `border_color` (`border = 0.0` for none).
    pub fn rounded_rect(
        &mut self,
        rect: Rect,
        radius: f32,
        fill: [f32; 4],
        border: f32,
        border_color: [f32; 4],
    ) {
        let [x, y, w, h] = rect;
        let radius = radius.clamp(0.0, w.min(h) / 2.0);
        let shape = Shape {
            center: [x + w / 2.0, y + h / 2.0],
            half: [w / 2.0, h / 2.0],
            radius,
            border,
            softness: 0.0,
            border_color,
        };
        let white = self.white.id();
        self.push_quad(white, rect, [0.0, 0.0, 1.0, 1.0], fill, shape);
    }

    /// Queues a soft drop shadow for a `radius`-rounded `rect`, fading out
    /// over `blur` pixels around its edge. Queue it before the shape itself.
    pub fn shadow(&mut self, rect: Rect, radius: f32, blur: f32, color: [f32; 4]) {
        let [x, y, w, h] = rect;
        let blur = blur.max(1.0);
        let shape = Shape {
            center: [x + w / 2.0, y + h / 2.0],
            half: [w / 2.0, h / 2.0],
            radius: radius.clamp(0.0, w.min(h) / 2.0),
            border: 0.0,
            softness: blur,
            border_color: [0.0; 4],
        };
        let quad = [x - blur, y - blur, w + 2.0 * blur, h + 2.0 * blur];
        let white = self.white.id();
        self.push_quad(white, quad, [0.0, 0.0, 1.0, 1.0], color, shape);
    }

    /// Queues the `uv` region (`[u0, v0, u1, v1]`, 0..1, v down) of
    /// `texture`, stretched over `dst` and multiplied by `color`.
    pub fn image(&mut self, texture: &Texture, dst: Rect, uv: [f32; 4], color: [f32; 4]) {
        self.push_quad(texture.id(), dst, uv, color, NO_SHAPE);
    }

    fn push_quad(
        &mut self,
        texture: gl::types::GLuint,
        dst: Rect,
        uv: [f32; 4],
        c: [f32; 4],
        s: Shape,
    ) {
        let [x, y, w, h] = dst;
        let [u0, v0, u1, v1] = uv;
        let b = s.border_color;
        let corner = |px: f32, py: f32, u: f32, v: f32| {
            [
                px,
                py,
                u,
                v,
                c[0],
                c[1],
                c[2],
                c[3],
                px - s.center[0],
                py - s.center[1],
                s.half[0],
                s.half[1],
                s.radius,
                s.border,
                s.softness,
                b[0],
                b[1],
                b[2],
                b[3],
            ]
        };
        let (tl, tr) = (corner(x, y, u0, v0), corner(x + w, y, u1, v0));
        let (bl, br) = (corner(x, y + h, u0, v1), corner(x + w, y + h, u1, v1));
        for v in [tl, bl, br, tl, br, tr] {
            self.vertices.extend_from_slice(&v);
        }
        let clip = self.clip;
        match self.batches.last_mut() {
            Some(last) if last.texture == texture && last.clip == clip => last.vertex_count += 6,
            _ => self.batches.push(Batch {
                texture,
                clip,
                first_vertex: self.vertices.len() / VERTEX_FLOATS - 6,
                vertex_count: 6,
            }),
        }
    }

    /// Draws everything queued, in order, on a `width`×`height` pixel
    /// window, then empties the queue and lifts any clip. Leaves depth
    /// testing as it found it.
    ///
    /// # Safety
    /// Requires a current GL context.
    pub unsafe fn flush(&mut self, width: f32, height: f32) {
        self.clip = None;
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
                match batch.clip {
                    Some([x, y, w, h]) => {
                        gl::Enable(gl::SCISSOR_TEST);
                        // GL's scissor origin is the bottom-left corner.
                        gl::Scissor(x, height as i32 - (y + h), w, h);
                    }
                    None => gl::Disable(gl::SCISSOR_TEST),
                }
                gl::BindTexture(gl::TEXTURE_2D, batch.texture);
                gl::DrawArrays(
                    gl::TRIANGLES,
                    batch.first_vertex as i32,
                    batch.vertex_count as i32,
                );
            }
            gl::Disable(gl::SCISSOR_TEST);
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
