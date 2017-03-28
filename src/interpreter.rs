// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::rc::Rc;
use std::char;

use ast;
use ast::{Assign, BinOp, BinTerm, Block, Coord, FnCall, FnDef, Idents};
use ast::{Num, PutAt, Return, Stmt, Term, Unit};
use error::{Error, Result};
use elements::{Color, Vec2};
use pretty::Formatter;
use runtime::{Builtin, FontMap, Frame, Env, Val};
use types::ValType;

// Expression interpreter.

fn eval_expr<'a>(fm: &mut FontMap,
                 env: &Env<'a>,
                 term: &'a Term<'a>)
                 -> Result<Val<'a>> {
    match *term {
        Term::String(ref s) => Ok(eval_string(s)),
        Term::Number(ref x) => eval_num(env, x),
        Term::Color(ref co) => Ok(eval_color(co)),
        Term::Idents(ref i) => env.lookup(i),
        Term::Coord(ref co) => eval_coord(fm, env, co),
        Term::BinOp(ref bo) => eval_binop(fm, env, bo),
        Term::FnCall(ref f) => eval_call(fm, env, f),
        Term::FnDef(ref fd) => Ok(Val::FnExtrin(fd)),
        Term::Block(ref bk) => eval_block(fm, env, bk),
    }
}

fn eval_string<'a>(s: &'a str) -> Val<'a> {
    // Strip off the quotes at the start and end.
    let string = String::from(&s[1..s.len() - 1]);
    let mut string = String::with_capacity(s.len() - 2);

    // TODO: Move escape sequence handling into the parser.

    // Parsing escape sequences in a string literal is a small state machine
    // with the following states.
    enum EscState {
        Normal,
        Escape,
        Unicode(u32, u32),
    }

    let mut st = EscState::Normal;

    // Iterate all characters except for the enclosing quotes.
    for ch in s[1..s.len() - 1].chars() {
        match st {
            EscState::Normal => {
                match ch {
                    '\\' => st = EscState::Escape,
                    _ => string.push(ch),
                }
            }
            EscState::Escape => {
                match ch {
                    '\\' => { string.push('\\'); st = EscState::Normal; }
                    '"' => { string.push('"'); st = EscState::Normal; }
                    'u' => { st = EscState::Unicode(0, 6); }
                    _ => panic!("Invalid escape code. TODO: Turn into parse error."),
                }
            }
            EscState::Unicode(codepoint, num_left) => {
                // An unicode escape sequence of the form \u1f574 consists of at
                // most 6 hexadecimal characters, and ends at the first non-hex
                // character. Examples:
                // "\u"        -> U+0
                // "\u1f574z"  -> U+1F574, U+7A
                // "\u1f5741"  -> U+1F5741
                // "\u01f5741" -> U+1F574, U+31
                if ch.is_digit(16) && num_left > 0 {
                    // Parsing the digit will succeed, because we checked above.
                    let d = ch.to_digit(16).unwrap();
                    st = EscState::Unicode(codepoint * 16 + d, num_left - 1);
                } else {
                    let c = char::from_u32(codepoint)
                        .expect("Invalid code point. TODO: Turn into parse error.");
                    string.push(c);
                    string.push(ch);
                    st = EscState::Normal;
                }
            }
        }
    }

    match st {
        EscState::Unicode(codepoint, _num_left) => {
            let c = char::from_u32(codepoint)
                .expect("Invalid code point. TODO: Turn into parse error.");
            string.push(c);
        }
        _ => { }
    }

    Val::Str(string)
}

fn eval_num<'a>(env: &Env<'a>, num: &'a Num) -> Result<Val<'a>> {
    let Num(x, opt_unit) = *num;
    if let Some(unit) = opt_unit {
        match unit {
            Unit::W => Ok(Val::Num(1920.0 * x, 1)),
            Unit::H => Ok(Val::Num(1080.0 * x, 1)),
            Unit::Pt => Ok(Val::Num(1.0 * x, 1)),
            Unit::Em => {
                // The variable "font_size" should always be set, it is present
                // in the global environment.
                let ident_font_size = Idents(vec!["font_size"]);
                let emsize = env.lookup_len(&ident_font_size)?;
                Ok(Val::Num(emsize * x, 1))
            }
        }
    } else {
        Ok(Val::Num(x, 0))
    }
}

fn eval_color<'a>(col: &ast::Color) -> Val<'a> {
    let ast::Color(rbyte, gbyte, bbyte) = *col;
    let cf64 = Color::new(rbyte as f64 / 255.0, gbyte as f64 / 255.0, bbyte as f64 / 255.0);
    Val::Col(cf64)
}

fn eval_coord<'a>(fm: &mut FontMap,
                  env: &Env<'a>,
                  coord: &'a Coord<'a>)
                  -> Result<Val<'a>> {
    let x = eval_expr(fm, env, &coord.0)?;
    let y = eval_expr(fm, env, &coord.1)?;
    match (x, y) {
        (Val::Num(a, d), Val::Num(b, e)) if d == e => Ok(Val::Coord(a, b, d)),
        _ => {
            let msg = "Type error: coord must be (num, num) or (len, len), \
                       but found (<TODO>, <TODO>) instead.";
            Err(Error::Other(String::from(msg)))
        }
    }
}

fn eval_binop<'a>(fm: &mut FontMap,
                  env: &Env<'a>,
                  binop: &'a BinTerm<'a>)
                  -> Result<Val<'a>> {
    let lhs = eval_expr(fm, env, &binop.0)?;
    let rhs = eval_expr(fm, env, &binop.2)?;
    match binop.1 {
        BinOp::Adj => eval_adj(lhs, rhs),
        BinOp::Add => eval_add(lhs, rhs),
        BinOp::Sub => eval_sub(lhs, rhs),
        BinOp::Mul => eval_mul(lhs, rhs),
        BinOp::Div => eval_div(lhs, rhs),
        BinOp::Exp => panic!("TODO: eval exp"),
    }
}

/// Adjoins two frames.
fn eval_adj<'a>(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
    match (lhs, rhs) {
        (Val::Frame(f0), Val::Frame(f1)) => {
            let mut frame = (*f0).clone();
            let anchor = f0.get_anchor();
            for pe in f1.get_elements() {
                frame.place_element(anchor + pe.position, pe.element.clone());
            }
            frame.set_anchor(anchor + f1.get_anchor());
            Ok(Val::Frame(Rc::new(frame)))
        }
        (lhs, rhs) => {
            Err(Error::binop_type("~", ValType::Frame, lhs.get_type(), rhs.get_type()))
        }
    }
}

fn eval_add<'a>(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
    match (lhs, rhs) {
        (Val::Num(x0, d0), Val::Num(x1, d1)) if d0 == d1 => {
            Ok(Val::Num(x0 + x1, d0))
        }
        (Val::Coord(x0, y0, d0), Val::Coord(x1, y1, d1)) if d0 == d1 => {
            Ok(Val::Coord(x0 + x1, y0 + y1, d0))
        }
        (Val::Str(a), Val::Str(b)) => {
            Ok(Val::Str(a + &b))
        }
        (lhs, rhs) => {
            let mut f = Formatter::new();
            f.print("Type error: '+' expects operands of the same type, \
                     num or len or coords thereof, \
                     but found '");
            f.print(lhs);
            f.print("' and '");
            f.print(rhs);
            f.print("' instead.");
            Err(Error::Other(f.into_string()))
        }
    }
}

fn eval_sub<'a>(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
    match (lhs, rhs) {
        (Val::Num(x0, d0), Val::Num(x1, d1)) if d0 == d1 => {
            Ok(Val::Num(x0 - x1, d0))
        }
        (Val::Coord(x0, y0, d0), Val::Coord(x1, y1, d1)) if d0 == d1 => {
            Ok(Val::Coord(x0 - x1, y0 - y1, d0))
        }
        (lhs, rhs) => {
            let mut f = Formatter::new();
            f.print("Type error: '-' expects operands of the same type, \
                     num or len or coords thereof, \
                     but found '");
            f.print(lhs);
            f.print("' and '");
            f.print(rhs);
            f.print("' instead.");
            Err(Error::Other(f.into_string()))
        }
    }
}

fn eval_mul<'a>(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
    match (lhs, rhs) {
        (Val::Num(x, d), Val::Num(y, e)) => Ok(Val::Num(x * y, d + e)),
        _ => {
            let msg = "Type error: '*' expects num or len operands, \
                       but found <TODO> and <TODO> instead.";
            Err(Error::Other(String::from(msg)))
        }
    }
}

fn eval_div<'a>(lhs: Val<'a>, rhs: Val<'a>) -> Result<Val<'a>> {
    match (lhs, rhs) {
        (Val::Num(x, d), Val::Num(y, e)) => Ok(Val::Num(x / y, d - e)),
        _ => {
            let msg = "Type error: '/' expects num or len operands, \
                       but found <TODO> and <TODO> instead.";
            Err(Error::Other(String::from(msg)))
        }
    }
}

fn eval_call<'a>(fm: &mut FontMap,
                 env: &Env<'a>,
                 call: &'a FnCall<'a>)
                 -> Result<Val<'a>> {
    let mut args = Vec::with_capacity(call.1.len());
    for arg in &call.1 {
        args.push(eval_expr(fm, env, arg)?);
    }
    let func = eval_expr(fm, env, &call.0)?;
    match func {
        // For a user-defined function, we evaluate the function body.
        Val::FnExtrin(fn_def) => eval_call_def(fm, env, fn_def, args),
        // For a builtin function, the value carries a function pointer,
        // so we can just call that.
        Val::FnIntrin(Builtin(intrin)) => intrin(fm, env, args),
        // Other things are not callable.
        _ => {
            let msg = "Type error: attempting to call value of type <TODO>. \
                       Only functions can be called.";
            Err(Error::Other(String::from(msg)))
        }
    }
}

fn eval_call_def<'a>(fm: &mut FontMap,
                     env: &Env<'a>,
                     fn_def: &'a FnDef<'a>,
                     args: Vec<Val<'a>>)
                     -> Result<Val<'a>> {
    // Ensure that a value is provided for every argument, and no more.
    if fn_def.0.len() != args.len() {
        let msg = format!("Arity error: function takes {} arguments, \
                           but {} were provided.",
                          fn_def.0.len(), args.len());
        return Err(Error::Other(msg))
    }

    // For a function call, bring the argument in scope as variables, and then
    // evaluate the body block.
    let mut inner_env = env.clone();
    for (arg_name, val) in fn_def.0.iter().zip(args) {
        inner_env.put(arg_name, val);
    }

    eval_block(fm, &inner_env, &fn_def.1)
}

fn eval_block<'a>(fm: &mut FontMap,
                  env: &Env<'a>,
                  block: &'a Block<'a>)
                  -> Result<Val<'a>> {
    // A block is evaluated in its enclosing environment, but it does not modify
    // the environment, it gets a copy.
    let inner_env = (*env).clone();
    let mut frame = Frame::from_env(inner_env);

    for statement in &block.0 {
        match *statement {
            // A return statement in a block determines the value that the block
            // evalates to, if a return is present.
            Stmt::Return(Return(ref r)) => return eval_expr(fm, frame.get_env(), r),
            // A block statemen to make a frame can only be used at the top
            // level.
            Stmt::Block(..) => {
                let msg = "Error: slides can only be introduced at the top level. \
                           Note: use 'at (0w, 0w) put { ... }' to place a frame.";
                return Err(Error::Other(String::from(msg)));
            }
            // Otherwise, evaluating a statement just mutates the environment.
            _ => {
                let maybe_frame = eval_statement(fm, &mut frame, statement)?;
                assert!(maybe_frame.is_none());
            }
        }
    }

    Ok(Val::Frame(Rc::new(frame)))
}

// Statement interpreter.

pub fn eval_statement<'a>(fm: &mut FontMap,
                          frame: &mut Frame<'a>,
                          stmt: &'a Stmt<'a>)
                          -> Result<Option<Rc<Frame<'a>>>> {
    match *stmt {
        Stmt::Import(ref _i) => {
            println!("TODO: eval import");
            Ok(None)
        }
        Stmt::Assign(ref a) => {
            eval_assign(fm, frame, a)?;
            Ok(None)
        }
        Stmt::Return(..) => {
            // The return case is handled in block evaluation. A bare return
            // statement does not make sense.
            Err(Error::Other(String::from("Syntax error: 'return' cannot be used here.")))
        }
        Stmt::Block(ref bk) => {
            if let Val::Frame(frame) = eval_block(fm, frame.get_env(), bk)? {
                Ok(Some(frame))
            } else {
                let msg = "Type error: top-level blocks must evaluate to frames, \
                           but a <TODO> was encountered instead.";
                Err(Error::Other(String::from(msg)))
            }
        }
        Stmt::PutAt(ref pa) => {
            eval_put_at(fm, frame, pa)?;
            Ok(None)
        }
    }
}

fn eval_assign<'a>(fm: &mut FontMap,
                   frame: &mut Frame<'a>,
                   stmt: &'a Assign<'a>)
                   -> Result<()> {
    let Assign(target, ref expression) = *stmt;
    let value = eval_expr(fm, frame.get_env(), expression)?;
    frame.put_in_env(target, value);
    Ok(())
}

fn eval_put_at<'a>(fm: &mut FontMap,
                   frame: &mut Frame<'a>,
                   put_at: &'a PutAt<'a>)
                   -> Result<()> {
    let content = match eval_expr(fm, frame.get_env(), &put_at.0)? {
        Val::Frame(f) => f,
        _ => {
            let msg = "Cannot place <TODO>. Only frames can be placed.";
            return Err(Error::Other(String::from(msg)));
        }
    };

    let pos = match eval_expr(fm, frame.get_env(), &put_at.1)? {
        // TODO: Make Coord type carry Vec2 instead of separate x, y.
        Val::Coord(x, y, 1) => Vec2::new(x, y),
        _ => {
            let msg = "Placement requires a coordinate with length units, \
                       but a <TODO> was found instead.";
            return Err(Error::Other(String::from(msg)));
        }
    };

    for pe in content.get_elements() {
        frame.place_element(pos + pe.position, pe.element.clone());
    }

    Ok(())
}
