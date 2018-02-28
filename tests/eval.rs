// Pris -- A language for designing slides
// Copyright 2018 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

//! Tests in this file evaluate a script fully, and then check that the value of
//! the 'result' variable in the top-level environment equals a desired value.

extern crate pris;

use pris::ast::Idents;
use pris::interpreter;
use pris::lexer;
use pris::parser;
use pris::runtime;
use pris::pretty;

fn eval<'a>(input: &'a [u8]) -> String {
    let doc = lexer::lex(input)
        .and_then(|tokens| parser::parse(&tokens[..]))
        .expect("Test script contains syntax error.");

    let mut fm = runtime::FontMap::new();
    let mut stmt_interpreter = interpreter::StmtInterpreter::new(&mut fm);
    for statement in &doc.0 {
        stmt_interpreter
            .eval_statement(statement)
            .expect("Test script failed with an error.");
    }
    let result = stmt_interpreter
        .env()
        .lookup(&Idents(vec!["result"]))
        .expect("Test script did not assign to 'result' variable.");

    pretty::print(result)
}

#[test]
fn eval_num() {
    assert_eq!(eval(b"result = 32"), "32 : num");
}

#[test]
fn eval_coord_x() {
    let src = br#"
    coord = (1w, 2w)
    result = coord.x
    "#;
    assert_eq!(eval(src), "1920 : len");
}

#[test]
fn eval_coord_y() {
    let src = br#"
    coord = (1w, 2w)
    result = coord.y
    "#;
    assert_eq!(eval(src), "3840 : len");
}
