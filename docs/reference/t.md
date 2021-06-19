# t

    t(text: str) -> frame

Render a piece of text. This function will likely be renamed to `text` in the
future, when string prefixes are supported.

The origin of the returned frame is on the baseline of the first line.

## Font selection

The font can be selected with the `font_family` and `font_style` variables. Font
features can be provided as a list of strings in the `font_features` variable.
Features must be specified in [`hb_feature_from_string`][hb-feature] format. For
example:

    // Enable stylistic set 1 and 2.
    font_features = ["ss01"; "ss02"]

    // Disable ligatures and kerning, when the font enables them by default.
    font_features = ["-liga"; "-kern"]

[hb-feature]: https://harfbuzz.github.io/harfbuzz-hb-common.html#hb-feature-from-string

## Alignment

Alignment can be controlled with the `text_align` variable, which must be one of
`"left"`, `"center"`, or `"right"`. Line height is controlled by the
`line_height` variable, and size by the `font_size` variable.

## Color

The text color is taken from the `color` variable.
