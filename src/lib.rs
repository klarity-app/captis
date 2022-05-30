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
pub use windows::*;

#[cfg(target_os = "windows")]
pub fn init_capturer() -> Option<impl Capturer> {
    WindowsCapturer::new()
}

#[cfg(target_os = "macos")]
pub fn init_capturer() -> Option<impl Capturer> {
    todo!()
}

#[cfg(target_os = "linux")]
pub fn init_capturer() -> Option<impl Capturer> {
    todo!()
}
