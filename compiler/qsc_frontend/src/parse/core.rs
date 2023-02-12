// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{kw, scan::Scanner, Result};
use crate::lex::{Delim, TokenKind};
use qsc_ast::ast::{
    CallableDecl, CallableKind, DeclMeta, Ident, Item, ItemKind, Namespace, NodeId, Package, Path,
    Span,
};

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

    let mut items = Vec::new();
    while let Some(item) = opt(s, item)? {
        items.push(item);
    }

    s.expect(TokenKind::Close(Delim::Brace))?;
    let hi = s.span().hi;
    Ok(Namespace {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        name,
        items,
    })
}

fn item(s: &mut Scanner) -> Result<Item> {
    let lo = s.span().lo;
    let meta = DeclMeta {
        attrs: Vec::new(),
        visibility: None,
    };

    let kind = if s.keyword(kw::FUNCTION).is_ok() {
        let decl = callable_decl(s, CallableKind::Function)?;
        Ok(ItemKind::Callable(meta, decl))
    } else if s.keyword(kw::OPERATION).is_ok() {
        let decl = callable_decl(s, CallableKind::Operation)?;
        Ok(ItemKind::Callable(meta, decl))
    } else {
        Err(s.error("Expecting namespace item.".to_string()))
    }?;

    let hi = s.span().hi;
    Ok(Item {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        kind,
    })
}

fn callable_decl(s: &mut Scanner, kind: CallableKind) -> Result<CallableDecl> {
    let lo = s.span().lo;
    let name = ident(s)?;

    let ty_params = if s.expect(TokenKind::Lt).is_ok() {
        let ty_params = comma_sep(s, ty_param);
        s.expect(TokenKind::Gt)?;
        ty_params
    } else {
        Vec::new()
    };

    let hi = s.span().hi;
    Ok(CallableDecl {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        kind,
        name,
        ty_params,
        input: todo!(),
        output: todo!(),
        functors: None,
        body: todo!(),
    })
}

fn ty_param(s: &mut Scanner) -> Result<Ident> {
    s.expect(TokenKind::Apos)?;
    ident(s)
}

fn comma_sep<T>(s: &mut Scanner, mut f: impl FnMut(&mut Scanner) -> Result<T>) -> Vec<T> {
    let mut items = Vec::new();
    while let Ok(item) = f(s) {
        items.push(item);
        if s.expect(TokenKind::Comma).is_err() {
            break;
        }
    }

    items
}

fn opt<T>(s: &mut Scanner, mut f: impl FnMut(&mut Scanner) -> Result<T>) -> Result<Option<T>> {
    let span = s.span();
    match f(s) {
        Ok(x) => Ok(Some(x)),
        Err(_) if span == s.span() => Ok(None),
        Err(err) => Err(err),
    }
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
