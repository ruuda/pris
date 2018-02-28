// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

extern crate freetype;

mod builtins;
mod elements;
mod fontconfig;
mod harfbuzz;
mod parser_utils;
mod png;
mod rsvg;
mod types;

#[macro_use]
pub mod pretty;

pub mod ast;
pub mod cairo;
pub mod driver;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod runtime;

// This is the compiler entry point for the library, which is used by the
// command-line program. The source for that program is in bin/pris.rs.
