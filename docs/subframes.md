# Subframes

**Note**: This is still an idea, it has not been implemented.

## Problem

Currently, slides in Pris are not first class. There is no way to assign slides
to variables, or to return slides from functions. I would like to be able to
write:

    title_slide = function(title) { ... }

    title_slide("Section 1")

    {
      // First slide of section 1.
    }

    title_slide("Section 2")

    {
      // First slide of section 2.
    }

Instead of having to write

    {
      put title_slide("Section 1")
    }

In particular, because you manually have to set the background color each time
in the latter case (although you could also `put fill_rectangle((1w, 1h))` to
achieve a background color — and perhaps that is better anyway).

Apart from this, Pris does not yet have animation. For this I had the idea of
introducing a `pause` statement, like so:

    {
      put fill_circle(1em) at (0.5w, 0.5h)
      pause
      put fill_circle(1em) at (0.25w, 0.5h)
      put fill_circle(1em) at (0.75w, 0.5h)
    }

The first circle would appear on the first subframe and each subframe following
it, the next two circles would appear on the second subframe only.

## The time dimension

Just like frames have a bounding box that demarcates them in space, they could
have a duration that does so in time. The above box frame would have a duration
of 2 frames. A `put` would place an animated frame (consisting of multiple
subframes) at the current time, and `pause` would advance the current time by
one.

Analogous to how `at` tranlates a frame in space, `delay` would translate a
frame in time. (Let's, use a `f` suffix for units of time, meaning “frame”.)
This would make the dots appear one by one:

    {
      put fill_circle(1em) at (0.25w, 0.5h) delay 0f
      put fill_circle(1em) at (0.50w, 0.5h) delay 1f
      put fill_circle(1em) at (0.75w, 0.5h) delay 2f
    }

Just like how bounding boxes grow to contain everything placed, the duration
would grow to encapsulate all the entire time range (3 subframes here). In fact,
with `delay` we do not need `pause` any more, but it might still be nice to
have, to act as an implicit counter.

Pris does not yet have clipping masks for spatial limiting, but I do want to add
them in the future. Analogously, we can clip a frame in time:

    {
      put fill_circle(1em) at (0.33w, 0.5h) until 1f
      put fill_circle(1em) at (0.66w, 0.5h) delay 1f
    }

This would make the circle jump, appearing on the left in the first frame, and
on the right in the second frame.

> **Open question**: `until` suggests that the time argument is relative to the
> start of the current frame, but that would make it dependent on external
> state. It would be more natural to just limit the duration, but then “until”
> may not be the best name. Perhaps `up_to`, that aligns nicely with `delay` too.

With this, we can theoretically build an entire presentation in one slide that
has subframes. In fact, the previous example is the single-slide equivalent of

    {
      put fill_circle(1em) at (0.33w, 0.5h)
    }

    {
      put fill_circle(1em) at (0.66w, 0.5h) delay 1f
    }

## A unified approach

I think this is the key: if slides *are* frames, then we can now have
first-class slides, because frames are already first class. This solves the
initial problem:

    put title_slide("Section 1") until 1f delay 0f

    put
    {
      // First slide of section 1.
    }
    until 1f delay 1f

    put title_slide("Section 2") until 1f delay 2f

    put
    {
      // First slide of section 2.
    }
    until 1f delay 3f

The only downside now, is that it is not quite ergonomic. Especially having to
manually specify frame times is annoying and unmaintainable, because you cannot
easily insert or swap frames. We can alleviate this with a counter though:

    t = 0f

    put title_slide("Section 1") until 1f delay t
    t = t + 1f

    put
    {
      // First slide of section 1.
    }
    until 1f delay t
    t = t + 1f

    put title_slide("Section 2") until 1f delay t
    t = t + 1f

    put
    {
      // First slide of section 2.
    }
    until 1f delay t

At this point, I feel that syntactic sugar would be helpful. Pris is a
<abbr>DSL</abbr> after all. We can introduce an implicit `t` variable,
make `put` implicitly delay everything by that, and add a `clear` statement
that caps everything placed so far in time, and advances `t`. `pause` would do
the same, but without the `until`, therefore not clearing the current elements.

    put title_slide("Section 1")

    clear
    put
    {
      // Uncover a bulleted list.
      put some_bullet_1()
      pause
      put some_bullet_2()
    }

    clear
    put title_slide("Section 2")

    clear
    put
    {
      // First slide of section 2.
    }

This does look like a workable solution to me. What bothers me though, is that
now there is an implicit “time origin” that `put` respects. For consistency,
should there be something similar for space? E.g.

    put t("First line")
    move (0em, line_height)
    put t("Second line")

Actually, now that I write this down, I realize that I have badly wanted this
for a long time. **Yes**, there should also be an implicit origin, with
statements to mutate it.

Also, it would be very natural to integrate this with the anchor. Currently, the
place of the anchor is a bit ad-hoc, and the user does not have direct control
over it, although you can `put {} at somewhere` as the last statement as a
workaround. If the anchor would simply be the final value of the implicit
origin, then that would unify space and time.

Also, an implicit origin/anchor in time is useful too! For placement in time,
the default should be to extend indefinitely into the future. I.e. once placed,
elements never disappear, unless you explicitly clip them in time. But that
raises a problem: what is the duration of an animation? Taking the convex hull
of the timespans would make it infinite, but there is a finite time at which an
animation ends, after which it is constant. We could define “duration”
arbitrarily as that point, but it may not be what you want. For example, it
would make it difficult to construct a frame with a duration of two subframes.
(You could `put {} delay 1f` as a workaround, just like with the bounding box.)
Having an anchor in time provides an elegant solution here: the last time offset
becomes the duration. And with `pause` as a statement to advance it, it also
usually has the right value.

When placing an animation itself though, you may want to advance the current
time by its duration. For example:

    a1 = animation_1()
    a2 = animation_2()

    {
      // Play animations in parrallel.
      put a1
      put a2
      pause max(a1.duration, a2.duration)
    }

    {
      // Play animations sequentially.
      put a1
      pause a1.duration
      put a2
      pause a2.duration
    }

The latter example has an equivalent in space that I find myself needing often:

    {
      put t("Line 1") at (1em, 1em + 1 * line_height)
      put t("Line 2") at (1em, 1em + 2 * line_height)
      put t("Line 3") at (1em, 1em + 3 * line_height)
    }

With `move`, we could write it as follows:

    {
      t1 = t("Line 1\n")
      t2 = t("Line 2\n")
      t3 = t("Line 3\n")
      move (1em, 1em)
      put t1
      move t1.anchor
      put t2
      move t2.anchor
      put t3
      move t3.anchor
    }

This raises the question: should there be two variants of `put`? One that places
an element without moving the anchor or advancing the time, and one that does?
For example, we may add a `@put` (syntax just an idea, better names or sigils
are welcome) and have:

    {
      // Play sequentially.
      @put a1
      @put a2
    }

    {
      // Place lines below one another.
      move (1em, 1em)
      @put t"(Line 1\n")
      @put t"(Line 2\n")
      @put t"(Line 3\n")
    }

That again raises the question then, should `delay` and `at` be the same
function, with coords having a time component in addition to space? And should
`move` and `pause` be the same statement, perhaps called `advance` instead?

Unifying coordinates in space and time does seem like a very natural thing to do
that will further eliminate special cases. The syntax for coordinates could
remain the same, with `(x, y)` constructing a coordinate where the time
component is zero, and `1f` (or maybe `@1f` or something like that to set a time
coordinate apart from a number with time dimensions) would be a coordinate where
the spatial component is zero.

## Conclusion

 * Frames extend in space (the bounding box) and in time.
 * Similar to how `at` translates a frame in space, `delay` translates in time.
 * `put` will no longer place things at the origin, it will place them at the
   current *anchor*, which starts out at the origin.
 * `move` can move the anchor by a relative offset.
 * Similarly, `put` will not place things at time 0, it will place them at the
   current *subframe*, which starts out at 0.
 * `pause` can advance the subframe by a relative offset. (It could be fixed to
   1, but maybe it is good for consistency with `move` to make it variable.)
 * `clear` would clip all currently placed elements in time up to the current
   subframe, and then advance the current subframe.
