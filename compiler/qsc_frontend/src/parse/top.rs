// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{kw, scan::Scanner, Result};
use crate::lex::{Delim, TokenKind};
use qsc_ast::ast::{Ident, Namespace, NodeId, Package, Path, Span};

pub(super) fn package(s: &mut Scanner) -> Result<Package> {
    let mut namespaces = Vec::new();
    while s.keyword(kw::NAMESPACE).is_ok() {
        namespaces.push(namespace(s)?);
    }

    s.expect(TokenKind::Eof)?;
    Ok(Package {
        id: NodeId::PLACEHOLDER,
        namespaces,
    })
}

fn namespace(s: &mut Scanner) -> Result<Namespace> {
    let lo = s.span().lo;
    let name = path(s)?;
    s.expect(TokenKind::Open(Delim::Brace))?;
    let items = Vec::new();
    s.expect(TokenKind::Close(Delim::Brace))?;
    let hi = s.span().hi;
    Ok(Namespace {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        name,
        items,
    })
}

fn path(s: &mut Scanner) -> Result<Path> {
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

fn ident(s: &mut Scanner) -> Result<Ident> {
    let name = s.ident()?.to_string();
    Ok(Ident {
        id: NodeId::PLACEHOLDER,
        span: s.span(),
        name,
    })
}
