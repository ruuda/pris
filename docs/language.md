# The language

This chapter characterizes the Pris domain specific language in various ways. It
may help to determine where Pris is positioned relative to other languages. If
you want to get your hands dirty, the [examples][examples] may be a better way
to get started.

[examples]: https://github.com/ruuda/pris/tree/master/examples

## Paradigm

**Pris is purely functional**, in the following sense:

 * Pris has first-class functions: functions can be passed around as values.
 * Functions in Pris are pure: they have no side effects.
 * Values in Pris are immutable, there are no mutable objects.

**Pris is imperative**,
in the sense that blocks consist of statements that are executed sequentially.
A statement is either assignment to a variable, a `put` statement that places a
frame, or a `return` from a block. Although statements mutate the current scope,
these mutations are only visible to current scope, and do not affect the
surrounding scope.

    x = 1
    y =
    {
      x = 2
      return x
    }
    // At this point, x = 1 and y = 2.

**Pris is dynamically scoped**,
in the sense that the variables that are in scope when a function is evaluated,
depend on the call site, not on the site where the function was defined. When
used carelessly, this can make make programs intractable quickly.  But when used
with care, dynamic scoping is a powerful tool for a graphics language,
reminiscent of the cascading properties of <abbr>CSS</abbr>. For example, we
can write a `title` function that makes no particular choice of color, so the
surrounding scope can determine it.

    title = function(text)
    {
        font_size = 0.2h
        text_align = "center"
        put t(title) at (0.5w, 0.5h)
    }

    {
      // This title will be red.
      color = #ff0000
      put title("Slide 1")
    }

    {
      // This title will be blue.
      color = #0000ff
      put title("Slide 2")
    }

Due to dynamic scoping, there are two ways to get a value into a function:

 * By setting a particular variable in the calling scope.
 * By passing it as a function argument.

Variables are useful for values that do not change often, to avoid having to
pass them all the time. For example, the [`t`](reference/t.md) function uses
the `font_family` and `font_style` from the surrounding scope, but accepts the
text as argument.

## Type system

**Pris is strongly typed**,
in the sense that it reports a type error for nonsensical operations,
rather than implicitly coercing values.
Number types have units:
a length is different from a dimensionless number.

**Pris is dynamically typed**,
in the sense that type errors in unreachable code
do not cause compilation to fail.

    // Calling this function will trigger a type error,
    // but defining it does not.
    trigger_error = function()
    {
      return "wrong" + 0
    }

Dynamic typing is not a deep design choice, it is simply easier to implement.
Pris may acquire static type checking in the future.

## Semantics

TODO

## Syntax

**Pris is whitespace-insensitive**. Whitespace separates tokens, but the amount,
and the distinction between spaces and newlines, are irrelevant. Tabs and
carriage returns are rejected by the parser.

**Pris does not have a statement separator**. Rather, the grammar is constructed
in such a way that statement boundaries are unambiguous.

    // You can put multiple statements on a line.
    // That doesn't mean it's a good idea though.
    x = 10 y = 12 put t("12") z = 1 + 2 w = z
