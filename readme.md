# Pris

Pris is a domain-specific language for designing slides and other graphics.

## Comparison to other technologies

 * Pris is similar to LaTeX with Beamer in the sense that you write your slides
   in a text-based, human readable format, from which a pdf is produced. Pris
   differs from LaTeX with Beamer in not doing any lay-out. All elements must be
   placed manually.
 * Pris is similar to reveal.js in the sense that you write your slides in a
   text-based, human readable format. Its control over visuals is superficially
   similar to css. It differs from reveal.js in requiring a separate compilation
   step that renders a pdf.
 * Pris is similar to TikZ in LaTeX, in the sense that it is a domain-specific
   language for creating graphics. It is similar in providing complete control
   over where elements are placed. Pris differs from TikZ in not being embedded
   in LaTeX. It has a more modern syntax, and it has first class support for
   computation. For instance, arithmetic with coordinates is supported out of
   the box.
 * Pris is vaguely similar to Powerpoint and graphical editors like Illustrator
   or Inkscape in providing complete control over where elements are placed. It
   differs in being a text-based format intended to be edited with a text
   editor, rather than a graphical editor.
