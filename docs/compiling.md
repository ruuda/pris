## Windows

On MSYS2, install the following packages:

    pacman -Sy --needed           \
      make gcc pkg-config         \
      mingw-w64-x86_64-cairo      \
      mingw-w64-x86_64-harfbuzz   \
      mingw-w64-x86_64-fontconfig \
      mingw-w64-x86_64-freetype   \
      mingw-w64-x86_64-librsvg

Then compile as follows from an MSYS2 MinGW 64-bit shell:

    RUSTFLAGS=$(pkg-config --libs-only-L \
      fontconfig harfbuzz librsvg-2.0 gobject-2.0 cairo freetype2
    ) cargo build --release

In theory it should be possible to make `build.rs` pass the right `-L` flags to
rustc, but in practice I have not been able to make this work. Also, librsvg and
gobject are not actually version 2.0, but for some reason pkg-config identifies
them by that name.
