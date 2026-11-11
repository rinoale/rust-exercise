#[cfg(target_os = "macos")]
fn main() -> anyhow::Result<()> {
    graphics_lab::voxel_app::run()
}
#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("block_world requires macOS and a Metal-capable GPU");
    std::process::exit(1);
}
