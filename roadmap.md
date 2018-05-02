# Roadmap

Points are ordered from highest priority to lowest priority.

## Done

* Add support for text (FreeType and Harfbuzz).
* Implement anchors and adjunction.
* Support loading svg images.
* Extend the syntax with unary negation.
* Add syntax for multiline string literals.
* Write a custom lexer to support comments.
* Support loading raster images.

## Near-term

* Extend the syntax to support string prefixes, define what they mean.
* Think about how to handle animation and subframes.
* Add support for lists, to enable polygons, bullet point lists, etc.

## Longer-term

* Add support for Opentype features (smcp, onum).
* Keep track of source location in AST nodes to provide helpful errors.
* Take proper font metrics into account for text bounding box.

## Eventually

* Add support for loops, to e.g. draw a clock, or a list of bullet points.
