#![cfg(target_os = "macos")]
#![allow(non_upper_case_globals)]

use super::*;
use core_graphics::{
    base::CGError,
    display::{kCGWindowListOptionAll, CGDisplay, CGRect},
    geometry::{CGPoint, CGSize},
};

const kCGErrorFailure: CGError = 1000;

const kCGErrorIllegalArgument: CGError = 1001;

#[repr(C)]
struct Pixel {
    b: u8,
    g: u8,
    r: u8,
    _padding: u8,
}

pub(crate) struct MacOSCapturer {
    displays: Vec<Display>,
}

impl MacOSCapturer {
    pub(crate) fn new() -> Result<Self, CGError> {
        Ok(Self {
            displays: Self::get_displays()?,
        })
    }

    fn get_displays() -> Result<Vec<Display>, CGError> {
        let active_displays = CGDisplay::active_displays()?;
        let mut displays: Vec<Display> = Vec::with_capacity(active_displays.len());
        for display_id in active_displays {
            let display = CGDisplay::new(display_id);
            let mut cg_rect = display.bounds();

            let rotations = [90.0, -90.0];

            if rotations.contains(&display.rotation()) {
                let width = cg_rect.size.width;
                cg_rect.size.width = cg_rect.size.height;
                cg_rect.size.height = width;
            }

            displays.push(cg_rect.into())
        }
        Ok(displays)
    }
}

impl Capturer for MacOSCapturer {
    fn capture(&self, index: usize) -> Result<RgbImage, CGError> {
        let display = *self.displays.get(index).ok_or(kCGErrorIllegalArgument)?;

        let cg_image = CGDisplay::screenshot(display.into(), kCGWindowListOptionAll, 0, 0)
            .ok_or(kCGErrorFailure)?;

        let data = cg_image.data();

        let bytes = data.bytes();

        let pixels =
            unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const Pixel, bytes.len()) };

        let (width, height) = (cg_image.width(), cg_image.height());

        let mut image = RgbImage::new(width as u32, height as u32);

        let mut i = 0;

        for pixel in image.pixels_mut() {
            let Pixel { r, g, b, .. } = pixels[i];
            pixel.0 = [r, g, b];
            i += 1;
        }

        Ok(image)
    }
    fn capture_all(&self) -> Result<Vec<RgbImage>, CGError> {
        let mut vec: Vec<RgbImage> = Vec::with_capacity(self.displays.len());
        for (i, _) in self.displays.iter().enumerate() {
            vec.push(self.capture(i)?);
        }
        Ok(vec)
    }
    fn displays(&self) -> &[Display] {
        &self.displays
    }
}

impl From<CGRect> for Display {
    fn from(cg_rect: CGRect) -> Self {
        Display {
            left: cg_rect.origin.x,
            top: cg_rect.origin.y,
            width: cg_rect.size.width,
            height: cg_rect.size.height,
        }
    }
}

impl From<Display> for CGRect {
    fn from(display: Display) -> Self {
        CGRect::new(
            &CGPoint {
                x: display.left,
                y: display.top,
            },
            &CGSize {
                width: display.width,
                height: display.height,
            },
        )
    }
}
