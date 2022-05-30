#![cfg(target_os = "windows")]

use super::{Capturer, Display};
use image::{Rgb, RgbImage};
use std::{mem, ptr};
use winapi::{
    shared::{
        minwindef::{BOOL, LPARAM, TRUE},
        windef::{HDC, HMONITOR, LPRECT, RECT},
    },
    um::{
        wingdi::{
            BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDeviceCaps,
            SelectObject, BITMAPINFO, BITMAPINFOHEADER, BITSPIXEL, BI_RGB, CAPTUREBLT,
            DIB_RGB_COLORS, RGBQUAD, SRCCOPY,
        },
        winuser::{EnumDisplayMonitors, GetWindowDC},
    },
};

pub struct WindowsCapturer {
    h_dc: HDC,
    h_compatible_dc: HDC,
    displays: Vec<Display>,
    bits_per_pixel: u16,
}

impl Drop for WindowsCapturer {
    fn drop(&mut self) {
        unsafe {
            DeleteDC(self.h_compatible_dc);
            DeleteDC(self.h_dc);
        }
    }
}

impl Capturer for WindowsCapturer {
    fn displays<'a>(&self) -> &[Display] {
        &self.displays
    }

    fn capture(&self, index: usize) -> Option<RgbImage> {
        let h_dc = self.h_dc;

        let h_compatible_dc = self.h_compatible_dc;

        let Display {
            width,
            height,
            top,
            left,
        } = *self.displays.get(index)?;

        unsafe {
            let bitmap_info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height,
                    biPlanes: 1,
                    biBitCount: self.bits_per_pixel,
                    biCompression: BI_RGB,
                    ..mem::zeroed()
                },
                ..mem::zeroed()
            };

            let mut data: *mut u8 = ptr::null_mut();

            let compatible_bitmap = CreateDIBSection(
                h_dc,
                &bitmap_info as *const BITMAPINFO,
                DIB_RGB_COLORS,
                &mut data as *mut *mut u8 as _,
                ptr::null_mut(),
                0,
            );

            SelectObject(h_compatible_dc as _, compatible_bitmap as _);

            BitBlt(
                h_compatible_dc,
                0,
                0,
                width,
                height,
                h_dc,
                left,
                top,
                SRCCOPY | CAPTUREBLT,
            );

            let slice = std::slice::from_raw_parts(data as *mut RGBQUAD, (width * height) as usize);

            let (width, height) = (width as u32, height as u32);

            let mut image: RgbImage = RgbImage::new(width, height);

            let mut i = 0;

            for y in 0..height {
                for x in 0..width {
                    let RGBQUAD {
                        rgbBlue,
                        rgbGreen,
                        rgbRed,
                        ..
                    } = slice[i];
                    image.put_pixel(x, y, Rgb([rgbRed, rgbGreen, rgbBlue]));
                    i += 1;
                }
            }

            DeleteObject(compatible_bitmap as _);

            Some(image)
        }
    }
}

impl WindowsCapturer {
    pub fn new() -> Option<Self> {
        unsafe {
            let mut displays: Vec<Display> = vec![];

            let h_dc = GetWindowDC(ptr::null_mut());

            if h_dc.is_null() {
                return None;
            }

            let h_compatible_dc = CreateCompatibleDC(h_dc);

            if h_compatible_dc.is_null() {
                return None;
            }

            let bits_per_pixel = GetDeviceCaps(h_dc, BITSPIXEL) as u16;

            if EnumDisplayMonitors(
                h_dc,
                ptr::null_mut(),
                Some(enum_display_callback),
                (&mut displays as *mut _) as _,
            ) == 0
            {
                return None;
            }

            if displays.is_empty() {
                return None;
            }

            Some(Self {
                h_dc,
                h_compatible_dc,
                displays,
                bits_per_pixel,
            })
        }
    }
}

impl From<RECT> for Display {
    fn from(rect: RECT) -> Self {
        Self {
            top: rect.top,
            left: rect.left,
            width: (rect.right - rect.left).abs(),
            height: (rect.bottom - rect.top).abs(),
        }
    }
}

/// This function will give us the data we need to capture each display
/// separately through knowing each display's coordinates.
unsafe extern "system" fn enum_display_callback(
    _h_monitor: HMONITOR,
    _h_dc: HDC,
    lp_rect: LPRECT,
    l_param: LPARAM,
) -> BOOL {
    let displays = &mut *(l_param as *mut Vec<Display>);
    displays.push((*lp_rect).into());
    TRUE
}
