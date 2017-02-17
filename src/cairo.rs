// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use libc::c_char;

#[allow(non_camel_case_types)]
enum cairo_surface_t {}

#[allow(non_camel_case_types)]
enum cairo_t {}

#[link(name = "cairo")]
extern {
    fn cairo_pdf_surface_create(fname: *const c_char, width: f64, height: f64) -> *mut cairo_surface_t;
    fn cairo_create(surf: *mut cairo_surface_t) -> *mut cairo_t;
    fn cairo_set_source_rgb(cr: *mut cairo_t, r: f64, g: f64, b: f64);
    fn cairo_set_line_width(cr: *mut cairo_t, width: f64);
    fn cairo_move_to(cr: *mut cairo_t, x: f64, y: f64);
    fn cairo_line_to(cr: *mut cairo_t, x: f64, y: f64);
    fn cairo_stroke(cr: *mut cairo_t);
    fn cairo_show_page(cr: *mut cairo_t);
    fn cairo_destroy(cr: *mut cairo_t);
    fn cairo_surface_destroy(surf: *mut cairo_surface_t);
}

pub struct Surface {
    ptr: *mut cairo_surface_t,
}

pub struct Cairo {
    ptr: *mut cairo_t,
}

impl Surface {
    pub fn new(fname: &str, width: f64, height: f64) -> Surface {
        use std::ffi::CString;
        let fname_cstr = CString::new(fname).unwrap();
        Surface {
            ptr: unsafe { cairo_pdf_surface_create(fname_cstr.as_ptr(), width, height) }
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { cairo_surface_destroy(self.ptr) }
    }
}

impl Cairo {
    pub fn new(surf: Surface) -> Cairo {
        // Note that we take the surface by value and destroy it afterwards.
        // Cairo employs refcounting internally, so this is safe to do.
        Cairo {
            ptr: unsafe { cairo_create(surf.ptr) }
        }
    }

    pub fn set_source_rgb(&mut self, r: f64, g: f64, b: f64) {
        unsafe { cairo_set_source_rgb(self.ptr, r, g, b) }
    }

    pub fn set_line_width(&mut self, width: f64) {
        unsafe { cairo_set_line_width(self.ptr, width) }
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        unsafe { cairo_move_to(self.ptr, x, y) }
    }

    pub fn line_to(&mut self, x: f64, y: f64) {
        unsafe { cairo_line_to(self.ptr, x, y) }
    }

    pub fn stroke(&mut self) {
        unsafe { cairo_stroke(self.ptr) }
    }
}

impl Drop for Cairo {
    fn drop(&mut self) {
        unsafe { cairo_destroy(self.ptr) }
    }
}
