// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    kw,
    prim::{comma_sep, ident, opt, pat, path},
    scan::Scanner,
    ty::{self, ty},
    Result,
};
use crate::lex::{Delim, TokenKind};
use qsc_ast::ast::{
    CallableBody, CallableDecl, CallableKind, DeclMeta, Item, ItemKind, Namespace, NodeId, Package,
    Span, Spec, SpecBody, SpecDecl, SpecGen,
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
        let ty_params = comma_sep(s, ty::var);
        s.expect(TokenKind::Gt)?;
        ty_params
    } else {
        Vec::new()
    };

    let input = pat(s)?;
    s.expect(TokenKind::Colon)?;
    let output = ty(s)?;
    let body = callable_body(s)?;
    let hi = s.span().hi;
    Ok(CallableDecl {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        kind,
        name,
        ty_params,
        input,
        output,
        functors: None,
        body,
    })
}

fn callable_body(s: &mut Scanner) -> Result<CallableBody> {
    s.expect(TokenKind::Open(Delim::Brace))?;
    let mut specs = Vec::new();
    while let Some(spec) = opt(s, spec_decl)? {
        specs.push(spec);
    }

    s.expect(TokenKind::Close(Delim::Brace))?;
    Ok(CallableBody::Specs(specs))
}

fn spec_decl(s: &mut Scanner) -> Result<SpecDecl> {
    let spec = if s.keyword(kw::BODY).is_ok() {
        Spec::Body
    } else if s.keyword(kw::ADJOINT).is_ok() {
        Spec::Adj
    } else if s.keyword(kw::CONTROLLED).is_ok() {
        if s.keyword(kw::ADJOINT).is_ok() {
            Spec::CtlAdj
        } else {
            Spec::Ctl
        }
    } else {
        return Err(s.error("Expecting specialization.".to_string()));
    };

    let lo = s.span().lo;
    let gen = if s.keyword(kw::AUTO).is_ok() {
        SpecGen::Auto
    } else if s.keyword(kw::DISTRIBUTE).is_ok() {
        SpecGen::Distribute
    } else if s.keyword(kw::INTRINSIC).is_ok() {
        SpecGen::Intrinsic
    } else if s.keyword(kw::INVERT).is_ok() {
        SpecGen::Invert
    } else if s.keyword(kw::SELF).is_ok() {
        SpecGen::Slf
    } else {
        return Err(s.error("Expecting specialization generator.".to_string()));
    };

    s.expect(TokenKind::Semi)?;
    let hi = s.span().hi;
    Ok(SpecDecl {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        spec,
        body: SpecBody::Gen(gen),
    })
}
