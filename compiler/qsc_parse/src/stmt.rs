// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use super::{
    expr::{self, expr, expr_stmt},
    item,
    keyword::Keyword,
    prim::{ident, many, opt, pat, seq, token},
    scan::ParserContext,
    Error, Result,
};
use crate::{
    lex::{Delim, TokenKind},
    prim::{barrier, keyword, recovering, recovering_semi, recovering_token},
    ErrorKind,
};
use qsc_ast::ast::{
    Block, Mutability, NodeId, QubitInit, QubitInitKind, QubitSource, Stmt, StmtKind,
};
use qsc_data_structures::{language_features::LanguageFeatures, span::Span};

pub(super) fn parse(s: &mut ParserContext) -> Result<Box<Stmt>> {
    let lo = s.peek().span.lo;
    let kind = if token(s, TokenKind::Semi).is_ok() {
        Box::new(StmtKind::Empty)
    } else if let Some(item) = opt(s, item::parse)? {
        Box::new(StmtKind::Item(item))
    } else if let Some(local) = opt(s, parse_local)? {
        local
    } else if let Some(qubit) = opt(s, parse_qubit)? {
        qubit
    } else {
        let e = expr_stmt(s)?;
        if token(s, TokenKind::Semi).is_ok() {
            Box::new(StmtKind::Semi(e))
        } else {
            Box::new(StmtKind::Expr(e))
        }
    };

    Ok(Box::new(Stmt {
        id: NodeId::default(),
        span: s.span(lo),
        kind,
    }))
}

#[allow(clippy::vec_box)]
pub(super) fn parse_many(s: &mut ParserContext) -> Result<Vec<Box<Stmt>>> {
    many(s, |s| recovering(s, default, &[TokenKind::Semi], parse))
}

pub(super) fn parse_block(s: &mut ParserContext) -> Result<Box<Block>> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Open(Delim::Brace))?;
    let stmts = barrier(s, &[TokenKind::Close(Delim::Brace)], parse_many)?;
    check_semis(s, &stmts);
    recovering_token(s, TokenKind::Close(Delim::Brace));
    Ok(Box::new(Block {
        id: NodeId::default(),
        span: s.span(lo),
        stmts: stmts.into_boxed_slice(),
    }))
}

#[allow(clippy::unnecessary_box_returns)]
fn default(span: Span) -> Box<Stmt> {
    Box::new(Stmt {
        id: NodeId::default(),
        span,
        kind: Box::new(StmtKind::Err),
    })
}

fn parse_local(s: &mut ParserContext) -> Result<Box<StmtKind>> {
    let mutability = if keyword(s, Keyword::Let).is_ok() {
        Mutability::Immutable
    } else if keyword(s, Keyword::Mutable).is_ok() {
        Mutability::Mutable
    } else {
        let token = s.peek();
        return Err(Error(ErrorKind::Rule(
            "variable binding",
            token.kind,
            token.span,
        )));
    };

    let lhs = pat(s)?;
    token(s, TokenKind::Eq)?;
    let rhs = expr(s)?;
    recovering_semi(s);
    Ok(Box::new(StmtKind::Local(mutability, lhs, rhs)))
}

fn parse_qubit(s: &mut ParserContext) -> Result<Box<StmtKind>> {
    let source = if keyword(s, Keyword::Use).is_ok() {
        QubitSource::Fresh
    } else if keyword(s, Keyword::Borrow).is_ok() {
        QubitSource::Dirty
    } else {
        return Err(Error(ErrorKind::Rule(
            "qubit binding",
            s.peek().kind,
            s.peek().span,
        )));
    };

    let lhs = pat(s)?;
    token(s, TokenKind::Eq)?;
    let rhs = parse_qubit_init(s)?;
    let block = if s.contains_language_feature(LanguageFeatures::V2PreviewSyntax) {
        None
    } else {
        opt(s, parse_block)?
    };

    if block.is_none() {
        recovering_semi(s);
    }

    Ok(Box::new(StmtKind::Qubit(source, lhs, rhs, block)))
}

fn parse_qubit_init(s: &mut ParserContext) -> Result<Box<QubitInit>> {
    let lo = s.peek().span.lo;
    s.push_prediction(vec![crate::Prediction::Qubit]);
    let kind = if let Ok(name) = ident(s) {
        if name.name.as_ref() != "Qubit" {
            return Err(Error(ErrorKind::Convert(
                "qubit initializer",
                "identifier",
                name.span,
            )));
        } else if token(s, TokenKind::Open(Delim::Paren)).is_ok() {
            token(s, TokenKind::Close(Delim::Paren))?;
            QubitInitKind::Single
        } else if token(s, TokenKind::Open(Delim::Bracket)).is_ok() {
            let size = expr(s)?;
            token(s, TokenKind::Close(Delim::Bracket))?;
            QubitInitKind::Array(size)
        } else {
            let token = s.peek();
            return Err(Error(ErrorKind::Rule(
                "qubit suffix",
                token.kind,
                token.span,
            )));
        }
    } else if token(s, TokenKind::Open(Delim::Paren)).is_ok() {
        let (inits, final_sep) = seq(s, parse_qubit_init)?;
        token(s, TokenKind::Close(Delim::Paren))?;
        final_sep.reify(inits, QubitInitKind::Paren, QubitInitKind::Tuple)
    } else {
        let token = s.peek();
        return Err(Error(ErrorKind::Rule(
            "qubit initializer",
            token.kind,
            token.span,
        )));
    };

    Ok(Box::new(QubitInit {
        id: NodeId::default(),
        span: s.span(lo),
        kind: Box::new(kind),
    }))
}

pub(super) fn check_semis(s: &mut ParserContext, stmts: &[Box<Stmt>]) {
    let leading_stmts = stmts.split_last().map_or([].as_slice(), |s| s.1);
    for stmt in leading_stmts {
        if matches!(&*stmt.kind, StmtKind::Expr(expr) if !expr::is_stmt_final(&expr.kind)) {
            let span = Span {
                lo: stmt.span.hi,
                hi: stmt.span.hi,
            };
            s.push_error(Error(ErrorKind::MissingSemi(span)));
        }
    }
}
