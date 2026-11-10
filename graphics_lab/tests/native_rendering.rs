#![cfg(target_os = "macos")]

use glam::{DMat3, DVec3};
use std::{
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn capture(directory: &Path, name: &str, angle: f64) -> Vec<u8> {
    let path = directory.join(format!("{name}.png"));
    let run = Command::new(env!("CARGO_BIN_EXE_own_gpu"))
        .arg("--capture")
        .arg(&path)
        .args([
            "--size",
            "128",
            "--angle",
            &angle.to_string(),
            "--tilt",
            "0.35",
        ])
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "Native render failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(path.with_extension("json")).unwrap()).unwrap();
    assert_eq!(receipt["arithmetic_self_test_passed"], true);
    assert_eq!(receipt["pixels_written"], 128 * 128);
    assert_eq!(receipt["gpu_dispatches"], 17);
    assert!(
        receipt["loaded_metal_images"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| !p.as_str().unwrap().contains("AGXMetal"))
    );
    let mut reader = png::Decoder::new(fs::File::open(path).unwrap())
        .read_info()
        .unwrap();
    let mut pixels = vec![0; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut pixels).unwrap();
    assert_eq!(frame.color_type, png::ColorType::Rgba);
    pixels.truncate(frame.buffer_size());
    pixels
}

fn reference_hit(x: usize, y: usize, angle: f64) -> bool {
    // Independent double-precision slab oracle. The application only uses the
    // GPU-produced image; this CPU implementation exists exclusively in tests.
    let rotation = DMat3::from_rotation_x(-0.35) * DMat3::from_rotation_y(-angle);
    let origin = rotation * DVec3::new(0.0, 0.0, 5.0);
    let ray = rotation
        * DVec3::new(
            (((x as f64 + 0.5) / 128.0) * 2.0 - 1.0) * 0.55,
            (1.0 - ((y as f64 + 0.5) / 128.0) * 2.0) * 0.55,
            -1.0,
        );
    let mut near = 0.0_f64;
    let mut far = f64::INFINITY;
    for axis in 0..3 {
        let a = (-1.0 - origin[axis]) / ray[axis];
        let b = (1.0 - origin[axis]) / ray[axis];
        near = near.max(a.min(b));
        far = far.min(a.max(b));
    }
    far > near
}

#[test]
fn native_gpu_cube_matches_geometric_silhouette_and_rotates() {
    let build = Command::new("sw_vers")
        .arg("-buildVersion")
        .output()
        .unwrap();
    let supported = String::from_utf8_lossy(&build.stdout).trim() == "25G83"
        && graphics_lab::direct::discover().unwrap().iter().any(|gpu| {
            gpu.info
                .properties
                .get("model")
                .and_then(serde_json::Value::as_str)
                == Some("Apple M5 Pro")
                && gpu
                    .info
                    .properties
                    .get("IOSourceVersion")
                    .and_then(serde_json::Value::as_str)
                    == Some("353.14")
        });
    if !supported {
        eprintln!("Native dispatch test requires the measured M5 Pro / 25G83 / AGX 353.14");
        return;
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("native-cube-{}-{stamp}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let mut images = Vec::new();
    for (name, angle) in [("first", 0.65), ("rotated", 1.2)] {
        let pixels = capture(&directory, name, angle);
        let mut mismatch = 0;
        let mut coverage = 0;
        let mut colors = std::collections::BTreeSet::new();
        for (i, pixel) in pixels.chunks_exact(4).enumerate() {
            let hit = pixel[..3] != [0x14, 0x20, 0x32];
            coverage += usize::from(hit);
            mismatch += usize::from(hit != reference_hit(i % 128, i / 128, angle));
            colors.insert(pixel.to_vec());
        }
        assert!(
            coverage > 2000 && coverage < 8000,
            "Unexpected cube coverage: {coverage}"
        );
        assert!(
            mismatch < 20,
            "GPU silhouette differs from the geometric oracle in {mismatch} pixels"
        );
        let view = DMat3::from_rotation_x(-0.35) * DMat3::from_rotation_y(-angle);
        let eye = view * DVec3::new(0.0, 0.0, 5.0);
        let visible_faces = eye
            .to_array()
            .iter()
            .filter(|coordinate| coordinate.abs() > 1.0)
            .count();
        assert_eq!(
            colors.len(),
            visible_faces + 1,
            "{name}: expected a distinct color for each visible cube face plus background; got {colors:?}"
        );
        images.push(pixels);
    }
    let changes = images[0]
        .chunks_exact(4)
        .zip(images[1].chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        changes > 1000,
        "Rotation did not change the GPU-rendered image"
    );
    fs::remove_dir_all(directory).unwrap();
}
