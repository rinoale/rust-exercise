//! Independent IOKit transport. This module does not import or call Metal.
//!
//! `g17` executes our compiled cube kernel through the private IOGPU transport.
//! The connection probe below remains a separate diagnostic mode.

pub mod g17;
pub mod native_app;
pub mod trace;

use std::{
    collections::BTreeMap,
    ffi::{CStr, c_char, c_void},
};

use anyhow::{Context, Result, anyhow, ensure};
use core_foundation::{
    array::CFArray,
    base::{CFType, CFTypeRef, TCFType},
    boolean::CFBoolean,
    dictionary::{CFDictionary, CFMutableDictionaryRef},
    number::CFNumber,
    string::{CFString, CFStringRef},
};
use serde::Serialize;
use serde_json::Value;

type IoObject = u32;

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOServiceMatching(name: *const c_char) -> CFMutableDictionaryRef;
    fn IOServiceGetMatchingServices(
        main_port: u32,
        matching: CFMutableDictionaryRef,
        iterator: *mut IoObject,
    ) -> i32;
    fn IOIteratorNext(iterator: IoObject) -> IoObject;
    fn IOObjectRelease(object: IoObject) -> i32;
    fn IOObjectGetClass(object: IoObject, class_name: *mut c_char) -> i32;
    fn IORegistryEntryGetRegistryEntryID(entry: IoObject, entry_id: *mut u64) -> i32;
    fn IORegistryEntryCreateCFProperty(
        entry: IoObject,
        key: CFStringRef,
        allocator: *const c_void,
        options: u32,
    ) -> CFTypeRef;
    fn IOServiceOpen(
        service: IoObject,
        owning_task: u32,
        client_type: u32,
        connection: *mut IoObject,
    ) -> i32;
    fn IOServiceClose(connection: IoObject) -> i32;
    fn IOConnectCallStructMethod(
        connection: IoObject,
        selector: u32,
        input: *const c_void,
        input_size: usize,
        output: *mut c_void,
        output_size: *mut usize,
    ) -> i32;
    fn IOConnectCallMethod(
        connection: IoObject,
        selector: u32,
        input: *const u64,
        input_count: u32,
        input_structure: *const c_void,
        input_size: usize,
        output: *mut u64,
        output_count: *mut u32,
        output_structure: *mut c_void,
        output_size: *mut usize,
    ) -> i32;
}

unsafe extern "C" {
    static mach_task_self_: u32;
    fn mach_error_string(code: i32) -> *const c_char;
}

struct Object(IoObject);

impl Drop for Object {
    fn drop(&mut self) {
        // This wrapper owns the send right returned by IOKit.
        unsafe {
            IOObjectRelease(self.0);
        }
    }
}

pub struct GpuService {
    object: Object,
    pub info: GpuInfo,
}

#[derive(Serialize)]
pub struct GpuInfo {
    pub registry_id: u64,
    pub driver_class: String,
    pub properties: BTreeMap<String, Value>,
}

#[derive(Serialize)]
pub struct Capabilities {
    pub registry_discovery: bool,
    pub driver_connection: bool,
}

#[derive(Serialize)]
pub struct ProbeReport {
    pub schema_version: u32,
    pub os: String,
    pub gpu: GpuInfo,
    pub abi_profile: &'static str,
    pub user_client_type: u32,
    pub open_return_code: String,
    pub open_message: String,
    pub capabilities: Capabilities,
    pub limitation: &'static str,
}

pub struct Device {
    connection: IoObject,
}

impl Drop for Device {
    fn drop(&mut self) {
        // IOServiceClose releases a connection; IOObjectRelease is for services/iterators.
        unsafe {
            IOServiceClose(self.connection);
        }
    }
}

impl Device {
    fn open(service: &GpuService, client_type: u32) -> std::result::Result<Self, i32> {
        let mut connection = 0;
        // This type was observed with the tracer, rather than guessed from old AGX code.
        // Opening a connection does not establish that any method ABI is compatible.
        let result = unsafe {
            IOServiceOpen(
                service.object.0,
                mach_task_self_,
                client_type,
                &mut connection,
            )
        };
        if result == 0 {
            Ok(Self { connection })
        } else {
            Err(result)
        }
    }
}

fn property_to_json(value: &CFType) -> Value {
    if let Some(string) = value.downcast::<CFString>() {
        return Value::String(string.to_string());
    }
    if let Some(number) = value.downcast::<CFNumber>() {
        return number.to_i64().map_or(Value::Null, |n| n.into());
    }
    if let Some(boolean) = value.downcast::<CFBoolean>() {
        return bool::from(boolean).into();
    }
    if let Some(array) = value.downcast::<CFArray>() {
        return Value::Array(
            array
                .get_all_values()
                .into_iter()
                .map(|item| {
                    // IORegistry property arrays contain retained CF property-list values.
                    property_to_json(&unsafe { CFType::wrap_under_get_rule(item) })
                })
                .collect(),
        );
    }
    if let Some(dictionary) = value.downcast::<CFDictionary>() {
        let (keys, values) = dictionary.get_keys_and_values();
        let mut result = serde_json::Map::new();
        for (key, value) in keys.into_iter().zip(values) {
            // The dictionary retains its entries for the duration of this loop.
            let key = unsafe { CFType::wrap_under_get_rule(key) };
            let value = unsafe { CFType::wrap_under_get_rule(value) };
            if let Some(key) = key.downcast::<CFString>() {
                result.insert(key.to_string(), property_to_json(&value));
            }
        }
        return Value::Object(result);
    }
    Value::Null
}

fn property(service: IoObject, name: &str) -> Option<Value> {
    let key = CFString::new(name);
    let value = unsafe {
        IORegistryEntryCreateCFProperty(service, key.as_concrete_TypeRef(), std::ptr::null(), 0)
    };
    if value.is_null() {
        return None;
    }
    // CreateCFProperty returns an owned Core Foundation object.
    Some(property_to_json(&unsafe {
        CFType::wrap_under_create_rule(value)
    }))
}

pub fn discover() -> Result<Vec<GpuService>> {
    let matching = unsafe { IOServiceMatching(c"IOGPU".as_ptr()) };
    ensure!(
        !matching.is_null(),
        "IOServiceMatching(IOGPU) returned null"
    );
    let mut iterator = 0;
    // GetMatchingServices consumes the matching dictionary, including on failure.
    let result = unsafe { IOServiceGetMatchingServices(0, matching, &mut iterator) };
    ensure!(
        result == 0,
        "GPU discovery failed: {}",
        error_message(result)
    );
    let iterator = Object(iterator);
    let mut services = Vec::new();
    loop {
        let next = unsafe { IOIteratorNext(iterator.0) };
        if next == 0 {
            break;
        }
        let object = Object(next);
        let mut name = [0_i8; 128];
        let result = unsafe { IOObjectGetClass(next, name.as_mut_ptr()) };
        ensure!(
            result == 0,
            "Cannot read GPU class: {}",
            error_message(result)
        );
        let driver_class = unsafe { CStr::from_ptr(name.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        let mut registry_id = 0;
        let result = unsafe { IORegistryEntryGetRegistryEntryID(next, &mut registry_id) };
        ensure!(
            result == 0,
            "Cannot read GPU registry ID: {}",
            error_message(result)
        );
        let properties = [
            "model",
            "IOClass",
            "CFBundleIdentifier",
            "IOSourceVersion",
            "gpu-core-count",
            "GPUConfigurationVariable",
            "MetalPluginName",
            "MetalPluginClassName",
            "CommandSubmissionEnabled",
        ]
        .into_iter()
        .filter_map(|key| property(next, key).map(|value| (key.to_owned(), value)))
        .collect();
        services.push(GpuService {
            object,
            info: GpuInfo {
                registry_id,
                driver_class,
                properties,
            },
        });
    }
    Ok(services)
}

pub fn error_message(code: i32) -> String {
    let pointer = unsafe { mach_error_string(code) };
    if pointer.is_null() {
        format!("IOKit error 0x{:08x}", code as u32)
    } else {
        unsafe { CStr::from_ptr(pointer) }
            .to_string_lossy()
            .into_owned()
    }
}

pub fn probe() -> Result<(ProbeReport, Option<Device>)> {
    let services = discover()?;
    let service = services
        .into_iter()
        .find(|s| s.info.driver_class.starts_with("AGX"))
        .ok_or_else(|| {
            anyhow!("No Apple AGX GPU service found; this experiment targets Apple Silicon")
        })?;
    let os = String::from_utf8_lossy(
        &std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()?
            .stdout,
    )
    .trim()
    .to_owned();
    // A new driver/OS requires a fresh reference trace, not guessed selectors.
    let profile = observed_profile(&os, &service.info).context(
        "No observed ABI profile for this OS/GPU driver. Use --trace first; update the profile only after checking its successful GPU open event.",
    )?;
    let (device, code) = match Device::open(&service, profile.client_type) {
        Ok(d) => (Some(d), 0),
        Err(code) => (None, code),
    };
    let report = ProbeReport {
        schema_version: 2,
        os,
        gpu: service.info,
        abi_profile: profile.name,
        user_client_type: profile.client_type,
        open_return_code: format!("0x{:08x}", code as u32),
        open_message: error_message(code),
        capabilities: Capabilities {
            registry_discovery: true,
            driver_connection: device.is_some(),
        },
        limitation: "This is the connection diagnostic mode. Run own_gpu without --probe to render the native GPU cube.",
    };
    Ok((report, device))
}

struct AbiProfile {
    name: &'static str,
    client_type: u32,
}

fn observed_profile(os: &str, gpu: &GpuInfo) -> Option<AbiProfile> {
    (os == "26.6.2"
        && gpu.driver_class == "AGXAcceleratorG17X"
        && gpu
            .properties
            .get("IOSourceVersion")
            .and_then(Value::as_str)
            == Some("353.14")
        && gpu.properties.get("model").and_then(Value::as_str) == Some("Apple M5 Pro"))
    .then_some(AbiProfile {
        name: "M5 Pro / macOS 26.6.2 / AGX 353.14",
        client_type: 0x100005,
    })
}
