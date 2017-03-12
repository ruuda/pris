// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use cairo;

#[derive(Clone)]
pub struct PlacedElement {
    pub x: f64,
    pub y: f64,
    pub element: Element,
}

#[derive(Clone)]
pub enum Element {
    Line(Line),
    Text(Text),
}

#[derive(Clone)]
pub struct Line {
    pub color: Color,
    pub line_width: f64,
    pub x: f64,
    pub y: f64,
}

// TODO: What color space is this? A linear RGB space would be nice.
#[derive(Copy, Clone)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Clone)]
pub struct Text {
    pub color: Color,
    pub glyphs: Vec<cairo::Glyph>,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Color {
        Color { r: r, g: g, b: b }
    }
}
