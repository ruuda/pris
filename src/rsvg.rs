// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use libc::{c_void, c_uchar, c_int, c_ulong};
use cairo::{Cairo, cairo_t};
use std::fs;
use std::io::{BufRead, BufReader};
use std::mem;
use std::path::Path;
use std::ptr;

pub enum RsvgHandle {}
enum GError {}

type gboolean = c_int;
type gsize = c_ulong;

#[link(name = "rsvg-2")]
extern {
    fn rsvg_handle_new() -> *mut RsvgHandle;
    fn rsvg_handle_write(handle: *mut RsvgHandle, buf: *const c_uchar, count: gsize, error: *mut *mut GError) -> gboolean;
    fn rsvg_handle_close(handle: *mut RsvgHandle, error: *mut *mut GError) -> gboolean;
    fn rsvg_handle_render_cairo(handle: *mut RsvgHandle, cr: *mut cairo_t) -> gboolean;
}

#[link(name = "gobject-2.0")]
extern {
    fn g_object_unref(object: *mut c_void);
}

pub struct Svg {
    handle: *mut RsvgHandle,
}

impl Svg {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Svg, ()> {
        let mut f = match fs::File::open(path) {
            Ok(f) => f,
            // TODO: Proper error handling.
            Err(..) => return Err(()),
        };
        let mut reader = BufReader::new(f);

        let handle = unsafe { rsvg_handle_new() };

        // Read chunks using the `BufReader`, and feed them into the rsvg
        // handle, which will parse the file incrementally.
        loop {
            let consumed = {
                let buffer = match reader.fill_buf() {
                    Ok(b) => b,
                    // TODO: Proper error handling.
                    Err(..) => return Err(()),
                };

                // An empty buffer indicates EOF.
                if buffer.len() == 0 { break }

                unsafe {
                    if rsvg_handle_write(handle,
                                         buffer.as_ptr(),
                                         buffer.len() as c_ulong,
                                         ptr::null_mut()) != 1 {
                        // TODO: Proper error handling.
                        return Err(())
                    }
                }

                buffer.len()
            };

            reader.consume(consumed);
        }

        unsafe {
            if rsvg_handle_close(handle, ptr::null_mut()) != 1 {
                return Err(())
            }
        }

        let result = Svg {
            handle: handle,
        };
        Ok(result)
    }

    pub fn draw(&mut self, cairo: &mut Cairo) -> Result<(), ()> {
        unsafe {
            if rsvg_handle_render_cairo(self.handle, cairo.get_raw_ptr()) != 1 {
                return Err(())
            }
        }
        Ok(())
    }
}

impl Drop for Svg {
    fn drop(&mut self) {
        unsafe { g_object_unref(mem::transmute(self.handle)) }
    }
}
