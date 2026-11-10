//! Experimental native G17 executor. No Metal API, shader compiler, or capture.
//!
//! Ported from the measured IOGPU launch recipe in AGXForge (MIT, Spencer
//! Bryngelson, 2026). See native/g17/AGXForge-LICENSE.txt and manifest.json.
//! Opaque layouts are deliberately restricted to the measured platform.

use super::*;
use anyhow::{Context, Result, ensure};
use block2::RcBlock;
use glam::{Mat3, Vec3};
use serde::{Deserialize, Serialize};
use std::{
    ffi::{CStr, c_char, c_void},
    ptr,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

const BASE: u64 = 0x10000000000;
const CODE_OFFSET: usize = 0x6c0;
const INPUT_OFFSET: usize = 0x1000;
const UNIFORM_OFFSET: usize = 0x6000;
const OUTPUT_OFFSET: usize = 0x9000;
const TILE: usize = 1024;
const MAX_DISPATCHES: u32 = 65536;
const ALLOCATIONS: &[u8] = include_bytes!("../../native/g17/allocations.bin");
const MEMORY: &[u8] = include_bytes!("../../native/g17/memory.bin");
const PAGES: &[u8] = include_bytes!("../../native/g17/pages.bin");
const SELF_TEST: &[u8] = include_bytes!("../../native/g17/self_test.bin");
const CUBE: &[u8] = include_bytes!("../../native/g17/cube.bin");

unsafe extern "C" {
    fn dlopen(path: *const c_char, mode: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> i32;
    fn dispatch_queue_create(label: *const c_char, attributes: *const c_void) -> *mut c_void;
    fn dispatch_release(object: *mut c_void);
    fn _dyld_image_count() -> u32;
    fn _dyld_get_image_name(index: u32) -> *const c_char;
    fn mach_vm_allocate(task: u32, address: *mut u64, size: u64, flags: i32) -> i32;
    fn mach_vm_deallocate(task: u32, address: u64, size: u64) -> i32;
}

type DevCreate = unsafe extern "C" fn(u32) -> *mut c_void;
type Release = unsafe extern "C" fn(*mut c_void);
type QueueCreate = unsafe extern "C" fn(*mut c_void, *const u8, usize) -> *mut c_void;
type NotifySet = unsafe extern "C" fn(*mut c_void, *mut c_void, u32) -> *mut c_void;
type NextId = unsafe extern "C" fn(*mut c_void) -> u64;
type Submit = unsafe extern "C" fn(*mut c_void, *mut c_void, u32, *const u8, u32, *mut u8) -> i32;

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct Shared {
    cpu: u64,
    size: u32,
    id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Receipt {
    pub backend: String,
    pub gpu: String,
    pub os_build: String,
    pub gpu_dispatches: u32,
    pub arithmetic_self_test_passed: bool,
    pub pixels_written: usize,
    pub elapsed_ms: f64,
    pub loaded_metal_images: Vec<String>,
    pub rendering: String,
}

/// Owned by the main thread. Completion blocks publish through atomics only.
pub struct Renderer {
    library: *mut c_void,
    device: *mut c_void,
    queue: *mut c_void,
    dispatch: *mut c_void,
    connection: u32,
    old_connection: u32,
    reservations: Vec<u64>,
    mapped: [*mut u8; 29],
    shared: [Shared; 2],
    next_id: NextId,
    submit: Submit,
    device_release: Release,
    queue_release: Release,
    pub dispatches: u32,
    pub self_test_passed: bool,
}

fn u32_at(b: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(b[offset..offset + 4].try_into().unwrap())
}
fn u64_at(b: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(b[offset..offset + 8].try_into().unwrap())
}

unsafe fn symbol<T: Copy>(library: *mut c_void, name: &CStr) -> Result<T> {
    let p = unsafe { dlsym(library, name.as_ptr()) };
    ensure!(
        !p.is_null(),
        "IOGPU symbol {} is absent",
        name.to_string_lossy()
    );
    ensure!(
        size_of::<T>() == size_of::<*mut c_void>(),
        "invalid function pointer type"
    );
    // All callers use the measured C function signatures above.
    Ok(unsafe { std::mem::transmute_copy(&p) })
}

pub fn metal_images() -> Vec<String> {
    let mut result = Vec::new();
    for i in 0..unsafe { _dyld_image_count() } {
        let p = unsafe { _dyld_get_image_name(i) };
        if !p.is_null() {
            let path = unsafe { CStr::from_ptr(p) }.to_string_lossy();
            if path.contains("/Metal.framework/") || path.contains("AGXMetal") {
                result.push(path.into_owned());
            }
        }
    }
    result
}

fn no_metal_driver() -> Result<()> {
    ensure!(
        !metal_images().iter().any(|p| p.contains("AGXMetal")),
        "An AGXMetal driver is loaded; independent execution cannot be claimed"
    );
    Ok(())
}

impl Renderer {
    pub fn new() -> Result<Self> {
        let build = std::process::Command::new("sw_vers")
            .arg("-buildVersion")
            .output()?;
        ensure!(
            String::from_utf8_lossy(&build.stdout).trim() == "25G83",
            "This measured native command layout requires macOS build 25G83"
        );
        let service = discover()?
            .into_iter()
            .find(|s| {
                s.info.driver_class == "AGXAcceleratorG17X"
                    && s.info.properties.get("model").and_then(Value::as_str)
                        == Some("Apple M5 Pro")
                    && s.info
                        .properties
                        .get("IOSourceVersion")
                        .and_then(Value::as_str)
                        == Some("353.14")
            })
            .context("Native rendering requires the measured M5 Pro / AGX 353.14")?;
        no_metal_driver()?;
        // Load only the lower driver transport. No MTLDevice or AGXMetal plugin.
        let library = unsafe {
            dlopen(
                c"/System/Library/PrivateFrameworks/IOGPU.framework/IOGPU".as_ptr(),
                2,
            )
        };
        ensure!(!library.is_null(), "Cannot load IOGPU");
        let create: DevCreate = unsafe { symbol(library, c"IOGPUDeviceCreate")? };
        let device_release = unsafe { symbol(library, c"IOGPUDeviceRelease")? };
        let queue_release = unsafe { symbol(library, c"IOGPUCommandQueueRelease")? };
        let next_id = unsafe { symbol(library, c"IOGPUDeviceGetNextGlobalTraceID")? };
        let submit = unsafe { symbol(library, c"IOGPUCommandQueueSubmitCommandBuffers")? };
        let device = unsafe { create(service.object.0) };
        ensure!(!device.is_null(), "IOGPUDeviceCreate failed");
        let mut renderer = Self {
            library,
            device,
            queue: ptr::null_mut(),
            dispatch: ptr::null_mut(),
            connection: 0,
            old_connection: unsafe {
                ptr::read_unaligned(device.cast::<u8>().add(0x14).cast::<u32>())
            },
            reservations: Vec::new(),
            mapped: [ptr::null_mut(); 29],
            shared: [Shared::default(); 2],
            next_id,
            submit,
            device_release,
            queue_release,
            dispatches: 0,
            self_test_passed: false,
        };
        let k = unsafe {
            IOServiceOpen(
                service.object.0,
                mach_task_self_,
                0x100005,
                &mut renderer.connection,
            )
        };
        ensure!(k == 0, "Native device open failed: {}", error_message(k));
        // Keep the framework's queue and our direct calls on the same device client.
        unsafe {
            ptr::write_unaligned(
                device.cast::<u8>().add(0x14).cast::<u32>(),
                renderer.connection,
            );
        }
        for _ in 0..4 {
            let mut address = 0;
            let k = unsafe { mach_vm_allocate(mach_task_self_, &mut address, 0x400000, 1) };
            ensure!(k == 0, "Native VA reservation failed: {}", error_message(k));
            renderer.reservations.push(address);
        }
        for (selector, size) in [(2, 536), (0, 64), (5, 32), (258, 456), (256, 152)] {
            renderer.struct_call(selector, &[], size)?;
        }
        let mut context = [0_u8; 72];
        context[0] = 11;
        context[46] = 2;
        renderer.method(261, &[], &context, 72)?;
        renderer.allocate()?;
        renderer.create_queue()?;
        renderer.install_program(SELF_TEST)?;
        let arithmetic = renderer.execute(TILE)?;
        ensure!(
            arithmetic
                .iter()
                .enumerate()
                .all(|(i, value)| *value == i as u32 * 3 + 1),
            "Native GPU arithmetic self-test failed"
        );
        renderer.self_test_passed = true;
        renderer.install_program(CUBE)?;
        no_metal_driver()?;
        Ok(renderer)
    }

    fn struct_call(&self, selector: u32, input: &[u8], size: usize) -> Result<Vec<u8>> {
        let mut output = vec![0_u8; size];
        let mut count = size;
        let k = unsafe {
            IOConnectCallStructMethod(
                self.connection,
                selector,
                if input.is_empty() {
                    ptr::null()
                } else {
                    input.as_ptr().cast()
                },
                input.len(),
                output.as_mut_ptr().cast(),
                &mut count,
            )
        };
        ensure!(
            k == 0 && count == size,
            "Native selector {selector}: status={} size={count}, expected {size}",
            error_message(k)
        );
        Ok(output)
    }

    fn method(&self, selector: u32, scalars: &[u64], input: &[u8], size: usize) -> Result<Vec<u8>> {
        let mut output = vec![0_u8; size];
        let mut count = size;
        let k = unsafe {
            IOConnectCallMethod(
                self.connection,
                selector,
                if scalars.is_empty() {
                    ptr::null()
                } else {
                    scalars.as_ptr()
                },
                scalars.len() as u32,
                if input.is_empty() {
                    ptr::null()
                } else {
                    input.as_ptr().cast()
                },
                input.len(),
                ptr::null_mut(),
                ptr::null_mut(),
                output.as_mut_ptr().cast(),
                &mut count,
            )
        };
        ensure!(
            k == 0 && count == size,
            "Native selector {selector}: status={} size={count}, expected {size}",
            error_message(k)
        );
        Ok(output)
    }

    fn allocate(&mut self) -> Result<()> {
        let mut payload_offset = 0;
        for row in ALLOCATIONS.chunks_exact(144) {
            let index = u32_at(row, 0) as usize;
            let kind = u32_at(row, 4);
            let parent = u32_at(row, 8) as usize;
            let offset = u64_at(row, 16) as usize;
            let address = u64_at(row, 24);
            let size = u64_at(row, 32) as usize;
            let mut request = row[40..].to_vec();
            if kind == 1 {
                ensure!(
                    parent < index && !self.mapped[parent].is_null(),
                    "Invalid native parent"
                );
                let p = self.mapped[parent] as u64;
                request[56..64].copy_from_slice(&(p + offset as u64).to_le_bytes());
                request[64..72].copy_from_slice(&p.to_le_bytes());
            }
            let output = self.method(9, &[], &request, 88)?;
            ensure!(
                u64_at(&output, 0) == address && u64_at(&output, 40) == size as u64,
                "Native allocation {index} layout differs from measured addresses"
            );
            if kind == 0 {
                let p = u64_at(&output, 8) as *mut u8;
                ensure!(!p.is_null(), "Native allocation {index} has no CPU mapping");
                ensure!(
                    payload_offset + size <= MEMORY.len(),
                    "Truncated native payload"
                );
                self.mapped[index] = p;
                unsafe {
                    ptr::copy_nonoverlapping(MEMORY[payload_offset..].as_ptr(), p, size);
                }
                payload_offset += size;
            } else if kind == 1 {
                ensure!(
                    u64_at(&output, 8) == 0 && u64_at(&output, 16) != 0,
                    "Native view {index} has unexpected fields"
                );
            }
        }
        ensure!(
            payload_offset == MEMORY.len(),
            "Native payload size mismatch"
        );
        let bindings = [
            BASE + 0x58000 + INPUT_OFFSET as u64,
            BASE + 0x58000 + UNIFORM_OFFSET as u64,
            BASE + 0x58000 + OUTPUT_OFFSET as u64,
        ];
        for (i, value) in bindings.iter().enumerate() {
            self.write(28, 0x1ba0 + i * 8, &value.to_le_bytes());
        }
        Ok(())
    }

    fn create_queue(&mut self) -> Result<()> {
        let create: QueueCreate = unsafe { symbol(self.library, c"IOGPUCommandQueueCreate")? };
        let notify: NotifySet =
            unsafe { symbol(self.library, c"IOGPUNotificationQueueSetDispatchQueue")? };
        let mut descriptor = [0_u8; 0x410];
        // A short application label fits both name slots in the observed descriptor.
        descriptor[..9].copy_from_slice(b"own_gpu\0\0");
        descriptor[0x3e3..0x3eb].copy_from_slice(b"own_gpu\0");
        descriptor[0x400..0x404].copy_from_slice(&2_u32.to_le_bytes());
        descriptor[0x408..0x40c].copy_from_slice(&u32::MAX.to_le_bytes());
        descriptor[0x40c..].copy_from_slice(&1_u32.to_le_bytes());
        self.queue = unsafe { create(self.device, descriptor.as_ptr(), descriptor.len()) };
        ensure!(!self.queue.is_null(), "Native queue creation failed");
        let notification =
            unsafe { ptr::read_unaligned(self.queue.cast::<u8>().add(0x28).cast::<*mut c_void>()) };
        ensure!(
            !notification.is_null(),
            "Native notification queue is absent"
        );
        self.dispatch =
            unsafe { dispatch_queue_create(c"rust.graphics.native".as_ptr(), ptr::null()) };
        ensure!(
            !self.dispatch.is_null(),
            "Cannot create completion dispatch queue"
        );
        ensure!(
            !unsafe { notify(notification, self.dispatch, 1) }.is_null(),
            "Cannot bind native notification queue"
        );
        self.struct_call(6, &[], 16)?;
        for i in 0..2 {
            let result = self.method(14, &[0x4000, i as u64], &[], 16)?;
            self.shared[i] = Shared {
                cpu: u64_at(&result, 0),
                size: u32_at(&result, 8),
                id: u32_at(&result, 12),
            };
            ensure!(
                self.shared[i].cpu != 0
                    && self.shared[i].size == 0x4000
                    && self.shared[i].id == i as u32 + 1,
                "Unexpected native command pages"
            );
        }
        unsafe {
            ptr::copy_nonoverlapping(PAGES.as_ptr(), self.shared[1].cpu as *mut u8, 0x4000);
            ptr::copy_nonoverlapping(
                PAGES[0x4000..].as_ptr(),
                self.shared[0].cpu as *mut u8,
                0x4000,
            );
        }
        Ok(())
    }

    fn write(&self, allocation: usize, offset: usize, bytes: &[u8]) {
        // All offsets are fixed and bounded by this executor's measured allocations.
        unsafe {
            ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                self.mapped[allocation].add(offset),
                bytes.len(),
            );
        }
    }

    fn install_program(&self, code: &[u8]) -> Result<()> {
        ensure!(
            !code.is_empty() && code.len() < 0x10000 - CODE_OFFSET,
            "Native shader exceeds its code allocation"
        );
        self.write(0, CODE_OFFSET, code);
        Ok(())
    }

    fn execute(&mut self, count: usize) -> Result<Vec<u32>> {
        ensure!(
            count == TILE && self.dispatches < MAX_DISPATCHES,
            "Native dispatch bound exceeded"
        );
        self.write(22, 0xa8, &(count as u32).to_le_bytes());
        self.write(22, 0xac, &1_u32.to_le_bytes());
        self.write(22, 0xb0, &1_u32.to_le_bytes());
        self.write(22, 0xb4, &32_u32.to_le_bytes());
        self.write(25, 0x10, &(count as u32).to_le_bytes());
        self.write(25, 0x1c, &32_u32.to_le_bytes());
        let output = unsafe { self.mapped[20].add(OUTPUT_OFFSET).cast::<u32>() };
        unsafe {
            ptr::write_bytes(self.mapped[20].add(OUTPUT_OFFSET - 256), 0xa5, 256);
            ptr::write_bytes(self.mapped[20].add(OUTPUT_OFFSET + count * 4), 0xa5, 256);
        }
        for i in 0..count {
            unsafe {
                ptr::write_volatile(output.add(i), 0xdeadbeef);
            }
        }
        let first = unsafe { (self.next_id)(self.device) };
        let second = unsafe { (self.next_id)(self.device) };
        ensure!(
            first != 0 && second == first + 1,
            "Native trace IDs are invalid"
        );
        unsafe {
            ptr::copy_nonoverlapping(
                (second as u32).to_le_bytes().as_ptr(),
                (self.shared[1].cpu + 0x234) as *mut u8,
                4,
            );
            for (offset, value) in [(0, first), (0x18, first), (0x28, second)] {
                ptr::copy_nonoverlapping(
                    value.to_le_bytes().as_ptr(),
                    (self.shared[0].cpu + offset) as *mut u8,
                    8,
                );
            }
            ptr::write_volatile((self.shared[0].cpu + 0x24) as *mut u32, 0x800000f0);
        }
        let scheduled_count = Arc::new(AtomicU32::new(0));
        let completed_count = Arc::new(AtomicU32::new(0));
        let s = scheduled_count.clone();
        let c = completed_count.clone();
        let scheduled = RcBlock::new(move || {
            s.fetch_add(1, Ordering::Release);
        });
        let completed = RcBlock::new(move || {
            c.fetch_add(1, Ordering::Release);
        });
        let mut record = [0_u8; 64];
        let mut result = [0_u8; 64];
        record[..4].copy_from_slice(&self.shared[1].id.to_le_bytes());
        record[4..8].copy_from_slice(&self.shared[0].id.to_le_bytes());
        record[0x10..0x18]
            .copy_from_slice(&((&*scheduled as *const _ as *const c_void) as u64).to_le_bytes());
        record[0x18..0x20]
            .copy_from_slice(&((&*completed as *const _ as *const c_void) as u64).to_le_bytes());
        self.dispatches += 1;
        let status = unsafe {
            (self.submit)(
                self.queue,
                ptr::null_mut(),
                1,
                record.as_ptr(),
                64,
                result.as_mut_ptr(),
            )
        };
        // Follow the reference's ownership rule: these private callback records are
        // not released/reused without a verified ownership ABI. Total dispatches are bounded.
        std::mem::forget(scheduled);
        std::mem::forget(completed);
        ensure!(
            status == 0 && u32_at(&result, 0) == 0,
            "Native Submit failed: {status}"
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        while completed_count.load(Ordering::Acquire) == 0 {
            if Instant::now() >= deadline {
                // Do not free mappings that could still be in use by an unretired GPU job.
                eprintln!("Native GPU completion timed out; terminating the client");
                std::process::exit(75);
            }
            thread::sleep(Duration::from_micros(100));
        }
        ensure!(
            scheduled_count.load(Ordering::Acquire) == 1
                && completed_count.load(Ordering::Acquire) == 1,
            "Native completion count mismatch"
        );
        let values: Vec<_> = (0..count)
            .map(|i| unsafe { ptr::read_volatile(output.add(i)) })
            .collect();
        ensure!(
            !values.contains(&0xdeadbeef),
            "Native shader left unwritten pixels"
        );
        let guards_intact = unsafe {
            std::slice::from_raw_parts(self.mapped[20].add(OUTPUT_OFFSET - 256), 256)
                .iter()
                .chain(std::slice::from_raw_parts(
                    self.mapped[20].add(OUTPUT_OFFSET + count * 4),
                    256,
                ))
                .all(|byte| *byte == 0xa5)
        };
        ensure!(
            guards_intact,
            "Native shader wrote outside its output region"
        );
        no_metal_driver()?;
        Ok(values)
    }

    pub fn render(&mut self, size: u32, angle: f32, tilt: f32) -> Result<(Vec<u32>, Receipt)> {
        ensure!(
            matches!(size, 128 | 256),
            "Native image size must be 128 or 256"
        );
        let start = Instant::now();
        let camera = Mat3::from_rotation_x(-tilt) * Mat3::from_rotation_y(-angle);
        let origin = camera * Vec3::new(0.0, 0.0, 5.0);
        let light = camera * Vec3::new(-0.45, 0.8, 0.65).normalize();
        let mut scene = Vec::new();
        for coordinate in origin.to_array() {
            scene.extend([(-1.0 - coordinate).to_bits(), (1.0 - coordinate).to_bits()]);
        }
        for axis in 0..3 {
            let diffuse = (light[axis] * origin[axis].signum()).max(0.0);
            let brightness = 0.32 + diffuse * 0.68;
            let r = (245.0 * brightness) as u32;
            let g = (153.0 * brightness) as u32;
            let b = (65.0 * brightness) as u32;
            scene.push(0xff000000 | (r << 16) | (g << 8) | b);
        }
        scene.push(0xff142032);
        self.write(20, UNIFORM_OFFSET, bytemuck::cast_slice(&scene));
        let length = size as usize * size as usize;
        let mut pixels = Vec::with_capacity(length);
        for offset in (0..length).step_by(TILE) {
            let mut rays = Vec::with_capacity(TILE * 3);
            for i in offset..offset + TILE {
                let x = ((i % size as usize) as f32 + 0.5) / size as f32 * 2.0 - 1.0;
                let y = 1.0 - ((i / size as usize) as f32 + 0.5) / size as f32 * 2.0;
                let direction = camera * Vec3::new(x * 0.55, y * 0.55, -1.0);
                for coordinate in direction.to_array() {
                    rays.push(
                        1.0 / if coordinate.abs() < 1e-8 {
                            1e-8_f32.copysign(coordinate)
                        } else {
                            coordinate
                        },
                    );
                }
            }
            self.write(20, INPUT_OFFSET, bytemuck::cast_slice(&rays));
            pixels.extend(self.execute(TILE)?);
        }
        let receipt = Receipt {
            backend: "Rust / native G17 machine code / private IOGPU transport".into(),
            gpu: "Apple M5 Pro".into(), os_build: "25G83".into(), gpu_dispatches: self.dispatches,
            arithmetic_self_test_passed: self.self_test_passed, pixels_written: pixels.len(),
            elapsed_ms: start.elapsed().as_secs_f64()*1000.0, loaded_metal_images: metal_images(),
            rendering: "GPU ray/cube intersection, visibility and face selection; CPU camera rays and lighting constants".into(),
        };
        Ok((pixels, receipt))
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            if !self.queue.is_null() {
                (self.queue_release)(self.queue);
            }
            if !self.device.is_null() {
                ptr::write_unaligned(
                    self.device.cast::<u8>().add(0x14).cast::<u32>(),
                    self.old_connection,
                );
                (self.device_release)(self.device);
            }
            if self.connection != 0 {
                IOServiceClose(self.connection);
            }
            if !self.dispatch.is_null() {
                dispatch_release(self.dispatch);
            }
            for address in &self.reservations {
                mach_vm_deallocate(mach_task_self_, *address, 0x400000);
            }
            dlclose(self.library);
        }
    }
}
