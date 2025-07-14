// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::fmt::{self, Display, Formatter};

/// Displays values separated by the provided string.
pub fn join(
    f: &mut Formatter,
    mut vals: impl Iterator<Item = impl Display>,
    sep: &str,
) -> fmt::Result {
    if let Some(v) = vals.next() {
        v.fmt(f)?;
    }
    for v in vals {
        write!(f, "{sep}")?;
        v.fmt(f)?;
    }
    Ok(())
}
