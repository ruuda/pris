// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValType {
    Num(LenDim),
    Str,
    Color,
    Coord(LenDim),
    Frame,
    Fn
}

/// Represents a number of length dimensions.
///
/// -2 means "per area".
/// -1 means "per length".
/// 0 indicates a dimensionless number.
/// 1 means "length".
/// 2 means "area".
/// 3 means "volume".
/// etc.
pub type LenDim = i32;
