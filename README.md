# pulga_win_lib

Windowing + raw-graphics layer for the `42run` project (42 school "OpenGL
graphic project" subject). Handles window/event creation and talks to
OpenGL/Vulkan directly — no engine abstraction sits on top of the graphics
API, per the subject's constraints.

- Window + input event handling — **built**: `winit` for the window/event loop,
  `glutin`/`glutin-winit` only for native GL *context* creation (not a GL
  wrapper), raw `gl` crate bindings for OpenGL itself. See `src/app.rs`
  (the `App` trait + `run()`) and `src/event.rs` (winit → our own `Event`
  enum, unit tested).
- OpenGL support (primary target for this project) — **built**: a live
  GL 4.1 context.
- Vulkan support — not started yet.
- Own 3D mesh/object parsing (no mesh-loading library) — **built**:
  `src/obj.rs`, a from-scratch Wavefront OBJ parser (fan-triangulation,
  missing-normal computation, centroid/main-axis framing heuristic), ported
  from a from-scratch C++ OBJ loader written for an earlier 42 project
  (`~/scop`). Unit tested, including against the bundled
  `assets/models/42.obj`.
- Camera — **built**: `src/camera.rs`, a `Camera` type (perspective + cached
  look-at params + dolly `zoom`), same design as `~/scop`'s. Unit tested.
- Own transformation math — **built**: `src/transform.rs` on top of the
  [`pulgamecanica_matrix`](https://crates.io/crates/pulgamecanica_matrix)
  dependency, which supplies both `projection()` (called directly from
  `camera.rs`/`mesh_demo.rs` — see below) and the vector ops
  (`dot`/`cross_product`/`norm`/`add`/`sub`) — `src/vec3.rs` is a thin
  `[f32; 3]` ergonomic wrapper over `pulgamecanica_matrix::Vector<f32>`,
  shared by `obj`, `transform` and `camera` so the actual math lives in one
  place (the crate), not reimplemented three times. `translate`/`scale`/
  `rotate_axis_angle`/`look_at` (the matrix formulas) ported from the same
  `~/scop` C++ math code. Unit tested.

  `projection()` briefly had a variable-shadowing bug (`b`, the bottom
  frustum bound, got reused as the Y-scale factor before the *original* `b`
  was needed again for the Y-centering term, shifting everything rendered
  off-screen vertically) — fixed upstream in `pulgamecanica_matrix` 1.0.2,
  so this crate calls it directly again rather than carrying a local
  workaround.
- Shader compilation/management — **built**: `src/shader.rs`
  (`link_program`, raw `glCompileShader`/`glLinkProgram` with error-log
  surfacing). `assets/shaders/mesh.{vert,frag}` adapted from `~/scop`'s
  Vulkan GLSL (dropped the `set`/`binding` layout qualifiers and the
  Vulkan-clip-space flip; kept the per-face hash coloring + diffuse shading).
  No texture sampling yet (`~/scop`'s triplanar/UV blend) — that needs a PNG
  decoder first.
- 2D & 3D canvas — not started yet.

`src/gl_mesh.rs` uploads a parsed `obj::Mesh` to a VAO/VBO/EBO and draws it.
`src/primitives.rs` generates procedural meshes (cube, ground plane) in the
same `Mesh` shape, for gameplay geometry that isn't OBJ-authored —
`pulga_run`'s player/obstacles/ground use these.
`cargo run --example mesh_demo` ties the OBJ pipeline together: loads
`assets/models/42.obj`, compiles the shader, and spins the logo in front of
an auto-framed camera.

`App::title()` (default `None`) lets a consumer update the window title
every frame it returns `Some` — `pulga_run` uses it as a crude distance
meter, since there's no on-screen text rendering yet.

## Usage

```sh
make build        # cargo build
make test         # cargo test (pure logic: event translation, OBJ parsing, transforms)
make coverage      # cargo tarpaulin --lib
make doc           # cargo doc --open
make window-demo    # open an empty window (needs a display)
make mesh-demo       # load+shade+spin the 42-logo OBJ (needs a display)
make               # make help
```

Part of the [42Run](../42Run) workspace of independent projects; see the root
README there for how this fits with `pulga_gui_lib` and `pulga_run`.
