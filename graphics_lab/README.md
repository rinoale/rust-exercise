# Rust graphics lab and block world

Rust apps explore 3D rendering and a small game without a game engine:

| App | Renderer | Scene |
|---|---|---|
| `block_world` | Native Metal API | First-person voxel world with digging and building |
| `diagonal_world` | Native Metal API | Buildable 5–12–13 world with three real ramp angles |
| `metal_cube` | Native Metal API | Two rotating textured cubes and a floor |
| `own_gpu` | Our G17 kernel and Rust executor using private IOGPU transport | One shaded cube, ray traced on the GPU |

The native app now renders an image. Its default command opens a window; `--probe`
is a separate connection diagnostic and deliberately does not render.

## Diagonal world

A small creative sandbox built from **5 × 5 × 5 cubes** and
**5 × 12 × 5 boxes**, plus their diagonal halves. The halves are solid triangular
prisms: their exposed surfaces, ray picking, and player collision all follow the
actual slopes, rather than approximating them with little stairs.

```sh
cd graphics_lab
cargo run --bin diagonal_world
```

Requires macOS and Metal, like `block_world`. Click to capture the mouse. Explore
three colored ramp lanes, a terraced hill, a house with a sloped roof, and a pair
of equal-height towers: **12 cubes × 5 = 5 boxes × 12 = 60 design units**.

| Piece | Dimensions in design units | Slope |
|---|---|---|
| Cube | 5 × 5 × 5 | Flat top |
| Tall box | 5 × 12 × 5 | Flat top |
| Long box | 12 × 5 × 5 | Flat top |
| Gentle wedge | 12 × 5 × 5 bounding box | 22.619865° |
| Cube wedge | 5 × 5 × 5 bounding box | 45° |
| Steep wedge | 5 × 12 × 5 bounding box | 67.380135° |

The long and tall boxes are orientations of the same box; the gentle and steep
wedges are orientations of the same 5–12–13 triangular prism. Five design units
equal one rendering unit. Placement snaps to **one design unit**, so both 5-unit
and 12-unit lengths fit on the same integer lattice.

| Control | Action |
|---|---|
| WASD / mouse | Walk / look |
| Space | Jump; rise in fly mode |
| Left click / right click | Remove a piece / place a piece |
| 1–6 or mouse wheel | Select a shape; pieces are unlimited |
| R | Rotate the next piece 90° horizontally |
| F | Toggle fly mode (passes through solids) |
| Left Shift / Left Ctrl | Descend in fly mode / move faster |
| V | Aerial overview, with flight enabled |
| T | Return to safe standing room at spawn |
| Esc / click | Release mouse and pause / resume |
| Q | Quit |

A green stippled preview marks valid placement; red means occupied, outside the
building limits, or overlapping your body. Axis-aligned faces attach flush;
targeting a ramp's slope places the next piece at the top of its bounding box.
This prototype reserves each piece's **entire bounding box** when building, so
it does not support inserting a second piece into the empty half of a wedge.
The foundation is fixed. All three slopes are walkable, including the steep
67.38° ramp; this is an intentional sandbox movement rule, not realistic friction.
The world is finite, and edits reset when you restart. There is no saving or
multiplayer.

```sh
# GPU screenshot (aerial view by default):
cargo run --bin diagonal_world -- --capture artifacts/diagonal-world.png
# First-person screenshot or short native-window smoke test:
cargo run --bin diagonal_world -- --capture artifacts/diagonal-walk.png --walk-view
cargo run --bin diagonal_world -- --overview --frames 5
# Geometry, exact ray hits, stack alignment, movement, and building rules:
cargo test --test diagonal_gameplay
```

`src/diagonal/` owns solids, terrain, building, and movement. `diagonal_app.rs`
handles the native window and input; `diagonal_presentation.rs` supplies the HUD
and placement preview. Both block demos share `voxel_renderer.rs` and its Metal
pipeline. No game engine or additional dependencies are required.

## Block world

A small Minecraft-like prototype written in Rust using `winit` for the window and
input, `glam` for math, and Metal for GPU rendering. No Unity, Unreal, or game
engine is used. Like `metal_cube`, this app requires **macOS and a Metal GPU**.
It uses the public Metal API and does not require the specific M5 configuration
of the separate `own_gpu` experiment.

```sh
cd graphics_lab
cargo run --bin block_world
```

Click the window to capture the mouse and start playing. That first click only
captures the mouse. Esc releases it and pauses movement; click again to resume.
Switching away from the window also releases the mouse and clears held keys.

| Control | Action |
|---|---|
| W / A / S / D | Walk forward / left / backward / right |
| Mouse | Look around |
| Space | Jump while standing on a block |
| Left click | Dig the highlighted block and collect it into inventory |
| Right click | Use one selected block item to place it against the targeted face |
| 1–4 or mouse wheel | Select grass, dirt, stone, or wood |
| Esc | Release the mouse and pause |
| R | Respawn in safe standing room |
| Q or window close | Quit |

The four-slot hotbar shows the selected item and available counts. You start
with 16 grass, 32 dirt, 32 stone, and 16 wood blocks. Digging adds an item;
placement consumes one. Blocks must be within six units of the crosshair.
Placement cannot overlap your body, replace an occupied block, or leave the
world bounds. Jump and place beneath yourself to build upward.

Terrain occupies a **40 × 24 × 40** integer grid, with Y as the vertical axis.
Grass and dirt sit over stone, with a few wood stacks near spawn. Gravity,
wall and ceiling collision, normalized diagonal movement, and falling into
holes are implemented. Falling below the world respawns the player. This is a
finite, single-player prototype: edits and inventory are **in memory only** and
reset when you restart. There is no crafting, saving, or infinite terrain yet.

### How the game works

- `src/voxel.rs`: block grid, exposed-face meshing, exact grid ray traversal,
  inventory actions, and player collision/physics. This logic has no GPU calls.
- `src/voxel_app.rs`: window, mouse capture, held keys, and the game loop.
- `src/voxel_renderer.rs` and `src/voxel.metal`: Metal depth rendering, procedural
  block surfaces, lighting, distance fog, target outline, crosshair, and hotbar.

Only exposed block faces are uploaded. The world mesh is cached and rebuilt
when digging or placement changes the grid. Physics uses small substeps to
prevent passing through blocks during slow frames. Rendering waits for each
frame to finish to keep GPU resource ownership simple for this learning app.

```sh
# Offscreen render and a short real-window smoke test:
cargo run --bin block_world -- --capture artifacts/block-world.png
cargo run --bin block_world -- --frames 5

# Game rules/physics and real Metal dig/place image validation:
cargo test --test voxel_gameplay --test voxel_rendering
```

## Native GPU cube

```sh
cd graphics_lab
cargo run --no-default-features --bin own_gpu
```

Use the arrow keys to rotate or tilt the cube, R to reset, and Esc to close.
The image is rendered at 256 × 256 and scaled for presentation. The app redraws
when the camera changes; it is an interactive experiment rather than a benchmark.

This runtime is verified specifically on **Apple M5 Pro, macOS 26.6.2 build
25G83, AGX driver 353.14**. It refuses other versions because its private resource
and launch layouts have not been validated there.

Capture an image and its execution receipt:

```sh
cargo run --no-default-features --bin own_gpu -- --capture artifacts/native-cube.png
cargo run --no-default-features --bin own_gpu -- --capture artifacts/rotated.png --angle 1.2
cargo run --no-default-features --bin own_gpu -- --size 128 --frames 3
cargo run --no-default-features --bin own_gpu -- --self-test
```

`--capture` runs without opening a window and also writes a JSON receipt beside
the PNG. Every new renderer first checks 1,024 GPU-produced arithmetic values
against `3*i+1`. Every cube tile must overwrite its output sentinels, preserve
surrounding guard bytes, and complete exactly once before its pixels are read.

### What executes where

```text
Rust camera setup
    ↓ inverse camera rays and lighting constants
Our compiled G17 cube shader
    ↓ GPU ray/cube intersections, visibility, face selection, pixel writes
Private IOGPU transport → Apple kernel driver → M5 GPU
    ↓ completed pixels
CoreGraphics window presentation
```

The cube is rendered with GPU compute, not the hardware triangle rasterizer.
The CPU prepares camera rays and three face colors. The GPU computes intersections
with the six cube planes, tests visibility, chooses the visible face, and writes
one pixel per lane. There is no CPU rendering fallback or prerecorded image.

The renderer makes no Metal API calls, creates no Metal pipeline, and uses no
AGXMetal driver. **Apple's IOGPU library does load the base Metal framework as a
runtime dependency**; the receipt reports that dependency instead of hiding it.
This is a substitute for Metal's rendering path, not a replacement for every Apple
GPU software component.

Window drawing can independently load AGXMetal through AppKit/CoreGraphics.
Consequently, interactive mode runs the cube renderer in a separate headless
worker process. That worker rejects a loaded AGXMetal driver; the presenter
receives only finished pixels and an execution receipt. Offscreen capture uses
that same native renderer directly, without the presenter.

The private callback ownership rules are not fully established. Following the
reference runtime, callback records are retained until process exit and each
renderer is bounded to 65,536 dispatches. A 256 × 256 frame uses 64 dispatches;
the initial arithmetic test uses one. Completion waits have a five-second timeout.

### Source and provenance

- `src/direct/g17.rs`: Rust allocation, queue setup, command publication,
  submission, completion, and cube rendering.
- `src/direct/native_app.rs`: command line, renderer worker protocol, PNG output,
  and CoreGraphics presentation through `softbuffer`.
- `tools/build_g17_assets.py`: our cube and arithmetic programs expressed in
  AGXForge IR, plus reproducible offline asset generation.
- `native/g17/manifest.json`: compiler revision, generated-file hashes, shader
  ABI information, and platform identity.

The resource/launch recipe and offline G17 compiler come from
[AGXForge](https://github.com/sbryngelson/AGXForge), pinned to commit
`c6a3b22c23f7668a6139b6194ba82ee27587da52`. The runtime was ported to Rust and
adapted to our cube kernel. Its MIT notice is retained in
`native/g17/AGXForge-LICENSE.txt`.

The included assets are constructed from the reference's measured fields and our
IR programs. Generating them reads no Metal capture and invokes no Metal API.
Python and AGXForge are needed only to regenerate the assets; the Rust app does
not invoke Python, Metal, or an external renderer at runtime. Reproduction:

```sh
# In a checkout of the pinned AGXForge revision, install its Python dependencies
# and build its two decoder tools:
make tools/agx3dis tools/libagx3dis.dylib
# Then, from graphics_lab with that Python environment:
python tools/build_g17_assets.py /path/to/AGXForge
```

The offline compiler uses Apple's GPUCompiler decoder to validate instructions.
Its generated machine code is embedded in the Rust executable.

## Metal cube scene

```sh
cargo run --bin metal_cube
cargo run --bin metal_cube -- --capture artifacts/metal.png --time 1.5
```

Space pauses rotation; W toggles wireframe; D toggles depth testing; T toggles
texture; arrows orbit; + / - zoom; R resets; Esc closes. The floor is drawn last
so disabling depth creates visible errors. The scene exercises perspective,
back-face culling, depth, mipmaps, anisotropic filtering, normals, and sRGB output.

Shaders compile at runtime, so full Xcode and the standalone `metal` command are
not required. The title's submit-and-wait timing includes a CPU wait for GPU
completion. The two apps use different scenes and rendering techniques and do
not constitute an equivalent-work performance comparison.

## Driver research tools

```sh
cargo run --no-default-features --bin own_gpu -- --probe --report artifacts/direct-probe.json
cargo build --bin metal_cube
cargo run --no-default-features --bin own_gpu -- --trace artifacts/metal-iokit.jsonl
```

The probe only discovers and opens the driver. The trace command observes a
separate Metal reference process and writes JSONL, a selector/size summary, and
its reference image. It intercepts synchronous calls on directly opened GPU
connections, with 128-byte structure prefixes and at most 16 scalars. Async calls,
auxiliary connections, mapped contents, and complete command buffers are absent.
Its pointers and handles are process-specific, so the trace is not replayable.

## Validation

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo test --no-default-features
```

The native integration test executes real GPU work, compares cube silhouettes at
two angles against an independent double-precision geometric oracle, checks three
visible face colors, and verifies the arithmetic and execution receipts. Native
GPU dispatch tests run only on the verified platform. The Metal integration test
checks real GPU images with depth, texture, wireframe, and animation controls.
Generated captures, traces, and research checkouts under `artifacts/` are ignored.
