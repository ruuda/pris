// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use ast::Idents;
use cairo::{Cairo, FontFace, Surface};
use elements::{Color, Element, PlacedElement, Vec2};
use runtime::{FontMap, Frame};

fn draw_background(cr: &mut Cairo, canvas_size: Vec2, color: Color) {
    cr.rectangle(0.0, 0.0, canvas_size.x, canvas_size.y);
    cr.set_source_rgb(color.r, color.g, color.b);
    cr.fill();
}

/// Draw the lines for a polygon, but don't stroke or fill it yet.
fn draw_polygon(cr: &mut Cairo, vertices: &[Vec2], close: bool) {
    debug_assert!(vertices.len() >= 2, "Polygon must have at least one line segment.");

    let v0 = vertices[0];
    cr.move_to(v0.x, v0.y);

    for v in &vertices[1..] {
        cr.line_to(v.x, v.y);
    }

    if close {
        cr.close_path();
    }
}

fn draw_element(fm: &mut FontMap, cr: &mut Cairo, pe: &PlacedElement) {
    match pe.element {
        Element::StrokePolygon(ref polygon) => {
            let matrix = cr.get_matrix();
            cr.translate(pe.position.x, pe.position.y);

            draw_polygon(cr, &polygon.vertices, polygon.close);

            cr.set_source_rgb(polygon.color.r, polygon.color.g, polygon.color.b);
            cr.set_line_width(polygon.line_width);
            cr.stroke();

            cr.set_matrix(&matrix);
        }

        Element::FillPolygon(ref polygon) => {
            let matrix = cr.get_matrix();
            cr.translate(pe.position.x, pe.position.y);

            let close = true;
            draw_polygon(cr, &polygon.vertices, close);

            cr.set_source_rgb(polygon.color.r, polygon.color.g, polygon.color.b);
            cr.fill();

            cr.set_matrix(&matrix);
        }

        Element::Text(ref text) => {
            // Cairo uses absolute positions for glyphs, so we need to add
            // the final positions to the glyph locations.
            let glyphs_offset: Vec<_> = text.glyphs.iter()
                                            // TODO: Make offset type take
                                            // Vec2.
                                            .map(|g| g.offset(pe.position.x, pe.position.y))
                                            .collect();
            // If we were able to shape the text, then the FT font must
            // exist still. TODO: Would it be better to just embed a
            // reference in the Text element instead of doing the lookup
            // twice?
            let ft_face = fm.get(&text.font_family, &text.font_style).unwrap();
            let cr_face = FontFace::from_ft_face(ft_face.clone());
            cr.set_font_face(&cr_face);
            cr.set_font_size(text.font_size);
            cr.set_source_rgb(text.color.r, text.color.g, text.color.b);
            cr.show_glyphs(&glyphs_offset);
            // TODO: The cr_font should outlive the Cairo, because Cairo
            // might internally reference the font still. How to model this?
        }

        Element::Scaled(ref elements, scale) => {
            // Store the current transform so we can restore it later.
            let matrix = cr.get_matrix();
            cr.translate(pe.position.x, pe.position.y);
            cr.scale(scale, scale);
            for inner_pe in elements {
                draw_element(fm, cr, inner_pe);
            }
            cr.set_matrix(&matrix);
        }

        Element::Svg(ref svg) => {
            // Store the current transform so we can restore it later.
            let matrix = cr.get_matrix();
            cr.translate(pe.position.x, pe.position.y);
            svg.draw(cr);
            cr.set_matrix(&matrix);
        }

        Element::Png(ref path) => {
            // TODO: This will need error handling.
            let png_surface = Surface::from_png(path);
            cr.set_source_surface(&png_surface, pe.position.x, pe.position.y);
            cr.paint();
        }

        #[cfg(not(feature = "hyperlink"))]
        Element::Hyperlink(..) => {
            println!(
                "Warning: hyperlink not created, Pris was compiled without \
                hyperlink support.");
        }

        #[cfg(feature = "hyperlink")]
        Element::Hyperlink(ref hyperlink) => {
            // Escape the uri: backslashes and single quotes must be escaped
            // with a backslash, to fit the format of the tag "attributes".
            let mut uri_escaped = String::with_capacity(hyperlink.uri.len());
            for ch in hyperlink.uri.chars() {
                match ch {
                    '\'' => uri_escaped.push_str("\\'"),
                    // Not sure why a single backslash should turn into *four*
                    // instead of two, but when I push two backslashes, nothing
                    // shows up in Evince. Could be a bug in Cairo or Evince
                    // too.
                    '\\' => uri_escaped.push_str("\\\\\\\\"),
                    _ => uri_escaped.push(ch),
                }
            }

            let (x, y) = cr.user_to_device(pe.position.x, pe.position.y);
            let (w, h) = cr.user_to_device_distance(hyperlink.size.x, hyperlink.size.y);
            let attributes = format!(
                "uri='{}' rect=[{:0.3} {:0.3} {:0.3} {:0.3}]\0",
                uri_escaped,
                x, y, w, h
            );
            cr.tag_link(&attributes);
        }
    }
}

pub fn render_frame<'a>(
    fm: &mut FontMap,
    cr: &mut Cairo,
    canvas_size: Vec2,
    frame: &Frame<'a>
) {
    // TODO: Ensure that writing to background_color only accepts a color value,
    // so a lookup failure here is never a type error.
    let var_bgcolor = Idents(vec!["background_color"]);

    for subframe in frame.get_subframes() {
        if let Ok(bgcolor) = frame.get_env().lookup_color(&var_bgcolor) {
            draw_background(cr, canvas_size, bgcolor);
        }

        for pe in subframe.get_elements() {
            draw_element(fm, cr, pe);
        }

        cr.show_page();
        cr.assert_status_success();
    }
}
