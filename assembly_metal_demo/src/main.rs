#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod native_app;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    native_app::run()
}
#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
fn main() -> anyhow::Result<()> {
    graphics_lab::portable_cube::run()
}
