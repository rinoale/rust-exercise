//! Observe a separate reference process. Nothing here replays private driver calls.

use std::{collections::BTreeMap, fs, path::Path, process::Command};

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

pub fn record(output: &Path) -> Result<()> {
    let output = std::path::absolute(output)?;
    let parent = output.parent().context("Trace path has no parent")?;
    fs::create_dir_all(parent)?;
    let build = parent.join("tracer");
    fs::create_dir_all(&build)?;
    let source = build.join("iokit_trace.c");
    let dylib = build.join("libgraphics_lab_trace.dylib");
    fs::write(&source, include_str!("../../native/iokit_trace.c"))?;
    let status = Command::new("xcrun")
        .args([
            "clang",
            "-dynamiclib",
            "-std=c11",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-O2",
        ])
        .arg(&source)
        .args(["-framework", "IOKit", "-framework", "CoreFoundation", "-o"])
        .arg(&dylib)
        .status()
        .context("Cannot launch clang; install Apple's Command Line Tools")?;
    ensure!(status.success(), "Building the IOKit tracer failed");
    let reference = std::env::current_exe()?
        .parent()
        .context("Executable has no directory")?
        .join("metal_cube");
    ensure!(
        reference.is_file(),
        "Build the Metal reference first: cargo build --bin metal_cube"
    );
    let capture = output.with_extension("reference.png");
    println!("Tracing the separate Metal reference process (this is not independent rendering).");
    let mut injected = dylib.as_os_str().to_owned();
    if let Some(existing) = std::env::var_os("DYLD_INSERT_LIBRARIES").filter(|s| !s.is_empty()) {
        injected.push(":");
        injected.push(existing);
    }
    let status = Command::new(reference)
        .env("DYLD_INSERT_LIBRARIES", injected)
        .env("GRAPHICS_LAB_TRACE", &output)
        .arg("--capture")
        .arg(&capture)
        .args(["--time", "1.5", "--width", "800", "--height", "600"])
        .status()
        .context("Cannot launch the Metal reference")?;
    ensure!(status.success(), "The traced Metal reference failed");
    let contents = fs::read_to_string(&output)
        .context("No trace produced; dynamic interposition may be unavailable")?;
    let events: Vec<Value> = contents
        .lines()
        .map(serde_json::from_str)
        .collect::<std::result::Result<_, _>>()?;
    ensure!(
        !events.is_empty(),
        "Trace is empty; no GPU calls were intercepted"
    );
    let summary = summarize(&events);
    let summary_path = output.with_extension("summary.json");
    fs::write(
        &summary_path,
        format!("{}\n", serde_json::to_string_pretty(&summary)?),
    )?;
    println!("Captured {} events in {}", events.len(), output.display());
    println!("Selector/size summary: {}", summary_path.display());
    println!("Reference image: {}", capture.display());
    println!(
        "Payloads are truncated to 128 bytes and scalars to 16 entries. This is an ABI observation, not a replayable command stream."
    );
    Ok(())
}

pub fn summarize(events: &[Value]) -> Value {
    let opens: Vec<_> = events
        .iter()
        .filter(|e| e["kind"] == "open")
        .cloned()
        .collect();
    let mut methods = BTreeMap::<String, u64>::new();
    for event in events.iter().filter(|e| e.get("selector").is_some()) {
        let key = format!(
            "{} selector={} scalars_in={} bytes_in={} scalars_out={} bytes_out={} result={}",
            event["kind"].as_str().unwrap_or("?"),
            event["selector"],
            event["input_count"],
            event["input_size"],
            event["output_count"],
            event["output_size"],
            event["return_code"]
        );
        *methods.entry(key).or_default() += 1;
    }
    json!({
        "schema_version": 1,
        "source": "Separate Metal reference process; not the independent renderer",
        "events": events.len(),
        "gpu_client_opens": opens,
        "method_signatures": methods,
        "limitations": [
            "Only synchronous IOConnectCallMethod, StructMethod, ScalarMethod and GPU service open/close are hooked",
            "Only connections opened from an IOGPU service are tracked; auxiliary connections may be absent",
            "Async calls, shared memory contents, mappings, and direct Mach calls are not captured",
            "Input/output byte prefixes are capped at 128; scalar arrays at 16",
            "No private method's meaning or hardware compatibility is inferred from its selector alone",
            "Pointer values and handles are process-specific; this trace must not be replayed"
        ]
    })
}
