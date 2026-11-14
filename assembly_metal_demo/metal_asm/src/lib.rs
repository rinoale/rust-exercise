//! Rust interfaces to Metal graphics and compute implemented in ARM64 assembly.

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub mod cube;

use std::fmt;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
unsafe extern "C" {
    fn asm_metal_add(a: *const u32, b: *const u32, output: *mut u32, count: usize) -> i32;
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    DifferentLengths,
    TooManyElements,
    MetalStage(i32),
    UnsupportedPlatform,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                write!(f, "ARM64 Metal compute requires Apple Silicon macOS")
            }
            Self::DifferentLengths => write!(f, "Input vectors must have equal lengths"),
            Self::TooManyElements => {
                write!(f, "Input vectors exceed the supported byte/index range")
            }
            Self::MetalStage(stage) => {
                let detail = match stage {
                    1 => "invalid assembly arguments",
                    2 => "no Metal GPU",
                    3 => "shader source string allocation",
                    4 => "shader library compilation",
                    5 => "kernel function creation",
                    6 => "compute pipeline creation",
                    7 => "command queue creation",
                    8 => "GPU buffer allocation",
                    9 => "command buffer or encoder creation",
                    10 => "GPU command execution",
                    11 => "result buffer mapping",
                    _ => "unknown assembly status",
                };
                write!(f, "Assembly/Metal stage {stage} failed: {detail}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Add corresponding elements on the GPU, with wrapping unsigned arithmetic.
///
/// The assembly routine copies the inputs into shared Metal buffers and waits
/// for GPU completion before copying results into the returned Rust allocation.
pub fn add_u32(a: &[u32], b: &[u32]) -> Result<Vec<u32>, Error> {
    if a.len() != b.len() {
        return Err(Error::DifferentLengths);
    }
    if a.len() > u32::MAX as usize || a.len().checked_mul(size_of::<u32>()).is_none() {
        return Err(Error::TooManyElements);
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let mut result = vec![0; a.len()];
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        // SAFETY: All pointers refer to live, correctly sized u32 allocations. The
        // output is disjoint from the inputs; assembly waits before returning.
        let status = unsafe { asm_metal_add(a.as_ptr(), b.as_ptr(), result.as_mut_ptr(), a.len()) };
        if status != 0 {
            return Err(Error::MetalStage(status));
        }
        Ok(result)
    }
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    {
        if a.is_empty() {
            Ok(Vec::new())
        } else {
            Err(Error::UnsupportedPlatform)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    #[test]
    fn gpu_add_handles_partial_threadgroups_and_unsigned_overflow() {
        for length in [1, 7, 1031] {
            let a: Vec<_> = (0..length).map(|i| u32::MAX - i as u32).collect();
            let b: Vec<_> = (0..length).map(|i| i as u32 * 3 + 5).collect();
            let result = add_u32(&a, &b).unwrap();
            let expected: Vec<_> = a.iter().zip(&b).map(|(a, b)| a.wrapping_add(*b)).collect();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn empty_inputs_and_length_mismatches_are_handled() {
        assert_eq!(add_u32(&[], &[]).unwrap(), Vec::<u32>::new());
        assert_eq!(add_u32(&[1], &[]), Err(Error::DifferentLengths));
    }
}
