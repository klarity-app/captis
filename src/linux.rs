#![cfg(target_os = "linux")]

use super::*;

use libc::{shmat, shmctl, shmdt, shmget, IPC_CREAT, IPC_PRIVATE, IPC_RMID, SHM_RDONLY};
use std::{
    io::{Error, ErrorKind},
    marker::PhantomData,
    mem, ptr,
};
use x11rb::{
    connection::{Connection, RequestConnection},
    errors::ConnectionError,
    protocol::{
        randr::{self, ConnectionExt},
        shm::{self, ConnectionExt as XShmConnectionExt, SegWrapper},
        xproto::{ConnectionExt as XProtoConnectionExt, ImageFormat, PixmapWrapper},
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
    screen: usize,
    connection: RustConnection,
    displays: Vec<Display>,
    shm_addr: *const u8,
    shm_id: Option<i32>,
    seg: Option<u32>,
}

impl X11Capturer {
    pub(crate) fn new() -> Result<X11Capturer, ConnectionError> {
        let (connection, screen) = x11rb::connect(None).or(Err(ConnectionError::UnknownError))?;

        if connection
            .extension_information(randr::X11_EXTENSION_NAME)?
            .is_none()
        {
            return Err(ConnectionError::IoError(Error::new(
                ErrorKind::NotFound,
                "Couldn't find XRANDR Extension",
            )));
        }

        let (seg, shm_id, shm_addr) = if connection
            .extension_information(shm::X11_EXTENSION_NAME)?
            .is_some()
        {
            (None, None, ptr::null())
        } else {
            let screen = &connection.setup().roots[screen];

            match connection.generate_id() {
                Ok(seg) => unsafe {
                    let shm_id = shmget(
                        IPC_PRIVATE,
                        ((screen.width_in_pixels as usize * screen.height_in_pixels as usize)
                            * mem::size_of::<Bgr>()),
                        IPC_CREAT | 0o777,
                    );

                    if shm_id < 0 {
                        return Err(ConnectionError::IoError(Error::last_os_error()));
                    }

                    let shm_addr = shmat(shm_id, ptr::null(), SHM_RDONLY);

                    if (shm_addr as isize) < 0 {
                        return Err(ConnectionError::IoError(Error::last_os_error()));
                    }

                    connection.shm_attach(seg, shm_id as u32, false)?;

                    (Some(seg), Some(shm_id), shm_addr as *const u8)
                },
                Err(_) => (None, None, ptr::null()),
            }
        };

        Ok(X11Capturer {
            screen,
            displays: get_displays(&connection, screen)?,
            connection,
            shm_addr,
            shm_id,
            seg,
        })
    }

    /// Captures the screen using standard protocols, which are a lot less inefficient.
    fn capture_standard(&self, index: usize) -> Result<RgbImage, ConnectionError> {
        let display = self.displays.get(index).ok_or_else(|| {
            ConnectionError::IoError(Error::new(
                ErrorKind::NotFound,
                "Couldn't find specified Display",
            ))
        })?;

        let screen = &self.connection.setup().roots[self.screen];

        let root = screen.root;

        let x11_image = self
            .connection
            .get_image(
                ImageFormat::Z_PIXMAP,
                root,
                display.left as i16,
                display.top as i16,
                display.width as u16,
                display.height as u16,
                PLANE_MASK,
            )?
            .reply_unchecked()?
            .ok_or(ConnectionError::UnknownError)?;

        let data: &[Bgr] = unsafe {
            std::slice::from_raw_parts(x11_image.data.as_ptr() as _, x11_image.data.len())
        };

        Ok(bgr_to_rgb_image(
            data,
            display.width as u32,
            display.height as u32,
        ))
    }

    /// Captures the screen using the XShm protocol and shared memory causing the program to run
    /// hella lot faster.
    fn capture_shm(&self, index: usize) -> Result<RgbImage, ConnectionError> {
        let display = self.displays.get(index).ok_or_else(|| {
            ConnectionError::IoError(Error::new(
                ErrorKind::NotFound,
                "Couldn't find specified Display",
            ))
        })?;

        let screen = &self.connection.setup().roots[self.screen];

        let root = screen.root;

        let reply = self
            .connection
            .shm_get_image(
                root,
                display.left as i16,
                display.top as i16,
                display.width as u16,
                display.height as u16,
                PLANE_MASK,
                ImageFormat::Z_PIXMAP.into(),
                unsafe { self.seg.unwrap_unchecked() },
                0,
            )?
            .reply_unchecked()?
            .ok_or(ConnectionError::UnknownError)?;

        let data: &[Bgr] =
            unsafe { std::slice::from_raw_parts(self.shm_addr as _, reply.size as usize) };

        Ok(bgr_to_rgb_image(
            data,
            display.width as u32,
            display.height as u32,
        ))
    }
}

impl Drop for X11Capturer {
    fn drop(&mut self) {
        if let Some(seg) = self.seg {
            self.connection.shm_detach(seg).ok();
            unsafe {
                shmdt(self.shm_addr as _);
                shmctl(self.shm_id.unwrap(), IPC_RMID, ptr::null_mut());
            }
        }
    }
}

impl Capturer for X11Capturer {
    fn capture(&self, index: usize) -> Result<RgbImage, ConnectionError> {
        let image = match self.seg {
            Some(_) => self.capture_shm(index)?,
            None => self.capture_standard(index)?,
        };

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

fn bgr_to_rgb_image(data: &[Bgr], width: u32, height: u32) -> RgbImage {
    let mut image = RgbImage::new(width, height);

    let mut i = 0;

    for image_pixel in image.pixels_mut() {
        let pixel = data[i];
        image_pixel.0 = [pixel.r, pixel.g, pixel.b];
        i += 1;
    }

    image
}

fn get_displays(
    connection: &RustConnection,
    screen: usize,
) -> Result<Vec<Display>, ConnectionError> {
    let screen = &connection.setup().roots[screen];
    let mut displays: Vec<Display> = vec![];

    // Literally copied from https://github.com/BoboTiG/python-mss/blob/master/mss/linux.py
    let crtcs = match connection.randr_get_screen_resources_current(screen.root) {
        Ok(resources) => {
            resources
                .reply_unchecked()?
                .ok_or_else(|| {
                    ConnectionError::IoError(Error::new(
                        ErrorKind::NotFound,
                        "Couldn't get_screen_resources",
                    ))
                })?
                .crtcs
        }
        Err(_) => {
            connection
                .randr_get_screen_resources(screen.root)?
                .reply_unchecked()?
                .ok_or_else(|| {
                    ConnectionError::IoError(Error::new(
                        ErrorKind::NotFound,
                        "Couldn't get_screen_resources",
                    ))
                })?
                .crtcs
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

    Ok(displays)
}
