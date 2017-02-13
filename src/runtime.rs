// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::collections::HashMap;
use std::rc::Rc;

use ast::{FnDef, Idents};
use builtins;
use error::{Error, Result};
use pretty::{Formatter, Print};
use types::{LenDim, ValType};

#[derive(Clone)]
pub enum Val<'a> {
    Num(f64, LenDim), // TODO: Be consistent about abbreviating things.
    Str(String),
    Col(f64, f64, f64),
    Coord(f64, f64, LenDim),
    Frame(Rc<Frame<'a>>),
    FnExtrin(&'a FnDef<'a>),
    FnIntrin(Builtin),
}

#[derive(Clone)]
pub struct Frame<'a> {
    env: Env<'a>,
}

#[derive(Clone)]
pub struct Env<'a> {
    bindings: HashMap<&'a str, Val<'a>>,
}

/// A "builtin" function is a function that takes an environment and a vector of
/// arguments, and produces a new value. We make a wrapper type to be able to
/// implement a no-op clone on it.
pub struct Builtin(pub for<'a> fn(&Env<'a>, Vec<Val<'a>>) -> Result<Val<'a>>);

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
        }
    }

    pub fn from_env(env: Env<'a>) -> Frame<'a> {
        Frame {
            env: env
        }
    }
}

impl<'a> Env<'a> {
    pub fn new() -> Env<'a> {
        let mut bindings = HashMap::new();
        // Default font size is 0.1h.
        bindings.insert("font_size", Val::Num(108.0, 1));
        bindings.insert("image", Val::FnIntrin(Builtin(builtins::image)));
        bindings.insert("fit", Val::FnIntrin(Builtin(builtins::fit)));
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
        if let Val::Num(x, 0) = self.lookup(idents)? {
            Ok(x)
        } else {
            let mut msg = Formatter::new();
            msg.print("Type error: expected num, but ");
            msg.print(idents);
            msg.print("is <TODO>.");
            Err(Error::Other(msg.into_string()))
        }
    }

    pub fn lookup_len(&self, idents: &Idents<'a>) -> Result<f64> {
        if let Val::Num(x, 1) = self.lookup(idents)? {
            Ok(x)
        } else {
            let mut msg = Formatter::new();
            msg.print("Type error: expected len, but ");
            msg.print(idents);
            msg.print("is <TODO>.");
            Err(Error::Other(msg.into_string()))
        }
    }

    pub fn put(&mut self, ident: &'a str, val: Val<'a>) {
        self.bindings.insert(ident, val);
    }
}

impl Clone for Builtin {
    fn clone(&self) -> Builtin {
        let Builtin(x) = *self;
        Builtin(x)
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
            Val::Col(r, g, b) => {
                f.print("(");
                f.print(r);
                f.print(", ");
                f.print(g);
                f.print(", ");
                f.print(b);
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
        f.print("frame\n");
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
