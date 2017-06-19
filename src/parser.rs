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

use ast::{Assign, BinOp, BinTerm, Block, Color, Coord, Document, FnCall, FnDef};
use ast::{Idents, Import, Num, PutAt, Return, Stmt, Term, UnOp, UnTerm, Unit};
use lexer::{Token, lex};
use error::{Error, Result};

struct Parser<'a> {
    tokens: &'a [(usize, Token<'a>, usize)],
    cursor: usize,
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

/// A parse result, either the parsed value, or a parse error.
type PResult<T> = result::Result<T, PError>;

/// Helper trait for implementation convenience. For parser internal use only.
trait ReplaceError {
    fn replace_error(self, new_message: &'static str) -> Self;
}

impl<T> ReplaceError for PResult<T> {
    fn replace_error(self, new_message: &'static str) -> PResult<T> {
        self.map_err(|e| PError { message: new_message, ..e })
    }
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [(usize, Token<'a>, usize)]) -> Parser<'a> {
        Parser {
            tokens: tokens,
            cursor: 0,
        }
    }

    /// Run the parser on the full input and return the resulting document.
    fn parse_document(&mut self) -> PResult<Document<'a>> {
        // TODO: Have a pre-pass that checks for balanced parens and brackets.
        // That will produce more helpful error messages than "unexpected token"
        // at the mismatched closing bracket.
        panic!("not_implemented");
    }

    fn parse_statements(&mut self) -> PResult<Vec<Stmt<'a>>> {
        debug_assert!(self.cursor < self.tokens.len());

        let mut statements = Vec::new();
        // TODO: Do not rely on error handling to determine next action:
        // actually we can just inspect the token. When parsing a block, we
        // expect to see a '}', when parsing a document, we expect to see EOF.
        while let Ok(stmt) = self.parse_statement() {
            statements.push(stmt);
            if self.cursor == self.tokens.len() { break }
        }
        Ok(statements)
    }

    fn parse_statement(&mut self) -> PResult<Stmt<'a>> {
        debug_assert!(self.cursor < self.tokens.len());

        match self.tokens[self.cursor].1 {
            Token::KwImport => self.parse_import().map(Stmt::Import),
            Token::Ident(..) => self.parse_assign().map(Stmt::Assign),
            Token::KwReturn => self.parse_return().map(Stmt::Return),
            Token::LBrace => self.parse_block().map(Stmt::Block),
            Token::KwPut | Token::KwAt => self.parse_put_at().map(Stmt::PutAt),
            _ => {
                let msg = "Parse error in statement: expected import, return, \
                           assignment, block, or put-at.";
                self.error(msg)
            }
        }
    }

    fn parse_idents(&mut self) -> PResult<Idents<'a>> {
        let mut idents = Vec::new();

        // Take one identifier. If it is followed by a dot, repeat. There is at
        // least one identifier.
        loop {
            idents.push(self.parse_ident()?);

            match self.peek() {
                Some(Token::Dot) => self.consume(),
                _ => break,
            }
        }

        Ok(Idents(idents))
    }

    fn parse_ident(&mut self) -> PResult<&'a str> {
        if let Some(Token::Ident(ident)) = self.peek() {
            self.consume();
            Ok(ident)
        } else {
            self.error("Parse error: expected identifier.")
        }
    }

    fn parse_import(&mut self) -> PResult<Import<'a>> {
        assert!(self.take() == Some(Token::KwImport));

        let msg = "Parse error in import: expected path like 'std.colors'.";
        self.parse_idents().map(Import).replace_error(msg)
    }

    fn parse_assign(&mut self) -> PResult<Assign<'a>> {
        // TODO: Add hints to these messages. It is possible to explain here
        // that nested assignments are not allowed.
        let msg = "Parse error: expected '='.";

        let ident = self.parse_ident()?;
        self.expect_consume(Token::Equals, msg)?;
        let expr = self.parse_expr()?;

        Ok(Assign(ident, expr))
    }

    fn parse_return(&mut self) -> PResult<Return<'a>> {
        assert!(self.take() == Some(Token::KwReturn));
        self.parse_expr().map(Return)
    }

    fn parse_block(&mut self) -> PResult<Block<'a>> {
        panic!("not implemented");
    }

    fn parse_put_at(&mut self) -> PResult<PutAt<'a>> {
        let (frame, coord) = match self.take() {
            Some(Token::KwAt) => {
                let coord = self.parse_expr()?;
                self.expect_consume(Token::KwPut, "Parse error: expected 'put'.")?;
                let frame = self.parse_expr()?;
                (frame, coord)
            }
            Some(Token::KwPut) => {
                let frame = self.parse_expr()?;
                self.expect_consume(Token::KwAt, "Parse error: expected 'at'.")?;
                let coord = self.parse_expr()?;
                (frame, coord)
            }
            _ => unreachable!("parse_put_at must be called with cursor on put or at."),
        };
        Ok(PutAt(frame, coord))
    }

    fn parse_expr(&mut self) -> PResult<Term<'a>> {
        // TODO: Operators.
        self.parse_term()
    }

    fn parse_term(&mut self) -> PResult<Term<'a>> {
        use parser_utils::unescape_string_literal;
        use parser_utils::unescape_raw_string_literal;
        use parser_utils::parse_color;

        let error = self.error("Parse error: expected expression.");

        let token = match self.peek() {
            Some(t) => t,
            None => return error,
        };

        match token {
            // TODO: Return the right kind of parse error there, or make the
            // type an enum.
            Token::String(ref s) => {
                self.consume();
                Ok(Term::String(unescape_string_literal(s).unwrap()))
            }
            // TODO: Return the right kind of parse error there, or make the
            // type an enum.
            Token::RawString(ref s) => {
                self.consume();
                Ok(Term::String(unescape_raw_string_literal(s)))
            }
            Token::Color(ref cs) => {
                self.consume();
                Ok(Term::Color(parse_color(cs)))
            }
            Token::Number(..) => self.parse_number().map(Term::Number),
            Token::Ident(..) => self.parse_idents().map(Term::Idents),
            Token::KwFunction => self.parse_fn_def().map(Term::FnDef),
            Token::LBrace => self.parse_block().map(Term::Block),
            // Only in the case of an opening paren, it is ambiguous what to
            // parse: it could become a coord or an expression between parens.
            Token::LParen => self.parse_coord_or_parens(),
            _ => error,
        }
    }

    fn parse_number(&mut self) -> PResult<Num> {
        use std::str::FromStr;

        let num_str = match self.take() {
            Some(Token::Number(x)) => x,
            _ => unreachable!("parse_number must be called with cursor at number token."),
        };

        // The unwrap here is safe, because the lexer guarantees that we get a
        // string of the right format.
        let x = f64::from_str(&num_str).unwrap();

        let unit = match self.peek() {
            Some(Token::UnitEm) => Some(Unit::Em),
            Some(Token::UnitPt) => Some(Unit::Pt),
            Some(Token::UnitW) => Some(Unit::W),
            Some(Token::UnitH) => Some(Unit::H),
            _ => None,
        };

        if unit.is_some() { self.consume(); }

        Ok(Num(x, unit))
    }

    fn parse_fn_def(&mut self) -> PResult<FnDef<'a>> {
        debug_assert!(self.peek() == Some(Token::KwFunction));

        // Step over the 'function' keyword.
        self.consume();

        let args = self.parse_fn_def_args()?;

        // Peek the body; if the token is not the expected one, we can give
        // more context (parse error in function definition) than block could.
        let msg = "Parse error in function definition: expected '{'.";
        self.expect_peek(Token::LBrace, msg)?;

        let body = self.parse_block()?;

        Ok(FnDef(args, body))
    }

    /// Parse arguments between parentheses, like "()" or "(a, b, c)".
    fn parse_fn_def_args(&mut self) -> PResult<Vec<&'a str>> {
        let msg = "Parse error in function definition: expected '('.";
        self.expect_consume(Token::LParen, msg)?;

        let mut args = Vec::new();

        // Take one identifier. If it is followed by a comma, repeat. If we find
        // a closing paren instead, we are done.
        loop {
            match self.take() {
                Some(Token::Ident(ident)) => args.push(ident),
                Some(Token::RParen) => break,
                _ => return self.error("Parse error in function definition: expected argument name or ')'."),
            }

            match self.take() {
                Some(Token::Comma) => continue,
                Some(Token::RParen) => break,
                _ => return self.error("Parse error in function definition: expected ',' or ')'."),
            }
        }

        Ok(args)
    }

    fn parse_coord_or_parens(&mut self) -> PResult<Term<'a>> {
        debug_assert!(self.peek() == Some(Token::LParen));

        // Step over the opening parenthesis.
        self.consume();

        let expr_x = self.parse_expr()?;

        // If we find a ')' then we are done, it was an expression between
        // parens. If we find a ',' then it is a coord. Otherwise an error.
        match self.take() {
            Some(Token::RParen) => return Ok(expr_x),
            Some(Token::Comma) => {},
            _ => return self.error("Parse error: expected ',' or ')'."),
        }

        // If we get here, we are in the coordinate case.
        let expr_y = self.parse_expr()?;
        self.expect_consume(Token::RParen, "Parse error in coordinate: expected ')'.")?;

        Ok(Term::Coord(Box::new(Coord(expr_x, expr_y))))
    }

    /// Return the token under the cursor, if there is one.
    fn peek(&self) -> Option<Token<'a>> {
        self.tokens.get(self.cursor).map(|t| t.1)
    }

    /// Advance the cursor by one token, consuming the token under the cursor.
    fn consume(&mut self) {
        self.cursor += 1;

        debug_assert!(self.cursor <= self.tokens.len(),
            "Cursor should not go more than one beyond the last token.");
    }

    /// Return the token under the cursor if there is one, advance the cursor by one.
    fn take(&mut self) -> Option<Token<'a>> {
        if self.cursor < self.tokens.len() {
            let token = self.tokens[self.cursor];
            self.consume();
            Some(token.1)
        } else {
            None
        }
    }

    /// Consume one token. If it does not match, return the error message.
    fn expect_consume(&mut self, expected: Token<'a>, message: &'static str) -> PResult<()> {
        match self.peek() {
            Some(token) if token == expected => {
                self.consume();
                Ok(())
            }
            _ => self.error(message),
        }
    }

    /// Like `expect`, but do not consume the token.
    fn expect_peek(&mut self, expected: Token<'a>, message: &'static str) -> PResult<()> {
        match self.peek() {
            Some(token) if token == expected => Ok(()),
            _ => self.error(message),
        }
    }

    /// Build a parse error at the current cursor location.
    fn error<T>(&self, message: &'static str) -> PResult<T> {
        let err = PError {
            token_index: self.cursor,
            message: message,
        };
        Err(err)
    }
}

#[test]
fn parse_parses_import() {
    let tokens = lex(b"import foo.bar").unwrap();
    let mut parser = Parser::new(&tokens);
    let import = parser.parse_import().unwrap();
    assert_eq!(&(import.0).0[..], ["foo", "bar"]);
    assert_eq!(parser.cursor, 4);
}

#[test]
fn parse_parses_idents_single() {
    let tokens = lex(b"foo 22").unwrap();
    let mut parser = Parser::new(&tokens);
    let idents = parser.parse_idents().unwrap();
    assert_eq!(&idents.0[..], ["foo"]);
    assert_eq!(parser.cursor, 1);
}

#[test]
fn parse_parses_idents_multiple() {
    let tokens = lex(b"foo.bar.baz").unwrap();
    let mut parser = Parser::new(&tokens);
    let idents = parser.parse_idents().unwrap();
    assert_eq!(&idents.0[..], ["foo", "bar", "baz"]);
    assert_eq!(parser.cursor, 5);
}

#[test]
fn parse_fails_empty() {
    let tokens = lex(b"put").unwrap(); // "put" is a keyword, not identifier.
    let mut parser = Parser::new(&tokens);
    let result = parser.parse_idents();
    assert_eq!(result.err().unwrap().token_index, 0);
}

#[test]
fn parse_fails_idents_unfinished_dot() {
    let tokens = lex(b"foo.").unwrap();
    let mut parser = Parser::new(&tokens);
    let result = parser.parse_idents();
    assert_eq!(result.err().unwrap().token_index, 2);
}

#[test]
fn parse_parses_string_literal() {
    let tokens = lex(b"\"foo\"").unwrap();
    let mut parser = Parser::new(&tokens);
    let lit = parser.parse_term().unwrap();
    assert!(lit == Term::String(String::from("foo")));
    assert_eq!(parser.cursor, 1);
}

#[test]
fn parse_parses_raw_string_literal() {
    let tokens = lex(b"  ---\n  foo\n  --- appendix").unwrap();
    let mut parser = Parser::new(&tokens);
    let lit = parser.parse_term().unwrap();
    assert!(lit == Term::String(String::from("foo")));
    assert_eq!(parser.cursor, 1);
}

#[test]
fn parse_parses_color() {
    let tokens = lex(b"#c0ffee").unwrap();
    let mut parser = Parser::new(&tokens);
    let color = parser.parse_term().unwrap();
    assert!(color == Term::Color(Color(0xc0, 0xff, 0xee)));
    assert_eq!(parser.cursor, 1);
}

#[test]
fn parse_parses_idents_term() {
    let tokens = lex(b"foo.bar.baz 22").unwrap();
    let mut parser = Parser::new(&tokens);
    let term = parser.parse_term().unwrap();
    match term {
        Term::Idents(idents) => assert_eq!(&idents.0[..], ["foo", "bar", "baz"]),
        _ => panic!("expected idents"),
    }
    assert_eq!(parser.cursor, 5);
}

#[test]
fn parse_parses_unitless_number_literal() {
    let tokens = lex(b"31seconds").unwrap();
    let mut parser = Parser::new(&tokens);
    let lit = parser.parse_term().unwrap();
    assert!(lit == Term::Number(Num(31.0, None)));
    assert_eq!(parser.cursor, 1);
}

#[test]
fn parse_parses_unitful_number_literal() {
    let tokens = lex(b"0.5em").unwrap();
    let mut parser = Parser::new(&tokens);
    let lit = parser.parse_term().unwrap();
    assert!(lit == Term::Number(Num(0.5, Some(Unit::Em))));
    assert_eq!(parser.cursor, 2);
}

#[test]
fn parse_parses_number_then_eof() {
    let tokens = lex(b"17").unwrap();
    let mut parser = Parser::new(&tokens);
    let lit = parser.parse_term().unwrap();
    assert!(lit == Term::Number(Num(17.0, None)));
    assert_eq!(parser.cursor, 1);
}

#[test]
fn parse_parses_fn_def_args_empty() {
    let tokens = lex(b"()").unwrap();
    let mut parser = Parser::new(&tokens);
    let args = parser.parse_fn_def_args().unwrap();
    let bogus = ["foobar"]; // Required to help type inference.
    assert_eq!(&args[..], &bogus[..0]);
    assert_eq!(parser.cursor, 2);
}

#[test]
fn parse_parses_fn_def_args_one() {
    let tokens = lex(b"(x)").unwrap();
    let mut parser = Parser::new(&tokens);
    let args = parser.parse_fn_def_args().unwrap();
    assert_eq!(&args[..], &["x"]);
    assert_eq!(parser.cursor, 3);
}

#[test]
fn parse_parses_fn_def_args_two() {
    let tokens = lex(b"(x, y)").unwrap();
    let mut parser = Parser::new(&tokens);
    let args = parser.parse_fn_def_args().unwrap();
    assert_eq!(&args[..], &["x", "y"]);
    assert_eq!(parser.cursor, 5);
}

#[test]
fn parse_parses_fn_def_empty() {
    let tokens = lex(b"function() {}").unwrap();
    let mut parser = Parser::new(&tokens);
    // TODO: Enable this once blocks are implemented.
    // let fn_def = parser.parse_fn_def(0).unwrap();
    // assert_eq!(parser.cursor, 5);
    // assert_eq!(fn_def.0.len(), 0); // Zero arguments.
    // assert_eq!((fn_def.1).0.len(), 0); // Zero statements.
}

#[test]
fn parse_parses_coord() {
    let tokens = lex(b"(1, 2)").unwrap();
    let mut parser = Parser::new(&tokens);
    let coord = parser.parse_term().unwrap();
    let one = Term::Number(Num(1.0, None));
    let two = Term::Number(Num(2.0, None));
    assert!(coord == Term::Coord(Box::new(Coord(one, two))));
    assert_eq!(parser.cursor, 5);
}

#[test]
fn parse_parses_expr_in_parens() {
    let tokens = lex(b"(1)").unwrap();
    let mut parser = Parser::new(&tokens);
    let term = parser.parse_term().unwrap();
    let one = Term::Number(Num(1.0, None));
    assert!(term == one);
    assert_eq!(parser.cursor, 3);
}

#[test]
fn parse_parses_put_at() {
    let tokens = lex(b"put 1 at 2").unwrap();
    let mut parser = Parser::new(&tokens);
    let put_at = parser.parse_put_at().unwrap();
    let one = Term::Number(Num(1.0, None));
    let two = Term::Number(Num(2.0, None));
    assert!(put_at.0 == one);
    assert!(put_at.1 == two);
    assert_eq!(parser.cursor, 4);
}

#[test]
fn parse_parses_at_put() {
    let tokens = lex(b"at 1 put 2").unwrap();
    let mut parser = Parser::new(&tokens);
    let put_at = parser.parse_put_at().unwrap();
    let one = Term::Number(Num(1.0, None));
    let two = Term::Number(Num(2.0, None));
    assert!(put_at.0 == two);
    assert!(put_at.1 == one);
    assert_eq!(parser.cursor, 4);
}

#[test]
fn parse_fails_at_not_put() {
    let tokens = lex(b"at 1 place 2").unwrap();
    let mut parser = Parser::new(&tokens);
    let result = parser.parse_put_at();
    assert_eq!(result.err().unwrap().token_index, 2);
}

#[test]
fn parse_fails_put_not_at() {
    let tokens = lex(b"put 1 on 2").unwrap();
    let mut parser = Parser::new(&tokens);
    let result = parser.parse_put_at();
    assert_eq!(result.err().unwrap().token_index, 2);
}
