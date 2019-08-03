// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

//! This module contains the Pris parser.
//!
//! The parser is a hand-written recursive descent parser. As the grammar is
//! LL(1), it runs in linear time (in the number of tokens). Pris used a
//! generated parser before, but it was replaced, mainly because it contributed
//! disproportionately to the build time and number of transitive dependencies.
//! Furthermore, a hand-written parser can generate more helpful error messages,
//! and a simple recursive descent parser like this one is still maintainable by
//! hand.
//!
//! A formal description of the grammar is available in the form of a Bison
//! grammar in etc/pris.y.

use std::result;

use ast::{Assign, BinOp, BinTerm, Block, Coord, Document, FnCall, FnDef};
use ast::{Idents, Import, Num, List, Put, Return, Stmt, Term, UnOp, UnTerm, Unit};
use error::{Error, Result};
use lexer::{Span, Token};

/// Parse a token stream into a document.
pub fn parse<'a>(tokens: &[(Token<'a>, Span)]) -> Result<Document<'a>> {
    // TODO: Have a pre-pass that checks for balanced parens and brackets.
    // That will produce more helpful error messages than "unexpected token"
    // at the mismatched closing bracket.
    match Parser::new(tokens).parse_document() {
        Ok(doc) => Ok(doc),
        Err(perr) => {
            assert!(perr.token_index <= tokens.len());
            let mut message = perr.message.into();
            let span = if perr.token_index == tokens.len() {
                message += " Found end of input instead.";
                let span = tokens[tokens.len() - 1].1;
                Span::new(span.end, span.end)
            } else {
                tokens[perr.token_index].1
            };
            // TODO: Make error take a span instead.
            let err = Error::parse(span.start, span.end, message);
            Err(err)
        }
    }
}

struct Parser<'t, 'a: 't> {
    tokens: &'t [(Token<'a>, Span)],
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

impl<'t, 'a: 't> Parser<'t, 'a> {
    fn new(tokens: &'t [(Token<'a>, Span)]) -> Parser<'t, 'a> {
        Parser {
            tokens: tokens,
            cursor: 0,
        }
    }

    /// Run the parser on the full input and return the resulting document.
    fn parse_document(&mut self) -> PResult<Document<'a>> {
        let mut statements = Vec::new();
        while self.cursor < self.tokens.len() {
            statements.push(self.parse_statement()?);
        }

        Ok(Document(statements))
    }

    fn parse_statement(&mut self) -> PResult<Stmt<'a>> {
        debug_assert!(self.cursor < self.tokens.len());

        match self.tokens[self.cursor].0 {
            Token::KwImport => self.parse_import().map(Stmt::Import),
            Token::Ident(..) => self.parse_assign().map(Stmt::Assign),
            Token::KwReturn => self.parse_return().map(Stmt::Return),
            Token::LBrace => self.parse_block().map(Stmt::Block),
            Token::KwPut => self.parse_put().map(Stmt::Put),
            _ => {
                let msg = "Parse error in statement: expected import, return, \
                           assignment, block, or put.";
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
        debug_assert!(self.peek() == Some(Token::KwReturn));

        // Step over the 'return' keyword.
        self.consume();

        self.parse_expr().map(Return)
    }

    fn parse_block(&mut self) -> PResult<Block<'a>> {
        debug_assert!(self.peek() == Some(Token::LBrace));

        // Step over the opening brace.
        self.consume();

        let mut statements = Vec::new();

        loop {
            // A closing brace marks the end of the block.
            match self.peek() {
                Some(Token::RBrace) => {
                    self.consume();
                    break
                }
                None => {
                    // TODO: Make the parencheck deal with this.
                    let msg = "Parse error in block: expected closing '}'.";
                    return self.error(msg)
                }
                Some(..) => {}
            }

            // Otherwise we expect a statement.
            statements.push(self.parse_statement()?);

            // Unlike idents, there are no separators for statements.
        }

        Ok(Block(statements))
    }

    fn parse_put(&mut self) -> PResult<Put<'a>> {
        debug_assert!(self.peek() == Some(Token::KwPut));

        // Step over the 'put' keyword.
        self.consume();

        self.parse_expr().map(Put)
    }

    fn parse_expr(&mut self) -> PResult<Term<'a>> {
        // Note: `parse_expr` is just a synonym for readability. There are
        // multiple levels of expressions to handle precedence.
        self.parse_expr_infix()
    }

    fn parse_expr_infix(&mut self) -> PResult<Term<'a>> {
        let mut term = self.parse_expr_add()?;

        loop {
            // The term so far could be it, or it could be the left-hand side
            // of an infix call expression. If we find an identifier next, it
            // might be an infix call. NOTE: This is the only place in the
            // parser where we need more than one token lookahead.
            match (self.peek(), self.peek_next()) {
                // If there is a '=' after the identifier, then the identifier
                // is not an infix call, but the target of an assignment.
                (Some(Token::Ident(..)), Some(Token::Equals)) => break,
                (Some(Token::Ident(..)), _) => {
                    // If there is no '=', then we expect identifiers for the
                    // function to call.
                    let infix = self.parse_idents().map(BinOp::Infix)?;
                    let rhs = self.parse_expr_add()?;
                    term = Term::bin_op(BinTerm(term, infix, rhs));
                }
                _ => break,
            }
        }
        Ok(term)
    }

    fn parse_expr_add(&mut self) -> PResult<Term<'a>> {
        let mut term = self.parse_expr_mul()?;

        loop {
            // The term so far could be it, or it could be part of a bigger
            // binary expression, if we encounter the right operator next.
            let maybe_op = match self.peek() {
                Some(Token::Plus) => Some(BinOp::Add),
                Some(Token::Minus) => Some(BinOp::Sub),
                Some(Token::Tilde) => Some(BinOp::Adj),
                _ => None,
            };

            if let Some(op) = maybe_op {
                self.consume();
                let rhs = self.parse_expr_mul()?;
                term = Term::bin_op(BinTerm(term, op, rhs));
            } else {
                return Ok(term)
            }
        }
    }

    fn parse_expr_mul(&mut self) -> PResult<Term<'a>> {
        let mut term = self.parse_expr_exp()?;

        loop {
            // The term so far could be it, or it could be part of a bigger
            // binary expression, if we encounter the right operator next.
            let maybe_op = match self.peek() {
                Some(Token::Star) => Some(BinOp::Mul),
                Some(Token::Slash) => Some(BinOp::Div),
                _ => None,
            };

            if let Some(op) = maybe_op {
                self.consume();
                let rhs = self.parse_expr_exp()?;
                term = Term::bin_op(BinTerm(term, op, rhs));
            } else {
                return Ok(term)
            }
        }
    }

    fn parse_expr_exp(&mut self) -> PResult<Term<'a>> {
        // Detect unary operators first, and handle them immediately.
        match self.peek() {
            // For now there is only minus. Might have ! in the future.
            Some(Token::Minus) => return self.parse_unop().map(Term::un_op),
            _ => {}
        }

        // If we are not dealing with a unary operator, then it is either a bare
        // term, exponentiation, or a function call.
        let term = self.parse_term()?;

        // Maybe we were dealing with exponentiation or a function call. If not,
        // just return the term.
        match self.peek() {
            Some(Token::Hat) => {
                self.consume();
                let rhs = self.parse_term()?;
                Ok(Term::bin_op(BinTerm(term, BinOp::Exp, rhs)))
            }
            Some(Token::LParen) => {
                let args = self.parse_fn_call_args()?;
                Ok(Term::fn_call(FnCall(term, args)))
            }
            _ => Ok(term)
        }
    }

    fn parse_fn_call_args(&mut self) -> PResult<Vec<Term<'a>>> {
        debug_assert!(self.peek() == Some(Token::LParen));

        // Step over the opening parenthesis.
        self.consume();

        // If there are no arguments, return immediately.
        if self.peek() == Some(Token::RParen) {
            self.consume();
            return Ok(Vec::new())
        }

        // Otherwise, parse one or more expressions separated by commas.
        let mut args = Vec::new();

        loop {
            args.push(self.parse_expr()?);

            match self.peek() {
                Some(Token::Comma) => {
                    self.consume();
                    continue
                }
                Some(Token::RParen) => {
                    self.consume();
                    break
                }
                _ => {
                    let msg = "Parse error in function call: expected ',' or ')'.";
                    return self.error(msg)
                }
            }
        }

        Ok(args)
    }

    fn parse_unop(&mut self) -> PResult<UnTerm<'a>> {
        let op = match self.take() {
            Some(Token::Minus) => UnOp::Neg,
            _ => unreachable!("parse_unop must be called with cursor on unop."),
        };

        // Note that what follows is not an arbitrary expression, but a term.
        // This ensures that unary operators bind most closely.
        let term = self.parse_term()?;

        Ok(UnTerm(op, term))
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
            Token::LBracket => self.parse_list().map(Term::List),
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

        // Short-circuit if there are no args, just the empty args list.
        if self.peek() == Some(Token::RParen) {
            self.consume();
            return Ok(args)
        }

        // Take one identifier. If it is followed by a comma, repeat. If we find
        // a closing paren instead, we are done.
        loop {
            match self.peek() {
                Some(Token::Ident(ident)) => args.push(ident),
                _ => return self.error("Parse error in function definition: expected argument name or ')'."),
            }

            self.consume();

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

        Ok(Term::coord(Coord(expr_x, expr_y)))
    }

    fn parse_list(&mut self) -> PResult<List<'a>> {
        debug_assert!(self.peek() == Some(Token::LBracket));

        // Step over the opening bracket.
        self.consume();

        let mut elements = Vec::new();

        loop {
            // If we find a closing bracket, then we are done here.
            if let Some(Token::RBracket) = self.peek() {
                self.consume();
                return Ok(List(elements));
            }

            // Otherwise, there must be an expression for the next element.
            elements.push(self.parse_expr()?);

            // After the element, either we immediately close the list with a
            // bracket, or there is a separator and more may follow.
            match self.peek() {
                Some(Token::RBracket) => {
                    self.consume();
                    return Ok(List(elements));
                }
                Some(Token::Semicolon) => {
                    self.consume();
                    continue;
                }
                Some(Token::Comma) => {
                    // Trying to use a comma as list separator is likely going
                    // to be a common mistake, as it is common from other
                    // languages. (Perhaps I should use comma after all for that
                    // reason.) Add a hint about that.
                    let msg = "Parse error in list: expected ';' or ']'. \
                               Note: use ';' to separate elements.";
                    return self.error(msg)
               }
                _ => {
                    let msg = "Parse error in list: expected ';' or ']'.";
                    return self.error(msg)
                }
            }
        }
    }


    /// Return the token under the cursor, if there is one.
    fn peek(&self) -> Option<Token<'a>> {
        self.tokens.get(self.cursor).map(|t| t.0)
    }

    /// Return the token after the cursor, if there is one.
    fn peek_next(&self) -> Option<Token<'a>> {
        self.tokens.get(self.cursor + 1).map(|t| t.0)
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
            Some(token.0)
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

#[cfg(test)]
mod test {
    use parser::Parser;
    use lexer::lex;
    use ast::{Assign, BinOp, BinTerm, Coord, Color, FnCall};
    use ast::{Idents, List, Num, Put, Stmt, Term, UnOp, UnTerm, Unit};

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
        assert_preq!(lit, Term::String(String::from("foo")));
        assert_eq!(parser.cursor, 1);
    }

    #[test]
    fn parse_parses_raw_string_literal() {
        let tokens = lex(b"  ---\n  foo\n  --- appendix").unwrap();
        let mut parser = Parser::new(&tokens);
        let lit = parser.parse_term().unwrap();
        assert_preq!(lit, Term::String(String::from("foo")));
        assert_eq!(parser.cursor, 1);
    }

    #[test]
    fn parse_parses_color() {
        let tokens = lex(b"#c0ffee").unwrap();
        let mut parser = Parser::new(&tokens);
        let color = parser.parse_term().unwrap();
        assert_preq!(color, Term::Color(Color(0xc0, 0xff, 0xee)));
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
        assert_preq!(lit, Term::Number(Num(31.0, None)));
        assert_eq!(parser.cursor, 1);
    }

    #[test]
    fn parse_parses_unitful_number_literal() {
        let tokens = lex(b"0.5em").unwrap();
        let mut parser = Parser::new(&tokens);
        let lit = parser.parse_term().unwrap();
        assert_preq!(lit, Term::Number(Num(0.5, Some(Unit::Em))));
        assert_eq!(parser.cursor, 2);
    }

    #[test]
    fn parse_parses_number_then_eof() {
        let tokens = lex(b"17").unwrap();
        let mut parser = Parser::new(&tokens);
        let lit = parser.parse_term().unwrap();
        assert_preq!(lit, Term::Number(Num(17.0, None)));
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
    fn parse_fails_fn_def_args_trailing_comma() {
        let tokens = lex(b"(a,)").unwrap();
        let mut parser = Parser::new(&tokens);
        let err = parser.parse_fn_def_args().err().unwrap();
        assert_eq!(err.token_index, 3);
    }

    #[test]
    fn parse_parses_fn_def_empty() {
        let tokens = lex(b"function() {}").unwrap();
        let mut parser = Parser::new(&tokens);
        let fn_def = parser.parse_fn_def().unwrap();
        assert_eq!(fn_def.0.len(), 0); // Zero arguments.
        assert_eq!((fn_def.1).0.len(), 0); // Zero statements.
        assert_eq!(parser.cursor, 5);
    }

    #[test]
    fn parse_parses_coord() {
        let tokens = lex(b"(1, 2)").unwrap();
        let mut parser = Parser::new(&tokens);
        let coord = parser.parse_term().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        assert_preq!(coord, Term::coord(Coord(one, two)));
        assert_eq!(parser.cursor, 5);
    }

    #[test]
    fn parse_parses_expr_in_parens() {
        let tokens = lex(b"(1)").unwrap();
        let mut parser = Parser::new(&tokens);
        let term = parser.parse_term().unwrap();
        let one = Term::Number(Num(1.0, None));
        assert_preq!(term, one);
        assert_eq!(parser.cursor, 3);
    }

    #[test]
    fn parse_parses_put_at() {
        let tokens = lex(b"put 1 at 2").unwrap();
        let mut parser = Parser::new(&tokens);
        let put = parser.parse_put().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        let at = BinTerm(one, BinOp::Infix(Idents(vec!["at"])), two);
        assert_preq!(put.0, Term::bin_op(at));
        assert_eq!(parser.cursor, 4);
    }

    #[test]
    fn does_not_parse_at_put() {
        let tokens = lex(b"at 1 put 2").unwrap();
        let mut parser = Parser::new(&tokens);
        let result = parser.parse_statement();
        assert_eq!(result.err().unwrap().token_index, 1);
    }

    #[test]
    fn does_not_parse_beyond_ident_then_space() {
        let tokens = lex(b"at 1 place 2").unwrap();
        let mut parser = Parser::new(&tokens);
        let result = parser.parse_expr().unwrap();
        assert_preq!(result, Term::Idents(Idents(vec!["at"])));
        assert_eq!(parser.cursor, 1);
    }

    #[test]
    fn parse_parses_infix_call_inside_statement() {
        let tokens = lex(b"put 1 on 2").unwrap();
        let mut parser = Parser::new(&tokens);
        let result = parser.parse_statement();
        match result {
          // Note, we can't match on the number here, that would break under a
          // future Rust release. https://github.com/rust-lang/rust/issues/41620
          Ok(Stmt::Put(Put(Term::BinOp(binterm)))) => {
              let one = Term::Number(Num(1.0, None));
              let two = Term::Number(Num(2.0, None));
              assert_preq!(binterm.0, one);
              assert_preq!(binterm.1, BinOp::Infix(Idents(vec!["on"])));
              assert_preq!(binterm.2, two);
          }
          _ => panic!("Unexpected parse result."),
        }
        assert_eq!(parser.cursor, 4); // Stopped at "on".
    }

    #[test]
    fn parse_parses_unop_neg() {
        let tokens = lex(b"-1").unwrap();
        let mut parser = Parser::new(&tokens);
        let unterm = parser.parse_unop().unwrap();
        let one = Term::Number(Num(1.0, None));
        assert_preq!(unterm.0, UnOp::Neg);
        assert_preq!(unterm.1, one);
        assert_eq!(parser.cursor, 2);
    }

    #[test]
    fn parse_parses_binop_exp() {
        let tokens = lex(b"1 ^ 2").unwrap();
        let mut parser = Parser::new(&tokens);
        let exp = parser.parse_expr_exp().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        let bt = BinTerm(one, BinOp::Exp, two);
        assert_preq!(exp, Term::bin_op(bt));
        assert_eq!(parser.cursor, 3);
    }

    #[test]
    fn parse_parses_binop_infix() {
        let tokens = lex(b"1 at 2").unwrap();
        let mut parser = Parser::new(&tokens);
        let exp = parser.parse_expr().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        let bt = BinTerm(one, BinOp::Infix(Idents(vec!["at"])), two);
        assert_preq!(exp, Term::bin_op(bt));
        assert_eq!(parser.cursor, 3);
    }

    #[test]
    fn parse_parses_fn_call() {
        let tokens = lex(b"1(2, 6)").unwrap();
        let mut parser = Parser::new(&tokens);
        let exp = parser.parse_expr_exp().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        let six = Term::Number(Num(6.0, None));
        let fc = FnCall(one, vec![two, six]);
        assert_preq!(exp, Term::fn_call(fc));
        assert_eq!(parser.cursor, 6);
    }

    #[test]
    fn parse_parses_single_exp() {
        let tokens = lex(b"1").unwrap();
        let mut parser = Parser::new(&tokens);
        let exp = parser.parse_expr_exp().unwrap();
        let one = Term::Number(Num(1.0, None));
        assert_preq!(exp, one);
        assert_eq!(parser.cursor, 1);
    }

    #[test]
    fn parse_parses_binop_mul() {
        let tokens = lex(b"1 * 2").unwrap();
        let mut parser = Parser::new(&tokens);
        let mul = parser.parse_expr_mul().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        let bt = BinTerm(one, BinOp::Mul, two);
        assert_preq!(mul, Term::bin_op(bt));
        assert_eq!(parser.cursor, 3);
    }

    #[test]
    fn parse_parses_binop_div() {
        let tokens = lex(b"1 / 2").unwrap();
        let mut parser = Parser::new(&tokens);
        let mul = parser.parse_expr_mul().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        let bt = BinTerm(one, BinOp::Div, two);
        assert_preq!(mul, Term::bin_op(bt));
        assert_eq!(parser.cursor, 3);
    }

    #[test]
    fn parse_parses_binop_mul_mixed_precedence() {
        let tokens = lex(b"1^6 * -2").unwrap();
        let mut parser = Parser::new(&tokens);
        let result = parser.parse_expr_mul().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        let six = Term::Number(Num(6.0, None));
        let lhs = Term::bin_op(BinTerm(one, BinOp::Exp, six));
        let rhs = Term::un_op(UnTerm(UnOp::Neg, two));
        let expected = Term::bin_op(BinTerm(lhs, BinOp::Mul, rhs));
        assert_preq!(result, expected);
        assert_eq!(parser.cursor, 6);
    }

    #[test]
    fn parse_parses_binop_single_mul() {
        let tokens = lex(b"1").unwrap();
        let mut parser = Parser::new(&tokens);
        let mul = parser.parse_expr_mul().unwrap();
        let one = Term::Number(Num(1.0, None));
        assert_preq!(mul, one);
        assert_eq!(parser.cursor, 1);
    }

    #[test]
    fn parse_parses_binop_add() {
        let tokens = lex(b"1 + 2").unwrap();
        let mut parser = Parser::new(&tokens);
        let add = parser.parse_expr_add().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        let bt = BinTerm(one, BinOp::Add, two);
        assert_preq!(add, Term::bin_op(bt));
        assert_eq!(parser.cursor, 3);
    }

    #[test]
    fn parse_parses_binop_sub() {
        let tokens = lex(b"1 - 2").unwrap();
        let mut parser = Parser::new(&tokens);
        let add = parser.parse_expr_add().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        let bt = BinTerm(one, BinOp::Sub, two);
        assert_preq!(add, Term::bin_op(bt));
        assert_eq!(parser.cursor, 3);
    }

    #[test]
    fn parse_parses_binop_adj() {
        let tokens = lex(b"1 ~ 2").unwrap();
        let mut parser = Parser::new(&tokens);
        let add = parser.parse_expr_add().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        let bt = BinTerm(one, BinOp::Adj, two);
        assert_preq!(add, Term::bin_op(bt));
        assert_eq!(parser.cursor, 3);
    }

    #[test]
    fn parse_binop_add_associates_left() {
        let tokens = lex(b"1 + 2 + 6").unwrap();
        let mut parser = Parser::new(&tokens);
        let add = parser.parse_expr_add().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        let six = Term::Number(Num(6.0, None));
        let inner = BinTerm(one, BinOp::Add, two);
        let outer = BinTerm(Term::bin_op(inner), BinOp::Add, six);
        assert_preq!(add, Term::bin_op(outer));
        assert_eq!(parser.cursor, 5);
    }

    #[test]
    fn parse_binop_mul_associates_left() {
        let tokens = lex(b"1 * 2 * 6").unwrap();
        let mut parser = Parser::new(&tokens);
        let add = parser.parse_expr_mul().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        let six = Term::Number(Num(6.0, None));
        let inner = BinTerm(one, BinOp::Mul, two);
        let outer = BinTerm(Term::bin_op(inner), BinOp::Mul, six);
        assert_preq!(add, Term::bin_op(outer));
        assert_eq!(parser.cursor, 5);
    }

    #[test]
    fn parse_parses_binop_single_add() {
        let tokens = lex(b"1").unwrap();
        let mut parser = Parser::new(&tokens);
        let add = parser.parse_expr_add().unwrap();
        let one = Term::Number(Num(1.0, None));
        assert_preq!(add, one);
        assert_eq!(parser.cursor, 1);
    }

    #[test]
    fn parse_parses_binop_add_mixed_precedence() {
        let tokens = lex(b"1 * 2 + 6 / 10").unwrap();
        let mut parser = Parser::new(&tokens);
        let result = parser.parse_expr_add().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        let six = Term::Number(Num(6.0, None));
        let ten = Term::Number(Num(10.0, None));
        let lhs = Term::bin_op(BinTerm(one, BinOp::Mul, two));
        let rhs = Term::bin_op(BinTerm(six, BinOp::Div, ten));
        let expected = Term::bin_op(BinTerm(lhs, BinOp::Add, rhs));
        assert_preq!(result, expected);
        assert_eq!(parser.cursor, 7);
    }

    #[test]
    fn parse_parses_block_empty() {
        let tokens = lex(b"{}").unwrap();
        let mut parser = Parser::new(&tokens);
        let block = parser.parse_block().unwrap();
        assert_eq!(block.0.len(), 0);
        assert_eq!(parser.cursor, 2);
    }

    #[test]
    fn parse_parses_block_single_statement() {
        let tokens = lex(b"{ x = 1 }").unwrap();
        let mut parser = Parser::new(&tokens);
        let block = parser.parse_block().unwrap();
        let one = Term::Number(Num(1.0, None));
        assert_eq!(block.0.len(), 1);
        assert_preq!(block.0[0], Stmt::Assign(Assign("x", one)));
        assert_eq!(parser.cursor, 5);
    }

    #[test]
    fn parse_parses_block_double_statement() {
        let tokens = lex(b"{ x = 1 y = 2 }").unwrap();
        let mut parser = Parser::new(&tokens);
        let block = parser.parse_block().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        assert_eq!(block.0.len(), 2);
        assert_preq!(block.0[0], Stmt::Assign(Assign("x", one)));
        assert_preq!(block.0[1], Stmt::Assign(Assign("y", two)));
        assert_eq!(parser.cursor, 8);
    }

    #[test]
    fn parse_block_can_separate_statements() {
        // In this case the most greedy expression is '1 * y',
        // and the parse error occurs at the unexpected '=' token.
        let tokens = lex(b"{ x = 1 * y = 2 }").unwrap();
        let mut parser = Parser::new(&tokens);
        let result = parser.parse_block();
        assert_eq!(result.err().unwrap().token_index, 6);
    }

    #[test]
    fn parse_block_requires_closing_brace() {
        let tokens = lex(b"{ x = 1 ").unwrap();
        let mut parser = Parser::new(&tokens);
        let result = parser.parse_block();
        assert_eq!(result.err().unwrap().token_index, 4);
    }

    #[test]
    fn parse_parses_document_double_statement() {
        let tokens = lex(b"x = 1 y = 2").unwrap();
        let mut parser = Parser::new(&tokens);
        let doc = parser.parse_document().unwrap();
        let one = Term::Number(Num(1.0, None));
        let two = Term::Number(Num(2.0, None));
        assert_eq!(doc.0.len(), 2);
        assert_preq!(doc.0[0], Stmt::Assign(Assign("x", one)));
        assert_preq!(doc.0[1], Stmt::Assign(Assign("y", two)));
        assert_eq!(parser.cursor, 6);
    }

    #[test]
    fn parse_parses_fn_call_args_empty() {
        let tokens = lex(b"()").unwrap();
        let mut parser = Parser::new(&tokens);
        let args = parser.parse_fn_call_args().unwrap();
        assert_eq!(args.len(), 0);
        assert_eq!(parser.cursor, 2);
    }

    #[test]
    fn parse_parses_fn_call_args_single() {
        let tokens = lex(b"(1)").unwrap();
        let mut parser = Parser::new(&tokens);
        let args = parser.parse_fn_call_args().unwrap();
        assert_eq!(args.len(), 1);
        assert_preq!(args[0], Term::Number(Num(1.0, None)));
        assert_eq!(parser.cursor, 3);
    }

    #[test]
    fn parse_parses_fn_call_args_double() {
        let tokens = lex(b"(1, 2)").unwrap();
        let mut parser = Parser::new(&tokens);
        let args = parser.parse_fn_call_args().unwrap();
        assert_eq!(args.len(), 2);
        assert_preq!(args[0], Term::Number(Num(1.0, None)));
        assert_preq!(args[1], Term::Number(Num(2.0, None)));
        assert_eq!(parser.cursor, 5);
    }
}
