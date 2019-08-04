// Pris -- A language for designing slides
// Copyright 2018 Ruud van Asseldonk

//! This module contains constants for the names of builtins.
//!
//! Using constants rather than inlining string literals protects against
//! typos. Furthermore, it gives an overview of all magic strings in one place.

// Shouting at the reader is impolite. Furthermore, these names are easier to
// grep for if they map 1:1.
#![allow(non_upper_case_globals)]

pub const at: &'static str = "at";
pub const canvas_size: &'static str = "canvas_size";
pub const color: &'static str = "color";
pub const fill_circle: &'static str = "fill_circle";
pub const fill_polygon: &'static str = "fill_polygon";
pub const fill_rectangle: &'static str = "fill_rectangle";
pub const fit: &'static str = "fit";
pub const font_family: &'static str = "font_family";
pub const font_size: &'static str = "font_size";
pub const font_style: &'static str = "font_style";
pub const glyph: &'static str = "glyph";
pub const hyperlink: &'static str = "hyperlink";
pub const image: &'static str = "image";
pub const line: &'static str = "line";
pub const line_height: &'static str = "line_height";
pub const line_width: &'static str = "line_width";
pub const str: &'static str = "str";
pub const t: &'static str = "t";
pub const text_align: &'static str = "text_align";
pub const width: &'static str = "width";
pub const height: &'static str = "height";
pub const size: &'static str = "size";
pub const offset: &'static str = "offset";
