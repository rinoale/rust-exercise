//! CPU assembly owns every Metal rendering call; Rust owns this context's storage.

use std::{ffi::c_void, fmt, marker::PhantomData, rc::Rc};

unsafe extern "C" {
    fn asm_cube_init(context: *mut usize, ns_view: *mut c_void) -> i32;
    fn asm_cube_frame(
        context: *mut usize,
        width: usize,
        height: usize,
        bgra: *mut u8,
        angle: f32,
    ) -> i32;
    fn asm_cube_destroy(context: *mut usize);
}

#[derive(Debug)]
pub struct CubeError(i32);

impl fmt::Display for CubeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let detail = match self.0 {
            1 => "no Metal GPU",
            2 => "shader compilation or function lookup",
            3 => "pipeline, depth state, queue, or layer creation",
            4 => "render texture allocation",
            5 => "render command encoding",
            6 => "GPU command execution",
            7 => "size must be 1..=4096 and angle must be finite",
            8 => "capture requires an offscreen renderer",
            _ => "unknown error",
        };
        write!(f, "Assembly cube stage {} failed: {detail}", self.0)
    }
}

impl std::error::Error for CubeError {}

pub struct Cube {
    // Assembly layout: device, library, vertex function, fragment function,
    // pipeline, queue, depth state, layer, depth texture, color texture,
    // width, height, borrowed NSView. All slots are 64 bits on this target.
    context: Box<[usize; 13]>,
    // AppKit and the attached layer must remain on their creating thread.
    _main_thread: PhantomData<Rc<()>>,
}

impl Cube {
    pub fn offscreen() -> Result<Self, CubeError> {
        // SAFETY: null selects the offscreen path without accessing AppKit.
        unsafe { Self::for_ns_view(std::ptr::null_mut()) }
    }

    /// Attach a Metal layer to a window's NSView.
    ///
    /// # Safety
    /// A non-null view must be a valid NSView on the main thread, and it must
    /// remain alive until this Cube is dropped. Calls and drop stay on this thread.
    pub unsafe fn for_ns_view(ns_view: *mut c_void) -> Result<Self, CubeError> {
        let mut cube = Self {
            context: Box::new([0; 13]),
            _main_thread: PhantomData,
        };
        // SAFETY: writable zeroed context matches the documented assembly layout;
        // the caller guarantees the borrowed view's lifetime and thread.
        let status = unsafe { asm_cube_init(cube.context.as_mut_ptr(), ns_view) };
        if status != 0 {
            return Err(CubeError(status));
        }
        Ok(cube)
    }

    pub fn render(&mut self, width: u32, height: u32, angle: f32) -> Result<(), CubeError> {
        self.frame(width, height, angle, std::ptr::null_mut())
    }

    /// Render offscreen and return tightly packed BGRA8 pixels after GPU completion.
    pub fn capture(&mut self, width: u32, height: u32, angle: f32) -> Result<Vec<u8>, CubeError> {
        Self::validate(width, height, angle)?;
        if self.context[7] != 0 {
            return Err(CubeError(8));
        }
        let mut pixels = vec![0; width as usize * height as usize * 4];
        self.frame(width, height, angle, pixels.as_mut_ptr())?;
        Ok(pixels)
    }

    fn validate(width: u32, height: u32, angle: f32) -> Result<(), CubeError> {
        if !(1..=4096).contains(&width) || !(1..=4096).contains(&height) || !angle.is_finite() {
            return Err(CubeError(7));
        }
        Ok(())
    }

    fn frame(
        &mut self,
        width: u32,
        height: u32,
        angle: f32,
        pixels: *mut u8,
    ) -> Result<(), CubeError> {
        Self::validate(width, height, angle)?;
        // SAFETY: initialized exclusive context; pixels is null or a live allocation
        // of width*height*4 bytes. Assembly waits for the GPU before reading back.
        let status = unsafe {
            asm_cube_frame(
                self.context.as_mut_ptr(),
                width as usize,
                height as usize,
                pixels,
                angle,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(CubeError(status))
        }
    }
}

impl Drop for Cube {
    fn drop(&mut self) {
        // SAFETY: context stays alive for cleanup; a borrowed view remains alive
        // by the constructor's contract. All frames finish synchronously.
        unsafe { asm_cube_destroy(self.context.as_mut_ptr()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn renders_colored_cube_rotates_and_resizes() {
        let mut cube = Cube::offscreen().unwrap();
        let first = cube.capture(192, 128, 0.55).unwrap();
        let background = [32, 16, 8, 255];
        assert_eq!(&first[..4], &background);
        let foreground: Vec<_> = first.chunks_exact(4).filter(|p| *p != background).collect();
        assert!(
            (800..15000).contains(&foreground.len()),
            "expected a visible cube"
        );
        assert!(
            foreground.iter().copied().collect::<HashSet<_>>().len() >= 3,
            "expected several visible colored faces"
        );
        let rotated = cube.capture(192, 128, 1.2).unwrap();
        assert_ne!(first, rotated, "rotation must change the rendered image");
        let resized = cube.capture(96, 160, 0.55).unwrap();
        assert_eq!(resized.len(), 96 * 160 * 4);
        assert!(resized.chunks_exact(4).any(|p| p != background));
        assert!(cube.capture(0, 100, 0.0).is_err());
        assert!(cube.capture(100, 100, f32::NAN).is_err());
    }
}
