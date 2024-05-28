// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::span::Span;

/// A line and column pair that describes a position in a string.
#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub struct Position {
    /// Line offset.
    pub line: u32,
    /// Column offset.
    /// When created using [`Encoding::Utf8`], this is the byte offset.
    /// When created using [`Encoding::Utf16`], this is the code unit (word) offset.
    pub column: u32,
}

/// The encoding to use when creating a [`Position`] from a source string and offset.
///
/// For reference, when **indexing directly** into a string:
///     Rust uses UTF-8 byte offsets (only available through conversion into a byte array)
///     JavaScript uses UTF-16 code unit offsets
///     Python uses character offsets
///
/// Additionally, all these languages provide alternate ways of iterating over
/// strings, which sometimes use code units and sometimes use characters,
/// confusing our understanding of what "nth character" means.
///
/// Therefore it's important to be aware of exactly how this column offset will be treated
/// by the code that's using it. e.g. in LSP (language server protocol) and
/// DAP (debug adapter protocol) it's explicitly defined to use UTF-16 code units.
#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub enum Encoding {
    Utf8,
    Utf16,
}

impl Position {
    /// For a given string and utf-8 byte offset, returns a [`Position`]
    /// that corresponds to that offset.
    ///
    /// The column information is expressed in code units, depending on the passed in `encoding`.
    /// [`Encoding::Utf8`] will use byte offsets, and [`Encoding::Utf16`] will use
    /// utf-16 code unit (word) offsets.
    ///
    /// If the given offset is past the end of the string, returns the position of
    /// the end of the string (e.g. for "hello" the end of the string is line=0, column=5).
    ///
    /// Note that this function does not validate whether the passed in offset
    /// is a valid utf8 char boundary. If an invalid offset is passed in,
    /// the next char boundary will be returned.
    #[must_use]
    pub fn from_utf8_byte_offset(
        encoding: Encoding,
        contents: &str,
        utf8_byte_offset: u32,
    ) -> Self {
        positions_from_utf8_byte_offsets(encoding, contents, [utf8_byte_offset])[0]
    }

    /// For a given string, returns the utf-8 byte offset that corresponds
    /// to this [`Position`] in that string.
    ///
    /// The column information in the position is interpreted based on the
    /// passed in `encoding`. [`Encoding::Utf8`] will treat the column information
    /// as byte offsets, and [`Encoding::Utf16`] will use utf-16 code unit (word) offsets.
    ///
    /// If the position is past the end of the string, returns the offset of
    /// the end of the string (e.g. for "hello" returns 5).
    #[must_use]
    pub fn to_utf8_byte_offset(&self, encoding: Encoding, contents: &str) -> u32 {
        let mut column: u32 = 0;
        let mut line: u32 = 0;

        for (byte_offset, c) in contents.char_indices() {
            if c == '\n' {
                line += 1;
                column = 0;
            } else {
                column += num_code_units(encoding, c);
            }

            if line > self.line || (line == self.line && column > self.column) {
                // We moved past the target line+column
                return u32(byte_offset);
            }
        }

        // return eof if we move past the end of the string
        u32(contents.len())
    }
}

/// Same as a [`Span`], but represented with [`Position`]s instead of offsets.
#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    /// For a given string and a span expressed in utf-8 byte offsets,
    /// returns a [`Range`] that corresponds to that span.
    ///
    /// Any offsets that go past the end of the string are mapped to the end of the string.
    /// (e.g. for "hello", the span (0-10) would map to (0,0-0,5)).
    ///
    /// This function does not validate span range. If `span.hi > span.lo`, the
    /// end offset is simply mapped to the end of the string.
    /// (e.g. for "hello", the invalid span (3-0) would map to (0,3-0,5)).
    ///
    /// This function also does not validate whether the passed in offset
    /// is a valid utf8 char boundary. If an invalid offset is passed in,
    /// the next char boundary will be returned.
    #[must_use]
    pub fn from_span(encoding: Encoding, contents: &str, span: &Span) -> Self {
        let [start, end] = positions_from_utf8_byte_offsets(encoding, contents, [span.lo, span.hi]);
        Self { start, end }
    }

    #[must_use]
    pub fn empty(&self) -> bool {
        self.start == self.end
    }
}

/// For a given string and array of utf-8 byte offsets, returns the [`Position`]s
/// corresponding to the byte offsets.
///
/// Since this function iterates over the string once, the array *MUST* be in sorted order.
///
/// This function takes a (const sized) array to avoid allocations.
fn positions_from_utf8_byte_offsets<const N: usize>(
    encoding: Encoding,
    contents: &str,
    sorted_utf8_byte_offsets: [u32; N],
) -> [Position; N] {
    // The below example contains characters that are encoded
    // with different numbers of code units in utf-8 and utf-16,
    // to demonstrate how code unit offset will differ depending
    // on the chosen encoding. Characters can be encoded with:
    //  One code unit (byte) in utf-8, one code unit (word) in utf-16
    //  Multiple code units in utf-8, one code unit in utf-16
    //  Multiple code units in utf-8, surrogate pair in utf-16

    // chars                    | 𝑓                 (        𝑥                 ⃗        )       Σ     <eof>
    // unicode code point       | 1d453             28       1d465             20d7     29     3a3
    // utf-8 units (bytes)      | f09d9193          28       f09d91a5          e28397   29     cea3
    // utf-16 units             | d835     dc53     0028     d835     dc65     20d7     0029   03a3
    // char offset              | 0                 1        2                 3        4      5     6
    // utf-8 byte offset        | 0                 4        5                 9        12     13    15
    // utf-16 code unit offset  | 0                 2        3                 5        6      7     8

    let mut positions = [Position { line: 0, column: 0 }; N];
    let mut i = 0;
    let mut column: u32 = 0;
    let mut line: u32 = 0;

    for (char_index, c) in contents.char_indices() {
        if i == N {
            // We've run out of offsets to look for
            break;
        }

        // We've moved past the next requested offset
        while char_index >= sorted_utf8_byte_offsets[i] as usize {
            positions[i] = Position { line, column };
            i += 1;

            if i == N {
                // We've run out of offsets to look for
                break;
            }
        }

        // Windows (\r\n) line endings will be handled
        // fine here, with the \r counting as an extra char
        // at the end of the line.
        if c == '\n' {
            line += 1;
            column = 0;
        } else {
            column += num_code_units(encoding, c);
        }
    }

    // If any offsets couldn't be mapped, map them to <eof>
    while i < N {
        positions[i] = Position { line, column };
        i += 1;
    }

    positions
}

fn num_code_units(encoding: Encoding, c: char) -> u32 {
    match encoding {
        Encoding::Utf8 => u32(c.len_utf8()),
        Encoding::Utf16 => u32(c.len_utf16()),
    }
}

fn u32(value: usize) -> u32 {
    // Compiler uses u32 for offsets, so safe to assume all strings
    // we deal with are shorter than u32::MAX.
    u32::try_from(value).expect("value should fit in u32")
}
