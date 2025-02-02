// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    prim::{many, opt, recovering, recovering_token, token},
    stmt, Result,
};
use crate::{
    ast::{LiteralKind, Program, Stmt, StmtKind},
    lex::{cooked::Literal, Delim, TokenKind},
    parser::{completion::WordKinds, expr},
};

use super::ParserContext;

pub(super) fn parse(s: &mut ParserContext) -> Result<Program> {
    let lo = s.peek().span.lo;
    let version = opt(s, parse_version)?;
    let stmts = parse_top_level_nodes(s)?;

    Ok(Program {
        span: s.span(lo),
        version,
        statements: stmts.into_boxed_slice(),
    })
}

fn parse_version(s: &mut ParserContext<'_>) -> Result<String> {
    let lo = s.peek().span.lo;
    s.expect(WordKinds::OpenQASM);
    token(s, TokenKind::Keyword(crate::keyword::Keyword::OpenQASM))?;
    let next = s.peek();
    if matches!(next.kind, TokenKind::Literal(Literal::Float)) {
        let l = expr::lit(s)?.expect("msg");
        match l.kind {
            LiteralKind::Float(f) => Ok(f.to_string()),
            _ => Err(crate::parser::error::Error::new(
                crate::parser::error::ErrorKind::Lit("version", next.span),
            )),
        }
    } else {
        Err(crate::parser::error::Error::new(
            crate::parser::error::ErrorKind::Rule("version number", next.kind, next.span),
        ))
    }
}

pub(super) fn parse_top_level_nodes(s: &mut ParserContext) -> Result<Vec<Box<Stmt>>> {
    const RECOVERY_TOKENS: &[TokenKind] = &[TokenKind::Semicolon, TokenKind::Close(Delim::Brace)];
    let nodes = {
        many(s, |s| {
            recovering(
                s,
                |span| {
                    Box::new(Stmt {
                        span,
                        annotations: Vec::new().into_boxed_slice(),
                        kind: Box::new(StmtKind::Err),
                    })
                },
                RECOVERY_TOKENS,
                parse_top_level_node,
            )
        })
    }?;
    recovering_token(s, TokenKind::Eof);
    Ok(nodes)
}

fn parse_top_level_node(s: &mut ParserContext) -> Result<Box<Stmt>> {
    if let Some(block) = opt(s, stmt::parse_block)? {
        Ok(Box::new(Stmt {
            span: block.span,
            annotations: Vec::new().into_boxed_slice(),
            kind: Box::new(StmtKind::Block(block)),
        }))
    } else {
        stmt::parse(s)
    }
}
