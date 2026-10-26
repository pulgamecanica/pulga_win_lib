//! Windowing + raw-OpenGL layer for the `42run` project.
//!
//! pulga_win_lib owns the window, the event loop, and the GL context; callers
//! implement [`App`] and talk to OpenGL directly (via the `gl` crate) once
//! [`run`] has started.

// Native-only until the WebGL backend lands: glutin has no wasm32 support.
#[cfg(not(target_arch = "wasm32"))]
mod app;
pub mod event;

#[cfg(not(target_arch = "wasm32"))]
pub use app::{App, WindowConfig, run};
pub use event::{Event, Key, MouseButton};

/// Re-exported so downstream crates (pulga_gui_lib, pulga_run) call GL without
/// picking their own version of the raw bindings.
pub use gl;

pub mod camera;
pub mod canvas;
pub mod gl_mesh;
pub mod obj;
pub mod primitives;
pub mod render;
#[cfg(not(target_arch = "wasm32"))]
pub mod screenshot;
pub mod shader;
pub mod texture;
pub mod time;

pub mod transform;
mod vec3;

pub use camera::Camera;

#[cfg(all(test, target_arch = "wasm32"))]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
