#[cfg(target_os = "macos")]
fn main() -> anyhow::Result<()> {
    graphics_lab::direct::native_app::run()
}

#[cfg(not(target_os = "macos"))]
fn main() -> anyhow::Result<()> {
    graphics_lab::portable_cube::run()
}
