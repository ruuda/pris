// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

extern crate lalrpop;
extern crate pkg_config;

fn main() {
    // At build time, run LALRPOP to convert syntax definitions into parsers.
    lalrpop::process_root().unwrap();

    // On Windows on MSYS2, we need to explicitly locate the dependencies using
    // pkg-config, because the compiler does not find them automatically like on
    // other platforms.
    /* pkg_config::probe_library("cairo").unwrap();
    pkg_config::probe_library("fontconfig").unwrap();
    pkg_config::probe_library("harfbuzz").unwrap();
    pkg_config::probe_library("rsvg-2").unwrap();
    pkg_config::probe_library("gobject-2.0").unwrap();
    pkg_config::probe_library("freetype").unwrap(); */
}
