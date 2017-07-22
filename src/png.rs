// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

//! This module implements reading limited png metadata.

use std::fs::File;
use std::io::Read;

use error::{Error, Result};

/// Get the width and height of a png file.
pub fn get_dimensions(fname: &str) -> Result<(u32, u32)> {
    // The width and height are present in the first 24 bytes of a png image.
    // Read those into a buffer.
    let mut buffer = [0u8; 24];

    if File::open(fname).map(|mut f| f.read_exact(&mut buffer)).is_err() {
        // TODO: There could be a different error than missing file.
        return Err(Error::missing_file(fname.into()))
    }

    const EXPECTED: [u8; 16] = [
        // The png signature.
        137, b'P', b'N', b'G', b'\r', b'\n', 26, b'\n',
        // The length of the IHDR chunk.
        0, 0, 0, 13,
        // The type of the IHDR chunk.
        b'I', b'H', b'D', b'R'
    ];

    if &buffer[..16] != &EXPECTED {
        return Err(Error::format(fname.into(), "It is not a valid png file."))
    }

    // Next are 2x4 bytes width and height.
    let w = read_u32_be(&buffer[16..20]);
    let h = read_u32_be(&buffer[20..24]);

    Ok((w, h))
}

/// Interpret four bytes as a big-endian (most significant byte first) integer.
fn read_u32_be(bytes: &[u8]) -> u32 {
    ((bytes[0] as u32) << 24) |
    ((bytes[1] as u32) << 16) |
    ((bytes[2] as u32) << 08) |
    ((bytes[3] as u32) << 00)
}

#[test]
fn get_dimensions_works_for_example() {
    let dim = get_dimensions("examples/image.png").unwrap();
    assert_eq!(dim, (256, 256));
}
