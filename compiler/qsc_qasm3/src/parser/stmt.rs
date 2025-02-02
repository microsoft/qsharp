// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::span::Span;

use super::{
    completion::WordKinds,
    error::{Error, ErrorKind},
    expr::{self},
    prim::{barrier, many, opt, recovering, recovering_path, recovering_token},
    Result,
};
use crate::{
    ast::{Annotation, Block, IncludeStmt, LiteralKind, Stmt, StmtKind},
    lex::{cooked::Literal, Delim, TokenKind},
};

use super::{prim::token, ParserContext};

pub(super) fn parse(s: &mut ParserContext) -> Result<Box<Stmt>> {
    let lo = s.peek().span.lo;
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
    token(s, TokenKind::At)?;
    s.expect(WordKinds::Annotation);
    let name = recovering_path(s, WordKinds::empty())?;

    Ok(Box::new(Annotation {
        span: s.span(lo),
        name: Box::new(name),
        value: None,
    }))
}

fn parse_include(s: &mut ParserContext) -> Result<StmtKind> {
    let lo = s.peek().span.lo;
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
