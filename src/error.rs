// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::result;

use ast::Idents;
use pretty::Formatter;
use types::ValType;

// Error message guidelines:
//
//  * Shorter is better.
//  * Simpler is better (no jargon).
//  * The expected thing goes first, the actual thing goes second.

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Arity(ArityError),
    Format(FormatError),
    MissingFile(MissingFileError),
    MissingFont(MissingFontError),
    Parse(ParseError),
    Type(TypeError),
    Value(ValueError),
    Other(String),
}

#[derive(Debug)]
pub struct FormatError {
    path: String,
    message: &'static str,
}

#[derive(Debug)]
pub struct MissingFileError {
    path: String,
}

#[derive(Debug)]
pub struct MissingFontError {
    family: String,
    style: String,
}

#[derive(Debug)]
pub struct ParseError {
    /// Index of the first byte in the source file that contains the error.
    pub start: usize,
    /// Index of the first byte after the error.
    pub end: usize,
    message: String,
}

#[derive(Debug)]
pub struct ArityError {
    #[allow(dead_code)] // Used in tests (TODO).
    expected: u32,
    #[allow(dead_code)] // Used in tests (TODO).
    actual: u32,
    message: String,
}

#[derive(Debug)]
pub struct TypeError {
    #[allow(dead_code)] // Used in tests (TODO).
    expected: ValType,
    #[allow(dead_code)] // Used in tests (TODO).
    actual: ValType,
    message: String,
}

#[derive(Debug)]
pub struct ValueError {
    message: String,
}

impl Error {
    pub fn arity(fn_name: &str, expected: u32, actual: u32) -> Error {
        let mut f = Formatter::new();
        f.print("'");
        f.print(fn_name);
        f.print("' takes ");
        f.print(expected);
        f.print(if expected == 1 { " argument" } else { " arguments" });
        f.print(", but ");
        f.print(actual);
        f.print(if actual == 1 { " was " } else { " were " });
        f.print("given.");
        let arity_error = ArityError {
            expected: expected,
            actual: actual,
            message: f.into_string(),
        };
        Error::Arity(arity_error)
    }

    pub fn binop_type(op_name: &str,
                      expected: ValType,
                      actual_lhs: ValType,
                      actual_rhs: ValType)
                      -> Error {
        let mut f = Formatter::new();
        f.print("'");
        f.print(op_name);
        f.print("' expects operands of type '");
        f.print(expected);
        f.print("', but found ");
        if actual_lhs == actual_rhs {
            f.print("'");
            f.print(actual_lhs);
            f.print("' instead.");
        } else {
            f.print("'");
            f.print(actual_lhs);
            f.print("' and '");
            f.print(actual_rhs);
            f.print("' instead.");
        }
        let type_error = TypeError {
            expected: expected,
            actual: actual_lhs, // TODO: What about actual rhs?
            message: f.into_string(),
        };
        Error::Type(type_error)
    }

    pub fn arg_type(fn_name: &str,
                    expected: ValType,
                    actual: ValType,
                    arg_num: u32)
                    -> Error {
        let mut f = Formatter::new();
        f.print("Expected '");
        f.print(expected);
        f.print("' but found '");
        f.print(actual);
        f.print("', in ");
        match arg_num {
            0 => f.print("first"),
            1 => f.print("second"),
            2 => f.print("third"),
            3 => f.print("fourth"),
            _ => panic!("Why does your function have so many arguments?"),
        }
        f.print(" argument of '");
        f.print(fn_name);
        f.print("'.");
        let type_error = TypeError {
            expected: expected,
            actual: actual,
            message: f.into_string(),
        };
        Error::Type(type_error)
    }

    pub fn var_type(var_name: &Idents,
                    expected: ValType,
                    actual: ValType)
                    -> Error {
        let mut f = Formatter::new();
        f.print("Expected '");
        f.print(var_name);
        f.print("' to have type '");
        f.print(expected);
        f.print("', but found '");
        f.print(actual);
        f.print("'.");
        let type_error = TypeError {
            expected: expected,
            actual: actual,
            message: f.into_string(),
        };
        Error::Type(type_error)
    }

    pub fn list_type(expected_element_type: ValType, actual_element_type: ValType) -> Error {
        let mut f = Formatter::new();
        f.print("Encountered elements of type '");
        f.print(expected_element_type);
        f.print("' as well as '");
        f.print(actual_element_type);
        f.print("' in one list, but all elements must have the same type.");
        let type_error = TypeError {
            expected: expected_element_type,
            actual: actual_element_type,
            message: f.into_string(),
        };
        Error::Type(type_error)
    }

    pub fn value(message: String) -> Error {
        let err = ValueError {
            message: message,
        };
        Error::Value(err)
    }

    pub fn missing_font(family: String, style: String) -> Error {
        let err = MissingFontError {
            family: family,
            style: style,
        };
        Error::MissingFont(err)
    }

    pub fn missing_file(path: String) -> Error {
        let err = MissingFileError {
            path: path,
        };
        Error::MissingFile(err)
    }

    pub fn format(path: String, message: &'static str) -> Error {
        let err = FormatError {
            path: path,
            message: message,
        };
        Error::Format(err)
    }

    pub fn parse(start: usize, end: usize, message: String) -> Error {
        let err = ParseError {
            start: start,
            end: end,
            message: message,
        };
        Error::Parse(err)
    }

    pub fn print(&self) {
        // Print in red using ANSI escape codes.
        print!("\x1b[31;1mError: \x1b[0m");
        match *self {
            Error::Arity(ref ae) => println!("{}\n", ae.message),
            Error::Format(ref f) => println!("The file '{}' is invalid. {}\n", f.path, f.message),
            Error::MissingFile(ref mf) => println!("The file '{}' does not exist.\n", mf.path),
            Error::MissingFont(ref mf) => println!("The font '{} {}' cannot be found.\n", mf.family, mf.style),
            Error::Other(ref ot) => println!("{}\n", ot),
            Error::Parse(ref pe) => println!("{}\n", pe.message),
            Error::Type(ref tye) => println!("{}\n", tye.message),
            Error::Value(ref ve) => println!("{}\n", ve.message),
        }
    }
}
