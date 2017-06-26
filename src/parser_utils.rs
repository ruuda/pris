// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

//! This module contains building blocks for the parser. The actual parser can
//! be found in the `parser` module.

use ast;
use lexer::Token;
use std::char;

/// Strips the '---' of a raw string literal and corrects its indentation.
pub fn unescape_raw_string_literal<'a>(literal: &'a str) -> String {
    debug_assert!(literal.len() >= 6,
                  "Raw string literal must include two '---' delimiters.");

    // The string literal without '---' is the maximum size we are going to
    // need, so reserve that much. We also strip at least one newline.
    let mut string = String::with_capacity(literal.len() - 7);
    let sliced = &literal[3..literal.len() - 3];

    // Find the last newline in the string. Everything after that should be
    // whitespace (because the closing '---' should go on its own line) and that
    // is the whitespace to be stripped.
    // TODO: Turn into proper error.
    let last_newline = sliced.rfind('\n').expect("Raw string literal must contain newline");
    let indent = sliced.len() - last_newline - 1;

    // TODO: Turn into proper error.
    assert_eq!(sliced.chars().next(), Some('\n'),
      "Raw string literal must have newline after opening '---'.");

    // Drop the starting newline and the final newline; these are the additional
    // newlines due to '---' being on their own line. Then iterate over the
    // inner parts to append every line, with indentation stripped.
    let mut left = &sliced[1..last_newline];

    // Iterate over lines manually because the std::str::lines() iterator
    // silently drops trailing newlines.
    while let Some(index) = left.find('\n') {
        // Do allow completely empty lines, because otherwise these would be
        // required to have trailing whitespace.
        if index == 0 {
            string.push('\n');
            left = &left[1..];
            continue;
        }
        // TODO: Proper error handling with Result.
        assert!(index > indent, "Newline in indent of raw string literal.");
        assert!(left.chars().take(indent).all(|x| x == ' '),
                "Non-space character in indent of raw string literal.");
        string.push_str(&left[indent..index + 1]);
        left = &left[index + 1..];
    }

    // And the final line.
    assert!(left.chars().take(indent).all(|x| x == ' '),
            "Non-space character in indent of raw string literal.");
    if left.len() >= indent {
        string.push_str(&left[indent..]);
    }

    // We should not have allocated accidentally.
    debug_assert_eq!(string.capacity(), literal.len() - 7);

    string
}

/// Turns a string literal into the string it represents.
///
/// For example, `"foo\"bar"` becomes `foo"bar`.
pub fn unescape_string_literal<'a>(s: &'a str)
                                   -> Result<String, String> {
    let mut string = String::with_capacity(s.len() - 2);

    // Parsing escape sequences in a string literal is a small state machine
    // with the following states.
    enum EscState {
        // The base state.
        Normal,
        // After a backslash.
        Escape,
        // After '\u'. The state is (current_value, bytes_left).
        Unicode(u32, u32),
    }

    let mut st = EscState::Normal;

    // Iterate all characters except for the enclosing quotes.
    for ch in s[1..s.len() - 1].chars() {
        match st {
            EscState::Normal => {
                match ch {
                    '\\' => st = EscState::Escape,
                    _ => string.push(ch),
                }
            }
            EscState::Escape => {
                match ch {
                    '\\' => { string.push('\\'); st = EscState::Normal; }
                    '"' => { string.push('"'); st = EscState::Normal; }
                    'n' => { string.push('\n'); st = EscState::Normal; }
                    'u' => { st = EscState::Unicode(0, 6); }
                    _ => return Err(format!("Invalid escape code '\\{}'.", ch)),
                }
            }
            EscState::Unicode(codepoint, num_left) => {
                // An unicode escape sequence of the form \u1f574 consists of at
                // most 6 hexadecimal characters, and ends at the first non-hex
                // character. Examples:
                // "\u"        -> U+0
                // "\u1f574z"  -> U+1F574, U+7A
                // "\u1f5741"  -> U+1F5741
                // "\u01f5741" -> U+1F574, U+31
                if ch.is_digit(16) && num_left > 0 {
                    // Parsing the digit will succeed, because we checked above.
                    let d = ch.to_digit(16).unwrap();
                    st = EscState::Unicode(codepoint * 16 + d, num_left - 1);
                } else {
                    // End of unicode escape, append the value and the current
                    // character which was not part of the escape.
                    string.push(char_from_codepoint(codepoint)?);
                    string.push(ch);
                    st = EscState::Normal;
                }
            }
        }
    }

    match st {
        // A string might end in an escape code.
        EscState::Unicode(codepoint, _num_left) => {
            string.push(char_from_codepoint(codepoint)?);
        }
        _ => { }
    }

    Ok(string)
}

fn char_from_codepoint<'a>(codepoint: u32) -> Result<char, String> {
    match char::from_u32(codepoint) {
        Some(c) => Ok(c),
        None => Err(format!("Invalid code point U+{:X}.", codepoint)),
    }
}

/// Parse a color string like `#aabbcc` into a `Color`.
///
/// This assumes that the string is of the expected format; it panics if it is
/// not. It is the task of the lexer and parser to ensure that only strings of
/// the valid format end up here.
pub fn parse_color(color: &str) -> ast::Color {
    debug_assert_eq!(&color[0..1], "#");
    let r = u8::from_str_radix(&color[1..3], 16).unwrap();
    let g = u8::from_str_radix(&color[3..5], 16).unwrap();
    let b = u8::from_str_radix(&color[5..7], 16).unwrap();
    ast::Color(r, g, b)
}

#[test]
fn unescape_raw_string_literal_strips_dashes() {
    let x = unescape_raw_string_literal("---\nhi\n---");
    assert_eq!("hi", &x);
}

#[test]
fn unescape_raw_string_literal_strips_indent() {
    let x = unescape_raw_string_literal("---\n  hi\n  ---");
    assert_eq!("hi", &x);
}

#[test]
fn unescape_raw_string_literal_preserves_newlines() {
    let x = unescape_raw_string_literal("---\n  hi\n    there\n  ---");
    assert_eq!("hi\n  there", &x);
}

#[test]
fn unescape_raw_string_literal_allows_blank_lines() {
    let x = unescape_raw_string_literal("---\n  hi\n\n  there\n  ---");
    assert_eq!("hi\n\nthere", &x);
}

#[test]
fn unescape_raw_string_literal_can_end_with_blank_line() {
    // What we have here is a line of a raw string literal (indented by two
    // spaces), followed by a blank line (entirely empty). This caused an index
    // out of bounds before.
    let x = unescape_raw_string_literal("---\n  First line.\n\n  ---");
    assert_eq!("First line.\n", &x);
}

#[test]
fn unescape_string_literal_strips_quotes() {
    let x = unescape_string_literal("\"\"");
    assert_eq!(Ok("".into()), x);
}

#[test]
fn unescape_string_literal_handles_escaped_quotes() {
    let x = unescape_string_literal("\"x\\\"y\"");
    assert_eq!(Ok("x\"y".into()), x);
}

#[test]
fn unescape_string_literal_handles_escaped_newlines() {
    let x = unescape_string_literal("\"\\n\"");
    assert_eq!(Ok("\n".into()), x);
}

#[test]
fn unescape_string_literal_handles_escaped_codepoints() {
    let x = unescape_string_literal("\"\\u1f574 Unicode 6 was a bad idea.\"");
    assert_eq!(Ok("\u{1f574} Unicode 6 was a bad idea.".into()), x);
}

#[test]
fn unescape_string_literal_handles_escaped_codepoints_at_end() {
    let x = unescape_string_literal("\"\\u1f574\"");
    assert_eq!(Ok("\u{1f574}".into()), x);
}

#[test]
fn unescape_string_literal_handles_short_escaped_codepoints() {
    let x = unescape_string_literal("\"\\u0anewline\"");
    assert_eq!(Ok("\nnewline".into()), x);
}

#[test]
fn unescape_string_literal_handles_long_escaped_codepoints() {
    let x = unescape_string_literal("\"\\u00000afg\"");
    assert_eq!(Ok("\nfg".into()), x);
    let y = unescape_string_literal("\"\\u0000afg\"");
    assert_eq!(Ok("\u{00af}g".into()), y);
}
