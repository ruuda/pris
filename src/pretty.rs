// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

//! The string formatting primitives in `std::fmt` are not really suitable for
//! pretty-printing code, because they do not support indentation in a proper
//! way. This module provides an alternative abstraction for pretty printing
//! that automatically inserts indents after newlines. It also assumes that
//! printing cannot fail, which avoid clumsy error handling.

use std::fmt::Write;
use std::rc::Rc;

pub fn print<P: Print>(content: P) -> String {
    let mut f = Formatter::new();
    f.print(content);
    f.into_string()
}

pub trait Print {
    fn print(&self, f: &mut Formatter);
}

pub struct Formatter {
    target: String,
    indent: u32,
}

impl<'a, P: Print> Print for &'a P {
    fn print(&self, f: &mut Formatter) {
        (*self).print(f);
    }
}

impl<P: Print> Print for Box<P> {
    fn print(&self, f: &mut Formatter) {
        (**self).print(f);
    }
}

impl<P: Print> Print for Rc<P> {
    fn print(&self, f: &mut Formatter) {
        (**self).print(f);
    }
}

impl<'a> Print for &'a str {
    fn print(&self, f: &mut Formatter) {
        f.target.push_str(self);
    }
}

impl Print for i32 {
    fn print(&self, f: &mut Formatter) {
        write!(&mut f.target, "{}", self).unwrap();
    }
}

impl Print for u32 {
    fn print(&self, f: &mut Formatter) {
        write!(&mut f.target, "{}", self).unwrap();
    }
}

impl Print for usize {
    fn print(&self, f: &mut Formatter) {
        write!(&mut f.target, "{}", self).unwrap();
    }
}

impl Print for f64 {
    fn print(&self, f: &mut Formatter) {
        write!(&mut f.target, "{}", self).unwrap();
    }
}

impl Formatter {
    pub fn new() -> Formatter {
        Formatter { target: String::new(), indent: 0 }
    }

    pub fn indent_more(&mut self) {
        self.indent += 1;
    }

    pub fn indent_less(&mut self) {
        assert!(self.indent > 0);
        self.indent -= 1;
    }

    pub fn print<P: Print>(&mut self, content: P) {
        content.print(self);
    }

    pub fn println<P: Print>(&mut self, content: P) {
        for _ in 0..self.indent * 2 {
            self.target.push(' ');
        }
        self.print(content);
    }

    pub fn print_hex_byte(&mut self, content: u8) {
        write!(&mut self.target, "{:2x}", content).unwrap();
    }

    pub fn into_string(self) -> String {
        self.target
    }
}
