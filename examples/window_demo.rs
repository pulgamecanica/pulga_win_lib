//! Opens a window, clears it to cornflower blue, closes on Escape.
//! `cargo run --example window_demo` (needs a display).

use pulga_win_lib::{App, Event, Key, WindowConfig, gl, run};

struct Demo;

impl App for Demo {
    fn event(&mut self, event: Event) {
        if event == Event::KeyDown(Key::Escape) {
            std::process::exit(0);
        }
    }

    fn draw(&mut self) {
        unsafe {
            gl::ClearColor(0.39, 0.58, 0.93, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(WindowConfig { title: "pulga_win_lib window_demo".into(), ..Default::default() }, Demo)
}
