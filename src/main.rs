// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

mod ast;
mod syntax;

use std::io;
use std::io::Read;

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    match syntax::parse_statement(&input) {
        Ok(num) => println!("{}", num),
        Err(err) => println!("{:?}", err),
    }
}
