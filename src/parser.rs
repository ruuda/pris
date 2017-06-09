// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

//! This module contains the Pris parser.
//!
//! The parser is a hand-written recursive descent parser. This is not the most
//! efficient kind of parser, but it is doable to maintain it by hand, and it
//! can generate helpful error messages.
//!
//! A formal description of the grammar is available in the form of a Bison
//! grammar in etc/pris.y.

use std::result;

use ast::{Assign, BinOp, BinTerm, Block, Coord, Document, FnCall, FnDef};
use ast::{Idents, Import, Num, PutAt, Return, Stmt, Term, UnOp, UnTerm, Unit};
use lexer::Token;
use error::{Error, Result};

struct Parser<'a> {
    tokens: Vec<(usize, Token<'a>, usize)>,
}

/// An intermediate parse error.
///
/// To report friendly parse errors, eventually we will construct an
/// `error::ParseError`. But during parsing, many operations can fail, which is
/// not fatal, it just means that the parser must backtrack. Constructing a
/// heap-allocated error message in a large struct with many pointer-sized
/// integers would be wasteful. So during parsing, errors are only collected
/// into this structure. If that error turns out to be fatal, all information
/// required to build a full parse error is here: the index of the wrong token,
/// which in turn contains the source location, and the prefix for the error
/// message, to which the "actually found" part still needs to be appended.
struct PError {
    token_index: usize,
    message: &'static str,
}

/// A parse result, either (next_token_index, value), or a parse error.
type PResult<T> = result::Result<(usize, T), PError>;

fn parse_error<T>(token_index: usize, message: &'static str) -> PResult<T> {
    let err = PError {
        token_index: token_index,
        message: message,
    };
    Err(err)
}

fn map<T, U, F: Fn(T) -> U>(f: F, result: PResult<T>) -> PResult<U> {
    result.map(|(i, x)| (i, f(x)))
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<(usize, Token<'a>, usize)>) -> Parser<'a> {
        Parser {
            tokens: tokens,
        }
    }

    /// Run the parser on the full input and return the resulting document.
    fn parse_document(&self, start: usize) -> PResult<Document<'a>> {
        panic!("not_implemented");
    }

    fn parse_statements(&self, start: usize) -> PResult<Vec<Stmt<'a>>> {
        let mut statements = Vec::new();
        let mut i = start;
        while let Ok((j, stmt)) = self.parse_statement(i) {
            statements.push(stmt);
            i = j;
        }
        Ok((i, statements))
    }

    fn parse_statement(&self, start: usize) -> PResult<Stmt<'a>> {
        match self.tokens[start].1 {
            Token::KwImport => map(Stmt::Import, self.parse_import(start)),
            Token::Ident(..) => map(Stmt::Assign, self.parse_assign(start)),
            Token::KwReturn => map(Stmt::Return, self.parse_return(start)),
            Token::LBrace => map(Stmt::Block, self.parse_block(start)),
            Token::KwPut | Token::KwAt => map(Stmt::PutAt, self.parse_put_at(start)),
            _ => {
                let msg = "in statement: expected import, return, assignment, block, or put-at.";
                parse_error(start, msg)
            }
        }
    }

    fn parse_import(&self, start: usize) -> PResult<Import<'a>> {
        debug_assert!(self.tokens[start].1 == Token::KwImport);

        match self.parse_idents(start + 1) {
            Ok((i, idents)) => Ok((i, Import(idents))),
            Err(err) => {
                let msg = "in import: expected path like 'std.colors'.";
                parse_error(err.token_index, msg)
            }
        }
    }

    fn parse_assign(&self, start: usize) -> PResult<Assign<'a>> {
        panic!("not implemented");
    }

    fn parse_return(&self, start: usize) -> PResult<Return<'a>> {
        panic!("not implemented");
    }

    fn parse_block(&self, start: usize) -> PResult<Block<'a>> {
        panic!("not implemented");
    }

    fn parse_put_at(&self, start: usize) -> PResult<PutAt<'a>> {
        panic!("not implemented");
    }

    fn parse_idents(&self, start: usize) -> PResult<Idents<'a>> {
        panic!("not implemented");
    }
}
