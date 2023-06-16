// Copyright (c) 2019 Craig Disselkoen
// Portions copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::cmp::{Ordering, PartialOrd};
use std::fmt;

/// Describes a "debug location" (source location)
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct DebugLoc {
    /// The source line number
    pub line: u32,
    /// The source column number
    ///
    /// `Instruction`s and `Terminator`s have this info (and will have `Some`
    /// here), while `GlobalVariable`s and `Function`s do not have this info (and
    /// will have `None`)
    pub col: Option<u32>,
    /// The source filename
    pub filename: String,
    /// The source directory, if available
    pub directory: Option<String>,
}

impl PartialOrd for DebugLoc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // compare in the order (directory, filename, line, col)
        Some(
            (&self.directory, &self.filename, &self.line, &self.col).cmp(&(
                &other.directory,
                &other.filename,
                &other.line,
                &other.col,
            )),
        )
    }
}

impl Ord for DebugLoc {
    fn cmp(&self, other: &Self) -> Ordering {
        // compare in the order (directory, filename, line, col)
        (&self.directory, &self.filename, &self.line, &self.col).cmp(&(
            &other.directory,
            &other.filename,
            &other.line,
            &other.col,
        ))
    }
}

impl fmt::Display for DebugLoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pretty_directory = match &self.directory {
            Some(dir) => dir,
            None => "",
        };
        let need_slash = match &self.directory {
            Some(dir) => !dir.is_empty() && !dir.ends_with('/') && !self.filename.starts_with('/'),
            None => false,
        };
        let pretty_filename = match &self.filename as &str {
            "" => "<no filename available>",
            filename if !pretty_directory.is_empty() => {
                filename.trim_start_matches(pretty_directory)
            }
            filename => filename,
        };
        let pretty_column = match self.col {
            Some(col) => format!(", col {col}"),
            None => String::new(),
        };
        write!(
            f,
            "{}{}{}, line {}{}",
            pretty_directory,
            if need_slash { "/" } else { "" },
            pretty_filename,
            self.line,
            pretty_column,
        )
    }
}

pub trait HasDebugLoc {
    /// Returns the `DebugLoc` associated with the given `Instruction`,
    /// `Terminator`, `GlobalVariable`, or `Function`; or `None` if it doesn't
    /// have a `DebugLoc`.
    ///
    /// Reasons something might not have a `DebugLoc` include:
    ///     (1) the file was compiled without debuginfo;
    ///     (2) for an `Instruction`, it might not directly correspond to any source
    ///     line. For instance, it may be just setting up the stack frame for a
    ///     function.
    fn get_debug_loc(&self) -> &Option<DebugLoc>;
}
