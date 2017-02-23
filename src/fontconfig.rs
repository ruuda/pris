// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use libc::{c_int, c_char, c_uchar};
use std::ffi::{CStr, CString, OsStr};
use std::mem;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr;

enum FcConfig {}
enum FcPattern {}

type FcBool = c_int;
type FcChar8 = c_uchar;
type FcMatchKind = c_int;
type FcResult = c_int;

// Note: this is an enum in C. We can define one in Rust, but the underlying
// type of an enum in C is 'int', and although in Rust we can opt for u32 or u64
// directly, we cannot pick the platform-dependent 'int' type. So define a bunch
// of constants instead. Not all of the variants are used, but let's include
// them anyway (and allow dead code).
mod fc {
    #![allow(non_upper_case_globals, dead_code)]

    use fontconfig::FcResult;
    pub const FcResultMatch: FcResult = 0;
    pub const FcResultNoMatch: FcResult = 1;
    pub const FcResultTypeMismatch: FcResult = 2;
    pub const FcResultNoId: FcResult = 3;
    pub const FcResultOutOfMemory: FcResult = 4;

    use fontconfig::FcMatchKind;
    pub const FcMatchPattern: FcMatchKind = 0;
    pub const FcMatchFont: FcMatchKind = 1;
    pub const FcMatchScan: FcMatchKind = 2;
}

#[link(name = "fontconfig")]
extern {
    fn FcInitLoadConfigAndFonts() -> *mut FcConfig;
    fn FcNameParse(fname: *const FcChar8) -> *mut FcPattern;
    fn FcConfigSubstitute(config: *mut FcConfig, pattern: *mut FcPattern, kind: FcMatchKind) -> FcBool;
    fn FcDefaultSubstitute(pattern: *mut FcPattern);
    fn FcFontMatch(config: *mut FcConfig, pattern: *mut FcPattern, result: *mut FcResult) -> *mut FcPattern;
    fn FcPatternGetString(pattern: *mut FcPattern, object: *const c_char, n: c_int, result: *mut *mut FcChar8) -> FcResult;
    fn FcPatternDestroy(pattern: *mut FcPattern);
}


/// Given a Fontconfig query such as "Cantarell" or "Cantarell:bold", returns
/// the absolute path to the corresponding font file, if it could be found.
pub fn get_font_location(font_query: &str) -> Option<PathBuf> {
    let mut result = None;

    unsafe {
        let mut config = FcInitLoadConfigAndFonts();

        // This is the FC_FILE constant in the C API.
        let fc_file = CStr::from_bytes_with_nul_unchecked(b"file\0");

        // Fontconfig insists on using a non-standard character type, but it
        // only differs in signedness, which is arbitrary for characters anyway.
        let query_cstr = CString::new(font_query).unwrap();
        let query_char8: *const FcChar8 = mem::transmute(query_cstr.as_ptr());
        let pattern = FcNameParse(query_char8);

        // The parsed pattern might not have some properties set, such as the
        // weight or slant. FcDefaultSubstitute fills these in.
        FcDefaultSubstitute(pattern);

        // The docs say that FcConfigSubstitute must be called too, although it
        // is unclear what its purpose is.
        assert!(0 != FcConfigSubstitute(config, pattern, fc::FcMatchPattern));

        let mut match_result = fc::FcResultNoMatch;
        let font_match = FcFontMatch(config, pattern, &mut match_result);

        if match_result == fc::FcResultMatch {
            // Retrieve the filename from the "match", if there was one. Doing
            // this should always succeed, otherwise there is a programming
            // error, or allocation failure.
            let mut fname_ptr: *mut FcChar8 = ptr::null_mut();
            let get_result = FcPatternGetString(font_match, fc_file.as_ptr(), 0, &mut fname_ptr);
            assert_eq!(get_result, fc::FcResultMatch);

            // Do the conversion dance: from *mut c_str to PathBuf. PathBuf owns
            // its contents: we make a copy of Fontconfig's string, so we can
            // free it afterwards. It is an extra copy, but it is far more
            // convenient than the alternative. Also transmute Fontconfig's
            // signed character strings once more.
            let fname_cstr = CStr::from_ptr(mem::transmute(fname_ptr));
            let fname_osstr = OsStr::from_bytes(fname_cstr.to_bytes());
            result = Some(PathBuf::from(fname_osstr));
        }

        FcPatternDestroy(font_match);
        FcPatternDestroy(pattern);
    }

    result
}
