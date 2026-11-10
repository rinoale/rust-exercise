#![cfg(all(target_os = "macos", feature = "metal-app"))]

use std::{
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn capture(directory: &Path, name: &str, flags: &[&str]) -> Vec<u8> {
    let path = directory.join(format!("{name}.png"));
    let output = Command::new(env!("CARGO_BIN_EXE_metal_cube"))
        .arg("--capture")
        .arg(&path)
        .args(["--width", "320", "--height", "240"])
        .args(flags)
        .output()
        .expect("launch Metal renderer");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let decoder = png::Decoder::new(fs::File::open(path).unwrap());
    let mut reader = decoder.read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut pixels).unwrap();
    assert_eq!(frame.color_type, png::ColorType::Rgba);
    pixels.truncate(frame.buffer_size());
    pixels
}

#[test]
fn real_gpu_draws_both_cubes_and_responds_to_rendering_controls() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("graphics-lab-{}-{stamp}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let baseline = capture(&directory, "baseline", &[]);
    let orange_pixels = baseline
        .chunks_exact(4)
        .filter(|p| p[0] > 80 && p[0] as f32 > p[1] as f32 * 1.3 && p[1] as f32 > p[2] as f32 * 1.1)
        .count();
    let blue_pixels = baseline
        .chunks_exact(4)
        .filter(|p| {
            p[2] > 100 && p[2] as f32 > p[0] as f32 * 1.4 && p[1] as f32 > p[0] as f32 * 1.2
        })
        .count();
    assert!(
        orange_pixels > 200,
        "orange cube should be visible, found {orange_pixels} pixels"
    );
    assert!(
        blue_pixels > 500,
        "blue cube should be visible, found {blue_pixels} pixels"
    );

    for (name, flags) in [
        ("depth-disabled", vec!["--no-depth"]),
        ("texture-disabled", vec!["--no-texture"]),
        ("wireframe", vec!["--wireframe"]),
        ("animated", vec!["--time", "3.0"]),
    ] {
        let changed = capture(&directory, name, &flags);
        let changed_pixels = baseline
            .chunks_exact(4)
            .zip(changed.chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        assert!(
            changed_pixels > 200,
            "{name} should affect the GPU image; only {changed_pixels} pixels changed"
        );
    }
    fs::remove_dir_all(directory).unwrap();
}
