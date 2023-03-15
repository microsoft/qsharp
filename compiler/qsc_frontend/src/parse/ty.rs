// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use super::{
    keyword::Keyword,
    prim::{ident, keyword, opt, path, seq, token},
    scan::Scanner,
    Error, Parser, Result,
};
use crate::lex::{ClosedBinOp, Delim, TokenKind};
use qsc_ast::ast::{
    CallableKind, Functor, FunctorExpr, FunctorExprKind, Ident, NodeId, SetOp, Ty, TyKind, TyPrim,
    TyVar,
};

pub(super) fn ty(s: &mut Scanner) -> Result<Ty> {
    let lo = s.peek().span.lo;
    let mut lhs = base(s)?;
    loop {
        if let Some(array) = opt(s, array)? {
            lhs = Ty {
                id: NodeId::default(),
                span: s.span(lo),
                kind: TyKind::App(Box::new(array), vec![lhs]),
            }
        } else if let Some(kind) = opt(s, arrow)? {
            let output = ty(s)?;
            let functors = if keyword(s, Keyword::Is).is_ok() {
                Some(functor_expr(s)?)
            } else {
                None
            };

            lhs = Ty {
                id: NodeId::default(),
                span: s.span(lo),
                kind: TyKind::Arrow(kind, Box::new(lhs), Box::new(output), functors),
            }
        } else {
            break Ok(lhs);
        }
    }
}

pub(super) fn var(s: &mut Scanner) -> Result<Ident> {
    token(s, TokenKind::Apos)?;
    ident(s)
}

fn array(s: &mut Scanner) -> Result<Ty> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Open(Delim::Bracket))?;
    token(s, TokenKind::Close(Delim::Bracket))?;
    Ok(Ty {
        id: NodeId::default(),
        span: s.span(lo),
        kind: TyKind::Prim(TyPrim::Array),
    })
}

fn arrow(s: &mut Scanner) -> Result<CallableKind> {
    if token(s, TokenKind::RArrow).is_ok() {
        Ok(CallableKind::Function)
    } else if token(s, TokenKind::FatArrow).is_ok() {
        Ok(CallableKind::Operation)
    } else {
        Err(Error::Rule("arrow type", s.peek().kind, s.peek().span))
    }
}

fn base(s: &mut Scanner) -> Result<Ty> {
    let lo = s.peek().span.lo;
    let kind = if keyword(s, Keyword::Underscore).is_ok() {
        Ok(TyKind::Hole)
    } else if keyword(s, Keyword::BigInt).is_ok() {
        Ok(TyKind::Prim(TyPrim::BigInt))
    } else if keyword(s, Keyword::Bool).is_ok() {
        Ok(TyKind::Prim(TyPrim::Bool))
    } else if keyword(s, Keyword::Double).is_ok() {
        Ok(TyKind::Prim(TyPrim::Double))
    } else if keyword(s, Keyword::Int).is_ok() {
        Ok(TyKind::Prim(TyPrim::Int))
    } else if keyword(s, Keyword::Pauli).is_ok() {
        Ok(TyKind::Prim(TyPrim::Pauli))
    } else if keyword(s, Keyword::Qubit).is_ok() {
        Ok(TyKind::Prim(TyPrim::Qubit))
    } else if keyword(s, Keyword::Range).is_ok() {
        Ok(TyKind::Prim(TyPrim::Range))
    } else if keyword(s, Keyword::Result).is_ok() {
        Ok(TyKind::Prim(TyPrim::Result))
    } else if keyword(s, Keyword::String).is_ok() {
        Ok(TyKind::Prim(TyPrim::String))
    } else if keyword(s, Keyword::Unit).is_ok() {
        Ok(TyKind::Tuple(Vec::new()))
    } else if let Some(var) = opt(s, var)? {
        Ok(TyKind::Var(TyVar::Name(var.name)))
    } else if let Some(path) = opt(s, path)? {
        Ok(TyKind::Path(path))
    } else if token(s, TokenKind::Open(Delim::Paren)).is_ok() {
        let (tys, final_sep) = seq(s, ty)?;
        token(s, TokenKind::Close(Delim::Paren))?;
        Ok(final_sep.reify(tys, |t| TyKind::Paren(Box::new(t)), TyKind::Tuple))
    } else {
        Err(Error::Rule("type", s.peek().kind, s.peek().span))
    }?;

    Ok(Ty {
        id: NodeId::default(),
        span: s.span(lo),
        kind,
    })
}

pub(super) fn functor_expr(s: &mut Scanner) -> Result<FunctorExpr> {
    // Intersection binds tighter than union.
    functor_op(s, ClosedBinOp::Plus, SetOp::Union, |s| {
        functor_op(s, ClosedBinOp::Star, SetOp::Intersect, functor_base)
    })
}

fn functor_base(s: &mut Scanner) -> Result<FunctorExpr> {
    let lo = s.peek().span.lo;
    let kind = if token(s, TokenKind::Open(Delim::Paren)).is_ok() {
        let e = functor_expr(s)?;
        token(s, TokenKind::Close(Delim::Paren))?;
        Ok(FunctorExprKind::Paren(Box::new(e)))
    } else if keyword(s, Keyword::Adj).is_ok() {
        Ok(FunctorExprKind::Lit(Functor::Adj))
    } else if keyword(s, Keyword::Ctl).is_ok() {
        Ok(FunctorExprKind::Lit(Functor::Ctl))
    } else {
        Err(Error::Rule("functor literal", s.peek().kind, s.peek().span))
    }?;

    Ok(FunctorExpr {
        id: NodeId::default(),
        span: s.span(lo),
        kind,
    })
}

fn functor_op(
    s: &mut Scanner,
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
            kind: FunctorExprKind::BinOp(set_op, Box::new(lhs), Box::new(rhs)),
        };
    }

    Ok(lhs)
}
