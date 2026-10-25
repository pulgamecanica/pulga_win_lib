//! Windowing + raw-OpenGL layer for the `42run` project.
//!
//! pulga_win_lib owns the window, the event loop, and the GL context; callers
//! implement [`App`] and talk to OpenGL directly (via the `gl` crate) once
//! [`run`] has started.

mod app;
pub mod event;

pub use app::{App, WindowConfig, run};
pub use event::{Event, Key};

/// Re-exported so downstream crates (pulga_gui_lib, pulga_run) call GL without
/// picking their own version of the raw bindings.
pub use gl;

pub mod camera;
pub mod gl_mesh;
pub mod obj;
pub mod primitives;
pub mod shader;
pub mod transform;
mod vec3;

pub use camera::Camera;
