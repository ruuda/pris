// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use freetype;
use freetype::freetype_sys::FT_Face;
use std::mem;
use std::os::raw::{c_char, c_int, c_ulong};
use std::path::Path;
use std::ffi::{CStr, CString};

#[allow(non_camel_case_types)]
enum cairo_surface_t {}

#[allow(non_camel_case_types)]
pub enum cairo_t {}

#[allow(non_camel_case_types)]
enum cairo_font_face_t {}

#[allow(non_camel_case_types)]
type cairo_status_t = c_int;

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub struct cairo_glyph_t {
    index: c_ulong,
    x: f64,
    y: f64,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub struct cairo_matrix_t {
    xx: f64,
    yx: f64,
    xy: f64,
    yy: f64,
    x0: f64,
    y0: f64,
}

#[link(name = "cairo")]
extern {
    fn cairo_create(surf: *mut cairo_surface_t) -> *mut cairo_t;
    fn cairo_image_surface_create_from_png(fname: *const c_char) -> *mut cairo_surface_t;
    fn cairo_pdf_surface_create(fname: *const c_char, width: f64, height: f64) -> *mut cairo_surface_t;
    fn cairo_set_source_surface(cr: *mut cairo_t, surface: *mut cairo_surface_t, x: f64, y: f64);
    fn cairo_set_source_rgb(cr: *mut cairo_t, r: f64, g: f64, b: f64);
    fn cairo_set_line_width(cr: *mut cairo_t, width: f64);
    fn cairo_move_to(cr: *mut cairo_t, x: f64, y: f64);
    fn cairo_line_to(cr: *mut cairo_t, x: f64, y: f64);
    fn cairo_close_path(cr: *mut cairo_t);
    fn cairo_rectangle(cr: *mut cairo_t, x: f64, y: f64, w: f64, h: f64);
    fn cairo_stroke(cr: *mut cairo_t);
    fn cairo_fill(cr: *mut cairo_t);
    fn cairo_paint(cr: *mut cairo_t);
    fn cairo_show_page(cr: *mut cairo_t);
    fn cairo_destroy(cr: *mut cairo_t);
    fn cairo_surface_destroy(surf: *mut cairo_surface_t);
    fn cairo_ft_font_face_create_for_ft_face(face: FT_Face, load_flags: c_int) -> *mut cairo_font_face_t;
    fn cairo_font_face_destroy(face: *mut cairo_font_face_t);
    fn cairo_set_font_face(cr: *mut cairo_t, font: *mut cairo_font_face_t);
    fn cairo_set_font_size(cr: *mut cairo_t, size: f64);
    fn cairo_show_glyphs(cr: *mut cairo_t, glyphs: *const cairo_glyph_t, num_glyphs: c_int);
    fn cairo_get_matrix(cr: *mut cairo_t, matrix: *mut cairo_matrix_t);
    fn cairo_set_matrix(cr: *mut cairo_t, matrix: *const cairo_matrix_t);
    fn cairo_translate(cr: *mut cairo_t, tx: f64, ty: f64);
    fn cairo_scale(cr: *mut cairo_t, sx: f64, sy: f64);
    fn cairo_tag_begin(cr: *mut cairo_t, tag_name: *const c_char, attributes: *const c_char);
    fn cairo_tag_end(cr: *mut cairo_t, tag_name: *const c_char);
    fn cairo_status(cr: *mut cairo_t) -> cairo_status_t;
    fn cairo_status_to_string(status: cairo_status_t) -> *const c_char;
}

pub struct Surface {
    ptr: *mut cairo_surface_t,
}

pub struct Cairo {
    ptr: *mut cairo_t,
}

pub struct FontFace {
    ptr: *mut cairo_font_face_t,
    // Own the FreeType face to keep it alive.
    #[allow(dead_code)]
    ft_face: freetype::Face<'static>,
}

#[derive(Copy, Clone)]
pub struct Glyph(cairo_glyph_t);

#[derive(Copy, Clone)]
pub struct Matrix(cairo_matrix_t);

impl Surface {
    pub fn new_pdf(fname: &Path, width: f64, height: f64) -> Surface {
        let fname_cstr = CString::new(fname.to_str().unwrap()).unwrap();
        Surface {
            ptr: unsafe { cairo_pdf_surface_create(fname_cstr.as_ptr(), width, height) }
        }
    }

    pub fn from_png(fname: &Path) -> Surface {
        let fname_cstr = CString::new(fname.to_str().unwrap()).unwrap();
        Surface {
            // TODO: Check cairo_surface_status, see the manual.
            ptr: unsafe { cairo_image_surface_create_from_png(fname_cstr.as_ptr()) }
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

    pub unsafe fn get_raw_ptr(&mut self) -> *mut cairo_t {
        self.ptr
    }

    pub fn assert_status_success(&mut self) {
        unsafe {
            let status = cairo_status(self.ptr);
            if status == 0 { return }
            let message_ptr = cairo_status_to_string(status);
            match CStr::from_ptr(message_ptr).to_str() {
                Ok(msg) => panic!("Cairo status is not success: {}.", msg),
                Err(_) => panic!("Cairo status is not success."),
            }
        }
    }

    pub fn set_source_rgb(&mut self, r: f64, g: f64, b: f64) {
        unsafe { cairo_set_source_rgb(self.ptr, r, g, b) }
    }

    pub fn set_source_surface(&mut self, surface: &Surface, x: f64, y: f64) {
        unsafe { cairo_set_source_surface(self.ptr, surface.ptr, x, y) }
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

    pub fn close_path(&mut self) {
        unsafe { cairo_close_path(self.ptr) }
    }

    pub fn rectangle(&mut self, x: f64, y: f64, w: f64, h: f64) {
        unsafe { cairo_rectangle(self.ptr, x, y, w, h) }
    }

    pub fn stroke(&mut self) {
        unsafe { cairo_stroke(self.ptr) }
    }

    pub fn fill(&mut self) {
        unsafe { cairo_fill(self.ptr) }
    }

    pub fn paint(&mut self) {
        unsafe { cairo_paint(self.ptr) }
    }

    pub fn show_page(&mut self) {
        unsafe { cairo_show_page(self.ptr) }
    }

    pub fn tag_begin_link(&mut self, attributes: &str) {
        unsafe {
            let tag_name = CStr::from_bytes_with_nul_unchecked(b"Link\0");
            let attrs = CStr::from_bytes_with_nul(attributes.as_bytes()).unwrap();
            cairo_tag_begin(self.ptr, tag_name.as_ptr(), attrs.as_ptr());
        }
        self.assert_status_success();
    }

    pub fn tag_end_link(&mut self) {
        unsafe {
            let tag_name = CStr::from_bytes_with_nul_unchecked(b"Link\0");
            cairo_tag_end(self.ptr, tag_name.as_ptr());
        }
        self.assert_status_success();
    }

    pub fn set_font_face(&mut self, face: &FontFace) {
        unsafe { cairo_set_font_face(self.ptr, face.ptr) }
    }

    pub fn set_font_size(&mut self, size: f64) {
        unsafe { cairo_set_font_size(self.ptr, size) }
    }

    pub fn show_glyphs(&mut self, glyphs: &[Glyph]) {
        unsafe {
            let cgs: *const cairo_glyph_t = mem::transmute(glyphs.as_ptr());
            cairo_show_glyphs(self.ptr, cgs, glyphs.len() as c_int);
        }
    }

    pub fn get_matrix(&self) -> Matrix {
        unsafe {
            let mut mtx: cairo_matrix_t = mem::uninitialized();
            cairo_get_matrix(self.ptr, &mut mtx);
            Matrix(mtx)
        }
    }

    pub fn set_matrix(&mut self, matrix: &Matrix) {
        let &Matrix(ref mtx) = matrix;
        unsafe { cairo_set_matrix(self.ptr, mtx) }
    }

    pub fn translate(&mut self, tx: f64, ty: f64) {
        unsafe { cairo_translate(self.ptr, tx, ty) }
    }

    pub fn scale(&mut self, sx: f64, sy: f64) {
        unsafe { cairo_scale(self.ptr, sx, sy) }
    }
}

impl Drop for Cairo {
    fn drop(&mut self) {
        unsafe { cairo_destroy(self.ptr) }
    }
}

impl FontFace {
    pub fn from_ft_face(mut ft_face: freetype::Face<'static>) -> FontFace {
        FontFace {
            ptr: unsafe {
                cairo_ft_font_face_create_for_ft_face(ft_face.raw_mut(), 0)
            },
            ft_face: ft_face,
        }
    }
}

impl Drop for FontFace {
    fn drop(&mut self) {
        // Because the struct also contains the FreeType font face, it
        // is guaranteed that the Cairo font face is destroyed before
        // the FreeType one is.
        unsafe { cairo_font_face_destroy(self.ptr) }
    }
}

impl Glyph {
    pub fn new(index: u64, x: f64, y: f64) -> Glyph {
        let cg = cairo_glyph_t {
            index: index as c_ulong,
            x: x,
            y: y,
        };
        Glyph(cg)
    }

    /// Make a copy of the glyph, offset by the specified amount.
    pub fn offset(&self, dx: f64, dy: f64) -> Glyph {
        Glyph::new(self.0.index as u64, self.0.x + dx, self.0.y + dy)
    }
}
