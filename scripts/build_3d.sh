#!/usr/bin/env bash
# Run from any directory. Extra arguments (e.g. --release) go to both builds.
set -euo pipefail
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cargo build --locked --manifest-path "$repo_root/graphics_lab/Cargo.toml" --bins "$@"
cargo build --locked --manifest-path "$repo_root/assembly_metal_demo/Cargo.toml" --workspace --bins "$@"
