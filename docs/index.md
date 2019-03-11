# Pris

Pris is a domain-specific language for designing slides and other graphics.

Pris allows you to write drawing commands in a lightweight programming language
tailored for the task, and compile that to <span class="smcp">PDF</span>.
Graphics can be parametrized easily, and instead of copy-pasting, common
elements can be extracted into reusable functions.

## Features

 * Compiles to <span class="smcp">PDF</span>.
 * Full typographic control.
 * First class graphics that can be inspected and manipulated as values.
 * First class functions.

## Example

The obligatory “hello world”:

    {
      put t("Hello world") at (0.1w, 0.5h)
    }

This produces a single page, with the text “Hello world” in the default
sans-serif font. The leftmost point on the text baseline is located at 10% of
the canvas width and 50% of the canvas height.

## Getting started

Pris needs to be built from source. See the [building](building.md) chapter of
the docs. Then you might want to take a look at the [examples][examples]. Please
note that Pris is alpha-quality software. Expect things to break.

[examples]: https://github.com/ruuda/pris/tree/master/examples
