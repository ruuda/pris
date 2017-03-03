// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use freetype::freetype_sys::FT_Face;
use freetype;
use libc::{c_char, c_int, c_uint, c_void};
use std::ptr;
use std::mem;

#[allow(non_camel_case_types)]
enum hb_font_t {}

#[allow(non_camel_case_types)]
enum hb_buffer_t {}

#[allow(non_camel_case_types)]
type hb_destroy_func_t = *mut extern fn(*mut c_void);

#[allow(non_camel_case_types)]
type hb_direction_t = c_int;

/// Text direction (Rust version of `hb_direction_t`).
#[allow(dead_code)] // Not all variants are used, but they're there anyway.
pub enum Direction {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

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
    fn hb_buffer_add_utf8(buffer: *mut hb_buffer_t, text: *const c_char, text_len: c_int, item_offset: c_uint, item_length: c_int);
}

pub struct Font {
    ptr: *mut hb_font_t,
}

pub struct Buffer {
    ptr: *mut hb_buffer_t,
}

impl Font {
    // TODO: Figure out ft_face ownership rules.
    pub fn from_ft_face(mut ft_face: &mut freetype::Face<'static>) -> Font {
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
    pub fn new(direction: Direction) -> Buffer {
        let hb_direction = match direction {
            Direction::RightToLeft => hb::HB_DIRECTION_RTL,
            Direction::LeftToRight => hb::HB_DIRECTION_LTR,
            Direction::TopToBottom => hb::HB_DIRECTION_TTB,
            Direction::BottomToTop => hb::HB_DIRECTION_BTT,
        };

        let ptr = unsafe {
            // Note: Harfbuzz buffers are refcounted, and creating one will set its
            // refcount to 1. It must be freed later with `hb_buffer_destroy()`.
            let ptr = hb_buffer_create();

            // TODO: If allocation fails, some flag is set somewhere. We should
            // panic in that case.

            // The buffer cannot be used without setting the direction. So we
            // might as well take it in the constructor, and set it here.
            hb_buffer_set_direction(ptr, hb_direction);

            ptr
        };

        Buffer {
            ptr: ptr,
        }
    }

    pub fn add_str(&mut self, string: &str) {
        // Rust strings are utf-8, and the Harfbuzz API takes a (ptr, len) pair
        // as opposed to a null-terminated string, so we can pass it into
        // Harfbuzz directly.
        let chars: *const c_char = unsafe { mem::transmute(string.as_bytes().as_ptr()) };
        let count = string.as_bytes().len() as i32;
        // TODO: What is the difference between the first two and the last two
        // characters?
        unsafe { hb_buffer_add_utf8(self.ptr, chars, count, 0, count) }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        // Note that Harfbufzz buffers are refcounted.
        // TODO: Assert that the refcount is 1 here.
        unsafe { hb_buffer_destroy(self.ptr) }
    }
}
