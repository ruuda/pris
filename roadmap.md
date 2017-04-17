# Roadmap

Points are ordered from highest priority to lowest priority.

## Done

* Add support for text (FreeType and Harfbuzz).
* Implement anchors and adjunction.
* Support loading svg images.
* Extend the syntax with unary negation.

## Near-term

* Write a custom lexer to support comments.
* Add support for Opentype features (smcp, onum).
* Extend the syntax to support string prefixes, define what they mean.

## Longer-term

* Support loading raster images.
* Add support for lists, to enabled polygons, bullet point lists, etc.
* Keep track of source location in AST nodes to provide helpful errors.
* Add support for loops, to e.g. draw a clock, or a list of bullet points.

## Eventually

* Think about how to handle animation and subframes.
