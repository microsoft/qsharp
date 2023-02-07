// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::ast::{Lit, Span};

pub(crate) struct Token {
    kind: TokenKind,
    span: Span,
}

pub(crate) enum TokenKind {
    Bar,
    BinOpEq(ClosedBinOp),
    ClosedBinOp(ClosedBinOp),
    CloseDelim(Delim),
    Colon,
    ColonColon,
    DollarQuote,
    DotDot,
    DotDotDot,
    Eq,
    EqEq,
    FatArrow,
    Gt,
    Gte,
    Ident(String),
    LArrow,
    Lit(Lit),
    Lt,
    Lte,
    Neq,
    OpenDelim(Delim),
    Question,
    RArrow,
    Semi,
    SingleQuote,
    WSlash,
    WSlashEq,
}

/// Binary operators whose input type is closed under the operation. These are the only binary
/// operators that can be used for compound assignment, like `set x += y`.
pub(crate) enum ClosedBinOp {
    /// Addition: `+`.
    Add,
    /// Bitwise AND: `&&&`.
    AndB,
    /// Logical AND: `and`.
    AndL,
    /// Division: `/`.
    Div,
    /// Exponentiation: `^`.
    Exp,
    /// Modulus: `%`.
    Mod,
    /// Multiplication: `*`.
    Mul,
    /// Bitwise OR: `|||`.
    OrB,
    /// Logical OR: `or`.
    OrL,
    /// Shift left: `<<<`.
    Shl,
    /// Shift right: `>>>`.
    Shr,
    /// Subtraction: `-`.
    Sub,
    /// Bitwise XOR: `^^^`.
    XorB,
}

pub(crate) enum Delim {
    Brace,
    Bracket,
    Paren,
}
