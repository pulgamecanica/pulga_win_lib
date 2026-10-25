//! Loads the bundled 42-logo OBJ, compiles the mesh shader, and spins it in
//! front of the camera — the "can you actually see a parsed+shaded mesh"
//! check for `obj`, `shader`, `gl_mesh` and `transform`.
//! `cargo run --example mesh_demo` (needs a display).

use pulga_win_lib::{App, Event, Key, WindowConfig, gl, gl_mesh::GlMesh, obj, run, shader, transform};
use pulgamecanica_matrix::pulgamecanica::projection_matrix::projection;
use std::time::Instant;

struct MeshDemo {
    mesh: Option<GlMesh>,
    program: gl::types::GLuint,
    proj_loc: gl::types::GLint,
    view_loc: gl::types::GLint,
    model_loc: gl::types::GLint,
    centroid: [f32; 3],
    eye_distance: f32,
    aspect: f32,
    started: Instant,
}

impl MeshDemo {
    fn new() -> Self {
        Self {
            mesh: None,
            program: 0,
            proj_loc: -1,
            view_loc: -1,
            model_loc: -1,
            centroid: [0.0; 3],
            eye_distance: 5.0,
            aspect: 16.0 / 9.0,
            started: Instant::now(),
        }
    }
}

impl App for MeshDemo {
    fn init(&mut self) {
        let obj_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models/42.obj");
        let mesh = obj::load(obj_path).expect("failed to load assets/models/42.obj");

        // Frame the camera from the mesh's own extent, so this works for
        // whatever OBJ gets swapped in later, not just this one asset.
        let radius = mesh
            .vertices
            .iter()
            .map(|v| {
                let d = [
                    v.pos[0] - mesh.centroid[0],
                    v.pos[1] - mesh.centroid[1],
                    v.pos[2] - mesh.centroid[2],
                ];
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
            })
            .fold(0.0f32, f32::max);
        self.centroid = mesh.centroid;
        self.eye_distance = (radius * 2.5).max(1.0);

        let vert_src = include_str!("../assets/shaders/mesh.vert");
        let frag_src = include_str!("../assets/shaders/mesh.frag");

        unsafe {
            self.program =
                shader::link_program(vert_src, frag_src).expect("shader compile/link failed");
            self.proj_loc = shader::uniform_location(self.program, "proj");
            self.view_loc = shader::uniform_location(self.program, "view");
            self.model_loc = shader::uniform_location(self.program, "model");
            self.mesh = Some(GlMesh::upload(&mesh));
            gl::Enable(gl::DEPTH_TEST);
        }
    }

    fn event(&mut self, event: Event) {
        match event {
            Event::KeyDown(Key::Escape) => std::process::exit(0),
            Event::Resized { width, height } if height > 0 => {
                self.aspect = width as f32 / height as f32;
            }
            _ => {}
        }
    }

    fn draw(&mut self) {
        let Some(mesh) = &self.mesh else { return };
        let t = self.started.elapsed().as_secs_f32();

        let eye = [
            self.centroid[0] + self.eye_distance * t.sin() * 0.6,
            self.centroid[1] + self.eye_distance * 0.3,
            self.centroid[2] + self.eye_distance * t.cos() * 0.6 + self.eye_distance * 0.4,
        ];
        let view = transform::look_at(eye, self.centroid, [0.0, 1.0, 0.0]);
        let proj = projection(70f32.to_radians(), self.aspect, 0.1, 100.0);
        let model = transform::rotate_axis_angle([0.0, 1.0, 0.0], t * 0.6);

        unsafe {
            gl::ClearColor(0.08, 0.09, 0.11, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::UseProgram(self.program);
            gl::UniformMatrix4fv(self.proj_loc, 1, gl::TRUE, transform::to_gl(&proj).as_ptr());
            gl::UniformMatrix4fv(self.view_loc, 1, gl::TRUE, transform::to_gl(&view).as_ptr());
            gl::UniformMatrix4fv(self.model_loc, 1, gl::TRUE, transform::to_gl(&model).as_ptr());
            mesh.draw();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(WindowConfig { title: "pulga_win_lib mesh_demo — 42".into(), ..Default::default() }, MeshDemo::new())
}
