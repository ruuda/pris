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
* Add support for lists, to enable polygons, bullet point lists, etc.

## Near-term

* Extend the syntax to support string prefixes, define what they mean.
* Think about how to handle animation and subframes.
* Think about how to make slides first class. Currently you can't return a full
  slide from a function. Things such as setting the background color have to be
  repeated on every slide. Even if the full drawing is encapsulated in a
  function, you need one `put draw_slide()` per slide. Without first class
  slides, e.g. adding page numbers is difficult. With first class slides, it
  could be a function `slides -> slides`. Perhaps equating slides and frames is
  the answer.

## Longer-term

* Add support for Opentype features (smcp, onum).
* Keep track of source location in AST nodes to provide helpful errors.
* Take proper font metrics into account for text bounding box.
* Intern strings at parse time, then replace `&'a str` with an integer
  everywhere, and get rid of the pervasive `'a` liftetime.

## Eventually

* Add a static type system.
* Add support for loops, to e.g. draw a clock, or a list of bullet points. Or
  perhaps, instead of loops, add folds.
