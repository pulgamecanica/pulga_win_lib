//! A ready-made 3D pipeline: meshes shaded in one flat color under a fixed
//! directional light (`assets/shaders/flat.{vert,frag}`), plus frame
//! clearing.
//!
//! ```no_run
//! # use pulga_win_lib::{Camera, gl_mesh::GlMesh, primitives, render::{self, MeshRenderer}, transform};
//! # unsafe {
//! let renderer = MeshRenderer::new();
//! let cube = GlMesh::upload(&primitives::cube(1.0));
//! let camera = Camera::new(1.0, 16.0 / 9.0, 0.1, 100.0);
//! // every frame:
//! render::clear([0.5, 0.7, 0.9, 1.0]);
//! renderer.begin(&camera.proj(), &camera.view());
//! renderer.draw(&cube, &transform::translate([0.0, 0.0, -3.0]), [1.0, 0.5, 0.2, 1.0]);
//! # }
//! ```

use std::marker::PhantomData;

use crate::gl_mesh::GlMesh;
use crate::shader;
use crate::transform::{self, Mat4};

/// Clears color and depth to start a frame.
///
/// # Safety
/// Requires a current GL context.
pub unsafe fn clear(color: [f32; 4]) {
    unsafe {
        gl::ClearColor(color[0], color[1], color[2], color[3]);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}

/// See the [module docs](self). `!Send`: it must stay on the GL thread.
pub struct MeshRenderer {
    program: gl::types::GLuint,
    proj_loc: gl::types::GLint,
    view_loc: gl::types::GLint,
    model_loc: gl::types::GLint,
    color_loc: gl::types::GLint,
    _gl_thread: PhantomData<*const ()>,
}

impl MeshRenderer {
    /// Compiles the flat shader.
    ///
    /// # Safety
    /// Requires a current GL context.
    pub unsafe fn new() -> Self {
        unsafe {
            let program = shader::link_program(
                include_str!("../assets/shaders/flat.vert"),
                include_str!("../assets/shaders/flat.frag"),
            )
            .expect("flat shader failed to compile/link");
            Self {
                program,
                proj_loc: shader::uniform_location(program, "proj"),
                view_loc: shader::uniform_location(program, "view"),
                model_loc: shader::uniform_location(program, "model"),
                color_loc: shader::uniform_location(program, "color"),
                _gl_thread: PhantomData,
            }
        }
    }

    /// Binds the shader with this camera, and turns on depth testing and
    /// alpha blending. Call before a run of [`MeshRenderer::draw`]s.
    ///
    /// # Safety
    /// Requires a current GL context.
    pub unsafe fn begin(&self, proj: &Mat4, view: &Mat4) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::UseProgram(self.program);
            gl::UniformMatrix4fv(self.proj_loc, 1, gl::TRUE, transform::to_gl(proj).as_ptr());
            gl::UniformMatrix4fv(self.view_loc, 1, gl::TRUE, transform::to_gl(view).as_ptr());
        }
    }

    /// Draws `mesh` placed by `model`, in RGBA `color`.
    ///
    /// # Safety
    /// Requires a current GL context and a preceding [`MeshRenderer::begin`]
    /// with no other program bound since.
    pub unsafe fn draw(&self, mesh: &GlMesh, model: &Mat4, color: [f32; 4]) {
        unsafe {
            gl::UniformMatrix4fv(
                self.model_loc,
                1,
                gl::TRUE,
                transform::to_gl(model).as_ptr(),
            );
            gl::Uniform4f(self.color_loc, color[0], color[1], color[2], color[3]);
            mesh.draw();
        }
    }
}

impl Drop for MeshRenderer {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.program) };
    }
}
