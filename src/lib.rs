#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub type Error = windows::WindowsError;

#[cfg(target_os = "windows")]
pub type CoordinateType = i32;

#[cfg(target_os = "windows")]
pub type ProportionType = i32;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub type Error = x11rb::errors::ConnectionError;

#[cfg(target_os = "linux")]
pub type CoordinateType = i16;

#[cfg(target_os = "linux")]
pub type ProportionType = u16;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub type Error = macos::MacOSError;

#[cfg(target_os = "macos")]
pub type CoordinateType = f64;

#[cfg(target_os = "macos")]
pub type ProportionType = f64;

pub use image::RgbImage;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub(crate) struct Bgr {
    b: u8,
    g: u8,
    r: u8,
    _padding: u8,
}

pub trait Capturer {
    /// Returns a single image from the selected display.
    fn capture(&self, index: usize) -> Result<RgbImage, Error>;
    /// Captures a single image from the primary display.
    fn capture_primary(&self) -> Result<RgbImage, Error>;
    /// Captures a single image from all the displays available and returns them.
    fn capture_all(&self) -> Result<Vec<RgbImage>, Error>;
    /// Returns a reference to the currently available displays.
    fn displays(&self) -> &[Display];
    /// Refreshes the current displays.
    fn refresh_displays(&mut self) -> Result<(), Error>;
}

#[derive(Debug, Copy, Clone)]
pub struct Display {
    top: CoordinateType,
    left: CoordinateType,
    width: ProportionType,
    height: ProportionType,
}

impl Display {
    pub fn width(&self) -> ProportionType {
        self.width
    }
    pub fn height(&self) -> ProportionType {
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
