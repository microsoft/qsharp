// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    expr::expr,
    keyword::Keyword,
    prim::{keyword, many, opt, pat, seq, token, FinalSep},
    scan::Scanner,
    ErrorKind, Result,
};
use crate::lex::{Delim, TokenKind};
use qsc_ast::ast::{Block, NodeId, QubitInit, QubitInitKind, Stmt, StmtKind};

pub(super) fn block(s: &mut Scanner) -> Result<Block> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Open(Delim::Brace))?;
    let stmts = many(s, stmt)?;
    token(s, TokenKind::Close(Delim::Brace))?;
    Ok(Block {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
        stmts,
    })
}

pub(super) fn stmt(s: &mut Scanner) -> Result<Stmt> {
    let lo = s.peek().span.lo;
    let kind = if let Some(var) = opt(s, var_binding)? {
        Ok(var)
    } else if let Some(qubit) = opt(s, qubit_binding)? {
        Ok(qubit)
    } else {
        let e = expr(s)?;
        if token(s, TokenKind::Semi).is_ok() {
            Ok(StmtKind::Semi(e))
        } else {
            Ok(StmtKind::Expr(e))
        }
    }?;

    Ok(Stmt {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
        kind,
    })
}

fn var_binding(s: &mut Scanner) -> Result<StmtKind> {
    let mutable = if keyword(s, Keyword::Let).is_ok() {
        Ok(false)
    } else if keyword(s, Keyword::Mutable).is_ok() {
        Ok(true)
    } else {
        Err(s.error(ErrorKind::Rule("variable binding")))
    }?;

    let lhs = pat(s)?;
    token(s, TokenKind::Eq)?;
    let rhs = expr(s)?;
    token(s, TokenKind::Semi)?;
    if mutable {
        Ok(StmtKind::Mutable(lhs, rhs))
    } else {
        Ok(StmtKind::Let(lhs, rhs))
    }
}

fn qubit_binding(s: &mut Scanner) -> Result<StmtKind> {
    let borrow = if keyword(s, Keyword::Use).is_ok() {
        Ok(false)
    } else if keyword(s, Keyword::Borrow).is_ok() {
        Ok(true)
    } else {
        Err(s.error(ErrorKind::Rule("qubit binding")))
    }?;

    let lhs = pat(s)?;
    token(s, TokenKind::Eq)?;
    let rhs = qubit_init(s)?;
    let scope = opt(s, block)?;
    if scope.is_none() {
        token(s, TokenKind::Semi)?;
    }

    if borrow {
        Ok(StmtKind::Borrow(lhs, rhs, scope))
    } else {
        Ok(StmtKind::Use(lhs, rhs, scope))
    }
}

fn qubit_init(s: &mut Scanner) -> Result<QubitInit> {
    let lo = s.peek().span.lo;
    let kind = if keyword(s, Keyword::Qubit).is_ok() {
        if token(s, TokenKind::Open(Delim::Paren)).is_ok() {
            token(s, TokenKind::Close(Delim::Paren))?;
            Ok(QubitInitKind::Single)
        } else if token(s, TokenKind::Open(Delim::Bracket)).is_ok() {
            let size = expr(s)?;
            Ok(QubitInitKind::Array(Box::new(size)))
        } else {
            Err(s.error(ErrorKind::Rule("qubit initializer")))
        }
    } else if token(s, TokenKind::Open(Delim::Paren)).is_ok() {
        let (mut inits, final_sep) = seq(s, qubit_init)?;
        if final_sep == FinalSep::Missing && inits.len() == 1 {
            let init = inits.pop().expect("Sequence has exactly one initializer.");
            Ok(QubitInitKind::Paren(Box::new(init)))
        } else {
            Ok(QubitInitKind::Tuple(inits))
        }
    } else {
        Err(s.error(ErrorKind::Rule("qubit initializer")))
    }?;

    Ok(QubitInit {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
        kind,
    })
}
