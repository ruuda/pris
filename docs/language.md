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
A statement is either assignment to a variable,
a `put` statement that places a frame,
or a `return` from a block.
Although statements mutate the current scope,
these mutations are only visible to current scope,
and do not affect the surrounding scope.

    x = 1
    y =
    {
      x = 2
      return x
    }
    // At this point, x = 1 and y = 2.

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

## Syntax

**
