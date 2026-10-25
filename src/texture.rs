//! 2D RGBA textures uploaded with raw `gl` calls.
//!
//! Pixel data comes from the caller: generated in code (a font atlas) or
//! decoded by a single-format image library, as the subject allows. This
//! module never decodes files itself.

use std::marker::PhantomData;

/// How a texture is sampled when drawn bigger or smaller than its size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    /// Square, crisp texels. For pixel art and bitmap fonts.
    Nearest,
    /// Smooth interpolation. For photos and painted textures.
    Linear,
}

/// A texture on the GPU, freed on drop. `!Send`: it must stay on the GL
/// thread.
pub struct Texture {
    id: gl::types::GLuint,
    width: u32,
    height: u32,
    _gl_thread: PhantomData<*const ()>,
}

impl Texture {
    /// Uploads `pixels`: `width * height` RGBA texels, 4 bytes each, rows
    /// from the top.
    ///
    /// # Panics
    /// If `pixels.len() != width * height * 4`.
    ///
    /// # Safety
    /// Requires a current GL context.
    pub unsafe fn from_rgba8(width: u32, height: u32, pixels: &[u8], filter: Filter) -> Self {
        assert_eq!(
            pixels.len(),
            width as usize * height as usize * 4,
            "RGBA texture needs width * height * 4 bytes"
        );
        let filter = match filter {
            Filter::Nearest => gl::NEAREST,
            Filter::Linear => gl::LINEAR,
        } as i32;
        unsafe {
            let mut id = 0;
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_ptr().cast(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            Self {
                id,
                width,
                height,
                _gl_thread: PhantomData,
            }
        }
    }

    /// The raw GL texture name.
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    /// Size in texels, as `(width, height)`.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id) };
    }
}
