// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use freetype::freetype_sys::FT_Face;
use freetype;
use libc::{c_int, c_void};
use std::ptr;

#[allow(non_camel_case_types)]
enum hb_font_t {}

#[allow(non_camel_case_types)]
enum hb_buffer_t {}

#[allow(non_camel_case_types)]
type hb_destroy_func_t = *mut extern fn(*mut c_void);

#[allow(non_camel_case_types)]
type hb_direction_t = c_int;

// Note: this is an enum in C. We can define one in Rust, but the underlying
// type of an enum in C is 'int', and although in Rust we can opt for u32 or u64
// directly, we cannot pick the platform-dependent 'int' type. So define a bunch
// of constants instead. Not all of the variants are used, but let's include
// them anyway (and allow dead code).
mod hb {
    #![allow(dead_code)]

    use harfbuzz::hb_direction_t;
    pub const HB_DIRECTION_INVALID: hb_direction_t = 0;
    pub const HB_DIRECTION_LTR: hb_direction_t = 1;
    pub const HB_DIRECTION_RTL: hb_direction_t = 2;
    pub const HB_DIRECTION_TTB: hb_direction_t = 3;
    pub const HB_DIRECTION_BTT: hb_direction_t = 4;
}

#[link(name = "harfbuzz")]
extern {
    fn hb_ft_font_create(ft_face: FT_Face, destroy: hb_destroy_func_t) -> *mut hb_font_t;
    fn hb_buffer_create() -> *mut hb_buffer_t;
    fn hb_buffer_destroy(buffer: *mut hb_buffer_t);
    fn hb_buffer_set_direction(buffer: *mut hb_buffer_t, direction: hb_direction_t);
}

pub struct Font {
    ptr: *mut hb_font_t,
}

pub struct Buffer {
    ptr: *mut hb_buffer_t,
}

impl Font {
    // TODO: Figure out ft_face ownership rules.
    pub fn from_ft_face(mut ft_face: freetype::Face<'static>) -> Font {
        Font {
            ptr: unsafe { hb_ft_font_create(ft_face.raw_mut(), ptr::null_mut()) },
        }
    }
}

impl Drop for Font {
    fn drop(&mut self) {
        // TODO: Figure out lifetimes.
    }
}

impl Buffer {
    pub fn new() -> Buffer {
        // Note: Harfbuzz buffers are refcounted, and creating one will set its
        // refcount to 1. It must be freed later with `hb_buffer_destroy()`.
        Buffer {
            ptr: unsafe { hb_buffer_create() },
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        // Note that Harfbufzz buffers are refcounted.
        // TODO: Assert that the refcount is 1 here.
        unsafe { hb_buffer_destroy(self.ptr) }
    }
}
