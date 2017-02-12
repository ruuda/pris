// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::rc::Rc;

use interpreter::{Env, Frame, Result, Val};

pub fn fit<'a>(_env: &Env<'a>, mut args: Vec<Val<'a>>) -> Result<Val<'a>> {
    if args.len() != 2 {
        let msg = format!("Arity error: 'fit' takes two arguments, \
                           but {} were provided.", args.len());
        return Err(msg)
    }
    let frame = match args.remove(0) {
        Val::Frame(f) => f,
        _ => {
            let msg = "Type error: 'fit' expects a frame as first argument, \
                       but a <TODO> was given instead.";
            return Err(String::from(msg))
        }
    };
    let size = match args.remove(0) {
        Val::Coord(w, h, 1) => (w, h),
        _ => {
            let msg = "Type error: 'fit' expects a coord of len as second argument, \
                       but a <TODO> was given instead.";
            return Err(String::from(msg))
        }
    };
    println!("TODO: Should fit frame in ({}, {}) and return it as frame.", size.0, size.1);
    Ok(Val::Frame(frame))
}

pub fn image<'a>(_env: &Env<'a>, mut args: Vec<Val<'a>>) -> Result<Val<'a>> {
    if args.len() != 1 {
        let msg = format!("Arity error: 'image' takes a single argument, \
                           but {} were provided.", args.len());
        return Err(msg)
    }
    let fname = match args.remove(0) {
        Val::Str(s) => s,
        _ => {
            let msg = "Type error: 'image' expects a string, \
                       but a <TODO> was given instead.";
            return Err(String::from(msg))
        }
    };

    println!("TODO: Should load image '{}' and return it as frame.", fname);

    let frame = Frame::new();
    Ok(Val::Frame(Rc::new(frame)))
}
