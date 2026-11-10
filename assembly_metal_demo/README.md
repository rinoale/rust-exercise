# Rust → ARM64 assembly → Metal → 3D cube

A rotating, colored 3D cube with perspective and depth testing. Rust manages
an ordinary macOS window and calls an ARM64 assembly library. The assembly
makes every Metal rendering call, including presentation through CAMetalLayer.

## Run

Requires an Apple Silicon Mac, a native ARM64 Rust toolchain, and Xcode Command
Line Tools (`xcode-select --install`).

```sh
cd assembly_metal_demo
cargo run
```

A window opens with the rotating cube. Press **Space** to pause/resume, or
**Esc** to quit. Resizing the window recreates its depth attachment.

Render the same cube into a PNG without opening a window:

```sh
cargo run -- --capture /tmp/assembly-metal-cube.png
```

Run for a fixed number of frames and exit:

```sh
cargo run -- --frames 120
```

The earlier vector-add computation remains available separately:

```sh
cargo run --bin vector_add
```

## How the pieces connect

| File | Responsibility |
| --- | --- |
| [src/main.rs](src/main.rs) | Uses winit for the window, advances the rotation angle, calls the assembly package, and optionally saves pixels as PNG. |
| [metal_asm/src/cube.rs](metal_asm/src/cube.rs) | Rust ownership and validation around the assembly functions' C ABI. |
| [metal_asm/native/metal_cube.S](metal_asm/native/metal_cube.S) | Creates the Metal device, shader library, render pipeline, depth state, command queue, textures, and CAMetalLayer. Encodes draw calls, presents, waits, and releases resources. |
| [metal_asm/native/cube.metal](metal_asm/native/cube.metal) | GPU vertex and fragment shaders: 12 triangles, six face colors, rotation, and perspective projection. |
| [metal_asm/build.rs](metal_asm/build.rs) | Assembles the static library and links Apple's frameworks and Objective-C runtime. |
| [src/bin/vector_add.rs](src/bin/vector_add.rs) | The earlier compute-only example. |

Rust calls three assembly entry points:

```rust
unsafe extern "C" {
    fn asm_cube_init(context: *mut usize, ns_view: *mut std::ffi::c_void) -> i32;
    fn asm_cube_frame(
        context: *mut usize,
        width: usize,
        height: usize,
        bgra: *mut u8,
        angle: f32,
    ) -> i32;
    fn asm_cube_destroy(context: *mut usize);
}
```

Inside assembly, `bl _MTLCreateSystemDefaultDevice` calls Metal's exported C
entry point directly. Metal object methods use `_objc_msgSend`, with `x0`
holding the receiver and `x1` holding the selector. For the cube's draw call:

```asm
csel LC_draw         // selector: drawPrimitives:vertexStart:vertexCount:
mov x0, x28         // MTLRenderCommandEncoder
mov x2, #3          // MTLPrimitiveTypeTriangle
mov x3, #0          // first vertex
mov x4, #36         // 12 triangles
bl _objc_msgSend
```

The assembly also handles ARM64 calling conventions for floating-point
arguments and structures such as CGSize, MTLClearColor, and MTLRegion.
There is no C or Objective-C rendering wrapper between assembly and Metal.

The **ARM64 assembly runs on the CPU** and submits work. The **Metal shaders
run on the GPU**, where the triangles are transformed and rasterized. This
uses Apple's supported Metal API. Shader source is embedded in the assembly
library and compiled by Metal at runtime; the standalone `metal` compiler
is not required.

The pipeline and command queue are reused. Each frame waits synchronously for
completion to keep resource lifetimes and capture simple. Rendering supports
sizes from 1 to 4096 pixels per dimension. This is a learning example, with
no claim that assembly makes the rendering faster.

## Checks

```sh
MTL_DEBUG_LAYER=1 cargo test --workspace
MTL_DEBUG_LAYER=1 cargo run -- --frames 120
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

The graphics test reads pixels rendered by the GPU. It checks the background,
visible cube area, several face colors, rotation, resizing, and invalid inputs.
The compute tests check uneven threadgroups, unsigned overflow, empty inputs,
and mismatched lengths. Tests require a working Metal device.
