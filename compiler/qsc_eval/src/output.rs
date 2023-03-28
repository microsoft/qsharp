// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::fmt::Write;

use num_bigint::BigUint;
use num_complex::Complex64;

#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Error;

pub trait Receiver {
    /// Recieve state output
    /// # Errors
    /// This will return an error if handling the output fails.
    fn state(&mut self, state: Vec<(BigUint, Complex64)>) -> Result<(), Error>;

    /// Recieve generic message output
    /// # Errors
    /// This will return an error if handling the output fails.
    fn message(&mut self, msg: String) -> Result<(), Error>;
}

#[derive(Default)]
pub struct StdoutReceiver {}

impl Receiver for StdoutReceiver {
    fn state(&mut self, state: Vec<(BigUint, Complex64)>) -> Result<(), Error> {
        println!("{state:?}");
        Ok(())
    }

    fn message(&mut self, msg: String) -> Result<(), Error> {
        println!("{msg}");
        Ok(())
    }
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
        self.writer
            .write_fmt(format_args!("{state:?}\n"))
            .map_err(|_| Error)
    }

    fn message(&mut self, msg: String) -> Result<(), Error> {
        self.writer
            .write_fmt(format_args!("{msg}\n"))
            .map_err(|_| Error)
    }
}
