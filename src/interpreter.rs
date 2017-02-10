// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::collections::HashMap;

use ast::{Assign, FnDef, Idents, Num, Stmt, Term, Unit};
use std::rc::Rc;
use std::result;

// Types used for the interpreter: values and an environment.

#[derive(Clone, Debug)]
enum Val<'a> {
    Num(f64), // TODO: Be consistent about abbreviating things.
    Len(f64),
    Str(&'a str),
    Col(f64, f64, f64),
    NumCoord(f64, f64),
    LenCoord(f64, f64),
    Box(Rc<Env<'a>>),
    Fn(Rc<FnDef<'a>>),
}

#[derive(Debug)]
pub struct Env<'a> {
    bindings: HashMap<&'a str, Val<'a>>,
}

impl<'a> Env<'a> {
    pub fn new() -> Env<'a> {
        Env { bindings: HashMap::new() }
    }

    pub fn lookup(&self, idents: &Idents<'a>) -> Result<Val<'a>> {
        panic!("TODO: implement lookup.");
    }

    pub fn lookup_num(&self, idents: &Idents<'a>) -> Result<f64> {
        if let Val::Num(x) = self.lookup(idents)? {
            Ok(x)
        } else {
            let msg = format!("Type error: expected num, but {} is <TODO>.", idents);
            Err(msg)
        }
    }

    pub fn lookup_len(&self, idents: &Idents<'a>) -> Result<f64> {
        if let Val::Len(x) = self.lookup(idents)? {
            Ok(x)
        } else {
            let msg = format!("Type error: expected num, but {} is <TODO>.", idents);
            Err(msg)
        }
    }

    pub fn put(&mut self, ident: &'a str, val: Val<'a>) {
        self.bindings.insert(ident, val);
    }
}

pub type Result<T> = result::Result<T, Error>;

pub type Error = String;

// Expression interpreter.

fn eval_expr<'a>(env: &Env<'a>, term: &Term<'a>) -> Result<Val<'a>> {
    match *term {
        Term::String(ref s) => Ok(Val::Str(s)),
        Term::Number(ref x) => Ok(eval_num(env, x)),
        Term::Color(ref c) => panic!("TODO: eval color"),
        Term::Idents(ref path) => env.lookup(path),
        Term::Coord(ref co) => panic!("TODO: eval coordinate"),
        Term::BinOp(ref bo) => panic!("TODO: eval binop"),
        Term::FnCall(ref fc) => panic!("TODO: eval fncall"),
        Term::FnDef(ref fd) => panic!("TODO: eval fndef"),
        Term::Block(ref bk) => panic!("TODO: eval block"),
    }
}

fn eval_num<'a>(env: &Env<'a>, num: &Num) -> Val<'a> {
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

// Statement interpreter.

pub fn eval_statement<'a>(env: &mut Env<'a>, stmt: &Stmt<'a>) -> Result<()> {
    match *stmt {
        Stmt::Import(ref i) => panic!("TODO: eval import"),
        Stmt::Assign(ref a) => eval_assign(env, a),
        Stmt::Return(ref r) => panic!("TODO: eval return"),
        Stmt::Block(ref bk) => panic!("TODO: eval block"),
        Stmt::PutAt(ref pa) => panic!("TODO: eval put at"),
    }
}

fn eval_assign<'a>(env: &mut Env<'a>, stmt: &Assign<'a>) -> Result<()> {
    let Assign(target, ref expression) = *stmt;
    let value = eval_expr(env, expression)?;
    env.put(target, value);
    Ok(())
}
