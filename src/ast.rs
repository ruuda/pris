// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use pretty::{Formatter, Print};

pub struct Document<'a>(pub Vec<Stmt<'a>>);

pub enum Stmt<'a> {
    Import(Import<'a>),
    Assign(Assign<'a>),
    Return(Return<'a>),
    Block(Block<'a>),
    PutAt(PutAt<'a>),
}

pub struct Import<'a>(pub Idents<'a>);

pub struct Idents<'a>(pub Vec<&'a str>);

pub struct Assign<'a>(pub &'a str, pub Term<'a>);

pub enum Term<'a> {
    String(&'a str),
    Number(Num),
    Color(Color),
    Idents(Idents<'a>),
    Coord(Box<Coord<'a>>),
    BinOp(Box<BinTerm<'a>>),
    FnCall(Box<FnCall<'a>>),
    FnDef(FnDef<'a>),
    Block(Block<'a>),
}

pub struct Num(pub f64, pub Option<Unit>);

#[derive(Copy, Clone)]
pub enum Unit {
  W,
  H,
  Em,
  Pt,
}

pub struct Color(pub u8, pub u8, pub u8);

pub struct Coord<'a>(pub Term<'a>, pub Term<'a>);

pub struct BinTerm<'a>(pub Term<'a>, pub BinOp, pub Term<'a>);

#[derive(Copy, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Exp,
}

pub struct FnCall<'a>(pub Term<'a>, pub Vec<Term<'a>>);

pub struct FnDef<'a>(pub Vec<&'a str>, pub Block<'a>);

pub struct Block<'a>(pub Vec<Stmt<'a>>);

pub struct Return<'a>(pub Term<'a>);

pub struct PutAt<'a>(pub Term<'a>, pub Term<'a>);

// Pretty-printers.

impl<'a> Print for Document<'a> {
    fn print(&self, f: &mut Formatter) {
        for item in &self.0 {
            f.println(item);
            f.print("\n");
        }
    }
}

impl<'a> Print for Stmt<'a> {
    fn print(&self, f: &mut Formatter) {
        match *self {
            Stmt::Import(ref i) => f.print(i),
            Stmt::Assign(ref a) => f.print(a),
            Stmt::Return(ref r) => f.print(r),
            Stmt::Block(ref bk) => f.print(bk),
            Stmt::PutAt(ref pa) => f.print(pa),
        }
    }
}

impl<'a> Print for Import<'a> {
    fn print(&self, f: &mut Formatter) {
        f.print("import ");
        f.print(&self.0);
    }
}

impl<'a> Print for Idents<'a> {
    fn print(&self, f: &mut Formatter) {
        assert!(self.0.len() > 0);
        let mut parts = self.0.iter();
        f.print(parts.next().unwrap());
        for p in parts {
            f.print(".");
            f.print(p);
        }
    }
}

impl<'a> Print for Assign<'a> {
    fn print(&self, f: &mut Formatter) {
        f.print(&self.0);
        f.print(" = ");
        f.print(&self.1);
    }
}

impl<'a> Print for Term<'a> {
    fn print(&self, f: &mut Formatter) {
        match *self {
            Term::String(string) => f.print(string),
            Term::Number(ref nm) => f.print(nm),
            Term::Color(ref col) => f.print(col),
            Term::Idents(ref is) => f.print(is),
            Term::Coord(ref coo) => f.print(coo),
            Term::BinOp(ref bop) => f.print(bop),
            Term::FnCall(ref fc) => f.print(fc),
            Term::FnDef(ref fdf) => f.print(fdf),
            Term::Block(ref blk) => f.print(blk),
        }
    }
}

impl Print for Num {
    fn print(&self, f: &mut Formatter) {
        f.print(self.0);
        if let Some(unit) = self.1 {
            f.print(unit);
        }
    }
}

impl Print for Unit {
    fn print(&self, f: &mut Formatter) {
        match *self {
            Unit::W => f.print("w"),
            Unit::H => f.print("h"),
            Unit::Em => f.print("em"),
            Unit::Pt => f.print("pt"),
        }
    }
}

impl Print for Color {
    fn print(&self, f: &mut Formatter) {
        f.print("#");
        f.print_hex_byte(self.0);
        f.print_hex_byte(self.1);
        f.print_hex_byte(self.2);
    }
}

impl<'a> Print for Coord<'a> {
    fn print(&self, f: &mut Formatter) {
        f.print("(");
        f.print(&self.0);
        f.print(", ");
        f.print(&self.1);
        f.print(")");
    }
}

impl<'a> Print for BinTerm<'a> {
    fn print(&self, f: &mut Formatter) {
        f.print("(");
        f.print(&self.0);
        f.print(" ");
        f.print(&self.1);
        f.print(" ");
        f.print(&self.2);
        f.print(")");
    }
}

impl Print for BinOp {
    fn print(&self, f: &mut Formatter) {
        match *self {
            BinOp::Add => f.print("+"),
            BinOp::Sub => f.print("-"),
            BinOp::Mul => f.print("*"),
            BinOp::Div => f.print("/"),
            BinOp::Exp => f.print("^"),
        }
    }
}

impl<'a> Print for FnCall<'a> {
    fn print(&self, f: &mut Formatter) {
        f.print(&self.0);
        f.print("(");
        let mut first = true;
        for arg in &self.1 {
            if !first { f.print(", "); }
            f.print(arg);
            first = false;
        }
        f.print(")");
    }
}

impl<'a> Print for FnDef<'a> {
    fn print(&self, f: &mut Formatter) {
        f.print("function(");
        let mut first = true;
        for arg in &self.0 {
            if !first { f.print(", "); }
            f.print(arg);
            first = false;
        }
        f.print(")");
        f.print(&self.1);
    }
}

impl<'a> Print for Block<'a> {
    fn print(&self, f: &mut Formatter) {
        f.println("\n");
        f.println("{\n");
        f.indent_more();
        for stmt in &self.0 {
            f.println(stmt);
            f.print("\n");
        }
        f.indent_less();
        f.println("}");
    }
}

impl<'a> Print for Return<'a> {
    fn print(&self, f: &mut Formatter) {
        f.print("return ");
        f.print(&self.0);
    }
}

impl<'a> Print for PutAt<'a> {
    fn print(&self, f: &mut Formatter) {
        f.print("put ");
        f.print(&self.0);
        f.print(" at ");
        f.print(&self.1);
    }
}
