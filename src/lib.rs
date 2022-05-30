#[cfg(target_os = "windows")]
mod windows;

use image::RgbImage;

pub trait Capturer {
    fn capture(&self, index: usize) -> Option<RgbImage>;
    fn displays(&self) -> &[Display];
}

#[derive(Debug, Copy, Clone)]
pub struct Display {
    top: i32,
    left: i32,
    width: i32,
    height: i32,
}

#[cfg(target_os = "windows")]
/// Initializes a struct that implements [`Capturer`].
pub fn init_capturer() -> Option<impl Capturer> {
    use windows::*;
    WindowsCapturer::new()
}

#[cfg(target_os = "macos")]
/// Initializes a struct that implements [`Capturer`].
pub fn init_capturer() -> Option<impl Capturer> {
    None
}

#[cfg(target_os = "linux")]
/// Initializes a struct that implements [`Capturer`].
pub fn init_capturer() -> Option<impl Capturer> {
    None
}
