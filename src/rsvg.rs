// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use libc::{c_void, c_uchar, c_int, c_ulong};

use cairo::cairo_t;

enum RsvgHandle {}
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
