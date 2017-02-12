# Ideas

 * First impressions are important. Putting things at `(0, 0)` happens often,
   but it is a type error, as it requires units. As the first thing people witll
   try is likely `put "hello world" at (0, 0)`, this is a bad experience.
 * Also, aligning a string to the bottom-left at the baseline and then putting
   it at `(0, 0)` where `(0, 0)` is the top-left corner of the screen, will
   result in an off-screen string.
 * Both might be solved by making `put t("Hello world") at (1em, 2em)` the
   canonical "Hello world". But still, accepting `(0, 0)` as length might be
   useful.
