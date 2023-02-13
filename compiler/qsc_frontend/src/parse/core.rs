// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{kw, scan::Scanner, Parser, Result};
use crate::lex::{Delim, TokenKind};
use qsc_ast::ast::{
    CallableBody, CallableDecl, CallableKind, DeclMeta, Ident, Item, ItemKind, Namespace, NodeId,
    Package, Pat, PatKind, Path, Span, Spec, SpecBody, SpecDecl, SpecGen, Ty, TyKind, TyPrim,
    TyVar,
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
        let ty_params = comma_sep(s, ty_var);
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

fn ty(s: &mut Scanner) -> Result<Ty> {
    let lo = s.span().lo;
    let mut acc = ty_base(s)?;
    loop {
        if let Some(array) = opt(s, ty_array)? {
            let hi = s.span().hi;
            acc = Ty {
                id: NodeId::PLACEHOLDER,
                span: Span { lo, hi },
                kind: TyKind::App(Box::new(array), vec![acc]),
            }
        } else if s.expect(TokenKind::RArrow).is_ok() {
            let output = ty(s)?;
            let hi = s.span().hi;
            acc = Ty {
                id: NodeId::PLACEHOLDER,
                span: Span { lo, hi },
                kind: TyKind::Arrow(
                    CallableKind::Function,
                    Box::new(acc),
                    Box::new(output),
                    None,
                ),
            }
        } else if s.expect(TokenKind::FatArrow).is_ok() {
            let output = ty(s)?;
            let hi = s.span().hi;
            acc = Ty {
                id: NodeId::PLACEHOLDER,
                span: Span { lo, hi },
                kind: TyKind::Arrow(
                    CallableKind::Operation,
                    Box::new(acc),
                    Box::new(output),
                    None,
                ),
            }
        } else {
            return Ok(acc);
        }
    }
}

fn ty_base(s: &mut Scanner) -> Result<Ty> {
    let lo = s.span().lo;
    let kind = if s.expect(TokenKind::Open(Delim::Paren)).is_ok() {
        let tys = comma_sep(s, ty);
        s.expect(TokenKind::Close(Delim::Paren))?;
        Ok(TyKind::Tuple(tys))
    } else if s.keyword(kw::BIG_INT).is_ok() {
        Ok(TyKind::Prim(TyPrim::BigInt))
    } else if s.keyword(kw::BOOL).is_ok() {
        Ok(TyKind::Prim(TyPrim::Bool))
    } else if s.keyword(kw::DOUBLE).is_ok() {
        Ok(TyKind::Prim(TyPrim::Double))
    } else if s.keyword(kw::INT).is_ok() {
        Ok(TyKind::Prim(TyPrim::Int))
    } else if s.keyword(kw::PAULI).is_ok() {
        Ok(TyKind::Prim(TyPrim::Pauli))
    } else if s.keyword(kw::QUBIT).is_ok() {
        Ok(TyKind::Prim(TyPrim::Qubit))
    } else if s.keyword(kw::RANGE).is_ok() {
        Ok(TyKind::Prim(TyPrim::Range))
    } else if s.keyword(kw::RESULT).is_ok() {
        Ok(TyKind::Prim(TyPrim::Result))
    } else if s.keyword(kw::STRING).is_ok() {
        Ok(TyKind::Prim(TyPrim::String))
    } else if s.keyword(kw::UNIT).is_ok() {
        Ok(TyKind::Tuple(Vec::new()))
    } else if let Some(var) = opt(s, ty_var)? {
        Ok(TyKind::Var(TyVar::Name(var.name)))
    } else if let Some(path) = opt(s, path)? {
        if path.namespace.is_none() && path.name.name == "_" {
            Ok(TyKind::Hole)
        } else {
            Ok(TyKind::Path(path))
        }
    } else {
        Err(s.error("Expecting type.".to_string()))
    }?;

    let hi = s.span().hi;
    Ok(Ty {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        kind,
    })
}

fn ty_array(s: &mut Scanner) -> Result<Ty> {
    s.expect(TokenKind::Open(Delim::Bracket))?;
    let lo = s.span().lo;
    s.expect(TokenKind::Close(Delim::Bracket))?;
    let hi = s.span().hi;
    Ok(Ty {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        kind: TyKind::Prim(TyPrim::Array),
    })
}

fn ty_var(s: &mut Scanner) -> Result<Ident> {
    s.expect(TokenKind::Apos)?;
    ident(s)
}

fn pat(s: &mut Scanner) -> Result<Pat> {
    let lo = s.span().lo;
    let kind = if let Some(name) = opt(s, ident)? {
        let ty = if s.expect(TokenKind::Colon).is_ok() {
            Some(ty(s)?)
        } else {
            None
        };
        if name.name == "_" {
            Ok(PatKind::Discard(ty))
        } else {
            Ok(PatKind::Bind(name, ty))
        }
    } else if s.expect(TokenKind::DotDotDot).is_ok() {
        Ok(PatKind::Elided)
    } else if s.expect(TokenKind::Open(Delim::Paren)).is_ok() {
        let pats = comma_sep(s, pat);
        s.expect(TokenKind::Close(Delim::Paren))?;
        Ok(PatKind::Tuple(pats))
    } else {
        Err(s.error("Expecting pattern.".to_string()))
    }?;

    let hi = s.span().hi;
    Ok(Pat {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        kind,
    })
}

fn comma_sep<T>(s: &mut Scanner, mut p: impl Parser<T>) -> Vec<T> {
    let mut items = Vec::new();
    while let Ok(item) = p(s) {
        items.push(item);
        if s.expect(TokenKind::Comma).is_err() {
            break;
        }
    }

    items
}

fn opt<T>(s: &mut Scanner, mut p: impl Parser<T>) -> Result<Option<T>> {
    let span = s.span();
    match p(s) {
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
