# t

    t(text: str) -> frame

Render a piece of text. This function will likely be renamed to `text` in the
future, when string prefixes are supported.

The font can be selected with the `font_family` and `font_style` variables.

Alignment can be controlled with the `text_align` variable, which must be one of
`"left"`, `"center"`, or `"right"`.  Line height is controlled by the
`line_height` variable. The origin of the returned frame is on the baseline of
the first line.

The text color is taken from the `color` variable.
