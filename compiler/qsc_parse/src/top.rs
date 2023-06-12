// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use super::{
    expr::expr,
    keyword::Keyword,
    prim::{dot_ident, ident, keyword, many, opt, pat, seq, token},
    scan::Scanner,
    stmt::{self, stmt},
    ty::{self, ty},
    Error, Result,
};
use crate::{
    lex::{Delim, TokenKind},
    prim::shorten,
    ErrorKind,
};
use qsc_ast::ast::{
    Attr, Block, CallableBody, CallableDecl, CallableKind, Ident, Item, ItemKind, Namespace,
    NodeId, Path, Spec, SpecBody, SpecDecl, SpecGen, Stmt, Ty, TyDef, TyDefKind, TyKind,
    Visibility, VisibilityKind,
};

pub enum Fragment {
    Namespace(Namespace),
    Stmt(Box<Stmt>),
}

pub(super) fn namespaces(s: &mut Scanner) -> Result<Vec<Namespace>> {
    let namespaces = many(s, namespace)?;
    token(s, TokenKind::Eof)?;
    Ok(namespaces)
}

pub(super) fn fragments(s: &mut Scanner) -> Result<Vec<Fragment>> {
    let fragments = many(s, fragment)?;
    token(s, TokenKind::Eof)?;
    Ok(fragments)
}

fn fragment(s: &mut Scanner) -> Result<Fragment> {
    if let Some(namespace) = opt(s, namespace)? {
        Ok(Fragment::Namespace(namespace))
    } else {
        stmt(s).map(Fragment::Stmt)
    }
}

fn namespace(s: &mut Scanner) -> Result<Namespace> {
    let lo = s.peek().span.lo;
    let docs = many(s, doc)?;
    keyword(s, Keyword::Namespace)?;
    let name = dot_ident(s)?;
    token(s, TokenKind::Open(Delim::Brace))?;
    let items = many(s, item)?;
    token(s, TokenKind::Close(Delim::Brace))?;
    Ok(Namespace {
        id: NodeId::default(),
        span: s.span(lo),
        doc: docs.join("\n").into(),
        name,
        items: items.into_boxed_slice(),
    })
}

pub(super) fn item(s: &mut Scanner) -> Result<Box<Item>> {
    let lo = s.peek().span.lo;
    let docs = many(s, doc)?;
    let attrs = many(s, attr)?;
    let visibility = opt(s, visibility)?;
    let kind = if let Some(open) = opt(s, item_open)? {
        Ok(open)
    } else if let Some(ty) = opt(s, item_ty)? {
        Ok(ty)
    } else if let Some(callable) = opt(s, callable_decl)? {
        Ok(Box::new(ItemKind::Callable(callable)))
    } else {
        Err(Error(ErrorKind::Rule("item", s.peek().kind, s.peek().span)))
    }?;

    Ok(Box::new(Item {
        id: NodeId::default(),
        span: s.span(lo),
        doc: docs.join("\n").into(),
        attrs: attrs.into_boxed_slice(),
        visibility,
        kind,
    }))
}

fn doc(s: &mut Scanner) -> Result<String> {
    if s.peek().kind == TokenKind::DocComment {
        let prefix_len = if s.read().starts_with("/// ") { 4 } else { 3 };
        let content = shorten(prefix_len, 0, s.read());
        s.advance();
        Ok(content.to_string())
    } else {
        Err(Error(ErrorKind::Token(
            TokenKind::DocComment,
            s.peek().kind,
            s.peek().span,
        )))
    }
}

fn attr(s: &mut Scanner) -> Result<Box<Attr>> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::At)?;
    let name = ident(s)?;
    let arg = expr(s)?;
    Ok(Box::new(Attr {
        id: NodeId::default(),
        span: s.span(lo),
        name,
        arg,
    }))
}

fn visibility(s: &mut Scanner) -> Result<Visibility> {
    let lo = s.peek().span.lo;
    keyword(s, Keyword::Internal)?;
    Ok(Visibility {
        id: NodeId::default(),
        span: s.span(lo),
        kind: VisibilityKind::Internal,
    })
}

fn item_open(s: &mut Scanner) -> Result<Box<ItemKind>> {
    keyword(s, Keyword::Open)?;
    let name = dot_ident(s)?;
    let alias = if keyword(s, Keyword::As).is_ok() {
        Some(dot_ident(s)?)
    } else {
        None
    };
    token(s, TokenKind::Semi)?;
    Ok(Box::new(ItemKind::Open(name, alias)))
}

fn item_ty(s: &mut Scanner) -> Result<Box<ItemKind>> {
    keyword(s, Keyword::Newtype)?;
    let name = ident(s)?;
    token(s, TokenKind::Eq)?;
    let def = ty_def(s)?;
    token(s, TokenKind::Semi)?;
    Ok(Box::new(ItemKind::Ty(name, def)))
}

fn ty_def(s: &mut Scanner) -> Result<Box<TyDef>> {
    let lo = s.peek().span.lo;
    let kind = if token(s, TokenKind::Open(Delim::Paren)).is_ok() {
        let (defs, final_sep) = seq(s, ty_def)?;
        token(s, TokenKind::Close(Delim::Paren))?;
        Ok(final_sep.reify(defs, TyDefKind::Paren, TyDefKind::Tuple))
    } else {
        let field_ty = ty(s)?;
        if token(s, TokenKind::Colon).is_ok() {
            let name = ty_as_ident(field_ty)?;
            let field_ty = ty(s)?;
            Ok(TyDefKind::Field(Some(name), Box::new(field_ty)))
        } else {
            Ok(TyDefKind::Field(None, Box::new(field_ty)))
        }
    }?;

    Ok(Box::new(TyDef {
        id: NodeId::default(),
        span: s.span(lo),
        kind: Box::new(kind),
    }))
}

fn ty_as_ident(ty: Ty) -> Result<Box<Ident>> {
    let TyKind::Path(path) = *ty.kind else {
        return Err(Error(ErrorKind::Convert("identifier", "type", ty.span)));
    };
    if let Path {
        namespace: None,
        name,
        ..
    } = *path
    {
        Ok(name)
    } else {
        Err(Error(ErrorKind::Convert("identifier", "type", ty.span)))
    }
}

fn callable_decl(s: &mut Scanner) -> Result<Box<CallableDecl>> {
    let lo = s.peek().span.lo;
    let kind = if keyword(s, Keyword::Function).is_ok() {
        Ok(CallableKind::Function)
    } else if keyword(s, Keyword::Operation).is_ok() {
        Ok(CallableKind::Operation)
    } else {
        let token = s.peek();
        Err(Error(ErrorKind::Rule(
            "callable declaration",
            token.kind,
            token.span,
        )))
    }?;

    let name = ident(s)?;
    let generics = if token(s, TokenKind::Lt).is_ok() {
        let params = seq(s, ty::param)?.0;
        token(s, TokenKind::Gt)?;
        params
    } else {
        Vec::new()
    };

    let input = pat(s)?;
    token(s, TokenKind::Colon)?;
    let output = ty(s)?;
    let functors = if keyword(s, Keyword::Is).is_ok() {
        Some(Box::new(ty::functor_expr(s)?))
    } else {
        None
    };
    let body = callable_body(s)?;

    Ok(Box::new(CallableDecl {
        id: NodeId::default(),
        span: s.span(lo),
        kind,
        name,
        generics: generics.into_boxed_slice(),
        input,
        output: Box::new(output),
        functors,
        body: Box::new(body),
    }))
}

fn callable_body(s: &mut Scanner) -> Result<CallableBody> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Open(Delim::Brace))?;
    let specs = many(s, spec_decl)?;
    if specs.is_empty() {
        let stmts = many(s, stmt)?;
        token(s, TokenKind::Close(Delim::Brace))?;
        Ok(CallableBody::Block(Box::new(Block {
            id: NodeId::default(),
            span: s.span(lo),
            stmts: stmts.into_boxed_slice(),
        })))
    } else {
        token(s, TokenKind::Close(Delim::Brace))?;
        Ok(CallableBody::Specs(specs.into_boxed_slice()))
    }
}

fn spec_decl(s: &mut Scanner) -> Result<Box<SpecDecl>> {
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
        Err(Error(ErrorKind::Rule(
            "specialization",
            s.peek().kind,
            s.peek().span,
        )))
    }?;

    let body = if let Some(gen) = opt(s, spec_gen)? {
        token(s, TokenKind::Semi)?;
        SpecBody::Gen(gen)
    } else {
        SpecBody::Impl(pat(s)?, stmt::block(s)?)
    };

    Ok(Box::new(SpecDecl {
        id: NodeId::default(),
        span: s.span(lo),
        spec,
        body,
    }))
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
        Err(Error(ErrorKind::Rule(
            "specialization generator",
            s.peek().kind,
            s.peek().span,
        )))
    }
}
