// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
pub(crate) mod tests;

use qsc_data_structures::span::Span;

use super::{
    completion::WordKinds,
    error::{Error, ErrorKind},
    expr::{self, designator},
    prim::{self, barrier, many, opt, recovering, recovering_semi, recovering_token},
    Result,
};
use crate::{
    ast::{
        Annotation, Block, IncludeStmt, LiteralKind, PathKind, Pragma, QubitDeclaration, Stmt,
        StmtKind,
    },
    lex::{cooked::Literal, Delim, TokenKind},
};

use super::{prim::token, ParserContext};

pub(super) fn parse(s: &mut ParserContext) -> Result<Box<Stmt>> {
    let lo = s.peek().span.lo;
    if let Some(pragma) = opt(s, parse_pragma)? {
        return Ok(Box::new(Stmt {
            span: s.span(lo),
            annotations: [].into(),
            kind: Box::new(StmtKind::Pragma(pragma)),
        }));
    }
    let attrs = many(s, parse_annotation)?;
    let kind = if token(s, TokenKind::Semicolon).is_ok() {
        if attrs.is_empty() {
            Box::new(StmtKind::Empty)
        } else {
            let err_item = default(s.span(lo));
            s.push_error(Error::new(ErrorKind::FloatingAnnotation(err_item.span)));
            return Ok(err_item);
        }
    } else if let Some(v) = opt(s, parse_include)? {
        Box::new(v)
    } else if let Some(decl) = opt(s, parse_quantum_decl)? {
        Box::new(decl)
    } else {
        return Err(Error::new(ErrorKind::Rule(
            "statement",
            s.peek().kind,
            s.peek().span,
        )));
    };

    Ok(Box::new(Stmt {
        span: s.span(lo),
        annotations: attrs.into_boxed_slice(),
        kind,
    }))
}

#[allow(clippy::vec_box)]
pub(super) fn parse_many(s: &mut ParserContext) -> Result<Vec<Box<Stmt>>> {
    many(s, |s| {
        recovering(s, default, &[TokenKind::Semicolon], parse)
    })
}

pub(super) fn parse_block(s: &mut ParserContext) -> Result<Box<Block>> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Open(Delim::Brace))?;
    let stmts = barrier(s, &[TokenKind::Close(Delim::Brace)], parse_many)?;
    recovering_token(s, TokenKind::Close(Delim::Brace));
    Ok(Box::new(Block {
        span: s.span(lo),
        stmts: stmts.into_boxed_slice(),
    }))
}

#[allow(clippy::unnecessary_box_returns)]
fn default(span: Span) -> Box<Stmt> {
    Box::new(Stmt {
        span,
        annotations: Vec::new().into_boxed_slice(),
        kind: Box::new(StmtKind::Err),
    })
}

fn parse_annotation(s: &mut ParserContext) -> Result<Box<Annotation>> {
    let lo = s.peek().span.lo;
    s.expect(WordKinds::Annotation);
    token(s, TokenKind::Annotation)?;
    // parse name
    // parse value
    recovering_semi(s);
    Ok(Box::new(Annotation {
        span: s.span(lo),
        name: Box::new(PathKind::default()),
        value: None,
    }))
}

fn parse_include(s: &mut ParserContext) -> Result<StmtKind> {
    let lo = s.peek().span.lo;
    s.expect(WordKinds::Include);
    token(s, TokenKind::Keyword(crate::keyword::Keyword::Include))?;
    let next = s.peek();

    let v = expr::lit(s)?;
    if let Some(v) = v {
        if let LiteralKind::String(v) = v.kind {
            let r = IncludeStmt {
                span: s.span(lo),
                filename: v.to_string(),
            };
            token(s, TokenKind::Semicolon)?;
            return Ok(StmtKind::Include(r));
        }
    };
    Err(Error::new(ErrorKind::Rule(
        "include statement",
        TokenKind::Literal(Literal::String),
        next.span,
    )))
}

fn parse_pragma(s: &mut ParserContext) -> Result<Pragma> {
    let lo = s.peek().span.lo;
    s.expect(WordKinds::Pragma);
    token(s, TokenKind::Keyword(crate::keyword::Keyword::Pragma))?;
    // parse name
    // parse value

    Ok(Pragma {
        span: s.span(lo),
        name: Box::new(PathKind::default()),
        value: None,
    })
}

fn parse_quantum_decl(s: &mut ParserContext) -> Result<StmtKind> {
    let lo = s.peek().span.lo;
    s.expect(WordKinds::Qubit);
    token(s, TokenKind::Keyword(crate::keyword::Keyword::Qubit))?;
    let size = opt(s, designator)?;
    let name = prim::ident(s)?;

    recovering_semi(s);
    Ok(StmtKind::QuantumDecl(QubitDeclaration {
        span: s.span(lo),
        qubit: *name,
        size,
    }))
}
