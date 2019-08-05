// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::path::PathBuf;
use std::rc::Rc;

use ast::Idents;
use cairo;
use elements::{Element, FillPolygon, Hyperlink, StrokePolygon, PolygonKind, Text, Vec2};
use error::{Error, Result};
use freetype;
use harfbuzz;
use names;
use png;
use pretty::Formatter;
use rsvg;
use runtime::{BoundingBox, Frame, Subframe, Val};
use types::ValType;

// TODO: Put that somewhere else.
use interpreter::ExprInterpreter;

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

pub fn at<'i, 'a>(_interpreter: &mut ExprInterpreter<'i, 'a>,
                  mut args: Vec<Val<'a>>)
                  -> Result<Val<'a>> {

    // TODO: This generic `validate_args` does lose a bit on good error
    // messages. Write a custom one, as "at" is so prevalent?
    validate_args(names::at, &[ValType::Frame, ValType::Coord(1)], &args)?;
    // let msg = "Cannot translate (at) <something else>. Only frames can be translated.";
    // let msg = "Translation (at) requires a coordinate with length units, \
    //            but a <something else> was found instead.";
    let frame = match args.remove(0) {
        Val::Frame(f) => f,
        _ => unreachable!(),
    };
    let off = match args.remove(0) {
        Val::Coord(x, y, 1) => Vec2::new(x, y),
        _ => unreachable!(),
    };

    let mut new_frame = Frame::from_env(frame.get_env().clone());

    for subframe in frame.get_subframes() {
        let mut dest_sf = Subframe::new();
        for pe in subframe.get_elements() {
            dest_sf.place_element(pe.position + off, pe.element.clone());
        }
        new_frame.push_subframe(dest_sf);
    }

    new_frame.union_bounding_box(&frame.get_bounding_box().offset(off));

    // Pris always included the origin in the bounding box. This has
    // advantages and disadvantages. For example, you can create a space to
    // ajoin between elements by doing
    //
    //     hspace = function(dx) { put {} at (dx, 0em) }
    //
    // However, I am now leaning towards removing this behavior; I don't
    // like the special case, and there should just be a primitive to create
    // an empty bounding box.
    let bb = BoundingBox::new(off, Vec2::new(0.0, 0.0));
    new_frame.union_bounding_box(&bb);

    new_frame.set_anchor(frame.get_anchor() + off);

    Ok(Val::Frame(Rc::new(new_frame)))
}

pub fn fit<'i, 'a>(_interpreter: &mut ExprInterpreter<'i, 'a>,
                   mut args: Vec<Val<'a>>)
                   -> Result<Val<'a>> {
    validate_args(names::fit, &[ValType::Frame, ValType::Coord(1)], &args)?;
    let frame = match args.remove(0) {
        Val::Frame(f) => f,
        _ => unreachable!(),
    };
    let size = match args.remove(0) {
        Val::Coord(w, h, 1) => (w, h),
        _ => unreachable!(),
    };

    let bb = frame.get_bounding_box();

    // Avoid division by zero in the aspect ratio computation. Fitting into a
    // box of which either size has length 0 is nonsense anyway.
    if size.0 == 0.0 || size.1 == 0.0 {
        return Err(Error::Other("Cannot fit frame in a box with \
                                 width or height equal to 0. \
                                 Simply don't place the frame then.".into()))
    }

    let scale = if bb.height != 0.0 {
        if (bb.width / bb.height) > (size.0 / size.1) {
            // The frame is constrained by width.
            size.0 / bb.width
        } else {
            // The frame is constrained by height.
            size.1 / bb.height
        }
    } else if bb.width != 0.0 {
        if (bb.height / bb.width) > (size.1 / size.0) {
            // The frame is constrained by height.
            size.1 / bb.height
        } else {
            // The frame is constrained by width.
            size.0 / bb.width
        }
    } else {
        return Err(Error::Other("Cannot fit a frame of size (0w, 0w).".into()))
    };

    let mut scaled_frame = Frame::from_env(frame.get_env().clone());

    // As the frame is immutable anyway, it would actually be possible to refer
    // to the subframes in the frame, instead of copying them. If performance
    // ever becomes a concern, this would be a good place to start.
    for subframe in frame.get_subframes() {
        let elements: Vec<_> = subframe.get_elements().iter().cloned().collect();
        let mut new_sf = Subframe::new();
        new_sf.place_element(Vec2::zero(), Element::Scaled(elements, scale));
        scaled_frame.push_subframe(new_sf);
    }

    scaled_frame.set_anchor(frame.get_anchor() * scale);
    scaled_frame.union_bounding_box(&frame.get_bounding_box().scale(scale));

    Ok(Val::Frame(Rc::new(scaled_frame)))
}

pub fn line<'i, 'a>(interpreter: &mut ExprInterpreter<'i, 'a>,
                    mut args: Vec<Val<'a>>)
                    -> Result<Val<'a>> {
    validate_args(names::line, &[ValType::Coord(1)], &args)?;
    let offset = match args.remove(0) {
        Val::Coord(x, y, 1) => Vec2::new(x, y),
        _ => unreachable!(),
    };

    let line = StrokePolygon {
        // TODO: Better idents type for non-ast use?
        // Could have a "borrowed ident" that is like &[&str],
        // similar to the std::Path and std::PathBuf distinctions.
        color: interpreter.env.lookup_color(&Idents(vec![names::color]))?,
        line_width: interpreter.env.lookup_len(&Idents(vec![names::line_width]))?,
        close: false,
        vertices: vec![Vec2::zero(), offset],
        kind: PolygonKind::Lines,
    };

    let mut frame = Frame::new();
    frame.place_element_on_last_subframe(Vec2::zero(), Element::StrokePolygon(line));
    frame.set_anchor(offset);
    // TODO: Make bounding box take Vec2.
    frame.union_bounding_box(&BoundingBox::sized(offset.x, offset.y));

    Ok(Val::Frame(Rc::new(frame)))
}

enum DrawKind {
    Fill,
    Stroke { close: bool },
}

fn make_polygon_element<'i, 'a>(
    interpreter: &mut ExprInterpreter<'i, 'a>,
    vertices: Vec<Vec2>,
    polygon_kind: PolygonKind,
    draw_kind: DrawKind,
) -> Result<Frame<'a>> {
    // TODO: Better idents type for non-ast use?
    let color = interpreter.env.lookup_color(&Idents(vec![names::color]))?;

    let mut frame = Frame::new();

    for v in &vertices {
        frame.union_bounding_box(&BoundingBox::empty().offset(*v));
    }

    let element = match draw_kind {
        DrawKind::Stroke { close } => {
            let line_width = interpreter.env.lookup_len(&Idents(vec![names::line_width]))?;

            let polygon = StrokePolygon {
                color: color,
                line_width: line_width,
                close: close,
                vertices: vertices,
                kind: polygon_kind,
            };

            Element::StrokePolygon(polygon)
        }
        DrawKind::Fill => {
            let polygon = FillPolygon {
                color: color,
                vertices: vertices,
                kind: polygon_kind,
            };

            Element::FillPolygon(polygon)
        }
    };

    frame.place_element_on_last_subframe(Vec2::zero(), element);

    Ok(frame)
}

fn draw_circle<'i, 'a>(
    interpreter: &mut ExprInterpreter<'i, 'a>,
    mut args: Vec<Val<'a>>,
    name: &'static str,
    kind: DrawKind,
) -> Result<Val<'a>> {

    validate_args(name, &[ValType::Num(1)], &args)?;
    let r = match args.remove(0) {
        Val::Num(r, 1) => r,
        _ => unreachable!(),
    };

    // See also http://spencermortensen.com/articles/bezier-circle/.
    let c = 0.551915024494 * r;
    let z = 0.0;

    let vertices = vec![
        Vec2::new( r,  z), // Right
        Vec2::new( r, -c),
        Vec2::new( c, -r),
        Vec2::new( z, -r), // Top
        Vec2::new(-c, -r),
        Vec2::new(-r, -c),
        Vec2::new(-r,  z), // Left
        Vec2::new(-r,  c),
        Vec2::new(-c,  r),
        Vec2::new( z,  r), // Bottom
        Vec2::new( c,  r),
        Vec2::new( r,  c),
    ];

    let mut frame = make_polygon_element(interpreter, vertices, PolygonKind::Curves, kind)?;

    frame.set_anchor(Vec2::zero());

    Ok(Val::Frame(Rc::new(frame)))
}

pub fn fill_circle<'i, 'a>(
    interpreter: &mut ExprInterpreter<'i, 'a>,
    args: Vec<Val<'a>>
) -> Result<Val<'a>> {
    draw_circle(interpreter, args, names::fill_circle, DrawKind::Fill)
}

pub fn stroke_circle<'i, 'a>(
    interpreter: &mut ExprInterpreter<'i, 'a>,
    args: Vec<Val<'a>>
) -> Result<Val<'a>> {
    let kind = DrawKind::Stroke {
        close: true,
    };
    draw_circle(interpreter, args, names::stroke_circle, kind)
}

fn draw_rectangle<'i, 'a>(
    interpreter: &mut ExprInterpreter<'i, 'a>,
    mut args: Vec<Val<'a>>,
    name: &'static str,
    kind: DrawKind,
) -> Result<Val<'a>> {
    validate_args(name, &[ValType::Coord(1)], &args)?;

    let (w, h) = match args.remove(0) {
        Val::Coord(x, y, 1) => (x, y),
        _ => unreachable!(),
    };

    let vertices = vec![
        Vec2::zero(),
        Vec2::new(0.0, h),
        Vec2::new(w, h),
        Vec2::new(w, 0.0),
    ];

    let mut frame = make_polygon_element(interpreter, vertices, PolygonKind::Lines, kind)?;

    frame.set_anchor(Vec2::new(w, h));

    Ok(Val::Frame(Rc::new(frame)))
}

pub fn fill_rectangle<'i, 'a>(
    interpreter: &mut ExprInterpreter<'i, 'a>,
    args: Vec<Val<'a>>,
) -> Result<Val<'a>> {
    draw_rectangle(interpreter, args, names::fill_rectangle, DrawKind::Fill)
}

pub fn stroke_rectangle<'i, 'a>(
    interpreter: &mut ExprInterpreter<'i, 'a>,
    args: Vec<Val<'a>>,
) -> Result<Val<'a>> {
    let kind = DrawKind::Stroke {
        close: true,
    };
    draw_rectangle(interpreter, args, names::stroke_rectangle, kind)
}

fn draw_polygon<'i, 'a>(
    interpreter: &mut ExprInterpreter<'i, 'a>,
    mut args: Vec<Val<'a>>,
    name: &'static str,
    polygon_kind: PolygonKind,
    draw_kind: DrawKind,
) -> Result<Val<'a>> {
    validate_args(name, &[ValType::List], &args)?;

    let coords = match args.remove(0) {
        Val::List(vs) => vs,
        _ => unreachable!(),
    };

    let mut vertices = Vec::with_capacity(coords.len());

    // Collect the vertices, ensuring that they have the right type.
    for vertex in &coords {
        match vertex {
            &Val::Coord(x, y, 1) => {
                let v = Vec2::new(x, y);
                vertices.push(v);
            }
            not_coord_of_len => {
                let arg_num = 0;
                let actual_type = not_coord_of_len.get_type();
                let err = Error::arg_type(name, ValType::Coord(1), actual_type, arg_num);
                return Err(err);
            }
        }
    }

    // TODO: Demand at least two coords.
    let anchor = vertices.last().cloned().unwrap_or(Vec2::zero());
    let mut frame = make_polygon_element(interpreter, vertices, polygon_kind, draw_kind)?;
    frame.set_anchor(anchor);

    Ok(Val::Frame(Rc::new(frame)))
}

pub fn fill_polygon<'i, 'a>(
    interpreter: &mut ExprInterpreter<'i, 'a>,
    args: Vec<Val<'a>>,
) -> Result<Val<'a>> {
    draw_polygon(interpreter, args, names::fill_polygon, PolygonKind::Lines, DrawKind::Fill)
}

pub fn stroke_polygon<'i, 'a>(
    interpreter: &mut ExprInterpreter<'i, 'a>,
    args: Vec<Val<'a>>,
) -> Result<Val<'a>> {
    let kind = DrawKind::Stroke {
        // TODO: Make this a variable.
        close: true,
    };
    draw_polygon(interpreter, args, names::stroke_polygon, PolygonKind::Lines, kind)
}

pub fn fill_curve<'i, 'a>(
    interpreter: &mut ExprInterpreter<'i, 'a>,
    args: Vec<Val<'a>>,
) -> Result<Val<'a>> {
    draw_polygon(interpreter, args, names::fill_curve, PolygonKind::Curves, DrawKind::Fill)
}

pub fn stroke_curve<'i, 'a>(
    interpreter: &mut ExprInterpreter<'i, 'a>,
    args: Vec<Val<'a>>,
) -> Result<Val<'a>> {
    let kind = DrawKind::Stroke {
        // TODO: Make this a variable.
        close: true,
    };
    draw_polygon(interpreter, args, names::stroke_curve, PolygonKind::Curves, kind)
}

pub fn hyperlink<'i, 'a>(_interpreter: &mut ExprInterpreter<'i, 'a>,
                         mut args: Vec<Val<'a>>)
                         -> Result<Val<'a>> {
    validate_args(names::hyperlink, &[ValType::Str, ValType::Coord(1)], &args)?;
    let uri = match args.remove(0) {
        Val::Str(s) => s,
        _ => unreachable!(),
    };
    let size = match args.remove(0) {
        Val::Coord(w, h, 1) => Vec2::new(w, h),
        _ => unreachable!(),
    };

    let link = Hyperlink {
        size: size,
        uri: uri,
    };

    let mut frame = Frame::new();
    frame.place_element_on_last_subframe(Vec2::zero(), Element::Hyperlink(link));
    frame.set_anchor(size);
    // TODO: Make bounding box take Vec2.
    frame.union_bounding_box(&BoundingBox::sized(size.x, size.y));

    Ok(Val::Frame(Rc::new(frame)))
}

pub fn str<'i, 'a>(_interpreter: &mut ExprInterpreter<'i, 'a>,
                   mut args: Vec<Val<'a>>)
                   -> Result<Val<'a>> {
    // TODO: Make this generic over the dimension?
    validate_args(names::str, &[ValType::Num(0)], &args)?;
    let num = match args.remove(0) {
        Val::Num(x, _) => x,
        _ => unreachable!(),
    };

    Ok(Val::Str(format!("{}", num)))
}

pub fn sqrt<'i, 'a>(
    _interpreter: &mut ExprInterpreter<'i, 'a>,
    args: Vec<Val<'a>>
) -> Result<Val<'a>> {
    // Slight hack: we want sqrt to be generic over the dimension. That means
    // that the regular `validat_args` does not work, because it takes a fixed
    // type. Therefore, inspect the first argument first, and report the error
    // later. The expected type will be 'num', in case you pass in something
    // else, but that is okay for now.
    let (num, dim): (f64, i32) = match args.first() {
        Some(&Val::Num(x, n)) => (x, n),
        _ => {
            validate_args(names::sqrt, &[ValType::Num(0)], &args)?;
            unreachable!("First argument would have matched.");
        }
    };

    // Now that we now the dimension, validate args for this dimension
    // specifically. The first argument is going to pass, but there could
    // be an arity mismatch still.
    validate_args(names::sqrt, &[ValType::Num(dim)], &args)?;

    // TODO: Do proper error reporting.
    assert!(dim % 2 == 0, "Dimension must be multiple of 2.");

    Ok(Val::Num(num.sqrt(), dim / 2))
}

/// Typesets a single line of text.
///
/// Returns the glyphs as well as the width of the line.
fn typeset_line(ft_face: &mut freetype::Face,
                font_size: f64,
                text: &str)
                -> (Vec<cairo::Glyph>, f64) {
    // Shape the text using Harfbuzz: convert the UTF-8 string and input font
    // into a list of glyphs with offsets.
    let mut hb_font = harfbuzz::Font::from_ft_face(ft_face);

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

    (cr_glyphs, cur_x)
}

/// Split a string on newlines.
///
/// Unlike `std::str::lines`, the final newline is not swallowed.
fn split_lines(text: &str) -> Vec<&str> {
    // TODO: This might return an iterator instead of a vector.
    // At this point it is not worth the trouble performance-wise.
    let mut lines = Vec::new();
    let mut left = text;
    while let Some(index) = left.find("\n") {
        lines.push(&left[0..index]);
        left = &left[index + 1..];
    }
    lines.push(left);
    lines
}

#[test]
fn split_lines_returns_as_many_lines_as_newlines_plus_one() {
    let text = "\nfoo\nbar\n";
    let lines = split_lines(text);
    assert_eq!(&lines, &["", "foo", "bar", ""]);
}

pub fn t<'i, 'a>(interpreter: &mut ExprInterpreter<'i, 'a>,
                 mut args: Vec<Val<'a>>)
                 -> Result<Val<'a>> {
    validate_args(names::t, &[ValType::Str], &args)?;
    let text = match args.remove(0) {
        Val::Str(s) => s,
        _ => unreachable!(),
    };
    let text_lines = split_lines(&text);

    enum TextAlign { Left, Center, Right }

    // Read the font details from the 'font_family' and 'font_style' variables,
    // and locate the corresponding FreeType face. The line height is a bit of a
    // problem; we could make it dimensionless and relative to the font size --
    // which would make it scale automatically -- but then specifying absolute
    // line heights would be a bit of a hassle. We could make it absolute, but
    // then it does not scale automatically. Or we could allow both here:
    // numbers have units, so we could figure out what to do. But my gut feeling
    // is that dynamic typing will be confusing in the end.
    let font_family = interpreter.env.lookup_str(&Idents(vec![names::font_family]))?;
    let font_style = interpreter.env.lookup_str(&Idents(vec![names::font_style]))?;
    let font_size = interpreter.env.lookup_len(&Idents(vec![names::font_size]))?;
    let line_height = interpreter.env.lookup_len(&Idents(vec![names::line_height]))?;
    let text_align = interpreter.env.lookup_str(&Idents(vec![names::text_align]))?;
    let ft_face = match interpreter.font_map.get(&font_family, &font_style) {
        Some(face) => face,
        None => return Err(Error::missing_font(font_family, font_style)),
    };
    let ta = match text_align.as_ref() {
        "left" => TextAlign::Left,
        "center" => TextAlign::Center,
        "right" => TextAlign::Right,
        other => {
            // TODO: Move this error to an error at assignment time, not at
            // evaluation time. More type safety is more better.
            let mut fmt = Formatter::new();
            fmt.print("'");
            fmt.print(other);
            fmt.print("' is not a valid value for 'text_align'. ");
            fmt.print("Must be one of 'left', 'center', 'right'.");
            return Err(Error::value(fmt.into_string()))
        }
    };

    // TODO: Extract this, warn properly.
    if ft_face.family_name().as_ref() != Some(&font_family) {
        println!("Warning: requested font family '{}', but loaded '{}'.",
                 font_family, ft_face.family_name().unwrap_or("?".into()));
    }
    if ft_face.style_name().as_ref() != Some(&font_style) {
        println!("Warning: requested font style '{}', but loaded '{}'.",
                 font_style, ft_face.style_name().unwrap_or("?".into()));
    }

    let mut glyphs = Vec::new();
    let mut max_width: f64 = 0.0;
    let mut min_offset: f64 = 0.0;
    let mut cur_x = 0.0;
    let mut cur_y = 0.0;
    for line in text_lines {
        let (line_glyphs, width) = typeset_line(ft_face, font_size, line);

        // Apply x offset to enforce text alignment.
        let offset = match ta {
            TextAlign::Left => 0.0,
            TextAlign::Center => width * -0.5,
            TextAlign::Right => width * -1.0,
        };

        for g in line_glyphs {
            glyphs.push(g.offset(offset, cur_y));
        }

        max_width = max_width.max(width);
        min_offset = min_offset.min(offset);
        cur_y += line_height;
        cur_x = offset + width;
    }

    let text_elem = Text {
        color: interpreter.env.lookup_color(&Idents(vec!["color"]))?,
        font_family: font_family,
        font_style: font_style,
        font_size: font_size,
        glyphs: glyphs,
    };

    let mut frame = Frame::new();
    frame.place_element_on_last_subframe(Vec2::zero(), Element::Text(text_elem));
    frame.set_anchor(Vec2::new(cur_x, cur_y - line_height));

    let top_left = Vec2::new(min_offset, -line_height);
    let size = Vec2::new(max_width, cur_y);
    frame.union_bounding_box(&BoundingBox::new(top_left, size));

    Ok(Val::Frame(Rc::new(frame)))
}

pub fn glyph<'i, 'a>(interpreter: &mut ExprInterpreter<'i, 'a>,
                     mut args: Vec<Val<'a>>)
                     -> Result<Val<'a>> {
    validate_args(names::glyph, &[ValType::Num(0)], &args)?;
    let index_f64 = match args.remove(0) {
        Val::Num(x, 0) => x,
        _ => unreachable!(),
    };

    let index = index_f64 as u64;

    if index as f64 != index_f64 {
        let msg = format!("Expected an unsigned integer glyph index, found {}.", index_f64);
        return Err(Error::value(msg))
    }

    // TODO: This was copy-pasted from the `t()` function. Extract the common
    // stuff.

    let font_family = interpreter.env.lookup_str(&Idents(vec![names::font_family]))?;
    let font_style = interpreter.env.lookup_str(&Idents(vec![names::font_style]))?;
    let font_size = interpreter.env.lookup_len(&Idents(vec![names::font_size]))?;
    let line_height = interpreter.env.lookup_len(&Idents(vec![names::line_height]))?;
    let ft_face = match interpreter.font_map.get(&font_family, &font_style) {
        Some(face) => face,
        None => return Err(Error::missing_font(font_family, font_style)),
    };

    // Compensate for the fixed font size which is set for the Freetype font.
    // There is a 16.6 factor that `linear_hori_advance` adds according to the
    // docs, but it turns out that actually the advance with is returned with a
    // multiplication factor of 1024.
    let size_factor = font_size / 1000.0 / 1024.0;

    // Get the x-advance from the font, which will be used as the glyph width.
    match ft_face.load_glyph(index as u32, freetype::face::LoadFlag::empty()) {
        Ok(..) => {}
        // TODO: Better structural error.
        Err(..) => return Err(Error::Other(format!("Could not load glyph {}.", index))),
    }
    let width = ft_face.glyph().linear_hori_advance() as f64 * size_factor;

    let glyphs = vec![cairo::Glyph::new(index, 0.0, 0.0)];

    let text_elem = Text {
        color: interpreter.env.lookup_color(&Idents(vec![names::color]))?,
        font_family: font_family,
        font_style: font_style,
        font_size: font_size,
        glyphs: glyphs,
    };

    let mut frame = Frame::new();
    frame.place_element_on_last_subframe(Vec2::zero(), Element::Text(text_elem));
    frame.set_anchor(Vec2::new(width, 0.0));

    let top_left = Vec2::new(0.0, -line_height);
    let size = Vec2::new(width, 0.0);
    frame.union_bounding_box(&BoundingBox::new(top_left, size));

    Ok(Val::Frame(Rc::new(frame)))
}

pub fn image<'i, 'a>(_interpreter: &mut ExprInterpreter<'i, 'a>,
                     mut args: Vec<Val<'a>>)
                     -> Result<Val<'a>> {
    validate_args(names::image, &[ValType::Str], &args)?;
    let path = match args.remove(0) {
        Val::Str(s) => s,
        _ => unreachable!(),
    };

    // TODO: Make path relative to the source file. Some kind of state would be
    // required for this. Time to replace the FontMap everywhere with a more
    // elaborate state structure.

    let (width, height, element) = match () {
        _ if path.ends_with(".svg") => image_svg(path)?,
        _ if path.ends_with(".png") => image_png(path)?,
        _ => {
            let msg = format!("Cannot load '{}', only svg and png images are supported for now.", path);
            return Err(Error::Other(msg))
        }
    };

    let mut frame = Frame::new();
    frame.place_element_on_last_subframe(Vec2::zero(), element);
    frame.union_bounding_box(&BoundingBox::sized(width, height));

    // The image anchor is in the top right, so images can be adjoined easily:
    // the origin is top left.
    frame.set_anchor(Vec2::new(width, 0.0));

    Ok(Val::Frame(Rc::new(frame)))
}

fn image_svg<'a>(path: String) -> Result<(f64, f64, Element)> {
    let svg = match rsvg::Svg::open(&path) {
        Ok(svg) => svg,
        // TODO: Actually, the cause does not have to be a missing file, it
        // might be an ill-formed file or some other kind of IO error too.
        // Move error handling into the rsvg module proper.
        Err(()) => return Err(Error::missing_file(path)),
    };
    let (width, height) = svg.size();

    Ok((width as f64, height as f64, Element::Svg(svg)))
 }

fn image_png<'a>(path: String) -> Result<(f64, f64, Element)> {
    let (width, height) = png::get_dimensions(&path)?;
    Ok((width as f64, height as f64, Element::Png(PathBuf::from(path))))
}
