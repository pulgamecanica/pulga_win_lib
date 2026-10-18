//! Uploads a CPU-side [`crate::obj::Mesh`] to a VAO/VBO/EBO and draws it —
//! raw `gl` calls, no wrapper on top of OpenGL.

use crate::obj::{Mesh, Vertex};

pub struct GlMesh {
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    ebo: gl::types::GLuint,
    index_count: i32,
}

impl GlMesh {
    /// # Safety
    /// Requires a current GL context.
    pub unsafe fn upload(mesh: &Mesh) -> Self {
        unsafe {
            let mut vao = 0;
            let mut vbo = 0;
            let mut ebo = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.vertices.len() * size_of::<Vertex>()) as isize,
                mesh.vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mesh.indices.len() * size_of::<u32>()) as isize,
                mesh.indices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            let stride = size_of::<Vertex>() as i32;
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                std::mem::offset_of!(Vertex, pos) as *const _,
            );
            gl::EnableVertexAttribArray(0);

            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                std::mem::offset_of!(Vertex, normal) as *const _,
            );
            gl::EnableVertexAttribArray(1);

            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                std::mem::offset_of!(Vertex, uv) as *const _,
            );
            gl::EnableVertexAttribArray(2);

            gl::VertexAttribIPointer(
                3,
                1,
                gl::UNSIGNED_INT,
                stride,
                std::mem::offset_of!(Vertex, face_id) as *const _,
            );
            gl::EnableVertexAttribArray(3);

            gl::BindVertexArray(0);

            Self { vao, vbo, ebo, index_count: mesh.indices.len() as i32 }
        }
    }

    /// # Safety
    /// Requires a current GL context and a bound/linked shader program.
    pub unsafe fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, self.index_count, gl::UNSIGNED_INT, std::ptr::null());
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for GlMesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.ebo);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
