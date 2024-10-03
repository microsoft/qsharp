// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use super::{
    keyword::Keyword,
    prim::{apos_ident, opt, path, seq, token},
    scan::ParserContext,
    Error, Parser, Result,
};
use crate::{
    item::throw_away_doc,
    lex::{ClosedBinOp, Delim, TokenKind},
    prim::ident,
    ErrorKind,
};
use qsc_ast::ast::{
    CallableKind, Functor, FunctorExpr, FunctorExprKind, Ident, NodeId, SetOp, Ty, TyBounds,
    TyKind, TyParam,
};
use qsc_data_structures::span::{Span, WithSpan};

pub(super) fn ty(s: &mut ParserContext) -> Result<Ty> {
    let lo = s.peek().span.lo;
    let lhs = base(s)?;
    array_or_arrow(s, lhs, lo)
}

pub(super) fn array_or_arrow(s: &mut ParserContext<'_>, mut lhs: Ty, lo: u32) -> Result<Ty> {
    loop {
        if let Some(()) = opt(s, array)? {
            lhs = Ty {
                id: NodeId::default(),
                span: s.span(lo),
                kind: Box::new(TyKind::Array(Box::new(lhs))),
            }
        } else if let Some(kind) = opt(s, arrow)? {
            let output = ty(s)?;
            let functors = if token(s, TokenKind::Keyword(Keyword::Is)).is_ok() {
                Some(Box::new(functor_expr(s)?))
            } else {
                None
            };

            lhs = Ty {
                id: NodeId::default(),
                span: s.span(lo),
                kind: Box::new(TyKind::Arrow(
                    kind,
                    Box::new(lhs),
                    Box::new(output),
                    functors,
                )),
            }
        } else {
            break Ok(lhs);
        }
    }
}

pub(super) fn param(s: &mut ParserContext) -> Result<TyParam> {
    throw_away_doc(s);
    let lo = s.peek().span.lo;
    let generic = apos_ident(s)?;
    let bounds = if token(s, TokenKind::Colon).is_ok() {
        Some(ty_bounds(s)?)
    } else {
        None
    };

    Ok(TyParam::new(
        *generic,
        bounds.unwrap_or_else(|| TyBounds(Box::new([]))),
        s.span(lo),
    ))
}

fn ty_bounds(s: &mut ParserContext) -> Result<TyBounds> {
    let mut bounds = Vec::new();
    loop {
        bounds.push(*ident(s)?);
        if token(s, TokenKind::ClosedBinOp(ClosedBinOp::Plus)).is_err() {
            break;
        }
    }
    Ok(TyBounds(bounds.into_boxed_slice()))
}

fn array(s: &mut ParserContext) -> Result<()> {
    token(s, TokenKind::Open(Delim::Bracket))?;
    token(s, TokenKind::Close(Delim::Bracket))?;
    Ok(())
}

fn arrow(s: &mut ParserContext) -> Result<CallableKind> {
    if token(s, TokenKind::RArrow).is_ok() {
        Ok(CallableKind::Function)
    } else if token(s, TokenKind::FatArrow).is_ok() {
        Ok(CallableKind::Operation)
    } else {
        Err(Error::new(ErrorKind::Rule(
            "arrow type",
            s.peek().kind,
            s.peek().span,
        )))
    }
}

/// the base type of a type, which can be a hole, a type parameter, a path, or a parenthesized type
/// (or a tuple)
/// This parses the part before the arrow or array in a type, if an arrow or array is present.
fn base(s: &mut ParserContext) -> Result<Ty> {
    throw_away_doc(s);
    let lo = s.peek().span.lo;
    let kind = if token(s, TokenKind::Keyword(Keyword::Underscore)).is_ok() {
        Ok(TyKind::Hole)
    } else if let Some(name) = opt(s, param)? {
        Ok(TyKind::Param(name))
    } else if let Some(path) = opt(s, path)? {
        Ok(TyKind::Path(path))
    } else if token(s, TokenKind::Open(Delim::Paren)).is_ok() {
        let (tys, final_sep) = seq(s, ty)?;
        token(s, TokenKind::Close(Delim::Paren))?;
        Ok(final_sep.reify(tys, |t| TyKind::Paren(Box::new(t)), TyKind::Tuple))
    } else {
        Err(Error::new(ErrorKind::Rule(
            "type",
            s.peek().kind,
            s.peek().span,
        )))
    }?;

    Ok(Ty {
        id: NodeId::default(),
        span: s.span(lo),
        kind: Box::new(kind),
    })
}

pub(super) fn functor_expr(s: &mut ParserContext) -> Result<FunctorExpr> {
    // Intersection binds tighter than union.
    functor_op(s, ClosedBinOp::Plus, SetOp::Union, |s| {
        functor_op(s, ClosedBinOp::Star, SetOp::Intersect, functor_base)
    })
}

fn functor_base(s: &mut ParserContext) -> Result<FunctorExpr> {
    let lo = s.peek().span.lo;
    let kind = if token(s, TokenKind::Open(Delim::Paren)).is_ok() {
        let e = functor_expr(s)?;
        token(s, TokenKind::Close(Delim::Paren))?;
        Ok(FunctorExprKind::Paren(Box::new(e)))
    } else if token(s, TokenKind::Keyword(Keyword::Adj)).is_ok() {
        Ok(FunctorExprKind::Lit(Functor::Adj))
    } else if token(s, TokenKind::Keyword(Keyword::Ctl)).is_ok() {
        Ok(FunctorExprKind::Lit(Functor::Ctl))
    } else {
        Err(Error::new(ErrorKind::Rule(
            "functor literal",
            s.peek().kind,
            s.peek().span,
        )))
    }?;

    Ok(FunctorExpr {
        id: NodeId::default(),
        span: s.span(lo),
        kind: Box::new(kind),
    })
}

fn functor_op(
    s: &mut ParserContext,
    bin_op: ClosedBinOp,
    set_op: SetOp,
    mut p: impl Parser<FunctorExpr>,
) -> Result<FunctorExpr> {
    let lo = s.peek().span.lo;
    let mut lhs = p(s)?;

    while token(s, TokenKind::ClosedBinOp(bin_op)).is_ok() {
        let rhs = p(s)?;
        lhs = FunctorExpr {
            id: NodeId::default(),
            span: s.span(lo),
            kind: Box::new(FunctorExprKind::BinOp(set_op, Box::new(lhs), Box::new(rhs))),
        };
    }

    Ok(lhs)
}
