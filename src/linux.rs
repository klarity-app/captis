#![cfg(target_os = "linux")]

use super::*;

use std::{
    io::{Error, ErrorKind},
    marker::PhantomData,
};
use x11rb::{
    connection::Connection,
    errors::ConnectionError,
    protocol::{
        randr::ConnectionExt,
        xproto::{ConnectionExt as XProtoConnectionExt, ImageFormat},
    },
    rust_connection::RustConnection,
};

const PLANE_MASK: u32 = !1;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct Bgr {
    b: u8,
    g: u8,
    r: u8,
    _padding: u8,
}

pub(crate) struct X11Capturer {
    screen: u32,
    connection: RustConnection,
    displays: Vec<Display>,
    _phantom_data: PhantomData<*const ()>,
}

impl X11Capturer {
    pub(crate) fn new() -> Result<X11Capturer, ConnectionError> {
        let (connection, _count) = x11rb::connect(None).or(Err(ConnectionError::UnknownError))?;

        Ok(X11Capturer {
            screen: connection.setup().roots.first().unwrap().root,
            displays: get_displays(&connection)?,
            connection,
            _phantom_data: PhantomData,
        })
    }
}

impl Capturer for X11Capturer {
    fn capture(&self, index: usize) -> Result<RgbImage, ConnectionError> {
        let display = self.displays.get(index).ok_or_else(|| {
            ConnectionError::IoError(Error::new(
                ErrorKind::NotFound,
                "Couldn't find specified Display",
            ))
        })?;

        let x11_image = self
            .connection
            .get_image(
                ImageFormat::Z_PIXMAP,
                self.screen,
                display.left as i16,
                display.top as i16,
                display.width as u16,
                display.height as u16,
                PLANE_MASK,
            )?
            .reply_unchecked()?
            .ok_or(ConnectionError::UnknownError)?;

        let data: &[Bgr] = unsafe {
            std::slice::from_raw_parts(x11_image.data.as_ptr() as *const Bgr, x11_image.data.len())
        };

        let (width, height) = (display.width as u32, display.height as u32);

        let mut image = RgbImage::new(width, height);

        let mut i = 0;

        for image_pixel in image.pixels_mut() {
            let pixel = data[i];
            image_pixel.0 = [pixel.r, pixel.g, pixel.b];
            i += 1;
        }

        Ok(image)
    }

    fn capture_all(&self) -> Result<Vec<RgbImage>, ConnectionError> {
        let mut vec = vec![];
        for i in 0..self.displays.len() {
            vec.push(self.capture(i)?);
        }
        Ok(vec)
    }

    fn displays(&self) -> &[Display] {
        &self.displays
    }
}

fn get_displays(connection: &RustConnection) -> Result<Vec<Display>, ConnectionError> {
    let screens = &connection.setup().roots;
    let mut displays: Vec<Display> = vec![];

    for screen in screens {
        // Literally copied from https://github.com/BoboTiG/python-mss/blob/master/mss/linux.py
        let crtcs = match connection.randr_get_screen_resources_current(screen.root) {
            Ok(resources) => {
                let resources = resources.reply_unchecked()?;

                match resources {
                    Some(resources) => {
                        if resources.crtcs.is_empty() {
                            continue;
                        }
                        resources.crtcs
                    }
                    None => continue,
                }
            }
            Err(_) => {
                let resources = connection
                    .randr_get_screen_resources(screen.root)?
                    .reply_unchecked()?;

                match resources {
                    Some(resources) => {
                        if resources.crtcs.is_empty() {
                            continue;
                        }
                        resources.crtcs
                    }
                    None => continue,
                }
            }
        };

        for crtc in crtcs {
            if let Some(crtc_info) = connection.randr_get_crtc_info(crtc, 0)?.reply_unchecked()? {
                displays.push(Display {
                    top: crtc_info.y.into(),
                    left: crtc_info.x.into(),
                    width: crtc_info.width.into(),
                    height: crtc_info.height.into(),
                });
            }
        }
    }

    Ok(displays)
}
