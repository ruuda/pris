# Parser

The current parser used in Pris is a hand-written parser. See `src/parser.rs`
for more information.

Previously, a parser generated by [`lalrpop`][lalrpop] was used. This parser was
replaced mainly because of the long compile times caused by `lalrpop` and its
many dependencies. The opportunity for better error messages and reduced binary
size are other advantages.

## Build time

Build times on Rust 1.19.0-beta.1, median and standard deviation of 3 clean
builds, excluding crate download time:

| Version   | Mode    | Build time (s) |
| --------- | ------- | --------------:|
| `b504558` | debug   |     83.2 ± 2.2 |
| `4bb1e60` | debug   |     16.2 ± 0.5 |
| `b504558` | release |    225.9 ± 3.9 |
| `4bb1e60` | release |     36.8 ± 1.9 |

`b504558` is the `lalrpop`-based revision, `4bb1e60` uses the new parser.

## Binary size

The binary size for a release build, after stripping:

| Version   | Binary size (KiB) |
| --------- | -----------------:|
| `b504558` |            1644.0 |
| `4bb1e60` |            1380.0 |

That is almost a 20% reduction in binary size.

[lalrpop]: https://crates.io/crates/lalrpop