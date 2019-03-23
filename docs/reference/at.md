# at

    at(frame: frame, offset: coord) -> frame

Move a frame by the given offset. This is used to position elements on the
canvas, or relative to a scope, and the function is usually called with infix
notation. Example:

    // Place an image in the top-left corner of the canvas.
    put image("logo.png") at (1em, 1em)
