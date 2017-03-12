// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use cairo::{Cairo, FontFace};
use elements::Element;
use runtime::{FontMap, Frame};

// TODO: Should take a path, not str.
pub fn render_frame<'a>(fm: &mut FontMap, cr: &mut Cairo, frame: &Frame<'a>) {
    for pe in frame.get_elements() {
        match pe.element {
            Element::Line(ref line) => {
                cr.move_to(pe.x, pe.y);
                cr.set_source_rgb(line.color.r, line.color.g, line.color.b);
                cr.set_line_width(line.line_width);
                cr.line_to(pe.x + line.x, pe.y + line.y);
                cr.stroke();
            }

            Element::Text(ref text) => {
                // Cairo uses absolute positions for glyphs, so we need to add
                // the final positions to the glyph locations.
                let glyphs_offset: Vec<_> = text.glyphs.iter()
                                                .map(|g| g.offset(pe.x, pe.y))
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
        }
    }
    cr.show_page()
}
