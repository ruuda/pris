## Windows

On MSYS2, install the following packages:

```
pacman -Syy
pacman -S
make
gcc
pkg-config
mingw-w64-x86_64-cairo
mingw-w64-x86_64-harfbuzz
mingw-w64-x86_64-fontconfig
mingw-w64-x86_64-freetype
mingw-w64-x86_64-librsvg
```

Manually including the pkg-config flags (including the double ones) did the
trick for me, I was able to compile a working binary.