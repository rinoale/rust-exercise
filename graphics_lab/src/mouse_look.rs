//! Camera input is independent of the rendering API. WSLg's virtual pointer
//! can expose absolute XInput values as raw motion, so use window coordinates
//! there. Never feed both event streams into the camera.
use anyhow::{Result, bail};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    window::{CursorGrabMode, Window},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseInput {
    Raw,
    Cursor,
}

pub struct MouseLook {
    mode: MouseInput,
    active: bool,
    previous: Option<PhysicalPosition<f64>>,
}
impl MouseLook {
    pub fn from_environment() -> Result<Self> {
        let mode = match std::env::var("GRAPHICS_MOUSE_INPUT").as_deref() {
            Ok("raw") => MouseInput::Raw,
            Ok("cursor") => MouseInput::Cursor,
            Ok(_) => bail!("GRAPHICS_MOUSE_INPUT must be raw or cursor"),
            Err(_)
                if cfg!(target_os = "linux")
                    && (std::env::var_os("WSL_DISTRO_NAME").is_some()
                        || std::path::Path::new("/mnt/wslg").exists()) =>
            {
                MouseInput::Cursor
            }
            Err(_) => MouseInput::Raw,
        };
        Ok(Self::new(mode))
    }
    pub fn new(mode: MouseInput) -> Self {
        Self {
            mode,
            active: false,
            previous: None,
        }
    }
    pub fn capture(&mut self, window: &Window) -> Result<()> {
        match self.mode {
            MouseInput::Raw => {
                window
                    .set_cursor_grab(CursorGrabMode::Locked)
                    .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))?;
                window.set_cursor_visible(false);
            }
            MouseInput::Cursor => {
                // Do not warp/recenter: WSLg can report the synthetic warp or
                // snap-back as another movement and create a rotation loop.
                // Confinement is optional; keyboard play still works without it.
                if let Err(error) = window.set_cursor_grab(CursorGrabMode::Confined) {
                    eprintln!(
                        "Cursor confinement unavailable ({error}); cursor look remains enabled"
                    );
                }
                window.set_cursor_visible(true);
            }
        }
        self.activate();
        println!(
            "Mouse look: {:?} | arrows also look | H levels view | Esc releases",
            self.mode
        );
        Ok(())
    }
    fn activate(&mut self) {
        self.active = true;
        self.reset_reference();
    }
    pub fn release(&mut self, window: &Window) {
        self.active = false;
        self.reset_reference();
        let _ = window.set_cursor_grab(CursorGrabMode::None);
        window.set_cursor_visible(true);
    }
    pub fn reset_reference(&mut self) {
        self.previous = None;
    }
    pub fn raw_motion(&self, delta: (f64, f64)) -> Option<(f64, f64)> {
        (self.active && self.mode == MouseInput::Raw && delta.0.is_finite() && delta.1.is_finite())
            .then_some(delta)
    }
    pub fn cursor_motion(
        &mut self,
        position: PhysicalPosition<f64>,
        size: PhysicalSize<u32>,
    ) -> Option<(f64, f64)> {
        if !self.active || self.mode != MouseInput::Cursor {
            return None;
        }
        if !position.x.is_finite()
            || !position.y.is_finite()
            || position.x < 0.
            || position.y < 0.
            || position.x >= f64::from(size.width)
            || position.y >= f64::from(size.height)
        {
            self.reset_reference();
            return None;
        }
        let previous = self.previous.replace(position)?;
        let delta = (position.x - previous.x, position.y - previous.y);
        // Ignore pointer discontinuities (focus/resize/desktop repositioning),
        // not just the first event. Stationary absolute positions yield zero.
        if delta.0.abs() > f64::from(size.width) * 0.5
            || delta.1.abs() > f64::from(size.height) * 0.5
            || delta == (0., 0.)
        {
            return None;
        }
        Some(delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn position(x: f64, y: f64) -> PhysicalPosition<f64> {
        PhysicalPosition::new(x, y)
    }
    const SIZE: PhysicalSize<u32> = PhysicalSize::new(1100, 760);

    #[test]
    fn wsl_absolute_raw_values_never_rotate_the_camera() {
        let mut mouse = MouseLook::new(MouseInput::Cursor);
        mouse.activate();
        for _ in 0..100 {
            assert_eq!(mouse.raw_motion((28000., 19000.)), None);
            mouse.cursor_motion(position(1050., 700.), SIZE);
        }
        assert_eq!(
            mouse.cursor_motion(position(1056., 696.), SIZE),
            Some((6., -4.))
        );
        for _ in 0..100 {
            assert_eq!(mouse.cursor_motion(position(1056., 696.), SIZE), None);
        }
    }
    #[test]
    fn capture_reentry_and_pointer_jumps_rebase_without_rotation() {
        let mut mouse = MouseLook::new(MouseInput::Cursor);
        assert_eq!(mouse.cursor_motion(position(400., 300.), SIZE), None);
        mouse.activate();
        assert_eq!(mouse.cursor_motion(position(900., 650.), SIZE), None);
        assert_eq!(
            mouse.cursor_motion(position(910., 660.), SIZE),
            Some((10., 10.))
        );
        mouse.reset_reference();
        assert_eq!(mouse.cursor_motion(position(100., 50.), SIZE), None);
        assert_eq!(mouse.cursor_motion(position(900., 700.), SIZE), None);
        assert_eq!(
            mouse.cursor_motion(position(904., 702.), SIZE),
            Some((4., 2.))
        );
        assert_eq!(mouse.cursor_motion(position(f64::NAN, 0.), SIZE), None);
        assert_eq!(mouse.cursor_motion(position(100., 50.), SIZE), None);
    }
    #[test]
    fn raw_mode_uses_only_device_events_and_ignores_invalid_values() {
        let mut mouse = MouseLook::new(MouseInput::Raw);
        assert_eq!(mouse.raw_motion((2., 3.)), None);
        mouse.activate();
        assert_eq!(mouse.raw_motion((2., 3.)), Some((2., 3.)));
        assert_eq!(mouse.cursor_motion(position(600., 400.), SIZE), None);
        assert_eq!(mouse.raw_motion((f64::INFINITY, 3.)), None);
    }
}
