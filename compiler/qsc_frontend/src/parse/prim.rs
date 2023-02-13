// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{scan::Scanner, Parser, Result};
use crate::lex::TokenKind;
use qsc_ast::ast::{Ident, NodeId, Path, Span};

pub(super) fn ident(s: &mut Scanner) -> Result<Ident> {
    let name = s.ident()?.to_string();
    Ok(Ident {
        id: NodeId::PLACEHOLDER,
        span: s.span(),
        name,
    })
}

pub(super) fn path(s: &mut Scanner) -> Result<Path> {
    let lo = s.span().lo;
    let mut parts = vec![ident(s)?];
    while s.expect(TokenKind::Dot).is_ok() {
        parts.push(ident(s)?);
    }

    let name = parts.pop().unwrap();
    let namespace = if parts.is_empty() {
        None
    } else {
        let lo = parts.first().unwrap().span.lo;
        let hi = parts.last().unwrap().span.hi;
        let names: Vec<_> = parts.into_iter().map(|i| i.name).collect();
        Some(Ident {
            id: NodeId::PLACEHOLDER,
            span: Span { lo, hi },
            name: names.join("."),
        })
    };

    let hi = s.span().hi;
    Ok(Path {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        namespace,
        name,
    })
}

pub(super) fn opt<T>(s: &mut Scanner, mut p: impl Parser<T>) -> Result<Option<T>> {
    let span = s.span();
    match p(s) {
        Ok(x) => Ok(Some(x)),
        Err(_) if span == s.span() => Ok(None),
        Err(err) => Err(err),
    }
}

pub(super) fn comma_sep<T>(s: &mut Scanner, mut p: impl Parser<T>) -> Vec<T> {
    let mut items = Vec::new();
    while let Ok(item) = p(s) {
        items.push(item);
        if s.expect(TokenKind::Comma).is_err() {
            break;
        }
    }

    items
}
