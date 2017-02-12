// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::collections::HashMap;
use std::rc::Rc;
use std::result;

use ast::{Assign, BinOp, BinTerm, Block, Color, Coord, FnCall, FnDef, Idents};
use ast::{Num, PutAt, Return, Stmt, Term, Unit};
use builtins;
use pretty::{Formatter, Print};

// Types used for the interpreter: values and an environment.

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

/// Represents a number of length dimensions.
///
/// -2 means "per area".
/// -1 means "per length".
/// 0 indicates a dimensionless number.
/// 1 means "length".
/// 2 means "area".
/// 3 means "volume".
/// etc.
type LenDim = i32;

#[derive(Clone)]
pub struct Frame<'a> {
    env: Env<'a>,
}

// A "builtin" function is a function that takes an environment and a vector of
// arguments, and produces a new value. We make a wrapper type to be able to
// implement a no-op clone on it.
pub struct Builtin(for<'a> fn(&Env<'a>, Vec<Val<'a>>) -> Result<Val<'a>>);

pub type Result<T> = result::Result<T, Error>;

pub type Error = String;

#[derive(Clone)]
pub struct Env<'a> {
    bindings: HashMap<&'a str, Val<'a>>,
}

impl Clone for Builtin {
    fn clone(&self) -> Builtin {
        let Builtin(x) = *self;
        Builtin(x)
    }
}

impl<'a> Frame<'a> {
    pub fn new() -> Frame<'a> {
        Frame {
            env: Env::new(),
        }
    }
}

impl<'a> Env<'a> {
    pub fn new() -> Env<'a> {
        let mut bindings = HashMap::new();
        // Default font size is 0.1h.
        bindings.insert("font_size", Val::Num(108.0, 1));
        bindings.insert("image", Val::FnIntrin(Builtin(builtins::image)));
        Env { bindings: bindings }
    }

    pub fn lookup(&self, idents: &Idents<'a>) -> Result<Val<'a>> {
        match self.bindings.get(idents.0[0]) {
            Some(val) => Ok(val.clone()), // TODO: Handle nested lookup.
            None => Err(format!("Variable '{}' does not exist.", idents.0[0])),
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
            Err(msg.into_string())
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
            Err(msg.into_string())
        }
    }

    pub fn put(&mut self, ident: &'a str, val: Val<'a>) {
        self.bindings.insert(ident, val);
    }
}

// Expression interpreter.

fn eval_expr<'a>(env: &Env<'a>, term: &'a Term<'a>) -> Result<Val<'a>> {
    match *term {
        Term::String(ref s) => Ok(eval_string(s)),
        Term::Number(ref x) => Ok(eval_num(env, x)),
        Term::Color(ref co) => Ok(eval_color(co)),
        Term::Idents(ref i) => env.lookup(i),
        Term::Coord(ref co) => eval_coord(env, co),
        Term::BinOp(ref bo) => eval_binop(env, bo),
        Term::FnCall(ref f) => eval_call(env, f),
        Term::FnDef(ref fd) => Ok(Val::FnExtrin(fd)),
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
            Unit::W => Val::Num(1920.0 * x, 1),
            Unit::H => Val::Num(1080.0 * x, 1),
            Unit::Pt => Val::Num(1.0 * x, 1),
            Unit::Em => {
                // The variable "font_size" should always be set, it is present
                // in the global environment.
                let ident_font_size = Idents(vec!["font_size"]);
                let emsize = env.lookup_len(&ident_font_size).unwrap();
                Val::Num(emsize * x, 1)
            }
        }
    } else {
        Val::Num(x, 0)
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
        (Val::Num(a, d), Val::Num(b, e)) if d == e => Ok(Val::Coord(a, b, d)),
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
        (Val::Num(x, d), Val::Num(y, e)) if d == e => Ok(Val::Num(x + y, d)),
        _ => {
            let msg = "Type error: '+' expects operands of the same type, \
                       num or len, but found <TODO> and <TODO> instead.";
            Err(String::from(msg))
        }
    }
}

fn eval_sub<'a>(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
    match (lhs, rhs) {
        (Val::Num(x, d), Val::Num(y, e)) if d == e => Ok(Val::Num(x - y, d)),
        _ => {
            let msg = "Type error: '-' expects operands of the same type, \
                       num or len, but found <TODO> and <TODO> instead.";
            Err(String::from(msg))
        }
    }
}

fn eval_mul<'a>(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
    match (lhs, rhs) {
        (Val::Num(x, d), Val::Num(y, e)) => Ok(Val::Num(x * y, d + e)),
        _ => {
            let msg = "Type error: '*' expects num or len operands, \
                       but found <TODO> and <TODO> instead.";
            Err(String::from(msg))
        }
    }
}

fn eval_div<'a>(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
    match (lhs, rhs) {
        (Val::Num(x, d), Val::Num(y, e)) => Ok(Val::Num(x / y, d - e)),
        _ => {
            let msg = "Type error: '/' expects num or len operands, \
                       but found <TODO> and <TODO> instead.";
            Err(String::from(msg))
        }
    }
}

fn eval_call<'a>(env: &Env<'a>, call: &'a FnCall<'a>) -> Result<Val<'a>> {
    let mut args = Vec::with_capacity(call.1.len());
    for arg in &call.1 {
        args.push(eval_expr(env, arg)?);
    }
    let func = eval_expr(env, &call.0)?;
    match func {
        // For a user-defined function, we evaluate the function body.
        Val::FnExtrin(fn_def) => eval_call_def(env, fn_def, args),
        // For a builtin function, the value carries a function pointer,
        // so we can just call that.
        Val::FnIntrin(Builtin(intrin)) => intrin(env, args),
        // Other things are not callable.
        _ => {
            let msg = "Type error: attempting to call value of type <TODO>. \
                       Only functions can be called.";
            Err(String::from(msg))
        }
    }
}

fn eval_call_def<'a>(env: &Env<'a>,
                     fn_def: &'a FnDef<'a>,
                     args: Vec<Val<'a>>)
                     -> Result<Val<'a>> {
    // Ensure that a value is provided for every argument, and no more.
    if fn_def.0.len() != args.len() {
        let msg = format!("Arity error: function takes {} arguments, \
                           but {} were provided.",
                          fn_def.0.len(), args.len());
        return Err(msg)
    }

    // For a function call, bring the argument in scope as variables, and then
    // evaluate the body block.
    let mut inner_env = env.clone();
    for (arg_name, val) in fn_def.0.iter().zip(args) {
        inner_env.put(arg_name, val);
    }

    eval_block(&inner_env, &fn_def.1)
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
        x @ Val::Coord(_, _, 1) => x,
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
            Val::Num(x, d) => {
                f.print_f64(x);
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
                f.print_f64(r);
                f.print(", ");
                f.print_f64(g);
                f.print(", ");
                f.print_f64(b);
                f.print(") : color");
            }
            Val::Coord(x, y, d) => {
                f.print("(");
                f.print_f64(x);
                f.print(", ");
                f.print_f64(y);
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

fn print_unit(f: &mut Formatter, d: LenDim) {
    match d {
        -3 => f.print("len⁻³"),
        -2 => f.print("len⁻²"),
        -1 => f.print("len⁻¹"),
        0 => f.print("num"),
        1 => f.print("len"),
        2 => f.print("len²"),
        3 => f.print("len³"),
        n => { f.print("len^"); f.print_i32(n); }
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
