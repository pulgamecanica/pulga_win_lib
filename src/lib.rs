//! Windowing + raw-OpenGL layer for the `42run` project.
//!
//! PulgaWinLib owns the window, the event loop, and the GL context; callers
//! implement [`App`] and talk to OpenGL directly (via the `gl` crate) once
//! [`run`] has started.

mod app;
pub mod event;

pub use app::{run, App, WindowConfig};
pub use event::{Event, Key};

/// Re-exported so downstream crates (PulgaGuiLib, pulga_run) call GL without
/// picking their own version of the raw bindings.
pub use gl;

pub mod camera;
pub mod gl_mesh;
pub mod obj;
pub mod primitives;
pub mod shader;
pub mod transform;

pub use camera::Camera;
