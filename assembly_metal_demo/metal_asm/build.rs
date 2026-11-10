use std::{env, path::PathBuf, process::Command};

fn run(command: &mut Command) {
    let output = command
        .output()
        .expect("Cannot run Xcode Command Line Tools");
    assert!(
        output.status.success(),
        "Native build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn main() {
    assert!(
        env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos")
            && env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("aarch64"),
        "This example requires ARM64 macOS (Apple Silicon)."
    );
    let native = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("native");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let archive = output.join("libmetal_asm.a");

    let mut objects = Vec::new();
    for name in ["metal_add", "metal_cube"] {
        let object = output.join(format!("{name}.o"));
        run(Command::new("xcrun")
            .current_dir(&native)
            .args(["clang", "-arch", "arm64", "-c"])
            .arg(format!("{name}.S"))
            .arg("-o")
            .arg(&object));
        objects.push(object);
        println!("cargo:rerun-if-changed=native/{name}.S");
    }
    run(Command::new("xcrun")
        .args(["ar", "crs"])
        .arg(&archive)
        .args(&objects));

    println!("cargo:rerun-if-changed=native/vector_add.metal");
    println!("cargo:rerun-if-changed=native/cube.metal");
    println!("cargo:rustc-link-search=native={}", output.display());
    println!("cargo:rustc-link-lib=static=metal_asm");
    println!("cargo:rustc-link-lib=framework=Metal");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=QuartzCore");
    println!("cargo:rustc-link-lib=dylib=objc");
}
