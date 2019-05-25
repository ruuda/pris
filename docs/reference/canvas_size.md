# canvas_size

    canvas_size: vec2

A variable that sets the aspect ratio of the canvas. Must be assigned in the
global scope. The last assignment to this variable determines the canvas size
for all slides in the document, it is currently not possible to have slides of
different sizes in a single document.

Note that after changing the canvas size, variables that had been assigned a
value relative to the canvas size, continue to hold their absolute value, which
may no longer make sense for the new canvas size. For example:

    canvas_size = (1, 1)
    half = 0.5w
    canvas_size = (2, 2)
    // "half" is now effectively 0.25w, no longer half the canvas width.
