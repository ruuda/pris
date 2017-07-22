// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

//! This module implements reading png metadata with libpng.

use std::os::raw::{c_void, c_char};

#[allow(non_camel_case_types)]
enum png_info {}

#[allow(non_camel_case_types)]
enum png_struct {}

#[allow(non_camel_case_types)]
enum png_error_ptr {}

#[link(name = "png")]
extern {
    fn png_create_read_struct(user_png_ver: *const c_char, error_ptr: *const c_void, error_fn: *mut png_error_ptr, warn_fn: *mut png_error_ptr) -> *mut png_struct;
    fn png_destroy_read_struct(png_ptr_ptr: *mut *mut png_struct, info_ptr_ptr: *mut *mut png_info, end_info_ptr_ptr: *mut *mut png_info);
    fn png_create_info_struct(png_ptr: *mut png_struct) -> *mut png_info;
}
