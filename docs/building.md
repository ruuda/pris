# Building

Pris is written in [Rust][rust] and builds with Cargo, the build tool bundled
with Rust. When system dependencies are installed, `cargo build` is sufficient.

    git clone https://github.com/ruuda/pris
    cd pris
    cargo build --release
    target/release/pris --help
    target/release/pris examples/lines.pris
    evince examples/lines.pdf

If building does not succeed at first, some system dependencies might not be
installed. See below.

## Linux

To install system dependencies on Ubuntu:

    apt install libcairo2-dev libharfbuzz-dev librsvg2-dev

To install system dependencies on Arch Linux:

    pacman -S --needed cairo harfbuzz librsvg fontconfig freetype2

## Mac

On Mac, system dependencies can be installed through Homebrew:

    brew install cairo harfbuzz librsvg fontconfig

## Windows

On Windows, system dependencies can be installed inside an [MSYS2][msys2]
environment. More information will follow soon. For now, check out the `windows`
branch for more information.

[rust]:  https://www.rust-lang.org/
[msys2]: http://www.msys2.org/
