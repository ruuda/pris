# image

    image(fname: str) -> frame

Load a <abbr>PNG</abbr> or <abbr>SVG</abbr> graphic from the file with path
`fname`. Currently the path is relative to the working directory, but it should
be made relative to the source file.

The origin of the returned frame is in the top-left corner.

The size of the image relative to the canvas depends on the input image, and is
often not meaningful. To resize the image, use the [`fit`](fit.md) function.
