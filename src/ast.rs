// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use pretty::{Formatter, Print};

pub struct Document<'a>(pub Vec<Stmt<'a>>);

#[derive(PartialEq)]
pub enum Stmt<'a> {
    Import(Import<'a>),
    Assign(Assign<'a>),
    Return(Return<'a>),
    Block(Block<'a>),
    Put(Put<'a>),
}

#[derive(PartialEq, Eq)]
pub struct Import<'a>(pub Idents<'a>);

#[derive(PartialEq, Eq)]
pub struct Idents<'a>(pub Vec<&'a str>);

#[derive(PartialEq)]
pub struct Assign<'a>(pub &'a str, pub Term<'a>);

#[derive(PartialEq)]
pub enum Term<'a> {
    String(String),
    Number(Num),
    Color(Color),
    Idents(Idents<'a>),
    Coord(Box<Coord<'a>>),
    BinOp(Box<BinTerm<'a>>),
    UnOp(Box<UnTerm<'a>>),
    FnCall(Box<FnCall<'a>>),
    FnDef(FnDef<'a>),
    Block(Block<'a>),
}

impl<'a> Term<'a> {
    /// Shorthand to construct a `Term::Coord`.
    pub fn coord(coord: Coord<'a>) -> Term<'a> {
        Term::Coord(Box::new(coord))
    }

    /// Shorthand to construct a `Term::BinOp`.
    pub fn bin_op(bin_op: BinTerm<'a>) -> Term<'a> {
        Term::BinOp(Box::new(bin_op))
    }

    /// Shorthand to construct a `Term::UnOp`.
    pub fn un_op(un_op: UnTerm<'a>) -> Term<'a> {
        Term::UnOp(Box::new(un_op))
    }

    /// Shorthand to construct a `Term::FnCall`.
    pub fn fn_call(fn_call: FnCall<'a>) -> Term<'a> {
        Term::FnCall(Box::new(fn_call))
    }
}

#[derive(PartialEq)]
pub struct Num(pub f64, pub Option<Unit>);

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Unit {
  W,
  H,
  Em,
  Pt,
}

#[derive(PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8);

#[derive(PartialEq)]
pub struct Coord<'a>(pub Term<'a>, pub Term<'a>);

/// A binary operation applied to two terms.
#[derive(PartialEq)]
pub struct BinTerm<'a>(pub Term<'a>, pub BinOp, pub Term<'a>);

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BinOp {
    /// Adjoin, `~`.
    Adj,
    /// Add, `+`.
    Add,
    /// Subtract, `-`.
    Sub,
    /// Multiply, `*`.
    Mul,
    /// Divide, `/`.
    Div,
    /// Exponentiate, `^`.
    Exp,
}

/// A unary operation applied to a term.
#[derive(PartialEq)]
pub struct UnTerm<'a>(pub UnOp, pub Term<'a>);

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum UnOp {
    /// Unary negation, '-'.
    Neg,
}

#[derive(PartialEq)]
pub struct FnCall<'a>(pub Term<'a>, pub Vec<Term<'a>>);

#[derive(PartialEq)]
pub struct FnDef<'a>(pub Vec<&'a str>, pub Block<'a>);

#[derive(PartialEq)]
pub struct Block<'a>(pub Vec<Stmt<'a>>);

#[derive(PartialEq)]
pub struct Return<'a>(pub Term<'a>);

#[derive(PartialEq)]
pub struct Put<'a>(pub Term<'a>);

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
            Stmt::Put(ref put)  => f.print(put),
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
            // TODO: Should escape strings.
            Term::String(ref st) => f.print(&st[..]),
            Term::Number(ref nm) => f.print(nm),
            Term::Color(ref col) => f.print(col),
            Term::Idents(ref is) => f.print(is),
            Term::Coord(ref coo) => f.print(coo),
            Term::BinOp(ref bop) => f.print(bop),
            Term::UnOp(ref unop) => f.print(unop),
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
            BinOp::Adj => f.print("~"),
            BinOp::Add => f.print("+"),
            BinOp::Sub => f.print("-"),
            BinOp::Mul => f.print("*"),
            BinOp::Div => f.print("/"),
            BinOp::Exp => f.print("^"),
        }
    }
}

impl<'a> Print for UnTerm<'a> {
    fn print(&self, f: &mut Formatter) {
        f.print("(");
        f.print(&self.0);
        f.print(&self.1);
        f.print(")");
    }
}

impl Print for UnOp {
    fn print(&self, f: &mut Formatter) {
        match *self {
            UnOp::Neg => f.print("-"),
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

impl<'a> Print for Put<'a> {
    fn print(&self, f: &mut Formatter) {
        f.print("put ");
        f.print(&self.0);
    }
}
