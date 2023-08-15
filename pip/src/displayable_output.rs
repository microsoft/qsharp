// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use num_bigint::BigUint;
use num_complex::{Complex64, ComplexFloat};
use qsc::{fmt_basis_state_label, fmt_complex, format_state_id, get_phase};

pub struct DisplayableState(pub Vec<(BigUint, Complex64)>, pub usize);

impl DisplayableState {
    pub fn to_plain(&self) -> String {
        format!(
            "STATE:{}",
            self.0
                .iter()
                .map(|(id, state)| format!(
                    "\n{}: {}",
                    format_state_id(id, self.1),
                    fmt_complex(state)
                ))
                .collect::<String>()
        )
    }

    pub fn to_html(&self) -> String {
        format!(
            include_str!("state_header_template.html"),
            self.0
                .iter()
                .map(|(id, state)| {
                    let amplitude = state.abs().powi(2) * 100.0;
                    format!(
                        include_str!("state_row_template.html"),
                        fmt_basis_state_label(id, self.1),
                        fmt_complex(state),
                        amplitude,
                        amplitude,
                        get_phase(state),
                        get_phase(state)
                    )
                })
                .collect::<String>()
        )
    }
}

pub enum DisplayableOutput {
    State(DisplayableState),
    Message(String),
}
