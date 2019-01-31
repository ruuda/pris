// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use cairo;
use rsvg::Svg;

use std::ops;
use std::path::PathBuf;

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
    FillPolygon(FillPolygon),
    Png(PathBuf),
    Scaled(Vec<PlacedElement>, f64),
    StrokePolygon(StrokePolygon),
    Svg(Svg),
    Text(Text),
    Hyperlink(Hyperlink),
}

#[derive(Copy, Clone)]
pub enum PolygonKind {
    /// The points are vertices are connected by lines.
    Lines,
    /// One point is an endpoint of a cubic Bézier segment, the next two are control points.
    Curves,
}

#[derive(Clone)]
pub struct FillPolygon {
    pub color: Color,
    pub vertices: Vec<Vec2>,
    pub kind: PolygonKind,
}

#[derive(Clone)]
pub struct StrokePolygon {
    pub color: Color,
    pub line_width: f64,
    pub close: bool,
    pub vertices: Vec<Vec2>,
    pub kind: PolygonKind,
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

#[derive(Clone)]
pub struct Hyperlink {
    pub size: Vec2,
    pub uri: String,
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

impl ops::Mul<f64> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f64) -> Vec2 {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl ops::Neg for Vec2 {
    type Output = Vec2;

    fn neg(self) -> Vec2 {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Color {
        Color { r: r, g: g, b: b }
    }
}
