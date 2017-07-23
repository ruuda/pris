# Changelog

## master

Not released yet.

Highlights:

 * The lexer and parser have been replaced with hand-written ones. This enables
   support for comments, it generates better error messages in the case of a
   parse error, and it reduces compile times and the size of the binary.
 * Unary negation is now supported on coordinates.
 * The bounding box offset is now exposed, enabling proper centering.
 * A `glyph()` function has been added to select a single glyph by glyph id.
 * Subframe support (not finished).
 * There now is basic hosted documentation.
 * Support for loading png images, in addition to svg.

Bugs fixed:

 * Raw string literals can now contain empty lines.

## 0.1.0

Released 2017-04-20.

Highlights:

 * The color and width of lines can now be set.
 * The background color of a slide can now be set.
 * Text support (including ligatures and alignment).
 * Frames now have an “anchor”, which is used for the adjoin operator (`~`).
 * Escape sequences can be used inside strings.
 * There is a new `image()` function that can load svg images.
 * Support for scaling frames with the `fit()` function.
 * The bounding box of frames is now exposed, allowing lay out computations.

Pris 0.1.0 was usable (though not convenient) for making simple slides.

## 0.0.0

Released 2017-02-17.

Initial release. At this point Pris could produce a pdf document with lines in
it from a Pris source file. There was no support for colors or text yet.
