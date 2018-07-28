// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

//! This module contains the Pris lexer.
//!
//! The lexer turns the input into a stream of tokens. It strips comments that
//! run to the end of the line, and it removes whitespace. The lexer is hand-
//! written for a few reasons:
//!
//!  * It can produce helpful error messages in this way.
//!  * It can support non-greedy triple quoted strings that cannot be expressed
//!    as regex without support for non-greedy matching.
//!  * The lexer was originally written to feed tokens into Lalrpop, because the
//!    lexer included with Lalrpop cannot handle comments that span to the end
//!    of the line. Today this lexer feeds the hand-written parser.
//!
//! The lexer is a state machine with only a few states (the `State` enum). To
//! avoid an explosion of states, the handler for every state can do a bit of
//! finite lookahead. This ensures that e.g. the start of a comment can be
//! detected fully from the base state. There is no need to make '/' switch to
//! an "after slash" state which would then go to comment, or back to the base
//! state. Instead, when a '/' is encountered in the base state, it will look
//! ahead one character. If there is another slash, switch to the comment state.
//! If there is not, handle the slash as is.

use error::{Error, Result};

/// Represents a contiguous region of source code.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Span {
    /// Index of the first byte.
    pub start: usize,
    /// Index after the last byte.
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Span {
        Span {
            start: start,
            end: end,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Token<'a> {
    // TODO: These should not contain slices, that information is redundant and
    // makes the variants unnecessarily big. Instead, make the parser extract
    // this from the token boundaries and the original source.
    String(&'a str),
    RawString(&'a str),
    Color(&'a str),
    Number(&'a str),
    Ident(&'a str),

    KwAt,
    KwFunction,
    KwImport,
    KwPut,
    KwReturn,

    UnitEm,
    UnitH,
    UnitW,
    UnitPt,

    Comma,
    Dot,
    Equals,
    Hat,
    Minus,
    Plus,
    Slash,
    Star,
    Tilde,

    LParen,
    RParen,
    LBrace,
    RBrace,
}

/// Lexes a UTF-8 input file into tokens with source location.
pub fn lex(input: &[u8]) -> Result<Vec<(Token, Span)>> {
    Lexer::new(input).run()
}

#[derive(Debug, Eq, PartialEq)]
enum State {
    Base,
    Done,
    InColor,
    InComment,
    InIdent,
    InNumber,
    InRawString,
    InString,
    Space,
}

struct Lexer<'a> {
    input: &'a [u8],
    start: usize,
    state: State,
    tokens: Vec<(Token<'a>, Span)>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a [u8]) -> Lexer<'a> {
        Lexer {
            input: input,
            start: 0,
            state: State::Base,
            tokens: Vec::new(),
        }
    }

    /// Run the lexer on the full input and return the tokens.
    fn run(mut self) -> Result<Vec<(Token<'a>, Span)>> {
        loop {
            let (start, state) = match self.state {
                State::Base => self.lex_base()?,
                State::InColor => self.lex_color()?,
                State::InComment => self.lex_comment()?,
                State::InIdent => self.lex_ident()?,
                State::InNumber => self.lex_number()?,
                State::InRawString => self.lex_raw_string()?,
                State::InString => self.lex_string()?,
                State::Space => self.lex_space()?,
                State::Done => break,
            };

            debug_assert!(start >= self.start || state == State::Done,
                          "Lexer cursor decrement from {} to {} after {:?} -> {:?}.",
                          self.start, start, self.state, state);

            self.start = start;
            self.state = state;
        }

        Ok(self.tokens)
    }

    /// Check whether the byte sequence occurs at an index.
    fn has_at(&self, at: usize, expected: &[u8]) -> bool {
        // There must at least be sufficient bytes left to match the entire
        // expected string.
        if at + expected.len() > self.input.len() {
            return false
        }

        // Then check that every byte matches.
        for (a, e) in self.input[at..].iter().zip(expected) {
            if a != e {
                return false
            }
        }

        true
    }

    /// Push a token where `start` is the index of its first byte and `end` the
    /// index past the last one.
    fn push_from(&mut self, start: usize, token: Token<'a>, end: usize) {
        let span = Span {
            start: start,
            end: end,
        };
        self.tokens.push((token, span));
    }


    /// Push a token starting at `self.start`.
    fn push(&mut self, token: Token<'a>, end: usize) {
        let start = self.start;
        self.push_from(start, token, end);
    }

    /// Push a single-byte token, and set the start of the next token past it.
    fn push_single(&mut self, at: usize, token: Token<'a>) {
        self.push_from(at, token, at + 1);
        self.start = at + 1;
    }

    /// Lex in the base state until a state change occurs.
    ///
    /// Returns new values for `self.start` and `self.state`.
    fn lex_base(&mut self) -> Result<(usize, State)> {
        for i in self.start..self.input.len() {
            match self.input[i] {
                // There are two characters that require a brief lookahead:
                // * '/', to find the start of a comment "//".
                // * '-', to find the start of a raw string "---".
                // If the lookahead does not match, these characters are matched
                // again as single-character tokens further below.
                b'/' if self.has_at(i + 1, b"/") => {
                    return change_state(i, State::InComment)
                }
                b'-' if self.has_at(i + 1, b"--") => {
                    return change_state(i, State::InRawString)
                }

                // A few characters signal a change of state immediately. Note
                // that only spaces and newlines are considered whitespace.
                // No tabs or carriage returns please.
                b'"' => {
                    return change_state(i, State::InString)
                }
                b' ' | b'\n' => {
                    return change_state(i, State::Space)
                }
                b'#' => {
                    return change_state(i, State::InColor)
                }
                byte if is_alphabetic_or_underscore(byte) => {
                    return change_state(i, State::InIdent)
                }
                byte if is_digit(byte) => {
                    return change_state(i, State::InNumber)
                }

                // A number of punctuation characters are tokens themselves. For
                // these we push a single-byte token and continue after without
                // changing state. Pushing a single token does reset the start
                // counter.
                b',' => self.push_single(i, Token::Comma),
                b'.' => self.push_single(i, Token::Dot),
                b'=' => self.push_single(i, Token::Equals),
                b'^' => self.push_single(i, Token::Hat),
                b'-' => self.push_single(i, Token::Minus),
                b'+' => self.push_single(i, Token::Plus),
                b'/' => self.push_single(i, Token::Slash),
                b'*' => self.push_single(i, Token::Star),
                b'~' => self.push_single(i, Token::Tilde),
                b'(' => self.push_single(i, Token::LParen),
                b')' => self.push_single(i, Token::RParen),
                b'{' => self.push_single(i, Token::LBrace),
                b'}' => self.push_single(i, Token::RBrace),

                // If we detect the start of a byte order mark, complain about a
                // wrong encoding. (No BOMs for UTF-8 either, please.)
                0xef | 0xfe | 0xff | 0x00 => {
                    return Err(make_encoding_error(i, &self.input[i..]))
                }

                // Anything else is invalid. Please, no tabs or carriage
                // returns. And *definitely* no levitating men in business
                // suits. (Note that all of those are fine in comments and
                // strings, so you can still document everything in a non-Latin
                // language, or make slides for that. Just keep the source clean
                // please.)
                _ => return Err(make_parse_error(i, &self.input[i..])),
            }
        }

        done_at_end_of_input()
    }

    /// Lex in the color state until a state change occurs.
    fn lex_color(&mut self) -> Result<(usize, State)> {
        debug_assert!(self.has_at(self.start, b"#"));

        // Skip over the first '#' byte.
        for i in self.start + 1..self.input.len() {
            let c = self.input[i];

            // A hexadecimal character, as expected.
            if i < self.start + 7 && is_hexadecimal(c) {
                continue
            }

            // We expected more hexadecimal digits, but found something else.
            if i < self.start + 7 {
                let msg = format!("Expected hexadecimal digit, found '{}'.", char::from(c));
                return Err(Error::parse(self.start, i + 1, msg))
            }

            // We expect at most 6 hexadecimal digits, but if another
            // alphanumeric character comes after this, we don't want to
            // terminate the color and switch to identifier; that would lead to
            // very confusing parse errors later on. Report an error here
            // instead.
            if i == self.start + 7 && is_hexadecimal(c) {
                let msg = "Expected only six hexadecimal digits, found one more.";
                return Err(Error::parse(self.start, i + 1, msg.into()))
            }
            if i == self.start + 7 && is_alphanumeric_or_underscore(c) {
                let msg = format!("Expected six hexadecimal digits, found extra '{}'.", char::from(c));
                return Err(Error::parse(self.start, i + 1, msg))
            }

            // The end of the color in a non-hexadecimal character, as expected.
            // Re-inspect the current character from the base state.
            if i == self.start + 7 && !is_hexadecimal(c) {
                // TODO: The new parser does not need the tokens to have content
                // if the source available to the parser. (Which it needs to be
                // anyway to generate errors.)
                let inner = self.parse_utf8_str(self.start, i).unwrap();
                self.push(Token::Color(inner), i);
                return change_state(i, State::Base)
            }

            assert!(i < self.start + 7, "Would enter infinite loop when lexing color.");
        }

        if self.start + 7 == self.input.len() {
            // The input ends in a color.
            let inner = self.parse_utf8_str(self.start, self.input.len()).unwrap();
            self.push(Token::Color(inner), self.input.len());
            done_at_end_of_input()
        } else {
            // The input ends in a color, but we were still expecting digits.
            let msg = "Expected six hexadecimal digits, but input ended.";
            Err(Error::parse(self.start, self.input.len(), msg.into()))
        }
    }

    /// Skip until a newline is found, then switch to the whitespace state.
    fn lex_comment(&mut self) -> Result<(usize, State)> {
        debug_assert!(self.has_at(self.start, b"//"));

        // Skip the first two bytes, those are the "//" characters.
        for i in self.start + 2..self.input.len() {
            if self.input[i] == b'\n' {
                // Change to the whitespace state, because the last character
                // we saw was whitespace after all. Continue immediately at
                // the next byte (i + 1), there is no need to re-inspect the
                // newline.
                return change_state(i + 1, State::Space)
            }
        }

        done_at_end_of_input()
    }

    /// Lex an identifier untl a state change occurs.
    fn lex_ident(&mut self) -> Result<(usize, State)> {
        debug_assert!(is_alphabetic_or_underscore(self.input[self.start]));

        // Skip the first byte, because we already know that it contains
        // either an alphabetic character or underscore. For the other
        // characters, digits are allowed too.
        for i in self.start + 1..self.input.len() {
            if !is_alphanumeric_or_underscore(self.input[i]) {
                // An identifier consists of alphanumeric characters or
                // underscores, so at the first one that is not one of those,
                // change to the base state and re-inspect it.
                let inner = self.parse_utf8_str(self.start, i).unwrap();
                self.push(make_keyword_or_ident(inner), i);
                return change_state(i, State::Base)
            }
        }

        // The input ended in an identifier.
        let inner = self.parse_utf8_str(self.start, self.input.len()).unwrap();
        self.push(make_keyword_or_ident(inner), self.input.len());
        done_at_end_of_input()
    }

    /// Lex in the number state until a state change occurs.
    fn lex_number(&mut self) -> Result<(usize, State)> {
        debug_assert!(is_digit(self.input[self.start]));

        let mut period_seen = false;

        // Skip over the first digit, as we know already that it is a digit.
        for i in self.start + 1..self.input.len() {
            match self.input[i] {
                c if is_digit(c) => {
                    continue
                }
                b'.' if !period_seen => {
                    // Allow a single decimal period in the number.
                    period_seen = true
                    // TODO: Enforce that the next byte is a digit; numbers
                    // should not end in a period. (Just for style). But the
                    // lexer is simpler if this is allowed.
                }
                // For the various unit suffixes, we emit a separate token,
                // after emitting the number token. Then switch to the base
                // state and continue after the suffix.
                b'e' if self.has_at(i + 1, b"m") => {
                    let inner = self.parse_utf8_str(self.start, i).unwrap();
                    self.push(Token::Number(inner), i);
                    self.push_from(i, Token::UnitEm, i + 2);
                    return change_state(i + 2, State::Base)
                }
                b'p' if self.has_at(i + 1, b"t") => {
                    let inner = self.parse_utf8_str(self.start, i).unwrap();
                    self.push(Token::Number(inner), i);
                    self.push_from(i, Token::UnitPt, i + 2);
                    return change_state(i + 2, State::Base)
                }
                b'h' => {
                    let inner = self.parse_utf8_str(self.start, i).unwrap();
                    self.push(Token::Number(inner), i);
                    self.push_single(i, Token::UnitH);
                    return change_state(i + 1, State::Base)
                }
                b'w' => {
                    let inner = self.parse_utf8_str(self.start, i).unwrap();
                    self.push(Token::Number(inner), i);
                    self.push_single(i, Token::UnitW);
                    return change_state(i + 1, State::Base)
                }
                _ => {
                    // Not a digit or first period, re-inspect this byte in the
                    // base state.
                    let inner = self.parse_utf8_str(self.start, i).unwrap();
                    self.push(Token::Number(inner), i);
                    return change_state(i, State::Base)
                }
            }
        }

        // The input ended in a number.
        let inner = self.parse_utf8_str(self.start, self.input.len()).unwrap();
        self.push(Token::Number(inner), self.input.len());
        done_at_end_of_input()
    }

    /// Lex in the raw string state until a "---" is found.
    fn lex_raw_string(&mut self) -> Result<(usize, State)> {
        debug_assert!(self.has_at(self.start, b"---"));

        // Skip over the first "---" that starts the literal.
        for i in self.start + 3..self.input.len() {
            match self.input[i] {
                b'-' if self.has_at(i + 1, b"--") => {
                    // Another "---" marks the end of the raw string. Continue
                    // in the base state after the last dash.
                    let inner = self.parse_utf8_str(self.start, i + 3)?;
                    self.push(Token::RawString(inner), i + 3);
                    return change_state(i + 3, State::Base)
                }
                _ => continue,
            }
        }

        // If we reach end of input inside a raw string, that's an error.
        let msg = "Raw string was not closed with '---' before end of input.";
        Err(Error::parse(self.start, self.start + 3, msg.into()))
    }

    /// Lex in the string state until a closing quote is found.
    fn lex_string(&mut self) -> Result<(usize, State)> {
        debug_assert!(self.has_at(self.start, b"\""));

        // Skip over the first quote that starts the literal.
        let mut skip_next = false;
        for i in self.start + 1..self.input.len() {
            if skip_next {
                skip_next = false;
                continue
            }
            match self.input[i] {
                b'\\' => {
                    // For the lexer, skip over anything after a backslash, even
                    // if it is not a valid escape code. The parser will handle
                    // those.
                    skip_next = true
                }
                b'"' => {
                    let inner = self.parse_utf8_str(self.start, i + 1)?;
                    self.push(Token::String(inner), i + 1);
                    // Continue in the base state after the closing quote.
                    return change_state(i + 1, State::Base)
                }
                _ => continue,
            }
        }

        // If we reach end of input inside a string, that's an error.
        let msg = "String was not closed with '\"' before end of input.";
        Err(Error::parse(self.start, self.start + 1, msg.into()))
    }

    /// Lex in the whitespace state until a state change occurs.
    fn lex_space(&mut self) -> Result<(usize, State)> {
        for i in self.start..self.input.len() {
            match self.input[i] {
                b' ' | b'\n' => {
                    continue
                }
                b'\t' | b'\r' => {
                    // Be very strict about whitespace; report an error for tabs
                    // and carriage returns. `make_parse_error()` generates a
                    // specialized error message for these.
                    return Err(make_parse_error(i, &self.input[i..]))
                }
                _ => {
                    // On anything else we switch back to the base state and
                    // inspect the current byte again in that state.
                    return change_state(i, State::Base)
                }
            }
        }

        done_at_end_of_input()
    }

    /// Extract a string literal as `&str`, or fail.
    fn parse_utf8_str(&self, start: usize, past_end: usize) -> Result<&'a str> {
        use std::str;
        let inner_slice = &self.input[self.start..past_end];
        str::from_utf8(inner_slice).map_err(|e| {
            let msg = "String literal contains invalid UTF-8.".into();
            let off = e.valid_up_to();
            Error::parse(start + off, past_end, msg)
        })
    }
}

/// Make `Lexer::run()` change to a different state, starting at the given byte.
///
/// This is only a helper function to make the lexer code a bit more readable,
/// the logic is in `Lexer::run()`.
fn change_state(at: usize, state: State) -> Result<(usize, State)> {
    Ok((at, state))
}

/// Signal end of input to the `Lexer::run()` method.
///
/// This is only a helper function to make the lexer code a bit more readable,
/// the logic is in `Lexer::run()`.
fn done_at_end_of_input() -> Result<(usize, State)> {
    Ok((0, State::Done))
}

/// Check whether a byte of UTF-8 is an ASCII letter.
fn is_alphabetic(byte: u8) -> bool {
    (b'a' <= byte && byte <= b'z') || (b'A' <= byte && byte <= b'Z')
}

/// Check whether a byte of UTF-8 is an ASCII letter or underscore.
fn is_alphabetic_or_underscore(byte: u8) -> bool {
    is_alphabetic(byte) || (byte == b'_')
}

/// Check whether a byte of UTF-8 is an ASCII letter, digit, or underscore.
fn is_alphanumeric_or_underscore(byte: u8) -> bool {
    is_alphabetic_or_underscore(byte) || (b'0' <= byte && byte <= b'9')
}

/// Check whether a byte of UTF-8 is an ASCII digit.
fn is_digit(byte: u8) -> bool {
    b'0' <= byte && byte <= b'9'
}

/// Check whether a byte of UTF-8 is a hexadecimal character.
fn is_hexadecimal(byte: u8) -> bool {
    is_digit(byte) || (b'a' <= byte && byte <= b'f') || (b'A' <= byte && byte <= b'F')
}

/// Returns either a keyword if one matches, or an identifier token otherwise.
fn make_keyword_or_ident(ident: &str) -> Token {
    match ident {
        "at" => Token::KwAt,
        "function" => Token::KwFunction,
        "import" => Token::KwImport,
        "put" => Token::KwPut,
        "return" => Token::KwReturn,
        _ => Token::Ident(ident),
    }
}

/// Detects a few byte order marks and returns an error
fn make_encoding_error(at: usize, input: &[u8]) -> Error {
    let (message, count) = if input.starts_with(&[0xef, 0xbb, 0xbf]) {
        // There is a special place in hell for people who use byte order marks
        // in UTF-8.
        ("Found UTF-8 byte order mark. Please remove it.", 3)
    } else if input.starts_with(&[0xfe, 0xff]) ||
              input.starts_with(&[0xff, 0xfe]) {
        ("Expected UTF-8 encoded file, but found UTF-16 byte order mark.", 2)
    } else if input.starts_with(&[0x00, 0x00, 0xfe, 0xff]) ||
              input.starts_with(&[0xff, 0xfe, 0x00, 0x00]) {
        ("Expected UTF-8 encoded file, but found UTF-32 byte order mark.", 4)
    } else {
        // If it was not a known byte order mark after all, complain about the
        // character as a normal parse error.
        return make_parse_error(at, input)
    };

    Error::parse(at, at + count, message.into())
}

fn make_parse_error(at: usize, input: &[u8]) -> Error {
    let message = match input[0] {
        b'\t' => {
            "Found tab character. Please use spaces instead.".into()
        }
        b'\r' => {
            "Found carriage return. Please use Unix line endings instead.".into()
        }
        x if x < 0x20 || x == 0x7f => {
            // An ASCII control character. In this case the character is likely
            // not printable as-is, so we include the byte in the message, and
            // an encoding hint.
            format!("Unexpected control character 0x{:x}. ", x) +
            "Note that Pris expects UTF-8 encoded files."
        }
        x if x < 0x7f => {
            // A regular ASCII character, but apparently not one we expected at
            // this place.
            format!("Unexpected character '{}'.", char::from(x))
        }
        x => {
            // If we find a non-ASCII byte, try to decode the next few bytes as
            // UTF-8. If that succeeds, complain about non-ASCII identifiers.
            // Otherwise complain about the encoding. Note that the unwrap will
            // succeed, as we have at least one byte in the input.
            let to = if input.len() < 8 { input.len() } else { 8 };
            match String::from_utf8_lossy(&input[..to]).chars().next().unwrap() {
                '\u{fffd}' => {
                    // U+FFFD is generated when decoding UTF-8 fails.
                    format!("Unexpected byte 0x{:x}. ", x) +
                    "Note that Pris expects UTF-8 encoded files."
                }
                c => {
                    format!("Unexpected character '{}'. ", c) +
                    "Note that identifiers must be ASCII."
                }
            }
        }
    };

    // The end index is not entirely correct for the non-ASCII but valid UTF-8
    // case, but meh.
    Error::parse(at, at + 1, message)
}

#[test]
fn lex_handles_a_simple_input() {
    let input = b"foo bar";
    let tokens = lex(input).unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0], (Token::Ident("foo"), Span::new(0, 3)));
    assert_eq!(tokens[1], (Token::Ident("bar"), Span::new(4, 7)));
}

#[test]
fn lex_handles_a_string_literal() {
    let input = br#"foo "bar""#;
    let tokens = lex(input).unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0], (Token::Ident("foo"), Span::new(0, 3)));
    assert_eq!(tokens[1], (Token::String("\"bar\""), Span::new(4, 9)));
}

#[test]
fn lex_handles_a_string_literal_with_escaped_quote() {
    let input = br#""bar\"baz""#;
    let tokens = lex(input).unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], (Token::String(r#""bar\"baz""#), Span::new(0, 10)));
}

#[test]
fn lex_handles_a_raw_string_literal() {
    let input = b"foo---bar---baz";
    let tokens = lex(input).unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0], (Token::Ident("foo"), Span::new(0, 3)));
    assert_eq!(tokens[1], (Token::RawString("---bar---"), Span::new(3, 12)));
    assert_eq!(tokens[2], (Token::Ident("baz"), Span::new(12, 15)));
}

#[test]
fn lex_strips_a_comment() {
    let input = b"foo\n// This is comment\nbar";
    let tokens = lex(input).unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0], (Token::Ident("foo"), Span::new(0, 3)));
    assert_eq!(tokens[1], (Token::Ident("bar"), Span::new(23, 26)));
}

#[test]
fn lex_handles_a_color() {
    let input = b"#f8f8f8 #cfcfcf";
    let tokens = lex(input).unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0], (Token::Color("#f8f8f8"), Span::new(0, 7)));
    assert_eq!(tokens[1], (Token::Color("#cfcfcf"), Span::new(8, 15)));
}

#[test]
fn lex_handles_numbers() {
    let input = b"31 31.0 2w 2h 2em 2pt 17";
    let tokens = lex(input).unwrap();
    assert_eq!(tokens.len(), 11);
    assert_eq!(tokens[0], (Token::Number("31"), Span::new(0, 2)));
    assert_eq!(tokens[1], (Token::Number("31.0"), Span::new(3, 7)));
    assert_eq!(tokens[2], (Token::Number("2"), Span::new(8, 9)));
    assert_eq!(tokens[3], (Token::UnitW, Span::new(9, 10)));
    assert_eq!(tokens[4], (Token::Number("2"), Span::new(11, 12)));
    assert_eq!(tokens[5], (Token::UnitH, Span::new(12, 13)));
    assert_eq!(tokens[6], (Token::Number("2"), Span::new(14, 15)));
    assert_eq!(tokens[7], (Token::UnitEm, Span::new(15, 17)));
    assert_eq!(tokens[8], (Token::Number("2"), Span::new(18, 19)));
    assert_eq!(tokens[9], (Token::UnitPt, Span::new(19, 21)));
    assert_eq!(tokens[10], (Token::Number("17"), Span::new(22, 24)));
}

#[test]
fn lex_handles_braces() {
    let input = b"{ }\n";
    let tokens = lex(input).unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0], (Token::LBrace, Span::new(0, 1)));
    assert_eq!(tokens[1], (Token::RBrace, Span::new(2, 3)));
}

#[test]
fn lex_handles_keywords() {
    let input = b"return the function put at the import";
    let tokens = lex(input).unwrap();
    assert_eq!(tokens.len(), 7);
    assert_eq!(tokens[0], (Token::KwReturn, Span::new(0, 6)));
    assert_eq!(tokens[1], (Token::Ident("the"), Span::new(7, 10)));
    assert_eq!(tokens[2], (Token::KwFunction, Span::new(11, 19)));
    assert_eq!(tokens[3], (Token::KwPut, Span::new(20, 23)));
    assert_eq!(tokens[4], (Token::KwAt, Span::new(24, 26)));
    assert_eq!(tokens[5], (Token::Ident("the"), Span::new(27, 30)));
    assert_eq!(tokens[6], (Token::KwImport, Span::new(31, 37)));
}

#[test]
fn lex_handles_invalid_utf8() {
    let input = [0x2a, 0xac];
    let tokens = lex(&input);
    assert!(tokens.is_err());
}
