# Rust graphics lab and block world

Rust apps explore 3D rendering and small games without a game engine.

| App | macOS | Windows | Linux / WSL2 |
|---|---|---|---|
| `metal_cube` (historical name) | Metal adapter | Direct3D 12 adapter | Vulkan / OpenGL adapter |
| `block_world` | Metal adapter | Direct3D 12 adapter | Vulkan / OpenGL adapter |
| `diagonal_world` | Metal adapter | Direct3D 12 adapter | Vulkan / OpenGL adapter |
| `own_gpu` | Original private Apple AGX experiment | Portable GPU cube | Portable GPU cube |

## Build and run on this machine

From the repository root, build **all five 3D app binaries**, including the
assembly demo's portable mode:

```sh
./scripts/build_3d.sh
cargo run --manifest-path graphics_lab/Cargo.toml --bin diagonal_world
cargo run --manifest-path graphics_lab/Cargo.toml --bin block_world
cargo run --manifest-path graphics_lab/Cargo.toml --bin metal_cube
cargo run --manifest-path graphics_lab/Cargo.toml --bin own_gpu
cargo run --manifest-path assembly_metal_demo/Cargo.toml
```

The build script accepts Cargo arguments such as `--release`. A Linux build is a
Linux executable, even under WSL; a native Windows build selects Direct3D 12.
On Windows, use the two `cargo build --manifest-path ... --bins` commands if Bash
is unavailable. Linux windows require X11 or Wayland (WSLg supplies both).
On WSL the apps default to X11 to avoid observed Wayland compositor disconnects
with small software-rendered windows. Other Linux desktops keep winit's default.
Set `GRAPHICS_WINDOW_SYSTEM=x11` or `wayland` to override this choice.
Offscreen `--capture` does not require a window or display server with Vulkan.

At startup the renderer prints the actual graphics backend, device name, and
device type. CPU adapters such as llvmpipe are accepted, allowing the apps to run
on WSL even without hardware Vulkan. This is slower than a hardware GPU.
The current WSL machine has been verified with **software Vulkan**, including
captures and window presentation. OpenGL capture works here, but OpenGL window presentation failed during
validation; use Vulkan on this machine.

To select a backend explicitly:

```sh
GRAPHICS_BACKEND=vulkan cargo run --bin block_world
GRAPHICS_BACKEND=gl cargo run --bin metal_cube -- --capture artifacts/gl.png
# Native Windows PowerShell:
# $env:GRAPHICS_BACKEND = "dx12"
# cargo run --bin block_world
```

Allowed choices are `metal` on macOS, `dx12` on Windows, and `vulkan` or `gl` on
Linux. Unsupported choices fail explicitly. No backend override is necessary
for normal use.

## Mouse input on WSL

Both world games default to **cursor input** on WSL. This computes motion from
successive positions inside the window and ignores raw device motion, which can
contain absolute values through WSLg and spin the camera into the floor.
The first movement after capture/reentry establishes a reference without turning.
The pointer stays visible, and the app never warps it back to the center.

Click to play, use WASD to walk, and move the cursor to look. **Arrow keys also
look around; H levels the view.** Since cursor mode is bounded by the window,
use arrows to continue turning at an edge, or press Esc, move the pointer back,
and click to resume. Losing focus clears held keys and releases the pointer.

Normal macOS, Windows, and Linux desktops retain raw mouse input. Override
with `GRAPHICS_MOUSE_INPUT=cursor` or `GRAPHICS_MOUSE_INPUT=raw` when needed;
use cursor mode on this WSL installation. An already-running executable must
be closed and relaunched after rebuilding to pick up the fix.

## Graphics abstraction

`src/graphics/mod.rs` defines **`GraphicsApi`**, the Rust trait equivalent of an
abstract graphics class. It exposes presentation, RGBA readback, surface size,
and adapter identity. `Frame` and `DrawCall` carry API-independent mesh data,
transforms, shading, and depth/wireframe settings.

- `MacAdapter` selects Metal.
- `WindowsAdapter` selects Direct3D 12.
- `LinuxAdapter` selects Vulkan/OpenGL, including software adapters on WSL.

All three implement `GraphicsApi` and share a `wgpu` command implementation and
WGSL shaders. The implementation owns device creation, pipelines, texture
mipmaps, mesh uploads, resize/reconfiguration, depth attachments, presentation,
and padded GPU readback. World meshes are retained on the GPU until the world
revision changes. Wireframe uses line geometry, so it does not require optional
hardware polygon-mode support.

The app loops use `CubeRenderer` or `VoxelRenderer` through this interface.
`presentation.rs` contains the shared voxel scene interface and HUD geometry;
gameplay modules contain no GPU API calls. Original direct Metal renderers are
retained as reference code under the `metal-app` feature. The three regular apps
also build with `--no-default-features`.

`own_gpu` on non-macOS and `assembly_metal_demo` on non-Apple-Silicon platforms
use the shared portable single-cube renderer. They print **Portable GPU mode**:
these modes do not execute or emulate Apple assembly, G17 machine code, private
IOGPU calls, or native execution receipts. The original native research paths
remain available on their supported Macs. Portable modes support `--capture`,
`--size`, `--angle`, `--frames`, and `--self-test`; Space pauses, arrows rotate,
R resets, and Esc exits. Native-only diagnostic flags are not portable.

## Diagonal world

A small creative sandbox built from **5 × 5 × 5 cubes** and
**5 × 12 × 5 boxes**, plus their diagonal halves. The halves are solid triangular
prisms: their exposed surfaces, ray picking, and player collision all follow the
actual slopes, rather than approximating them with little stairs.

```sh
cd graphics_lab
cargo run --bin diagonal_world
```

Runs on macOS, Windows, and Linux/WSL. Click to capture the mouse. Explore
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
| WASD | Walk |
| Mouse / arrow keys | Look around |
| H | Level the camera |
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
and placement preview. Both block demos share `voxel_renderer.rs` and the platform-independent
graphics pipeline. No game engine is required.

## Block world

A small Minecraft-like prototype written in Rust using `winit` for the window and
input, `glam` for math, and the shared graphics adapters for GPU rendering.
No Unity, Unreal, or game engine is used. It runs on **macOS, Windows, and
Linux/WSL**, independently of the separate M5-specific native experiment.

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
| Mouse / arrow keys | Look around |
| H | Level the camera |
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
- `src/voxel_renderer.rs` and `src/graphics/shader.wgsl`: depth rendering, procedural
  block surfaces, lighting, distance fog, target outline, crosshair, and hotbar.

Only exposed block faces are uploaded. The world mesh is cached and rebuilt
when digging or placement changes the grid. Physics uses small substeps to
prevent passing through blocks during slow frames. GPU resource lifetimes are managed by wgpu; captures wait for readback completion.

```sh
# Offscreen render and a short real-window smoke test:
cargo run --bin block_world -- --capture artifacts/block-world.png
cargo run --bin block_world -- --frames 5

# Game rules/physics and GPU API dig/place image validation:
cargo test --test voxel_gameplay --test voxel_rendering
```

## Native GPU cube (macOS research path)

The rest of this section describes the original macOS-only private AGX path.
On Windows/Linux the same binary uses the portable mode documented above.

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

WGSL shaders compile through the selected graphics adapter; full Xcode and the
standalone `metal` command are not required for the shared renderer. The title's
submit timing measures CPU submission, not GPU completion. The two apps use different scenes and rendering techniques and do
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
GPU dispatch tests run only on the verified platform. The portable integration tests
check rendered images with depth, texture, wireframe, animation, world edits,
camera changes, resize, and unaligned readback rows. Software Vulkan is valid
for these tests. Windows cross-compilation checks do not substitute for a
native Windows or macOS runtime test.
Generated captures, traces, and research checkouts under `artifacts/` are ignored.
