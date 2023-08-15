// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::io::{Cursor, Write};

use num_bigint::BigUint;
use num_complex::{Complex, Complex64};

#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Error;

#[must_use]
pub fn format_state_id(id: &BigUint, qubit_count: usize) -> String {
    format!("|{}⟩", fmt_basis_state_label(id, qubit_count))
}

#[must_use]
pub fn get_phase(c: &Complex<f64>) -> f64 {
    f64::atan2(c.im, c.re)
}

#[must_use]
pub fn fmt_complex(c: &Complex<f64>) -> String {
    // Format -0 as 0
    // Also using Unicode Minus Sign instead of ASCII Hyphen-Minus
    // and Unicode Mathematical Italic Small I instead of ASCII i.
    format!(
        "{}{:.4}{}{:.4}𝑖",
        if c.re <= -0.00005 { "−" } else { "" },
        c.re.abs(),
        if c.im <= -0.00005 { "−" } else { "+" },
        c.im.abs()
    )
}

#[must_use]
pub fn fmt_basis_state_label(id: &BigUint, qubit_count: usize) -> String {
    // This will generate a bit string that shows the qubits in the order
    // of allocation, left to right.
    format!("{:0>qubit_count$}", id.to_str_radix(2))
}

pub trait Receiver {
    /// Receive state output
    /// # Errors
    /// This will return an error if handling the output fails.
    fn state(&mut self, state: Vec<(BigUint, Complex64)>, qubit_count: usize) -> Result<(), Error>;

    /// Receive generic message output
    /// # Errors
    /// This will return an error if handling the output fails.
    fn message(&mut self, msg: &str) -> Result<(), Error>;
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
    fn state(&mut self, state: Vec<(BigUint, Complex64)>, qubit_count: usize) -> Result<(), Error> {
        writeln!(self.writer, "STATE:").map_err(|_| Error)?;
        for (id, state) in state {
            writeln!(
                self.writer,
                "{}: {}",
                format_state_id(&id, qubit_count),
                state
            )
            .map_err(|_| Error)?;
        }
        Ok(())
    }

    fn message(&mut self, msg: &str) -> Result<(), Error> {
        writeln!(self.writer, "{msg}").map_err(|_| Error)
    }
}

pub struct CursorReceiver<'a> {
    cursor: &'a mut Cursor<Vec<u8>>,
}

impl<'a> CursorReceiver<'a> {
    pub fn new(cursor: &'a mut Cursor<Vec<u8>>) -> Self {
        Self { cursor }
    }
    pub fn dump(&mut self) -> String {
        let v = self.cursor.get_mut();
        let s = match std::str::from_utf8(v) {
            Ok(v) => v.to_owned(),
            Err(e) => format!("Invalid UTF-8 sequence: {e}"),
        };
        v.clear();
        s.trim().to_string()
    }
}

impl<'a> Receiver for CursorReceiver<'a> {
    fn state(&mut self, state: Vec<(BigUint, Complex64)>, qubit_count: usize) -> Result<(), Error> {
        writeln!(self.cursor, "STATE:").map_err(|_| Error)?;
        for (id, state) in state {
            writeln!(
                self.cursor,
                "{}: {}",
                format_state_id(&id, qubit_count),
                state
            )
            .map_err(|_| Error)?;
        }
        Ok(())
    }

    fn message(&mut self, msg: &str) -> Result<(), Error> {
        writeln!(self.cursor, "{msg}").map_err(|_| Error)
    }
}
