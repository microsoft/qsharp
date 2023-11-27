// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use super::{keyword::Keyword, scan::Scanner, ty::ty, Error, Parser, Result};
use crate::{
    lex::{Delim, TokenKind},
    ErrorKind,
};
use qsc_ast::ast::{Ident, NodeId, Pat, PatKind, Path};
use qsc_data_structures::span::{Span, WithSpan};

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

pub(super) fn token(s: &mut Scanner, t: TokenKind) -> Result<()> {
    if s.peek().kind == t {
        s.advance();
        Ok(())
    } else {
        Err(Error(ErrorKind::Token(t, s.peek().kind, s.peek().span)))
    }
}

pub(super) fn apos_ident(s: &mut Scanner) -> Result<Box<Ident>> {
    let peek = s.peek();
    if peek.kind == TokenKind::AposIdent {
        let name = s.read().into();
        s.advance();
        Ok(Box::new(Ident {
            id: NodeId::default(),
            span: peek.span,
            name,
        }))
    } else {
        Err(Error(ErrorKind::Rule(
            "generic parameter",
            peek.kind,
            peek.span,
        )))
    }
}

pub(super) fn ident(s: &mut Scanner) -> Result<Box<Ident>> {
    let peek = s.peek();
    if peek.kind == TokenKind::Ident {
        let name = s.read().into();
        s.advance();
        Ok(Box::new(Ident {
            id: NodeId::default(),
            span: peek.span,
            name,
        }))
    } else {
        Err(Error(ErrorKind::Rule("identifier", peek.kind, peek.span)))
    }
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
    let kind = if token(s, TokenKind::Keyword(Keyword::Underscore)).is_ok() {
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
    let offset = s.peek().span.lo;
    match p(s) {
        Ok(x) => Ok(Some(x)),
        Err(error) if advanced(s, offset) => Err(error),
        Err(_) => Ok(None),
    }
}

pub(super) fn many<T>(s: &mut Scanner, mut p: impl Parser<T>) -> Result<Vec<T>> {
    let mut xs = Vec::new();
    while let Some(x) = opt(s, &mut p)? {
        xs.push(x);
    }
    Ok(xs)
}

pub(super) fn seq<T>(s: &mut Scanner, mut p: impl Parser<T>) -> Result<(Vec<T>, FinalSep)>
where
    T: Default + WithSpan,
{
    let mut xs = Vec::new();
    let mut final_sep = FinalSep::Missing;
    while s.peek().kind == TokenKind::Comma {
        let mut span = s.peek().span;
        span.hi = span.lo;
        s.push_error(Error(ErrorKind::MissingSeqEntry(span)));
        xs.push(T::default().with_span(span));
        s.advance();
    }
    while let Some(x) = opt(s, &mut p)? {
        xs.push(x);
        if token(s, TokenKind::Comma).is_ok() {
            while s.peek().kind == TokenKind::Comma {
                let mut span = s.peek().span;
                span.hi = span.lo;
                s.push_error(Error(ErrorKind::MissingSeqEntry(span)));
                xs.push(T::default().with_span(span));
                s.advance();
            }
            final_sep = FinalSep::Present;
        } else {
            final_sep = FinalSep::Missing;
            break;
        }
    }
    Ok((xs, final_sep))
}

pub(super) fn recovering<T>(
    s: &mut Scanner,
    default: impl FnOnce(Span) -> T,
    tokens: &[TokenKind],
    mut p: impl Parser<T>,
) -> Result<T> {
    let offset = s.peek().span.lo;
    match p(s) {
        Ok(value) => Ok(value),
        Err(error) if advanced(s, offset) => {
            s.push_error(error);
            s.recover(tokens);
            Ok(default(s.span(offset)))
        }
        Err(error) => Err(error),
    }
}

pub(super) fn recovering_semi(s: &mut Scanner) -> Result<()> {
    match token(s, TokenKind::Semi) {
        Ok(()) => Ok(()),
        Err(error) => {
            s.push_error(error);
            // no recovery, just move on to the next token
            Ok(())
        }
    }
}

pub(super) fn recovering_token(s: &mut Scanner, t: TokenKind) -> Result<()> {
    match token(s, t) {
        Ok(()) => Ok(()),
        Err(error) => {
            s.push_error(error);
            s.recover(&[t]);
            Ok(())
        }
    }
}

pub(super) fn barrier<'a, T>(
    s: &mut Scanner<'a>,
    tokens: &'a [TokenKind],
    mut p: impl Parser<T>,
) -> Result<T> {
    s.push_barrier(tokens);
    let result = p(s);
    s.pop_barrier().expect("barrier should be popped");
    result
}

pub(super) fn shorten(from_start: usize, from_end: usize, s: &str) -> &str {
    &s[from_start..s.len() - from_end]
}

fn advanced(s: &Scanner, from: u32) -> bool {
    s.peek().span.lo > from
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
    Error(match error.0 {
        ErrorKind::Rule(_, found, span) => ErrorKind::Rule(name, found, span),
        ErrorKind::Convert(_, found, span) => ErrorKind::Convert(name, found, span),
        kind => kind,
    })
}
