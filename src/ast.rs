// Pris -- A language for designing slides
// Copyright 2017 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::fmt::{Display, Error, Formatter};

#[derive(Copy, Clone)]
pub enum Unit {
  W,
  H,
  Em,
  Pt,
}

pub struct Num {
  pub val: f64,
  pub unit: Option<Unit>,
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            Unit::W => write!(f, "w"),
            Unit::H => write!(f, "h"),
            Unit::Em => write!(f, "em"),
            Unit::Pt => write!(f, "pt"),
        }
    }
}

impl Display for Num {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.val)?;
        if let Some(unit) = self.unit {
            write!(f, "{}", unit)?
        }
        Ok(())
    }
}
