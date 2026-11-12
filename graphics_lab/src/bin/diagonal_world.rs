#[cfg(target_os = "macos")]
fn main() -> anyhow::Result<()> {
    graphics_lab::diagonal_app::run()
}
#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("diagonal_world requires macOS and a Metal-capable GPU");
    std::process::exit(1);
}
