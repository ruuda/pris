// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use freetype::freetype_sys::FT_Face;
use freetype;
use libc::{c_char, c_int, c_uint, c_void};
use std::mem;
use std::ptr;
use std::slice;

#[allow(non_camel_case_types)]
enum hb_font_t {}

#[allow(non_camel_case_types)]
enum hb_buffer_t {}

#[allow(non_camel_case_types)]
enum hb_feature_t {}

#[allow(non_camel_case_types)]
type hb_destroy_func_t = *mut extern fn(*mut c_void);

#[allow(non_camel_case_types)]
type hb_direction_t = c_int;

#[repr(C, packed)]
#[allow(non_camel_case_types)]
struct hb_glyph_info_t {
    // The Harfbuzz type is hb_codepoint_t, which is a typedef for uint32_t.
    codepoint: u32,
    // The Harfbuzz type is hb_mask_t, which is a typedef for uint32_t.
    mask: u32,
    cluster: u32,
}

#[repr(C, packed)]
#[allow(non_camel_case_types)]
struct hb_glyph_position_t {
    // The Harfbuzz types are hb_position_t, which is a typedef for int32_t.
    x_advance: i32,
    y_advance: i32,
    x_offset: i32,
    y_offset: i32,
}

// Note: this is an enum in C. We can define one in Rust, but the underlying
// type of an enum in C is 'int', and although in Rust we can opt for u32 or u64
// directly, we cannot pick the platform-dependent 'int' type. So define a bunch
// of constants instead. Not all of the variants are used, but let's include
// them anyway (and allow dead code).
mod hb {
    #![allow(dead_code)]

    use harfbuzz::hb_direction_t;
    // Harfbuzz starts numbering at 0 and 4 explicitly for invalid and LTR.
    pub const HB_DIRECTION_INVALID: hb_direction_t = 0;
    pub const HB_DIRECTION_LTR: hb_direction_t = 4;
    pub const HB_DIRECTION_RTL: hb_direction_t = 5;
    pub const HB_DIRECTION_TTB: hb_direction_t = 6;
    pub const HB_DIRECTION_BTT: hb_direction_t = 7;
}

#[link(name = "harfbuzz")]
extern {
    fn hb_ft_font_create(ft_face: FT_Face, destroy: hb_destroy_func_t) -> *mut hb_font_t;
    fn hb_buffer_create() -> *mut hb_buffer_t;
    fn hb_buffer_destroy(buffer: *mut hb_buffer_t);
    fn hb_buffer_set_direction(buffer: *mut hb_buffer_t, direction: hb_direction_t);
    fn hb_buffer_add_utf8(buffer: *mut hb_buffer_t, text: *const c_char, text_len: c_int, item_offset: c_uint, item_length: c_int);
    fn hb_shape(font: *mut hb_font_t, buffer: *mut hb_buffer_t, features: *const hb_feature_t, num_features: c_uint);
    fn hb_buffer_get_glyph_infos(buffer: *mut hb_buffer_t, length: *mut c_uint) -> *mut hb_glyph_info_t;
    fn hb_buffer_get_glyph_positions(buffer: *mut hb_buffer_t, length: *mut c_uint) -> *mut hb_glyph_position_t;
}

pub struct Font {
    ptr: *mut hb_font_t,
}

pub struct Buffer {
    ptr: *mut hb_buffer_t,
}

/// Text direction (Rust version of `hb_direction_t`).
#[allow(dead_code)] // Not all variants are used, but they're there anyway.
pub enum Direction {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

#[derive(Copy, Clone, Debug)]
pub struct Glyph {
    codepoint: u32,
    x_advance: i32,
    y_advance: i32,
    x_offset: i32,
    y_offset: i32,
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

    pub fn shape(&mut self, font: &mut Font) {
        let features = ptr::null();
        let num_features = 0;
        unsafe { hb_shape(font.ptr, self.ptr, features, num_features) }
    }

    pub fn glyphs(&mut self) -> Vec<Glyph> {
        let (infos, poss) = unsafe {
            let mut ilen = 0;
            let mut plen = 0;
            // The Harfbuzz docs say that these pointers are valid as long as
            // the buffer is not modified. We could encode that properly in
            // Rust's type system, but I want to have the glyph info and
            // position in one struct anyway, so let's just make a copy and not
            // worry.
            let iptr = hb_buffer_get_glyph_infos(self.ptr, &mut ilen);
            let pptr = hb_buffer_get_glyph_positions(self.ptr, &mut plen);
            let islice = slice::from_raw_parts(iptr, ilen as usize);
            let pslice = slice::from_raw_parts(pptr, plen as usize);
            (islice, pslice)
        };
        infos.iter().zip(poss.iter()).map(|(info, pos)| {
            Glyph {
                codepoint: info.codepoint,
                x_offset: pos.x_offset,
                y_offset: pos.y_offset,
                x_advance: pos.x_advance,
                y_advance: pos.y_advance,
            }
        }).collect()
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        // Note that Harfbufzz buffers are refcounted.
        // TODO: Assert that the refcount is 1 here.
        unsafe { hb_buffer_destroy(self.ptr) }
    }
}
