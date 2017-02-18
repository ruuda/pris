// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::rc::Rc;

use ast::Idents;
use error::{Error, Result};
use elements::{Element, Line};
use runtime::{Env, Frame, Val};
use types::ValType;

fn validate_args<'a>(fn_name: &str,
                     expected: &[ValType],
                     actual: &[Val<'a>])
                     -> Result<()> {
    // First check that we have exactly the right number of arguments.
    if expected.len() != actual.len() {
        return Err(Error::arity(fn_name, expected.len() as u32, actual.len() as u32))
    }

    // Then check the type of each.
    for (i, (ex, ac)) in expected.iter().zip(actual).enumerate() {
        if *ex != ac.get_type() {
            return Err(Error::arg_type(fn_name, *ex, ac.get_type(), i as u32))
        }
    }

    Ok(())
}

pub fn fit<'a>(_env: &Env<'a>, mut args: Vec<Val<'a>>) -> Result<Val<'a>> {
    validate_args("fit", &[ValType::Frame, ValType::Coord(1)], &args)?;
    let frame = match args.remove(0) {
        Val::Frame(f) => f,
        _ => unreachable!(),
    };
    let size = match args.remove(0) {
        Val::Coord(w, h, 1) => (w, h),
        _ => unreachable!(),
    };
    println!("TODO: Should fit frame in ({}, {}) and return it as frame.", size.0, size.1);
    Ok(Val::Frame(frame))
}

pub fn image<'a>(_env: &Env<'a>, mut args: Vec<Val<'a>>) -> Result<Val<'a>> {
    validate_args("image", &[ValType::Str], &args)?;
    let fname = match args.remove(0) {
        Val::Str(s) => s,
        _ => unreachable!(),
    };

    println!("TODO: Should load image '{}' and return it as frame.", fname);

    Ok(Val::Frame(Rc::new(Frame::new())))
}

pub fn line<'a>(env: &Env<'a>, mut args: Vec<Val<'a>>) -> Result<Val<'a>> {
    validate_args("line", &[ValType::Coord(1)], &args)?;
    let (x, y) = match args.remove(0) {
        Val::Coord(x, y, 1) => (x, y),
        _ => unreachable!(),
    };

    let line = Line {
        // TODO: Better idents type for non-ast use?
        color: env.lookup_color(&Idents(vec!["color"]))?,
        line_width: env.lookup_len(&Idents(vec!["line_width"]))?,
        x: x,
        y: y,
    };

    let mut frame = Frame::new();
    frame.place_element(0.0, 0.0, Element::Line(line));

    Ok(Val::Frame(Rc::new(frame)))
}

pub fn str<'a>(_env: &Env<'a>, mut args: Vec<Val<'a>>) -> Result<Val<'a>> {
    // TODO: Make this generic over the dimension?
    validate_args("str", &[ValType::Num(0)], &args)?;
    let num = match args.remove(0) {
        Val::Num(x, _) => x,
        _ => unreachable!(),
    };

    Ok(Val::Str(format!("{}", num)))
}

pub fn t<'a>(_env: &Env<'a>, mut args: Vec<Val<'a>>) -> Result<Val<'a>> {
    validate_args("t", &[ValType::Str], &args)?;
    let text = match args.remove(0) {
        Val::Str(s) => s,
        _ => unreachable!(),
    };

    println!("TODO: Generate a text frame for the text '{}'.", text);
    Ok(Val::Frame(Rc::new(Frame::new())))
}
