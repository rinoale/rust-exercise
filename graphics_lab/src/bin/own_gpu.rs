#[cfg(target_os = "macos")]
fn main() -> anyhow::Result<()> {
    graphics_lab::direct::native_app::run()
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("own_gpu targets the macOS Apple Silicon AGX driver");
    std::process::exit(1);
}
