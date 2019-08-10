// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

extern crate pris;

use std::cmp;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io;
use std::path::{Path, PathBuf};

use pris::ast;
use pris::cairo;
use pris::driver;
use pris::error::Error;
use pris::interpreter;
use pris::lexer;
use pris::parser;
use pris::runtime;

const USAGE: &'static str = "
Pris, a language for designing slides.

Usage:
  pris [--] <infile> [<outfile>]
  pris (-h | --help)

Options:
  -h --help  Show this screen.

If the output file is not specified, it defaults to the input file, with
the extension replaced with '.pdf'. The input file name can optionally
be '-' to read from stdin. In that case the output file is mandatory.
";

fn print_help_and_exit(code: i32) {
    println!("{}", USAGE);
    std::process::exit(code);
}

fn main() {
    let mut fnames = Vec::new();

    for arg in std::env::args().skip(1) {
        match &arg[..] {
            "--" => continue,
            "-h" | "--help" => print_help_and_exit(0),
            _ => {},
        }
        fnames.push(arg);
    }

    if fnames.len() < 1 || fnames.len() > 2 {
        print_help_and_exit(1);
    }

    if fnames.len() == 1 && fnames[0] == "-" {
        println!("Specifiying an output file is required when reading from stdin.");
        std::process::exit(1);
    }

    let infile = Path::new(&fnames[0]);
    let outfile = if fnames.len() == 2 {
        PathBuf::from(&fnames[1])
    } else {
        infile.with_extension("pdf")
    };

    let mut input = Vec::new();

    // Allow reading from stdin by passing "-" as the input filename.
    if fnames[0] == "-" {
        io::stdin().read_to_end(&mut input).unwrap();
    } else {
        let f = File::open(infile)
            .expect("Failed to open input file");
        BufReader::new(f).read_to_end(&mut input)
            .expect("Failed to read input file");
    }

    let doc = parse_or_abort(&input);

    println!("Evaluating document ...");

    let mut frames = Vec::new();
    let mut fm = runtime::FontMap::new();
    let canvas_size: pris::Vec2;

    {
        let mut stmt_interpreter = interpreter::StmtInterpreter::new(&mut fm);
        for statement in &doc.0 {
            let result = match stmt_interpreter.eval_statement(statement) {
                Ok(x) => x,
                Err(e) => { e.print(); panic!("Abort after error.") }
            };
            if let Some(frame) = result { frames.push(frame); }
        }

        canvas_size = match stmt_interpreter
            .env()
            .lookup_coord_num(&ast::Idents(vec!["canvas_size"]))
        {
            Ok(sz) => sz,
            Err(e) => { e.print(); panic!("Abort after error.") }
        }
    }

    let surf = cairo::Surface::new_pdf(&outfile, canvas_size.x, canvas_size.y);
    let mut cr = cairo::Cairo::new(surf);
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_line_width(6.0);

    for (i, frame) in frames.iter().enumerate() {
        println!("[{}/{}] Painting frame ...", i + 1, frames.len());
        driver::render_frame(&mut fm, &mut cr, canvas_size, frame);
    }

    drop(cr);

    println!("Document written to {}.", outfile.to_str().unwrap());
}

fn report_error(input: &[u8], location: usize, len: usize) {
    // Locate the line that contains the error.
    // TODO: Deal with errors that span multiple lines.
    let mut line = 1;
    let mut start = 0;
    let mut end = 0;
    for (&c, i) in input.iter().zip(0..) {
        if i == location { break }
        if c == b'\n' {
            line += 1;
            start = i + 1;
        }
    }
    for (&c, i) in input[start..].iter().zip(start..) {
        if c == b'\n' {
            end = i;
            break
        }
    }

    // Try as best as we can to report the error. However, if the parse failed
    // because the input was invalid UTF-8, there is little we can do.
    let line_content = String::from_utf8_lossy(&input[start..end]);

    // The length of the mark can be longer than the line, for example when
    // token to mark was a multiline string literal. In that case, highlight
    // only up to the newline, don't extend the tildes too far.
    let mark_len = cmp::min(len, line_content.len() + start - location);

    println!("Parse error at line {}:\n", line);
    println!("{}", line_content);
    for _ in 0..location - start { print!(" "); }
    print!("^");
    for _ in 1..mark_len { print!("~"); }
    print!("\n");
}

fn parse_or_abort<'a>(input: &'a [u8]) -> ast::Document<'a> {
    match lexer::lex(input).and_then(|tokens| parser::parse(&tokens[..])) {
        Ok(doc) => doc,
        Err(Error::Parse(e)) => {
            report_error(input, e.start, e.end - e.start);
            Error::Parse(e).print();
            std::process::exit(1)
        }
        _ => unreachable!(),
    }
}
