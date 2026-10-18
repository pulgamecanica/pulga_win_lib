use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version};
use glutin::display::{Display, GetGlDisplay};
use glutin::prelude::*;
use glutin::surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowAttributes, WindowId};

use crate::event::{translate_key, Event, Key};

/// Configuration for the window PulgaWinLib opens.
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self { title: "42run".to_string(), width: 1280, height: 720 }
    }
}

/// Implemented by whatever drives the window (the game, an example, a GUI
/// gallery...). PulgaWinLib owns the window/event loop/GL context; the app
/// only reacts to translated [`Event`]s and draws when asked.
pub trait App {
    /// Called once, right after the GL context is current.
    fn init(&mut self) {}
    fn event(&mut self, event: Event);
    fn draw(&mut self);

    /// Returning `Some(title)` after a draw updates the window title (e.g.
    /// a distance meter); called every frame, so apps should debounce if the
    /// string changes rarely. `None` (the default) leaves the title as-is.
    fn title(&self) -> Option<String> {
        None
    }
}

struct GlState {
    #[allow(dead_code)] // kept alive: the surface/context borrow the display
    display: Display,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
    window: Window,
}

struct Handler<A: App> {
    config: WindowConfig,
    app: A,
    gl: Option<GlState>,
}

impl<A: App> ApplicationHandler for Handler<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gl.is_some() {
            return;
        }

        let window_attributes = WindowAttributes::default()
            .with_title(self.config.title.clone())
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.width,
                self.config.height,
            ));

        let template = ConfigTemplateBuilder::new().with_alpha_size(8);

        let (window, gl_config) = DisplayBuilder::new()
            .with_window_attributes(Some(window_attributes))
            .build(event_loop, template, |configs| {
                configs
                    .reduce(|accum, config| {
                        if config.num_samples() > accum.num_samples() { config } else { accum }
                    })
                    .expect("no GL configs available")
            })
            .expect("failed to build GL display/window");

        let window = window.expect("window attributes were provided, window should exist");
        let gl_display = gl_config.display();
        let raw_window_handle = window.window_handle().ok().map(|h| h.as_raw());

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 1))))
            .build(raw_window_handle);

        let not_current_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .expect("failed to create GL context")
        };

        let surface_attributes = window
            .build_surface_attributes(SurfaceAttributesBuilder::<WindowSurface>::new())
            .expect("failed to build surface attributes");
        let surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attributes)
                .expect("failed to create GL surface")
        };

        let context = not_current_context
            .make_current(&surface)
            .expect("failed to make GL context current");

        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });

        let _ = surface
            .set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()));

        self.app.init();
        self.gl = Some(GlState { display: gl_display, surface, context, window });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(gl) = self.gl.as_mut() else { return };
        match event {
            WindowEvent::CloseRequested => {
                self.app.event(Event::CloseRequested);
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    gl.window.resize_surface(&gl.surface, &gl.context);
                    unsafe { gl::Viewport(0, 0, size.width as i32, size.height as i32) };
                }
                self.app.event(Event::Resized { width: size.width, height: size.height });
            }
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                let key = match key_event.physical_key {
                    PhysicalKey::Code(code) => translate_key(code),
                    PhysicalKey::Unidentified(_) => Key::Other,
                };
                let translated =
                    if key_event.state.is_pressed() { Event::KeyDown(key) } else { Event::KeyUp(key) };
                self.app.event(translated);
            }
            WindowEvent::RedrawRequested => {
                self.app.event(Event::RedrawRequested);
                self.app.draw();
                if let Some(title) = self.app.title() {
                    gl.window.set_title(&title);
                }
                gl.surface.swap_buffers(&gl.context).expect("failed to swap buffers");
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(gl) = &self.gl {
            gl.window.request_redraw();
        }
    }
}

/// Opens a window and runs `app` until the window is closed.
pub fn run(config: WindowConfig, app: impl App + 'static) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut handler = Handler { config, app, gl: None };
    event_loop.run_app(&mut handler)?;
    Ok(())
}
