// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use super::{
    keyword::Keyword,
    prim::{dot_ident, ident, keyword, many, opt, pat, seq, token},
    scan::Scanner,
    stmt::{self, stmt},
    ty::{self, ty},
    ErrorKind, Result,
};
use crate::lex::{Delim, TokenKind};
use qsc_ast::ast::{
    Block, CallableBody, CallableDecl, CallableKind, DeclMeta, Item, ItemKind, Namespace, NodeId,
    Package, Spec, SpecBody, SpecDecl, SpecGen, Visibility, VisibilityKind,
};

pub(super) fn package(s: &mut Scanner) -> Result<Package> {
    let namespaces = many(s, namespace)?;
    token(s, TokenKind::Eof)?;
    Ok(Package {
        id: NodeId::PLACEHOLDER,
        namespaces,
    })
}

fn namespace(s: &mut Scanner) -> Result<Namespace> {
    let lo = s.peek().span.lo;
    keyword(s, Keyword::Namespace)?;
    let name = dot_ident(s)?;
    token(s, TokenKind::Open(Delim::Brace))?;
    let items = many(s, item)?;
    token(s, TokenKind::Close(Delim::Brace))?;
    Ok(Namespace {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
        name,
        items,
    })
}

fn item(s: &mut Scanner) -> Result<Item> {
    let lo = s.peek().span.lo;
    let kind = if let Some(meta) = opt(s, decl_meta)? {
        Ok(ItemKind::Callable(meta, callable_decl(s)?))
    } else if let Some(decl) = opt(s, callable_decl)? {
        let meta = DeclMeta {
            attrs: Vec::new(),
            visibility: None,
        };
        Ok(ItemKind::Callable(meta, decl))
    } else if keyword(s, Keyword::Open).is_ok() {
        let name = dot_ident(s)?;
        let alias = if keyword(s, Keyword::As).is_ok() {
            Some(dot_ident(s)?)
        } else {
            None
        };
        token(s, TokenKind::Semi)?;
        Ok(ItemKind::Open(name, alias))
    } else {
        Err(s.error(ErrorKind::Rule("item")))
    }?;

    Ok(Item {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
        kind,
    })
}

fn decl_meta(s: &mut Scanner) -> Result<DeclMeta> {
    let lo = s.peek().span.lo;
    keyword(s, Keyword::Internal)?;
    let visibility = Visibility {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
        kind: VisibilityKind::Internal,
    };
    Ok(DeclMeta {
        attrs: Vec::new(),
        visibility: Some(visibility),
    })
}

fn callable_decl(s: &mut Scanner) -> Result<CallableDecl> {
    let lo = s.peek().span.lo;
    let kind = if keyword(s, Keyword::Function).is_ok() {
        Ok(CallableKind::Function)
    } else if keyword(s, Keyword::Operation).is_ok() {
        Ok(CallableKind::Operation)
    } else {
        Err(s.error(ErrorKind::Rule("callable declaration")))
    }?;

    let name = ident(s)?;
    let ty_params = if token(s, TokenKind::Lt).is_ok() {
        let vars = seq(s, ty::var)?.0;
        token(s, TokenKind::Gt)?;
        vars
    } else {
        Vec::new()
    };

    let input = pat(s)?;
    token(s, TokenKind::Colon)?;
    let output = ty(s)?;
    let functors = if keyword(s, Keyword::Is).is_ok() {
        Some(ty::functor_expr(s)?)
    } else {
        None
    };
    let body = callable_body(s)?;

    Ok(CallableDecl {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
        kind,
        name,
        ty_params,
        input,
        output,
        functors,
        body,
    })
}

fn callable_body(s: &mut Scanner) -> Result<CallableBody> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Open(Delim::Brace))?;
    let specs = many(s, spec_decl)?;
    if specs.is_empty() {
        let stmts = many(s, stmt)?;
        token(s, TokenKind::Close(Delim::Brace))?;
        Ok(CallableBody::Block(Block {
            id: NodeId::PLACEHOLDER,
            span: s.span(lo),
            stmts,
        }))
    } else {
        token(s, TokenKind::Close(Delim::Brace))?;
        Ok(CallableBody::Specs(specs))
    }
}

fn spec_decl(s: &mut Scanner) -> Result<SpecDecl> {
    let lo = s.peek().span.lo;
    let spec = if keyword(s, Keyword::Body).is_ok() {
        Ok(Spec::Body)
    } else if keyword(s, Keyword::Adjoint).is_ok() {
        Ok(Spec::Adj)
    } else if keyword(s, Keyword::Controlled).is_ok() {
        if keyword(s, Keyword::Adjoint).is_ok() {
            Ok(Spec::CtlAdj)
        } else {
            Ok(Spec::Ctl)
        }
    } else {
        Err(s.error(ErrorKind::Rule("specialization")))
    }?;

    let body = if let Some(gen) = opt(s, spec_gen)? {
        token(s, TokenKind::Semi)?;
        SpecBody::Gen(gen)
    } else {
        SpecBody::Impl(pat(s)?, stmt::block(s)?)
    };

    Ok(SpecDecl {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
        spec,
        body,
    })
}

fn spec_gen(s: &mut Scanner) -> Result<SpecGen> {
    if keyword(s, Keyword::Auto).is_ok() {
        Ok(SpecGen::Auto)
    } else if keyword(s, Keyword::Distribute).is_ok() {
        Ok(SpecGen::Distribute)
    } else if keyword(s, Keyword::Intrinsic).is_ok() {
        Ok(SpecGen::Intrinsic)
    } else if keyword(s, Keyword::Invert).is_ok() {
        Ok(SpecGen::Invert)
    } else if keyword(s, Keyword::Slf).is_ok() {
        Ok(SpecGen::Slf)
    } else {
        Err(s.error(ErrorKind::Rule("specialization generator")))
    }
}
