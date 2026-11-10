#[cfg(target_os = "macos")]
fn main() -> anyhow::Result<()> {
    graphics_lab::metal_app::run()
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("metal_cube requires macOS and a Metal-capable GPU");
    std::process::exit(1);
}
