// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use cairo::{Cairo, cairo_t};
use std::fs;
use std::io::{BufRead, BufReader};
use std::mem;
use std::os::raw::{c_void, c_uchar, c_int, c_ulong};
use std::path::Path;
use std::ptr;

pub enum RsvgHandle {}
enum GError {}

#[allow(non_camel_case_types)]
type gboolean = c_int;

#[allow(non_camel_case_types)]
type gsize = c_ulong;

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub struct RsvgDimensionData {
    width: c_int,
    height: c_int,
    em: f64,
    ex: f64,
}

#[link(name = "rsvg-2")]
extern {
    fn rsvg_handle_new() -> *mut RsvgHandle;
    fn rsvg_handle_write(handle: *mut RsvgHandle, buf: *const c_uchar, count: gsize, error: *mut *mut GError) -> gboolean;
    fn rsvg_handle_close(handle: *mut RsvgHandle, error: *mut *mut GError) -> gboolean;
    fn rsvg_handle_render_cairo(handle: *mut RsvgHandle, cr: *mut cairo_t) -> gboolean;
    fn rsvg_handle_get_dimensions(handle: *mut RsvgHandle, dimension_data: *mut RsvgDimensionData);
}

#[link(name = "gobject-2.0")]
extern {
    fn g_object_ref(object: *mut c_void);
    fn g_object_unref(object: *mut c_void);
}

pub struct Svg {
    handle: *mut RsvgHandle,
}

impl Svg {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Svg, ()> {
        let f = match fs::File::open(path) {
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

    pub fn draw(&self, cairo: &mut Cairo) {
        unsafe {
            // Note: `rsvg_handle_render_cairo` takes the handle as mutable
            // pointer according to the docs; not as immutable. But
            // conceptually, drawing is an immutable operation. I am assuming
            // here that it indeed does not mutate the object.
            if rsvg_handle_render_cairo(self.handle, cairo.get_raw_ptr()) != 1 {
                panic!("Failed to draw svg, rsvg reported an error.");
            }
        }
    }

    pub fn size(&self) -> (u32, u32) {
        unsafe {
            let mut dims: RsvgDimensionData = mem::uninitialized();
            rsvg_handle_get_dimensions(self.handle, &mut dims);
            (dims.width as u32, dims.height as u32)
        }
    }
}

impl Drop for Svg {
    fn drop(&mut self) {
        unsafe { g_object_unref(mem::transmute(self.handle)) }
    }
}

impl Clone for Svg {
    fn clone(&self) -> Svg {
        // The RsvgHandle is a GObject, which is refcounted. So to clone, we can
        // bump the refcount, and after that we can safely alias the handle. We
        // can produce two mutable pointers to the same RsvgHandle in this way,
        // but this is fine because we do not mutate the object after
        // construction. (This assumes that drawing does not mutate.)
        unsafe { g_object_ref(mem::transmute(self.handle)) }
        Svg {
            handle: self.handle,
        }
    }
}
