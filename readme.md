## Observations

While a declarative source with a theme (such as html with css, or LaTeX) works
great for long documents, it is somewhat inflexible for slides. Good slides need
to be designed, and this is much more a manual process than typesetting large
blocks of text. If you want to be able to change themes with a single click,
then you need to shoehorn slides into a pretty limited format. Separating markup
from content is not a good idea for creative slides: markup is an important part
of the content. **Conclusion**: Don’t try to separate semantics from markup.
Specifying semantics is a non-goal. Offer full control over markup instead.

To design graphics (which is what good slides should be), a graphical editor can
be much nicer than a declarative format. Compare Inkscape with LaTeX and TikZ.
However, a text-based declarative format has several advantages: it is nicer to
work with in source control, and it can easily be scripted. Compare animating a
hand-drawn graph in Google Slides with animating a TikZ drawing in LaTeX and
Beamer. Manual animation often involves copying the entire thing, after which
changing parts becomes difficult. It is possible to build a graphical editor for
a text-based format, but these often produce messy output. **Conclusion**: A
declarative text-based input format that is intended to be written by humans is
preferred over an opaque format with a graphical editor.

Supporting external graphics (svg logos, jpeg photographs, png graphics) is a
must. For consistent graphics, having basic drawing capabilities embedded would
be useful (like TikZ in LaTex). The possibility to leverage external tools is
limited if consistency is a goal. (For instance, even if the same font is used
in externally drawn graphics, the graphic must be scaled to make font sizes
match.) Care must be taken to keep the scope narrow: TikZ is a huge project in
itself. **Conclusion**: Allow placing external graphics and expose a minimal but
sufficient set of embedded drawing operations.
