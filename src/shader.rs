//! Our own GLSL shader compilation — raw `gl` calls, no library wrapping
//! OpenGL itself, per the subject.

use std::ffi::CString;
use std::fmt;

#[derive(Debug)]
pub struct ShaderError(pub String);

impl fmt::Display for ShaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ShaderError {}

/// # Safety
/// Requires a current GL context.
unsafe fn compile(kind: gl::types::GLenum, src: &str) -> Result<gl::types::GLuint, ShaderError> {
    unsafe {
        let shader = gl::CreateShader(kind);
        let c_src = CString::new(src).expect("shader source must not contain a NUL byte");
        gl::ShaderSource(shader, 1, &c_src.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        let mut ok = gl::FALSE as gl::types::GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut ok);
        if ok != gl::TRUE as gl::types::GLint {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec![0u8; len.max(1) as usize];
            gl::GetShaderInfoLog(shader, len, std::ptr::null_mut(), buf.as_mut_ptr().cast());
            gl::DeleteShader(shader);
            buf.retain(|&b| b != 0);
            return Err(ShaderError(String::from_utf8_lossy(&buf).into_owned()));
        }
        Ok(shader)
    }
}

/// Compiles and links a vertex+fragment program.
///
/// # Safety
/// Requires a current GL context.
pub unsafe fn link_program(
    vertex_src: &str,
    fragment_src: &str,
) -> Result<gl::types::GLuint, ShaderError> {
    unsafe {
        let vs = compile(gl::VERTEX_SHADER, vertex_src)?;
        let fs = compile(gl::FRAGMENT_SHADER, fragment_src)?;

        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);

        gl::DeleteShader(vs);
        gl::DeleteShader(fs);

        let mut ok = gl::FALSE as gl::types::GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut ok);
        if ok != gl::TRUE as gl::types::GLint {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec![0u8; len.max(1) as usize];
            gl::GetProgramInfoLog(program, len, std::ptr::null_mut(), buf.as_mut_ptr().cast());
            gl::DeleteProgram(program);
            buf.retain(|&b| b != 0);
            return Err(ShaderError(String::from_utf8_lossy(&buf).into_owned()));
        }
        Ok(program)
    }
}

/// # Safety
/// Requires a current GL context and a linked `program`.
pub unsafe fn uniform_location(program: gl::types::GLuint, name: &str) -> gl::types::GLint {
    unsafe {
        let c_name = CString::new(name).unwrap();
        gl::GetUniformLocation(program, c_name.as_ptr())
    }
}
