// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::collections::HashMap;
use std::rc::Rc;
use std::result;

use ast::{Assign, BinOp, BinTerm, Block, Color, Coord, FnDef, Idents, Num};
use ast::{PutAt, Return, Stmt, Term, Unit};
use pretty::{Formatter, Print};

// Types used for the interpreter: values and an environment.

#[derive(Clone)]
pub enum Val<'a> {
    Num(f64), // TODO: Be consistent about abbreviating things.
    Len(f64),
    Str(String),
    Col(f64, f64, f64),
    NumCoord(f64, f64),
    LenCoord(f64, f64),
    Frame(Rc<Frame<'a>>),
    Fn(&'a FnDef<'a>),
}

#[derive(Clone)]
pub struct Frame<'a> {
    env: Env<'a>,
}

#[derive(Clone)]
pub struct Env<'a> {
    bindings: HashMap<&'a str, Val<'a>>,
}

impl<'a> Env<'a> {
    pub fn new() -> Env<'a> {
        let mut bindings = HashMap::new();
        // Default font size is 0.1h.
        bindings.insert("font_size", Val::Len(108.0));
        Env { bindings: bindings }
    }

    pub fn lookup(&self, idents: &Idents<'a>) -> Result<Val<'a>> {
        match self.bindings.get(idents.0[0]) {
            Some(val) => Ok(val.clone()), // TODO: Handle nested lookup.
            None => Err(format!("Variable '{}' does not exist.", idents.0[0])),
        }
    }

    pub fn lookup_num(&self, idents: &Idents<'a>) -> Result<f64> {
        if let Val::Num(x) = self.lookup(idents)? {
            Ok(x)
        } else {
            let mut msg = Formatter::new();
            msg.print("Type error: expected num, but ");
            msg.print(idents);
            msg.print("is <TODO>.");
            Err(msg.into_string())
        }
    }

    pub fn lookup_len(&self, idents: &Idents<'a>) -> Result<f64> {
        if let Val::Len(x) = self.lookup(idents)? {
            Ok(x)
        } else {
            let mut msg = Formatter::new();
            msg.print("Type error: expected len, but ");
            msg.print(idents);
            msg.print("is <TODO>.");
            Err(msg.into_string())
        }
    }

    pub fn put(&mut self, ident: &'a str, val: Val<'a>) {
        self.bindings.insert(ident, val);
    }
}

pub type Result<T> = result::Result<T, Error>;

pub type Error = String;

// Expression interpreter.

fn eval_expr<'a>(env: &Env<'a>, term: &'a Term<'a>) -> Result<Val<'a>> {
    match *term {
        Term::String(ref s) => Ok(eval_string(s)),
        Term::Number(ref x) => Ok(eval_num(env, x)),
        Term::Color(ref c) => Ok(eval_color(c)),
        Term::Idents(ref path) => env.lookup(path),
        Term::Coord(ref co) => eval_coord(env, co),
        Term::BinOp(ref bo) => eval_binop(env, bo),
        Term::FnCall(ref _fc) => panic!("TODO: eval fncall"),
        Term::FnDef(ref fd) => Ok(Val::Fn(fd)),
        Term::Block(ref bk) => eval_block(env, bk),
    }
}

fn eval_string<'a>(s: &'a str) -> Val<'a> {
    // Strip off the quotes at the start and end.
    let string = String::from(&s[1..s.len() - 1]);
    // TODO: Handle escape sequences.
    Val::Str(string)
}

fn eval_num<'a>(env: &Env<'a>, num: &'a Num) -> Val<'a> {
    let Num(x, opt_unit) = *num;
    if let Some(unit) = opt_unit {
        match unit {
            Unit::W => Val::Len(1920.0 * x),
            Unit::H => Val::Len(1080.0 * x),
            Unit::Pt => Val::Len(1.0 * x),
            Unit::Em => {
                // The variable "font_size" should always be set, it is present
                // in the global environment.
                let ident_font_size = Idents(vec!["font_size"]);
                let emsize = env.lookup_len(&ident_font_size).unwrap();
                Val::Len(emsize * x)
            }
        }
    } else {
        Val::Num(x)
    }
}

fn eval_color<'a>(col: &Color) -> Val<'a> {
    let Color(rbyte, gbyte, bbyte) = *col;
    Val::Col(rbyte as f64 / 255.0, gbyte as f64 / 255.0, bbyte as f64 / 255.0)
}

fn eval_coord<'a>(env: &Env<'a>, coord: &'a Coord<'a>) -> Result<Val<'a>> {
    let x = eval_expr(env, &coord.0)?;
    let y = eval_expr(env, &coord.1)?;
    match (x, y) {
        (Val::Num(a), Val::Num(b)) => Ok(Val::NumCoord(a, b)),
        (Val::Len(a), Val::Len(b)) => Ok(Val::LenCoord(a, b)),
        _ => {
            let msg = "Type error: coord must be (num, num) or (len, len), \
                       but found (<TODO>, <TODO>) instead.";
            Err(String::from(msg))
        }
    }
}

fn eval_binop<'a>(env: &Env<'a>, binop: &'a BinTerm<'a>) -> Result<Val<'a>> {
    let lhs = eval_expr(env, &binop.0)?;
    let rhs = eval_expr(env, &binop.2)?;
    match binop.1 {
        BinOp::Add => eval_add(lhs, rhs),
        BinOp::Sub => eval_sub(lhs, rhs),
        BinOp::Mul => eval_mul(lhs, rhs),
        BinOp::Div => eval_div(lhs, rhs),
        BinOp::Exp => panic!("TODO: eval exp"),
    }
}

fn eval_add<'a>(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
    match (lhs, rhs) {
        (Val::Num(x), Val::Num(y)) => Ok(Val::Num(x + y)),
        (Val::Len(x), Val::Len(y)) => Ok(Val::Len(x + y)),
        _ => {
            let msg = "Type error: '+' expects operands of the same type, \
                       num or len, but found <TODO> and <TODO> instead.";
            Err(String::from(msg))
        }
    }
}

fn eval_sub<'a>(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
    match (lhs, rhs) {
        (Val::Num(x), Val::Num(y)) => Ok(Val::Num(x - y)),
        (Val::Len(x), Val::Len(y)) => Ok(Val::Len(x - y)),
        _ => {
            let msg = "Type error: '-' expects operands of the same type, \
                       num or len, but found <TODO> and <TODO> instead.";
            Err(String::from(msg))
        }
    }
}

fn eval_mul<'a>(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
    match (lhs, rhs) {
        (Val::Num(x), Val::Num(y)) => Ok(Val::Num(x * y)),
        (Val::Len(x), Val::Num(y)) => Ok(Val::Len(x * y)),
        (Val::Num(x), Val::Len(y)) => Ok(Val::Len(x * y)),
        (Val::Len(_), Val::Len(_)) => {
            let msg = "Type error: multiplying two lengths would produce an area, \
                       but area values are not suppored.";
            Err(String::from(msg))
        }
        _ => {
            let msg = "Type error: '*' expects num or len operands, \
                       but found <TODO> and <TODO> instead.";
            Err(String::from(msg))
        }
    }
}

fn eval_div<'a>(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
    match (lhs, rhs) {
        (Val::Num(x), Val::Num(y)) => Ok(Val::Num(x / y)),
        (Val::Len(x), Val::Num(y)) => Ok(Val::Len(x / y)),
        (Val::Len(x), Val::Len(y)) => Ok(Val::Num(x / y)),
        (Val::Num(_), Val::Len(_)) => {
            let msg = "Type error: dividing a number by a length would produce \
                       a value of inverse length, but this is not supported.";
            Err(String::from(msg))
        }
        _ => {
            let msg = "Type error: '/' expects num or len operands, \
                       but found <TODO> and <TODO> instead.";
            Err(String::from(msg))
        }
    }
}

fn eval_block<'a>(env: &Env<'a>, block: &'a Block<'a>) -> Result<Val<'a>> {
    // A block is evaluated in its enclosing environment, but it does not modify
    // the environment, it gets a copy.
    let mut inner_env = (*env).clone();

    for statement in &block.0 {
        match *statement {
            // A return statement in a block determines the value that the block
            // evalates to, if a return is present.
            Stmt::Return(Return(ref r)) => return eval_expr(&inner_env, r),
            // A block statemen to make a frame can only be used at the top
            // level.
            Stmt::Block(..) => {
                let msg = "Error: slides can only be introduced at the top level. \
                           Note: use 'at (0w, 0w) put { ... }' to place a frame.";
                return Err(String::from(msg));
            }
            // Otherwise, evaluating a statement just mutates the environment.
            _ => {
                let maybe_frame = eval_statement(&mut inner_env, statement)?;
                assert!(maybe_frame.is_none());
            }
        }
    }

    let frame = Frame { env: inner_env };
    Ok(Val::Frame(Rc::new(frame)))
}

// Statement interpreter.

pub fn eval_statement<'a>(env: &mut Env<'a>,
                          stmt: &'a Stmt<'a>) -> Result<Option<Rc<Frame<'a>>>> {
    match *stmt {
        Stmt::Import(ref _i) => {
            println!("TODO: eval import");
            Ok(None)
        }
        Stmt::Assign(ref a) => {
            eval_assign(env, a)?;
            Ok(None)
        }
        Stmt::Return(..) => {
            // The return case is handled in block evaluation. A bare return
            // statement does not make sense.
            Err(String::from("Syntax error: 'return' cannot be used here."))
        }
        Stmt::Block(ref bk) => {
            if let Val::Frame(frame) = eval_block(env, bk)? {
                Ok(Some(frame))
            } else {
                let msg = "Type error: top-level blocks must evaluate to frames, \
                           but a <TODO> was encountered instead.";
                Err(String::from(msg))
            }
        }
        Stmt::PutAt(ref pa) => {
            eval_put_at(env, pa)?;
            Ok(None)
        }
    }
}

fn eval_assign<'a>(env: &mut Env<'a>, stmt: &'a Assign<'a>) -> Result<()> {
    let Assign(target, ref expression) = *stmt;
    let value = eval_expr(env, expression)?;
    env.put(target, value);
    Ok(())
}

fn eval_put_at<'a>(env: &Env<'a>, put_at: &'a PutAt<'a>) -> Result<()> {
    let content = match eval_expr(env, &put_at.0)? {
        x @ Val::Frame(..) => x,
        // TODO: Allow placing strings?
        _ => {
            let msg = "Cannot place <TODO>. Only frames can be placed.";
            return Err(String::from(msg));
        }
    };

    let location = match eval_expr(env, &put_at.1)? {
        x @ Val::LenCoord(..) => x,
        _ => {
            let msg = "Placement requires a coordinate with length units, \
                       but a <TODO> was found instead.";
            return Err(String::from(msg));
        }
    };

    let mut f = Formatter::new();
    f.print("TODO: at ");
    f.print(&location);
    f.print(" put ");
    f.print(&content);
    println!("{}", f.into_string());
    Ok(())
}

// Pretty printers for values and interpreter data structures.

impl<'a> Print for Val<'a> {
    fn print(&self, f: &mut Formatter) {
        match *self {
            Val::Num(x) => {
                f.print_f64(x);
                f.print(" (num)");
            }
            Val::Len(x) => {
                f.print_f64(x);
                f.print(" (len)");
            }
            Val::Str(ref s) => {
                f.print("\"");
                f.print(&s[..]); // TODO: Escaping.
                f.print("\"");
            }
            Val::Col(r, g, b) => {
                f.print("(");
                f.print_f64(r);
                f.print(", ");
                f.print_f64(g);
                f.print(", ");
                f.print_f64(b);
                f.print(") (color)");
            }
            Val::NumCoord(x, y) => {
                f.print("(");
                f.print_f64(x);
                f.print(", ");
                f.print_f64(y);
                f.print(") (num)");
            }
            Val::LenCoord(x, y) => {
                f.print("(");
                f.print_f64(x);
                f.print(", ");
                f.print_f64(y);
                f.print(") (len)");
            }
            Val::Frame(ref frame) => {
                f.print(frame);
            }
            Val::Fn(ref fndef) => {
                f.print(fndef);
            }
        }
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
