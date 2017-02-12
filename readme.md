# Pris

Pris is a domain-specific language for designing slides and other graphics.

## Comparison to other technologies

 * Pris is similar to LaTeX with Beamer in the sense that you write your slides
   in a text-based, human readable format, from which a pdf is produced. Pris
   differs from LaTeX with Beamer in not doing any lay-out. All elements must be
   placed manually.
 * Pris is similar to reveal.js in that its control over visuals superficially
   resembles css. It differs from reveal.js in requiring a separate compilation
   step that renders a pdf. It differs from html in being imperative rather than
   declarative.
 * Pris is similar to TikZ in LaTeX, in the sense that it is a domain-specific
   language for creating graphics. It is similar in providing complete control
   over where elements are placed. Pris differs from TikZ in not being embedded
   in LaTeX. It has a more modern syntax, and it has first class support for
   computation. For instance, arithmetic with coordinates is supported out of
   the box, and Pris has proper functions, rather than TeX macros.
 * Pris is vaguely similar to Powerpoint and graphical editors like Illustrator
   or Inkscape in providing complete control over where elements are placed. It
   differs in being a text-based format intended to be edited with a text
   editor, rather than with a graphical editor.
 * Pris is similar to an html canvas element, or to drawing with Skia or Cairo,
   in providing complete control over how graphics are drawn. It differs from
   direct canvas drawing in being more high-level (graphic elements can be
   manipulated as first-class values), and in being a domain-specific language
   rather than being controlled by a general-purpose scripting language.
