// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    prim::{many, opt, recovering, recovering_semi, recovering_token, token},
    stmt, Result,
};
use crate::{
    ast::{Program, Stmt, StmtKind, Version},
    lex::{Delim, TokenKind},
    parser::{completion::WordKinds, expr},
};

use super::ParserContext;

/// Grammar: `version? statementOrScope* EOF`.
pub(super) fn parse(s: &mut ParserContext) -> Result<Program> {
    let lo = s.peek().span.lo;
    let version = opt(s, parse_version)?;
    let stmts = parse_top_level_nodes(s)?;

    Ok(Program {
        span: s.span(lo),
        version,
        statements: stmts
            .into_iter()
            .map(Box::new)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    })
}

/// Grammar: `OPENQASM VersionSpecifier SEMICOLON`.
fn parse_version(s: &mut ParserContext<'_>) -> Result<Version> {
    s.expect(WordKinds::OpenQASM);
    token(s, TokenKind::Keyword(crate::keyword::Keyword::OpenQASM))?;
    let next = s.peek();
    if let Some(version) = expr::version(s)? {
        recovering_semi(s);
        Ok(version)
    } else {
        Err(crate::parser::error::Error::new(
            crate::parser::error::ErrorKind::Lit("version", next.span),
        ))
    }
}

pub(super) fn parse_top_level_nodes(s: &mut ParserContext) -> Result<Vec<Stmt>> {
    const RECOVERY_TOKENS: &[TokenKind] = &[TokenKind::Semicolon, TokenKind::Close(Delim::Brace)];
    let nodes = {
        many(s, |s| {
            recovering(
                s,
                |span| Stmt {
                    span,
                    annotations: Vec::new().into_boxed_slice(),
                    kind: Box::new(StmtKind::Err),
                },
                RECOVERY_TOKENS,
                parse_top_level_node,
            )
        })
    }?;
    recovering_token(s, TokenKind::Eof);
    Ok(nodes)
}

fn parse_top_level_node(s: &mut ParserContext) -> Result<Stmt> {
    if let Some(block) = opt(s, stmt::parse_block)? {
        Ok(Stmt {
            span: block.span,
            annotations: Vec::new().into_boxed_slice(),
            kind: Box::new(StmtKind::Block(block)),
        })
    } else {
        Ok(*stmt::parse(s)?)
    }
}
