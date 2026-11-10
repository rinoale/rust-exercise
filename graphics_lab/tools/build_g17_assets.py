#!/usr/bin/env python3
"""Build native launch assets and our cube kernel; no Metal APIs or captures.

Run with AGXForge's Python dependencies and decoder tools installed:
  python tools/build_g17_assets.py artifacts/research/AGXForge
AGXForge is pinned below. Only offline generation needs Python/the compiler.
"""
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys

PIN = "c6a3b22c23f7668a6139b6194ba82ee27587da52"
root = Path(__file__).resolve().parents[1]
reference = Path(sys.argv[1]).resolve()
assert subprocess.check_output(["git", "-C", str(reference), "rev-parse", "HEAD"], text=True).strip() == PIN
sys.path[:0] = [str(reference), str(reference / "tools")]
from agxforge.g17 import cc, ir
import g17authoredblock640payload as prefix
import g17structuredbasepayload as base
import g17structuredgraphpayload as graph
import g17structuredpages as pages
import g17structuredrequests as requests

destination = root / "native" / "g17"
destination.mkdir(parents=True, exist_ok=True)

def function(name):
    buffers = [ir.Buffer(n, i + 1) for i, n in enumerate(("rays", "scene", "pixels"))]
    fn = ir.Function(name, buffers)
    return fn, ir.Builder(fn, fn.block("entry")), *buffers

def lane_index(b):
    lane = b.builtin("thread_index_in_simdgroup")
    group = b.builtin("threadgroup_position_in_grid")
    return b.add(b.mul(group, b.const(32)), lane)

def arithmetic():
    fn, b, a, uniforms, out = function("native_arithmetic")
    i = lane_index(b)
    b.store_at(out, i, b.add(b.mul(i, b.const(3)), b.const(1)))
    b.ret()
    return cc.compile_function(fn)

def cube():
    # One GPU lane intersects one perspective camera ray with a cube in object
    # coordinates. The host supplies inverse ray directions and six plane offsets.
    # The GPU computes entry/exit distances, visibility, and the visible face.
    fn, b, rays, uniforms, out = function("native_ray_cube")
    i = lane_index(b)
    ray_index = b.mul(i, b.const(3))
    zero = b.const(0)
    near = far = color = None
    for axis in range(3):
        inverse = b.load(rays, b.add(ray_index, b.const(axis)))
        lo = b.fmul(inverse, b.load(uniforms, b.const(2 * axis)))
        hi = b.fmul(inverse, b.load(uniforms, b.const(2 * axis + 1)))
        entry, leave = b.fmin(lo, hi), b.fmax(lo, hi)
        face_color = b.load(uniforms, b.const(6 + axis))
        if axis == 0:
            near, far, color = entry, leave, face_color
        else:
            # Nonnegative IEEE float bit patterns have unsigned numeric ordering.
            take = b.icmp(b.fmax(entry, zero), b.fmax(near, zero), rel="ugt")
            color = b.csel(take, zero, face_color, color, rel="gt")
            near, far = b.fmax(near, entry), b.fmin(far, leave)
    hit = b.icmp(b.fmax(far, zero), b.fmax(near, zero), rel="ugt")
    background = b.load(uniforms, b.const(9))
    b.store_at(out, i, b.csel(hit, zero, color, background, rel="gt"))
    b.ret()
    return cc.compile_function(fn)

programs = {"self_test.bin": arithmetic(), "cube.bin": cube()}
blobs = {name: bytes(program.code) for name, program in programs.items()}
blobs["allocations.bin"] = requests.serialize()
blobs["pages.bin"] = pages.serialize()
memory = bytearray()
for index, _, aperture, size in requests.physical_requests():
    raw = (prefix.allocation_zero(prefix_bytes=0x400) if index == 0 else
           base.allocation(index) if index in (1, 2) else graph.allocation(index))
    assert len(raw) == size
    memory.extend(raw)
blobs["memory.bin"] = bytes(memory)
for name, data in blobs.items():
    (destination / name).write_bytes(data)
(destination / "AGXForge-LICENSE.txt").write_bytes((reference / "LICENSE").read_bytes())
manifest = {
    "reference": "https://github.com/sbryngelson/AGXForge",
    "reference_commit": PIN,
    "generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    "python": sys.version,
    "platform": "M5 Pro / macOS 26.6.2 (25G83) / AGX 353.14",
    "generation": "Launch layouts constructed from AGXForge's measured fields; our kernels compiled from IR. No Metal capture or Metal API.",
    "files": {name: {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()} for name, data in blobs.items()},
    "programs": {name: program.abi_plain(program.abi()) for name, program in programs.items()},
}
(destination / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
for name, data in blobs.items():
    print(name, len(data), hashlib.sha256(data).hexdigest())
