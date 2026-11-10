# Metal and native GPU cube experiments

Two Rust apps demonstrate different paths to the Apple GPU:

| App | Renderer | Scene |
|---|---|---|
| `metal_cube` | Native Metal API | Two rotating textured cubes and a floor |
| `own_gpu` | Our G17 kernel and Rust executor using private IOGPU transport | One shaded cube, ray traced on the GPU |

The native app now renders an image. Its default command opens a window; `--probe`
is a separate connection diagnostic and deliberately does not render.

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
