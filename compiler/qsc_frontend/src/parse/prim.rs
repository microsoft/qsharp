// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use super::{keyword::Keyword, scan::Scanner, ty::ty, Error, Parser, Result};
use crate::lex::{Delim, TokenKind};
use qsc_ast::ast::{Ident, NodeId, Pat, PatKind, Path};
use qsc_data_structures::span::Span;
use std::{cell::RefCell, str::FromStr};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FinalSep {
    Present,
    Missing,
}

impl FinalSep {
    pub(super) fn reify<T, U>(
        self,
        mut xs: Vec<T>,
        mut as_paren: impl FnMut(T) -> U,
        mut as_seq: impl FnMut(Box<[T]>) -> U,
    ) -> U {
        if self == Self::Missing && xs.len() == 1 {
            as_paren(xs.pop().expect("vector should have exactly one item"))
        } else {
            as_seq(xs.into_boxed_slice())
        }
    }
}

// omg why
thread_local!(pub(super) static CURRENT_COMMENT: RefCell<Vec<String>> = RefCell::new(Vec::new()));

pub(super) fn consume_comments(s: &mut Scanner) {
    let mut comments = Vec::new();
    while let Some(x) = {
        match comment(s) {
            Ok(x) => Some(x),
            Err(_) => None,
        }
    } {
        comments.push(x);
    }

    if !comments.is_empty() {
        CURRENT_COMMENT.with(|c| *c.borrow_mut() = comments);
    }
}

pub(super) fn comment(s: &mut Scanner) -> Result<String> {
    if s.peek().kind == TokenKind::Comment {
        let comment = s.read();
        eprintln!("comment: {comment}");
        s.advance();
        Ok(comment.to_string())
    } else {
        Err(Error::Rule("comment", s.peek().kind, s.peek().span))
    }
}

pub(super) fn token(s: &mut Scanner, kind: TokenKind) -> Result<()> {
    if s.peek().kind == kind {
        s.advance();
        Ok(())
    } else {
        Err(Error::Token(kind, s.peek().kind, s.peek().span))
    }
}

pub(super) fn keyword(s: &mut Scanner, kw: Keyword) -> Result<()> {
    if s.peek().kind == TokenKind::Ident && s.read() == kw.as_str() {
        s.advance();
        Ok(())
    } else {
        Err(Error::Keyword(kw, s.peek().kind, s.peek().span))
    }
}

pub(super) fn ident(s: &mut Scanner) -> Result<Box<Ident>> {
    if s.peek().kind != TokenKind::Ident {
        return Err(Error::Rule("identifier", s.peek().kind, s.peek().span));
    } else if let Ok(kw) = Keyword::from_str(s.read()) {
        return Err(Error::RuleKeyword("identifier", kw, s.peek().span));
    }

    let span = s.peek().span;
    let name = s.read().into();
    s.advance();
    Ok(Box::new(Ident {
        id: NodeId::default(),
        span,
        name,
    }))
}

pub(super) fn dot_ident(s: &mut Scanner) -> Result<Box<Ident>> {
    let p = path(s)?;
    let mut name = String::new();
    if let Some(namespace) = p.namespace {
        name.push_str(&namespace.name);
        name.push('.');
    }
    name.push_str(&p.name.name);

    Ok(Box::new(Ident {
        id: p.id,
        span: p.span,
        name: name.into(),
    }))
}

pub(super) fn path(s: &mut Scanner) -> Result<Box<Path>> {
    let lo = s.peek().span.lo;
    let mut parts = vec![ident(s)?];
    while token(s, TokenKind::Dot).is_ok() {
        parts.push(ident(s)?);
    }

    let name = parts.pop().expect("path should have at least one part");
    let namespace = match (parts.first(), parts.last()) {
        (Some(first), Some(last)) => {
            let lo = first.span.lo;
            let hi = last.span.hi;
            Some(Box::new(Ident {
                id: NodeId::default(),
                span: Span { lo, hi },
                name: join(parts.iter().map(|i| &i.name), ".").into(),
            }))
        }
        _ => None,
    };

    Ok(Box::new(Path {
        id: NodeId::default(),
        span: s.span(lo),
        namespace,
        name,
    }))
}

pub(super) fn pat(s: &mut Scanner) -> Result<Box<Pat>> {
    let lo = s.peek().span.lo;
    let kind = if keyword(s, Keyword::Underscore).is_ok() {
        let ty = if token(s, TokenKind::Colon).is_ok() {
            Some(Box::new(ty(s)?))
        } else {
            None
        };
        Ok(PatKind::Discard(ty))
    } else if token(s, TokenKind::DotDotDot).is_ok() {
        Ok(PatKind::Elided)
    } else if token(s, TokenKind::Open(Delim::Paren)).is_ok() {
        let (pats, final_sep) = seq(s, pat)?;
        token(s, TokenKind::Close(Delim::Paren))?;
        Ok(final_sep.reify(pats, PatKind::Paren, PatKind::Tuple))
    } else {
        let name = ident(s).map_err(|e| map_rule_name("pattern", e))?;
        let ty = if token(s, TokenKind::Colon).is_ok() {
            Some(Box::new(ty(s)?))
        } else {
            None
        };
        Ok(PatKind::Bind(name, ty))
    }?;

    Ok(Box::new(Pat {
        id: NodeId::default(),
        span: s.span(lo),
        kind: Box::new(kind),
    }))
}

pub(super) fn opt<T>(s: &mut Scanner, mut p: impl Parser<T>) -> Result<Option<T>> {
    consume_comments(s);
    let offset = s.peek().span.lo;
    match p(s) {
        Ok(x) => Ok(Some(x)),
        Err(_) if offset == s.peek().span.lo => Ok(None),
        Err(err) => Err(err),
    }
}

pub(super) fn try_many<T>(s: &mut Scanner, mut p: impl Parser<T>) -> Vec<T> {
    let mut xs = Vec::new();
    while let Ok(r) = opt(s, &mut p) {
        match r {
            Some(x) => xs.push(x),
            None => break,
        };
    }
    xs
}

pub(super) fn many<T>(s: &mut Scanner, mut p: impl Parser<T>) -> Result<Vec<T>> {
    let mut xs = Vec::new();
    while let Some(x) = opt(s, &mut p)? {
        xs.push(x);
    }
    Ok(xs)
}

pub(super) fn seq<T>(s: &mut Scanner, mut p: impl Parser<T>) -> Result<(Vec<T>, FinalSep)> {
    let mut xs = Vec::new();
    let mut final_sep = FinalSep::Missing;
    while let Some(x) = opt(s, &mut p)? {
        xs.push(x);
        if token(s, TokenKind::Comma).is_ok() {
            final_sep = FinalSep::Present;
        } else {
            final_sep = FinalSep::Missing;
            break;
        }
    }
    Ok((xs, final_sep))
}

fn join(mut strings: impl Iterator<Item = impl AsRef<str>>, sep: &str) -> String {
    let mut string = String::new();
    if let Some(s) = strings.next() {
        string.push_str(s.as_ref());
    }
    for s in strings {
        string.push_str(sep);
        string.push_str(s.as_ref());
    }
    string
}

fn map_rule_name(name: &'static str, error: Error) -> Error {
    match error {
        Error::Rule(_, found, span) => Error::Rule(name, found, span),
        Error::RuleKeyword(_, keyword, span) => Error::RuleKeyword(name, keyword, span),
        Error::Convert(_, found, span) => Error::Convert(name, found, span),
        _ => error,
    }
}
