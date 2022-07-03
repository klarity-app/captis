#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub type Error = windows::WindowsError;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub type Error = x11rb::errors::ConnectionError;

pub use image::RgbImage;

pub trait Capturer {
    /// Returns a single image from the selected display.
    fn capture(&self, index: usize) -> Result<RgbImage, Error>;
    /// Captures a single image from all the displays available and returns them.
    fn capture_all(&self) -> Result<Vec<RgbImage>, Error>;
    /// Returns a reference to the currently available displays.
    fn displays(&self) -> &[Display];
}

#[derive(Debug, Copy, Clone, Hash)]
pub struct Display {
    top: i32,
    left: i32,
    width: i32,
    height: i32,
}

impl Display {
    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn height(&self) -> i32 {
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
