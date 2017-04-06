// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use freetype;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use ast::{FnDef, Idents};
use builtins;
use elements::{Color, Element, PlacedElement, Vec2};
use error::{Error, Result};
use fontconfig;
use pretty::{Formatter, Print};
use types::{LenDim, ValType};

#[derive(Clone)]
pub enum Val<'a> {
    Num(f64, LenDim), // TODO: Be consistent about abbreviating things.
    Str(String),
    Col(Color),
    Coord(f64, f64, LenDim),
    Frame(Rc<Frame<'a>>),
    FnExtrin(&'a FnDef<'a>),
    FnIntrin(Builtin),
}

#[derive(Clone)]
pub struct Frame<'a> {
    env: Env<'a>,

    /// The bounding box of the elements in this frame. The x and y coordinates
    /// of the bounding box are relative to the origin of this frame.
    bounding_box: BoundingBox,

    /// The anchor of this frame; the position at which elements should be
    /// placed when a frame is adjoined, relative to the origin of this frame.
    anchor: Vec2,

    elements: Vec<PlacedElement>,
}

#[derive(Clone)]
pub struct Env<'a> {
    bindings: HashMap<&'a str, Val<'a>>,
}

#[derive(Clone)]
pub struct BoundingBox {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// A "builtin" function is a function that takes an environment and a vector of
/// arguments, and produces a new value. We make a wrapper type to be able to
/// implement a no-op clone on it.
pub struct Builtin(pub for<'a> fn(&mut FontMap, &Env<'a>, Vec<Val<'a>>) -> Result<Val<'a>>);

/// Keeps track of loaded Freetype fonts, indexed by (family name, style) pairs.
pub struct FontMap {
    freetype: freetype::Library,
    fonts: HashMap<(String, String), freetype::Face<'static>>,
}

impl<'a> Val<'a> {
    pub fn get_type(&self) -> ValType {
        match *self {
            Val::Num(_, d) => ValType::Num(d),
            Val::Str(..) => ValType::Str,
            Val::Col(..) => ValType::Color,
            Val::Coord(_, _, d) => ValType::Coord(d),
            Val::Frame(..) => ValType::Frame,
            Val::FnExtrin(..) => ValType::Fn,
            Val::FnIntrin(..) => ValType::Fn,
        }
    }
}

impl<'a> Frame<'a> {
    pub fn new() -> Frame<'a> {
        Frame {
            env: Env::new(),
            bounding_box: BoundingBox::empty(),
            anchor: Vec2::zero(),
            elements: Vec::new(),
        }
    }

    pub fn from_env(env: Env<'a>) -> Frame<'a> {
        Frame {
            env: env,
            bounding_box: BoundingBox::empty(),
            anchor: Vec2::zero(),
            elements: Vec::new(),
        }
    }

    pub fn get_env(&self) -> &Env<'a> {
        &self.env
    }

    pub fn put_in_env(&mut self, ident: &'a str, val: Val<'a>) {
        self.env.put(ident, val);
    }

    pub fn get_elements(&self) -> &[PlacedElement] {
        &self.elements
    }

    pub fn get_anchor(&self) -> Vec2 {
        self.anchor
    }

    pub fn set_anchor(&mut self, a: Vec2) {
        self.anchor = a;
    }

    pub fn union_bounding_box(&mut self, bb: &BoundingBox) {
        self.bounding_box = self.bounding_box.union(bb);
    }

    pub fn place_element(&mut self, position: Vec2, elem: Element) {
        let placed = PlacedElement {
            position: position,
            element: elem,
        };
        self.elements.push(placed);
        // TODO: Update bounding box.
    }
}

impl<'a> Env<'a> {
    pub fn new() -> Env<'a> {
        let mut bindings = HashMap::new();
        // Default font size is 0.1h.
        bindings.insert("font_size", Val::Num(108.0, 1));
        // The default font is "sans roman", which is usually DejaVu Sans Book.
        bindings.insert("font_family", Val::Str("sans".to_string()));
        bindings.insert("font_style", Val::Str("roman".to_string()));
        bindings.insert("text_align", Val::Str("left".to_string()));
        bindings.insert("line_width", Val::Num(10.8, 1));
        bindings.insert("color", Val::Col(Color::new(0.0, 0.0, 0.0)));
        bindings.insert("fit", Val::FnIntrin(Builtin(builtins::fit)));
        bindings.insert("image", Val::FnIntrin(Builtin(builtins::image)));
        bindings.insert("line", Val::FnIntrin(Builtin(builtins::line)));
        bindings.insert("str", Val::FnIntrin(Builtin(builtins::str)));
        bindings.insert("t", Val::FnIntrin(Builtin(builtins::t)));
        Env { bindings: bindings }
    }

    pub fn lookup(&self, idents: &Idents<'a>) -> Result<Val<'a>> {
        assert!(idents.0.len() > 0);
        match self.bindings.get(idents.0[0]) {
            Some(val) => {
                if idents.0.len() == 1 {
                    Ok(val.clone())
                } else {
                    match *val {
                        Val::Frame(ref frame) => {
                            let mut more = idents.0.clone();
                            more.remove(0);
                            frame.env.lookup(&Idents(more))
                        }
                        _ => {
                            let mut f = Formatter::new();
                            f.print("Type error while reading variable '");
                            f.print(idents);
                            f.print("'. Cannot look up '");
                            f.print(idents.0[1]);
                            f.print("' in '");
                            f.print(idents.0[0]);
                            f.print("' because it is not a frame.");
                            Err(Error::Other(f.into_string()))
                        }
                    }
                }
            }
            None => Err(Error::Other(format!("Variable '{}' does not exist.", idents.0[0]))),
        }
    }

    pub fn lookup_num(&self, idents: &Idents<'a>) -> Result<f64> {
        match self.lookup(idents)? {
            Val::Num(x, 0) => Ok(x),
            other => Err(Error::var_type(idents, ValType::Num(0), other.get_type())),
        }
    }

    pub fn lookup_len(&self, idents: &Idents<'a>) -> Result<f64> {
        match self.lookup(idents)? {
            Val::Num(x, 1) => Ok(x),
            other => Err(Error::var_type(idents, ValType::Num(1), other.get_type())),
        }
    }

    pub fn lookup_color(&self, idents: &Idents<'a>) -> Result<Color> {
        match self.lookup(idents)? {
            Val::Col(col) => Ok(col),
            other => Err(Error::var_type(idents, ValType::Color, other.get_type())),
        }
    }

    pub fn lookup_str(&self, idents: &Idents<'a>) -> Result<String> {
        match self.lookup(idents)? {
            Val::Str(s) => Ok(s),
            other => Err(Error::var_type(idents, ValType::Str, other.get_type())),
        }
    }

    pub fn put(&mut self, ident: &'a str, val: Val<'a>) {
        // TODO: Validate types for known variables, disallow assigning to
        // constants.
        self.bindings.insert(ident, val);
    }
}

impl BoundingBox {
    pub fn empty() -> BoundingBox {
        BoundingBox {
            // TODO: Use the Vec2 type?
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    /// Creates a bounding box at (0, 0) with the given size.
    pub fn sized(width: f64, height: f64) -> BoundingBox {
        BoundingBox {
            x: 0.0,
            y: 0.0,
            width: width,
            height: height,
        }
    }

    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        let x0 = self.x.min(other.x);
        let y0 = self.y.min(other.y);
        let x1 = (self.x + self.width).max(other.x + other.width);
        let y1 = (self.y + self.height).max(other.y + other.height);
        BoundingBox {
            x: x0,
            y: y0,
            width: x1 - x0,
            height: y1 - y0,
        }
    }
}

impl Clone for Builtin {
    fn clone(&self) -> Builtin {
        let Builtin(x) = *self;
        Builtin(x)
    }
}

impl FontMap {
    pub fn new() -> FontMap {
        FontMap {
            freetype: freetype::Library::init().expect("Failed to initialize Freetype."),
            fonts: HashMap::new(),
        }
    }

    pub fn get(&mut self, family: &str, style: &str) -> Option<&mut freetype::Face<'static>> {
        let key = (family.to_string(), style.to_string());

        let entry = match self.fonts.entry(key) {
            Entry::Occupied(x) => return Some(x.into_mut()),
            Entry::Vacant(x) => x,
        };

        // We don't have the font already, look up the file and load it with
        // Freetype.

        let mut query = family.to_string();
        query.push_str(":style=");
        query.push_str(style);
        let font_fname = match fontconfig::get_font_location(&query) {
            Some(fname) => fname,
            None => return None,
        };

        let ft_face = self.freetype
            .new_face(font_fname, 0)
            .expect("Failed to load font using Freetype.");

        // Set a standard size and DPI, so the Harfbuzz output will be relative
        // to this size, and we can scale ourselves when necessary.
        // TODO: Why does this method not take self as &mut? Ask on the Rust
        // Freetype bug tracker.
        ft_face.set_char_size(0, 1000, 72, 72).unwrap();

        let ft_face_ref = entry.insert(ft_face);
        Some(ft_face_ref)
    }
}

// Pretty printers for values and interpreter data structures.

impl<'a> Print for Val<'a> {
    fn print(&self, f: &mut Formatter) {
        match *self {
            Val::Num(x, d) => {
                f.print(x);
                f.print(" : ");
                print_unit(f, d);
            }
            Val::Str(ref s) => {
                f.print("\"");
                f.print(&s[..]); // TODO: Escaping.
                f.print("\"");
            }
            Val::Col(ref col) => {
                f.print("(");
                f.print(col.r);
                f.print(", ");
                f.print(col.g);
                f.print(", ");
                f.print(col.b);
                f.print(") : color");
            }
            Val::Coord(x, y, d) => {
                f.print("(");
                f.print(x);
                f.print(", ");
                f.print(y);
                f.print(") : coord of ");
                print_unit(f, d);
            }
            Val::Frame(ref frame) => {
                f.print(frame);
            }
            Val::FnExtrin(ref fndef) => {
                f.print(fndef);
            }
            Val::FnIntrin(..) => {
                f.print("function(...) { <built-in> }");
            }
        }
    }
}

impl Print for ValType {
    fn print(&self, f: &mut Formatter) {
        match *self {
            ValType::Num(d) => print_unit(f, d),
            ValType::Str => f.print("str"),
            ValType::Color => f.print("color"),
            ValType::Coord(d) => { f.print("coord of "); print_unit(f, d); }
            ValType::Frame => f.print("frame"),
            ValType::Fn => f.print("function"),
        }
    }
}

fn print_unit(f: &mut Formatter, d: LenDim) {
    match d {
        -3 => f.print("len⁻³"),
        -2 => f.print("len⁻²"),
        -1 => f.print("len⁻¹"),
        0 => f.print("num"),
        1 => f.print("len"),
        2 => f.print("len²"),
        3 => f.print("len³"),
        n => { f.print("len^"); f.print(n); }
    }
}

// Print implementation for variable bindings when printing env. Prints of the
// form "name = value".
impl<'a> Print for (&'a &'a str, &'a Val<'a>) {
    fn print(&self, f: &mut Formatter) {
        f.print(self.0);
        f.print(" = ");
        f.print(self.1);
    }
}

impl<'a> Print for Frame<'a> {
    fn print(&self, f: &mut Formatter) {
        f.print("frame with ");
        f.print(self.elements.len());
        f.print(" elements\n");
        f.print(&self.env);
    }
}

impl<'a> Print for Env<'a> {
    fn print(&self, f: &mut Formatter) {
        f.println("{\n");
        f.indent_more();
        for binding in self.bindings.iter() {
            f.println(binding);
            f.print("\n");
        }
        f.indent_less();
        f.println("}");
    }
}
