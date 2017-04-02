// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use cairo;
use rsvg::Svg;
use std::ops;

#[derive(Clone)]
pub struct PlacedElement {
    pub position: Vec2,
    pub element: Element,
}

/// A 2D vector type used for coordinates and offsets.
#[derive(Copy, Clone)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone)]
pub enum Element {
    Line(Line),
    Text(Text),
    Svg(Svg),
}

#[derive(Clone)]
pub struct Line {
    pub color: Color,
    pub line_width: f64,
    pub offset: Vec2,
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
    pub font_family: String,
    pub font_style: String,
    pub font_size: f64,
    pub glyphs: Vec<cairo::Glyph>,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Vec2 {
        Vec2 {
            x: x,
            y: y,
        }
    }

    pub fn zero() -> Vec2 {
        Vec2 {
            x: 0.0,
            y: 0.0,
        }
    }
}

impl ops::Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Color {
        Color { r: r, g: g, b: b }
    }
}
