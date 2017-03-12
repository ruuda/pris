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

pub enum Error {
    Arity(ArityError),
    Type(TypeError),
    MissingFont(MissingFontError),
    Other(String),
}

pub struct ArityError {
    #[allow(dead_code)] // Used in tests (TODO).
    expected: u32,
    #[allow(dead_code)] // Used in tests (TODO).
    actual: u32,
    message: String,
}

pub struct TypeError {
    #[allow(dead_code)] // Used in tests (TODO).
    expected: ValType,
    #[allow(dead_code)] // Used in tests (TODO).
    actual: ValType,
    message: String,
}

pub struct MissingFontError {
    family: String,
    style: String,
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

    pub fn missing_font(family: String, style: String) -> Error {
        let err = MissingFontError {
            family: family,
            style: style,
        };
        Error::MissingFont(err)
    }

    pub fn print(&self) {
        print!("\x1b[31;1mError: \x1b[0m");
        match *self {
            Error::Arity(ref ae) => println!("{}\n", ae.message),
            Error::Type(ref tye) => println!("{}\n", tye.message),
            Error::MissingFont(ref mf) => println!("The font '{} {}' cannot be found.\n", mf.family, mf.style),
            Error::Other(ref ot) => println!("{}\n", ot),
        }
    }
}
