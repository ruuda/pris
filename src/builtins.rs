// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::rc::Rc;

use ast::Idents;
use cairo;
use elements::{Element, Line, Text};
use error::{Error, Result};
use harfbuzz;
use runtime::{Env, FontMap, Frame, Val};
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

pub fn fit<'a>(_fm: &mut FontMap,
               _env: &Env<'a>,
               mut args: Vec<Val<'a>>)
               -> Result<Val<'a>> {
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

pub fn image<'a>(_fm: &mut FontMap,
                 _env: &Env<'a>,
                 mut args: Vec<Val<'a>>)
                 -> Result<Val<'a>> {
    validate_args("image", &[ValType::Str], &args)?;
    let fname = match args.remove(0) {
        Val::Str(s) => s,
        _ => unreachable!(),
    };

    println!("TODO: Should load image '{}' and return it as frame.", fname);

    Ok(Val::Frame(Rc::new(Frame::new())))
}

pub fn line<'a>(_fm: &mut FontMap,
                env: &Env<'a>,
                mut args: Vec<Val<'a>>)
                -> Result<Val<'a>> {
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

pub fn str<'a>(_fm: &mut FontMap,
               _env: &Env<'a>,
               mut args: Vec<Val<'a>>)
               -> Result<Val<'a>> {
    // TODO: Make this generic over the dimension?
    validate_args("str", &[ValType::Num(0)], &args)?;
    let num = match args.remove(0) {
        Val::Num(x, _) => x,
        _ => unreachable!(),
    };

    Ok(Val::Str(format!("{}", num)))
}

pub fn t<'a>(fm: &mut FontMap,
             env: &Env<'a>,
             mut args: Vec<Val<'a>>)
             -> Result<Val<'a>> {
    validate_args("t", &[ValType::Str], &args)?;
    let text = match args.remove(0) {
        Val::Str(s) => s,
        _ => unreachable!(),
    };

    // Read the font details from the 'font_family' and 'font_style' variables,
    // and locate the corresponding FreeType face.
    let font_family = env.lookup_str(&Idents(vec!["font_family"]))?;
    let font_style = env.lookup_str(&Idents(vec!["font_style"]))?;
    let font_size = env.lookup_len(&Idents(vec!["font_size"]))?;
    let mut ft_face = match fm.get(&font_family, &font_style) {
        Some(face) => face,
        None => return Err(Error::missing_font(font_family, font_style)),
    };

    // Shape the text using Harfbuzz: convert the UTF-8 string and input font
    // into a list of glyphs with offsets.
    let mut hb_font = harfbuzz::Font::from_ft_face(&mut ft_face);
    let mut hb_buffer = harfbuzz::Buffer::new(harfbuzz::Direction::LeftToRight);
    hb_buffer.add_str(&text);
    hb_buffer.shape(&mut hb_font);

    // Position all the glyphs: Harfbuzz gives offsets, but we need absolute
    // locations. Store them in the representation that Cairo expects.
    let hb_glyphs = hb_buffer.glyphs();
    let mut cr_glyphs = Vec::with_capacity(hb_glyphs.len());
    let (mut cur_x, mut cur_y) = (0.0, 0.0);
    // Compensate for the fixed font size which is set for the Freetype font,
    // and apply the desired font size.
    let size_factor = font_size / 1000.0;
    for hg in hb_glyphs {
        cur_x += hg.x_offset as f64 * size_factor;
        cur_y += hg.y_offset as f64 * size_factor;
        let cg = cairo::Glyph::new(hg.codepoint as u64, cur_x, cur_y);
        cur_x += hg.x_advance as f64 * size_factor;
        cur_y += hg.y_advance as f64 * size_factor;
        cr_glyphs.push(cg);
    }

    let text_elem = Text {
        color: env.lookup_color(&Idents(vec!["color"]))?,
        glyphs: cr_glyphs,
    };

    let mut frame = Frame::new();
    frame.place_element(0.0, 0.0, Element::Text(text_elem));
    Ok(Val::Frame(Rc::new(frame)))
}
