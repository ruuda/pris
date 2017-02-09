// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

extern crate lalrpop;

fn main() {
    // At build time, run LALRPOP to convert syntax definitions into parsers.
    lalrpop::process_root().unwrap();
}
