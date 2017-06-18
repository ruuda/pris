// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

// TODO: Remove when parser is done.
#![allow(dead_code)]
#![allow(unused_variables)]

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
use lexer::{Token, lex};
use error::{Error, Result};

struct Parser<'a> {
    tokens: &'a [(usize, Token<'a>, usize)],
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
#[derive(Debug)]
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
    fn new(tokens: &'a [(usize, Token<'a>, usize)]) -> Parser<'a> {
        Parser {
            tokens: tokens,
        }
    }

    /// Run the parser on the full input and return the resulting document.
    fn parse_document(&self, start: usize) -> PResult<Document<'a>> {
        // TODO: Have a pre-pass that checks for balanced parens and brackets.
        // That will produce more helpful error messages than "unexpected token"
        // at the mismatched closing bracket.
        panic!("not_implemented");
    }

    fn parse_statements(&self, start: usize) -> PResult<Vec<Stmt<'a>>> {
        debug_assert!(start < self.tokens.len());

        let mut statements = Vec::new();
        let mut i = start;
        while let Ok((j, stmt)) = self.parse_statement(i) {
            statements.push(stmt);
            i = j;
            if i == self.tokens.len() { break }
        }
        Ok((i, statements))
    }

    fn parse_statement(&self, start: usize) -> PResult<Stmt<'a>> {
        debug_assert!(start < self.tokens.len());

        match self.tokens[start].1 {
            Token::KwImport => map(Stmt::Import, self.parse_import(start)),
            Token::Ident(..) => map(Stmt::Assign, self.parse_assign(start)),
            Token::KwReturn => map(Stmt::Return, self.parse_return(start)),
            Token::LBrace => map(Stmt::Block, self.parse_block(start)),
            Token::KwPut | Token::KwAt => map(Stmt::PutAt, self.parse_put_at(start)),
            _ => {
                let msg = "Parse error in statement: expected import, return, \
                           assignment, block, or put-at.";
                parse_error(start, msg)
            }
        }
    }

    fn parse_idents(&self, start: usize) -> PResult<Idents<'a>> {
        let mut idents = Vec::new();
        let mut i = start;

        // Take one identifier. If it is followed by a dot, repeat.
        loop {
            let (j, ident) = self.parse_ident(i)?;

            idents.push(ident);
            i = j;

            if i >= self.tokens.len() { break }

            if self.tokens[i].1 == Token::Dot {
                i += 1
            } else {
                break
            }
        }

        Ok((i, Idents(idents)))
    }

    fn parse_ident(&self, start: usize) -> PResult<&'a str> {
        if start < self.tokens.len() {
            if let Token::Ident(ident) = self.tokens[start].1 {
                return Ok((start + 1, ident))
            }
        }
        parse_error(start, "Parse error: expected identifier.")
    }

    fn parse_import(&self, start: usize) -> PResult<Import<'a>> {
        debug_assert!(self.tokens[start].1 == Token::KwImport);

        match self.parse_idents(start + 1) {
            Ok((i, idents)) => Ok((i, Import(idents))),
            Err(err) => {
                let msg = "Parse error in import: expected path like 'std.colors'.";
                parse_error(err.token_index, msg)
            }
        }
    }

    fn parse_assign(&self, start: usize) -> PResult<Assign<'a>> {
        let (_, ident) = self.parse_ident(start)?;
        // TODO: Add hints to these messages. It is possible to explain here
        // that nested assignments are not allowed.
        let msg = "Parse error: expected '='.";
        let _ = self.expect_token(start + 1, Token::Equals, msg)?;
        let (i, expr) = self.parse_expr(start + 2)?;
        Ok((i, Assign(ident, expr)))
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

    fn parse_expr(&self, start: usize) -> PResult<Term<'a>> {
        // TODO: Operators.
        self.parse_term(start)
    }

    fn parse_term(&self, start: usize) -> PResult<Term<'a>> {
        use parser_utils::unescape_string_literal;
        use parser_utils::unescape_raw_string_literal;
        use parser_utils::parse_color;

        let msg = "Parse error in expression: expected one more token.";
        let (_, tok) = self.take_token(start, msg)?;

        match tok {
            Token::String(ref s) => {
                // TODO: Return the right kind of parse error there, or make the
                // type an enum.
                let contents = unescape_string_literal(s).unwrap();
                Ok((start + 1, Term::String(contents)))
            }
            Token::RawString(ref s) => {
                // TODO: Return the right kind of parse error there, or make the
                // type an enum.
                let contents = unescape_raw_string_literal(s);
                Ok((start + 1, Term::String(contents)))
            }
            Token::Color(ref cs) => {
                Ok((start + 1, Term::Color(parse_color(cs))))
            }
            Token::Number(..) => map(Term::Number, self.parse_number(start)),
            Token::Ident(..) => map(Term::Idents, self.parse_idents(start)),
            Token::KwFunction => map(Term::FnDef, self.parse_fn_def(start)),
            Token::LBrace => map(Term::Block, self.parse_block(start)),
            // Only in the case of an opening paren, it is ambiguous what to
            // parse: it could become a coord or an expression between parens.
            Token::LParen => self.parse_coord_or_parens(start),
            _ => parse_error(start, "Parse error: expected expression."),
        }
    }

    fn parse_number(&self, start: usize) -> PResult<Num> {
        use std::str::FromStr;

        let num_str = match self.tokens[start].1 {
            Token::Number(x) => x,
            _ => unreachable!("parse_number must be called with cursor at number token"),
        };

        // The unwrap here is safe, because the lexer guarantees that we get a
        // string of the right format.
        let x = f64::from_str(&num_str).unwrap();

        let unit = match self.take_token(start + 1, "<unused msg>").map(|t| t.1) {
            Ok(Token::UnitEm) => Some(Unit::Em),
            Ok(Token::UnitPt) => Some(Unit::Pt),
            Ok(Token::UnitW) => Some(Unit::W),
            Ok(Token::UnitH) => Some(Unit::H),
            _ => None,
        };

        match unit {
            Some(u) => Ok((start + 2, Num(x, unit))),
            None => Ok((start + 1, Num(x, None))),
        }
    }

    fn parse_fn_def(&self, start: usize) -> PResult<FnDef<'a>> {
        debug_assert!(self.tokens[start].1 == Token::KwFunction);

        let msg = "Parse error in function definition: expected '('.";
        self.expect_token(start + 1, Token::LParen, msg)?;

        let (i, args) = self.parse_fn_def_args(start + 1)?;

        let msg = "Parse error in function definition: expected '{'.";
        self.expect_token(i, Token::LBrace, msg)?;

        let (j, body) = self.parse_block(i)?;

        Ok((j, FnDef(args, body)))
    }

    /// Parse arguments between parentheses, like "()" or "(a, b, c)".
    fn parse_fn_def_args(&self, start: usize) -> PResult<Vec<&'a str>> {
        debug_assert!(self.tokens[start].1 == Token::LParen);

        let mut args = Vec::new();
        let mut i = start + 1;

        // Take one identifier. If it is followed by a comma, repeat. If we find
        // a closing paren instead, we are done.
        loop {
            let msg = "Parse error in function definition: expected argument name or ')'.";
            let (j, tok) = self.take_token(i, msg)?;
            match tok {
                Token::Ident(ident) => args.push(ident),
                Token::RParen => break,
                _ => return parse_error(i, msg),
            }

            i = j;

            let msg = "Parse error in function definition: expected ',' or ')'.";

            if i >= self.tokens.len() { return parse_error(i, msg) }

            match self.tokens[i].1 {
                Token::Comma => i += 1,
                Token::RParen => break,
                _ => return parse_error(i, msg),
            }
        }

        Ok((i + 1, args))
    }

    fn parse_coord_or_parens(&self, start: usize) -> PResult<Term<'a>> {
        debug_assert!(self.tokens[start].1 == Token::LParen);

        let (i, expr_x) = self.parse_expr(start + 1)?;

        // If we find a ')' then we are done, it was an expression between
        // parens. If we find a ',' then it is a coord. Otherwise an error.
        let msg = "Parse error: expected ',' or ')'.";
        match self.take_token(i, msg)?.1 {
            Token::RParen => return Ok((i + 1, expr_x)),
            Token::Comma => {},
            _ => return parse_error(i, msg),
        }

        // If we get here, we are in the coordinate case.
        let (j, expr_y) = self.parse_expr(i + 1)?;
        let msg = "Parse error in coordinate: expected ')'.";
        self.expect_token(j, Token::RParen, msg)?;

        let coord = Box::new(Coord(expr_x, expr_y));
        Ok((j + 1, Term::Coord(coord)))
    }

    fn take_token(&self,
                  at: usize,
                  error_message: &'static str) -> PResult<Token<'a>> {
        if at < self.tokens.len() {
            Ok((at + 1, self.tokens[at].1))
        } else {
            parse_error(at, error_message)
        }
    }

    fn expect_token(&self,
                    at: usize,
                    token: Token<'a>,
                    error_message: &'static str)
                    -> PResult<()> {
        if at < self.tokens.len() {
            if self.tokens[at].1 == token {
                return Ok((at + 1, ()))
            }
        }
        parse_error(at, error_message)
    }
}

#[test]
fn parse_parses_idents_single() {
    let tokens = lex(b"foo 22").unwrap();
    let (i, idents) = Parser::new(&tokens).parse_idents(0).unwrap();
    assert_eq!(i, 1);
    assert_eq!(&idents.0[..], ["foo"]);
}

#[test]
fn parse_parses_idents_multiple() {
    let tokens = lex(b"foo.bar.baz").unwrap();
    let (i, idents) = Parser::new(&tokens).parse_idents(0).unwrap();
    assert_eq!(i, tokens.len());
    assert_eq!(&idents.0[..], ["foo", "bar", "baz"]);
}

#[test]
fn parse_fails_empty() {
    let tokens = lex(b"put").unwrap(); // "put" is a keyword, not identifier.
    let result = Parser::new(&tokens).parse_idents(0);
    assert_eq!(result.err().unwrap().token_index, 0);
}

#[test]
fn parse_fails_idents_unfinished_dot() {
    let tokens = lex(b"foo.").unwrap();
    let result = Parser::new(&tokens).parse_idents(0);
    assert_eq!(result.err().unwrap().token_index, 2);
}

#[test]
fn parse_parses_string_literal() {
    let tokens = lex(b"\"foo\"").unwrap();
    let (i, lit) = Parser::new(&tokens).parse_term(0).unwrap();
    assert_eq!(i, 1);
    assert!(lit == Term::String(String::from("foo")));
}

#[test]
fn parse_parses_idents_term() {
    let tokens = lex(b"foo.bar.baz 22").unwrap();
    let (i, term) = Parser::new(&tokens).parse_term(0).unwrap();
    assert_eq!(i, 5);
    match term {
        Term::Idents(idents) => assert_eq!(&idents.0[..], ["foo", "bar", "baz"]),
        _ => panic!("expected idents"),
    }
}

#[test]
fn parse_parses_unitless_number_literal() {
    let tokens = lex(b"31seconds").unwrap();
    let (i, lit) = Parser::new(&tokens).parse_term(0).unwrap();
    assert_eq!(i, 1);
    assert!(lit == Term::Number(Num(31.0, None)));
}

#[test]
fn parse_parses_unitful_number_literal() {
    let tokens = lex(b"0.5em").unwrap();
    let (i, lit) = Parser::new(&tokens).parse_term(0).unwrap();
    assert_eq!(i, 2);
    assert!(lit == Term::Number(Num(0.5, Some(Unit::Em))));
}

#[test]
fn parse_parses_number_then_eof() {
    let tokens = lex(b"17").unwrap();
    let (i, lit) = Parser::new(&tokens).parse_term(0).unwrap();
    assert_eq!(i, 1);
    assert!(lit == Term::Number(Num(17.0, None)));
}

#[test]
fn parse_parses_fn_def_args_empty() {
    let tokens = lex(b"()").unwrap();
    let (i, args) = Parser::new(&tokens).parse_fn_def_args(0).unwrap();
    let bogus = ["foobar"]; // Required to help type inference.
    assert_eq!(i, 2);
    assert_eq!(&args[..], &bogus[..0]);
}

#[test]
fn parse_parses_fn_def_args_one() {
    let tokens = lex(b"(x)").unwrap();
    let (i, args) = Parser::new(&tokens).parse_fn_def_args(0).unwrap();
    assert_eq!(i, 3);
    assert_eq!(&args[..], &["x"]);
}

#[test]
fn parse_parses_fn_def_args_two() {
    let tokens = lex(b"(x, y)").unwrap();
    let (i, args) = Parser::new(&tokens).parse_fn_def_args(0).unwrap();
    assert_eq!(i, 5);
    assert_eq!(&args[..], &["x", "y"]);
}

#[test]
fn parse_parses_fn_def_empty() {
    let tokens = lex(b"function() {}").unwrap();
    // TODO: Enable this once blocks are implemented.
    // let (i, fn_def) = Parser::new(&tokens).parse_fn_def(0).unwrap();
    // assert_eq!(i, 5);
    // assert_eq!(fn_def.0.len(), 0); // Zero arguments.
    // assert_eq!((fn_def.1).0.len(), 0); // Zero statements.
}

#[test]
fn parse_parses_coord() {
    let tokens = lex(b"(1, 2)").unwrap();
    let (i, coord) = Parser::new(&tokens).parse_term(0).unwrap();
    let one = Term::Number(Num(1.0, None));
    let two = Term::Number(Num(2.0, None));
    assert_eq!(i, 5);
    assert!(coord == Term::Coord(Box::new(Coord(one, two))));
}

#[test]
fn parse_parses_expr_in_parens() {
    let tokens = lex(b"(1)").unwrap();
    let (i, term) = Parser::new(&tokens).parse_term(0).unwrap();
    let one = Term::Number(Num(1.0, None));
    assert_eq!(i, 3);
    assert!(term == one);
}
