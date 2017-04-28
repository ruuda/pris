#!/bin/sh

# This is the Windows build script, it is mainly responsible for ensuring
# dependencies are installed, and invoking Cargo in the right way. Other
# platforms do not need such a build script because we assume dependencies are
# installed already there, and Cargo works out of the box.
#
# This script must be executed inside an MSYS2 shell.

# Fail if any command fails.
set -e

# Install external dependencies (Cairo, Harfbuzz, etc.) and gcc and pkg-config,
# which are required to build some of the Rust dependencies.

pacman -Sqy --needed --noconfirm \
  make gcc pkg-config            \
  mingw-w64-x86_64-cairo         \
  mingw-w64-x86_64-harfbuzz      \
  mingw-w64-x86_64-fontconfig    \
  mingw-w64-x86_64-freetype      \
  mingw-w64-x86_64-librsvg

# Pass the library search path to rustc.

export RUSTFLAGS=$(pkg-config --libs-only-L \
  fontconfig harfbuzz librsvg-2.0 gobject-2.0 cairo freetype2)

cargo build
cargo test
