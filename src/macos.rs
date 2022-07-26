#![cfg(target_os = "macos")]
#![allow(non_upper_case_globals)]

use super::*;
use core_graphics::{
    base::CGError,
    display::{kCGWindowListOptionAll, CGDisplay, CGRect},
    geometry::{CGPoint, CGSize},
};
use std::{error::Error, fmt};

#[derive(Debug, Copy, Clone)]
pub enum MacOSError {
    CoreGraphicsError(CGError),
    CouldntFindDisplay,
    CouldntScreenshot,
}

impl From<i32> for MacOSError {
    fn from(number: i32) -> Self {
        Self::CoreGraphicsError(number)
    }
}

impl fmt::Display for MacOSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for MacOSError {}

pub(crate) struct MacOSCapturer {
    displays: Vec<Display>,
    primary_display_index: usize,
}

impl MacOSCapturer {
    pub(crate) fn new() -> Result<Self, MacOSError> {
        let (primary_display_index, displays) = Self::get_displays()?;

        Ok(Self {
            displays,
            primary_display_index,
        })
    }

    fn get_displays() -> Result<(usize, Vec<Display>), MacOSError> {
        let active_displays = CGDisplay::active_displays()?;

        let mut primary_display_index = 0;

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

            let display: Display = cg_rect.into();

            if display.top == 0.0 && display.left == 0.0 {
                primary_display_index = displays.len();
            }

            displays.push(display);
        }

        Ok((primary_display_index, displays))
    }
}

impl Capturer for MacOSCapturer {
    fn capture(&self, index: usize) -> Result<RgbImage, MacOSError> {
        use MacOSError::*;

        let display = *self.displays.get(index).ok_or(CouldntFindDisplay)?;

        let cg_image = CGDisplay::screenshot(display.into(), kCGWindowListOptionAll, 0, 0)
            .ok_or(CouldntScreenshot)?;

        let data = cg_image.data();

        let bytes = data.bytes();

        let pixels =
            unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const Bgr, bytes.len()) };

        let (width, height) = (cg_image.width(), cg_image.height());

        let mut image = RgbImage::new(width as u32, height as u32);

        let mut i = 0;

        for pixel in image.pixels_mut() {
            let Bgr { r, g, b, .. } = pixels[i];
            pixel.0 = [r, g, b];
            i += 1;
        }

        Ok(image)
    }

    fn capture_primary(&self) -> Result<RgbImage, MacOSError> {
        Ok(self.capture(self.primary_display_index)?)
    }

    fn capture_all(&self) -> Result<Vec<RgbImage>, MacOSError> {
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
