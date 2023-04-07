// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::io::Write;

use num_bigint::BigUint;
use num_complex::Complex64;

#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Error;

pub trait Receiver {
    /// Receive state output
    /// # Errors
    /// This will return an error if handling the output fails.
    fn state(&mut self, state: Vec<(BigUint, Complex64)>) -> Result<(), Error>;

    /// Receive generic message output
    /// # Errors
    /// This will return an error if handling the output fails.
    fn message(&mut self, msg: String) -> Result<(), Error>;
}

pub struct GenericReceiver<'a> {
    writer: &'a mut dyn Write,
}

impl<'a> GenericReceiver<'a> {
    pub fn new(writer: &'a mut impl Write) -> Self {
        Self { writer }
    }
}

impl<'a> Receiver for GenericReceiver<'a> {
    fn state(&mut self, state: Vec<(BigUint, Complex64)>) -> Result<(), Error> {
        writeln!(self.writer, "STATE:").map_err(|_| Error)?;
        for (id, state) in state {
            writeln!(self.writer, "|{}⟩: {}", id.to_str_radix(2), state).map_err(|_| Error)?;
        }
        Ok(())
    }

    fn message(&mut self, msg: String) -> Result<(), Error> {
        writeln!(self.writer, "{msg}").map_err(|_| Error)
    }
}
