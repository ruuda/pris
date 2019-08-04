# fill_polygon

    fill_polygon(vertices: list of coord) -> frame

Draw a solid polygon.

The fill color is taken from the `color` variable.

For example, drawing a rectangle with 2em sides:

    vertices = [(-1em, 1em); (1em, 1em); (1em, -1em); (-1em, -1em)]
    fill_polygon(vertices)
