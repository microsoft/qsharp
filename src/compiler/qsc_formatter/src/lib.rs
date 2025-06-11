// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module contains the Q# formatter, which can be used by calling
//! the format function available from this module. The formatting algorithm
//! uses the cooked and concrete tokens from the parser crate to create a
//! token stream of the given source code string. It then uses a sliding window
//! over this token stream to apply formatting rules when the selected tokens
//! match certain patterns. The formatting algorithm uses state to help make
//! correct decisions, particularly around indentation. Formatting rules will
//! generate text edit objects when the format of the input string does not
//! match the expected format, and these edits are returned on using the
//! formatter.

pub mod formatter;
