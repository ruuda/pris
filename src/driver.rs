// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use cairo::Cairo;
use elements::Element;
use runtime::Frame;

// TODO: Should take a path, not str.
pub fn render_frame<'a>(cr: &mut Cairo, frame: &Frame<'a>) {
    for pe in frame.get_elements() {
        cr.move_to(pe.x, pe.y);
        match pe.element {
            Element::Line(ref line) => {
                cr.set_source_rgb(line.color.r, line.color.g, line.color.b);
                cr.set_line_width(line.line_width);
                cr.line_to(pe.x + line.x, pe.y + line.y);
                cr.stroke();
            }
        }
    }
    cr.show_page()
}
