#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub type Error = windows::WindowsError;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub type Error = x11rb::errors::ConnectionError;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub type Error = core_graphics::base::CGError;

#[cfg(not(target_os = "macos"))]
pub type Number = i32;

#[cfg(target_os = "macos")]
pub type Number = f64;

pub use image::RgbImage;

pub trait Capturer {
    /// Returns a single image from the selected display.
    fn capture(&self, index: usize) -> Result<RgbImage, Error>;
    /// Captures a single image from all the displays available and returns them.
    fn capture_all(&self) -> Result<Vec<RgbImage>, Error>;
    /// Returns a reference to the currently available displays.
    fn displays(&self) -> &[Display];
}

#[derive(Debug, Copy, Clone)]
pub struct Display {
    top: Number,
    left: Number,
    width: Number,
    height: Number,
}

impl Display {
    pub fn width(&self) -> Number {
        self.width
    }
    pub fn height(&self) -> Number {
        self.height
    }
}

#[cfg(target_os = "windows")]
pub fn init_capturer() -> Result<impl Capturer, Error> {
    use windows::*;
    Ok(WindowsCapturer::new()?)
}

#[cfg(target_os = "linux")]
pub fn init_capturer() -> Result<impl Capturer, Error> {
    use linux::*;
    Ok(X11Capturer::new()?)
}

#[cfg(target_os = "macos")]
pub fn init_capturer() -> Result<impl Capturer, Error> {
    use macos::*;
    Ok(MacOSCapturer::new()?)
}
