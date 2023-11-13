// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use super::{
    expr::expr,
    keyword::Keyword,
    prim::{dot_ident, ident, many, opt, pat, seq, token},
    scan::Scanner,
    stmt,
    ty::{self, ty},
    Error, Result,
};
use crate::{
    lex::{Delim, TokenKind},
    prim::{barrier, recovering, recovering_token, shorten},
    stmt::check_semis,
    ty::array_or_arrow,
    ErrorKind,
};
use qsc_ast::ast::{
    Attr, Block, CallableBody, CallableDecl, CallableKind, Ident, Item, ItemKind, Namespace,
    NodeId, Pat, PatKind, Path, Spec, SpecBody, SpecDecl, SpecGen, StmtKind, TopLevelNode, Ty,
    TyDef, TyDefKind, TyKind, Visibility, VisibilityKind,
};
use qsc_data_structures::span::Span;

pub(super) fn parse(s: &mut Scanner) -> Result<Box<Item>> {
    let lo = s.peek().span.lo;
    let doc = parse_doc(s);
    let attrs = many(s, parse_attr)?;
    let visibility = opt(s, parse_visibility)?;
    let kind = if let Some(open) = opt(s, parse_open)? {
        open
    } else if let Some(ty) = opt(s, parse_newtype)? {
        ty
    } else if let Some(callable) = opt(s, parse_callable_decl)? {
        Box::new(ItemKind::Callable(callable))
    } else if visibility.is_some() {
        let err_item = default(s.span(lo));
        s.push_error(Error(ErrorKind::FloatingVisibility(err_item.span)));
        return Ok(err_item);
    } else if !attrs.is_empty() {
        let err_item = default(s.span(lo));
        s.push_error(Error(ErrorKind::FloatingAttr(err_item.span)));
        return Ok(err_item);
    } else if doc.is_some() {
        let err_item = default(s.span(lo));
        s.push_error(Error(ErrorKind::FloatingDocComment(err_item.span)));
        return Ok(err_item);
    } else {
        let p = s.peek();
        return Err(Error(ErrorKind::Rule("item", p.kind, p.span)));
    };

    Ok(Box::new(Item {
        id: NodeId::default(),
        span: s.span(lo),
        doc: doc.unwrap_or_default().into(),
        attrs: attrs.into_boxed_slice(),
        visibility,
        kind,
    }))
}

#[allow(clippy::vec_box)]
fn parse_many(s: &mut Scanner) -> Result<Vec<Box<Item>>> {
    const BARRIER_TOKENS: &[TokenKind] = &[
        TokenKind::At,
        TokenKind::Keyword(Keyword::Internal),
        TokenKind::Keyword(Keyword::Open),
        TokenKind::Keyword(Keyword::Newtype),
        TokenKind::Keyword(Keyword::Operation),
        TokenKind::Keyword(Keyword::Function),
    ];

    const RECOVERY_TOKENS: &[TokenKind] = &[TokenKind::Semi, TokenKind::Close(Delim::Brace)];

    barrier(s, BARRIER_TOKENS, |s| {
        many(s, |s| recovering(s, default, RECOVERY_TOKENS, parse))
    })
}

fn default(span: Span) -> Box<Item> {
    Box::new(Item {
        id: NodeId::default(),
        span,
        doc: "".into(),
        attrs: Vec::new().into_boxed_slice(),
        visibility: None,
        kind: Box::new(ItemKind::Err),
    })
}

pub(super) fn parse_namespaces(s: &mut Scanner) -> Result<Vec<Namespace>> {
    let namespaces = many(s, parse_namespace)?;
    recovering_token(s, TokenKind::Eof)?;
    Ok(namespaces)
}

pub(super) fn parse_top_level_nodes(s: &mut Scanner) -> Result<Vec<TopLevelNode>> {
    let nodes = many(s, parse_top_level_node)?;
    recovering_token(s, TokenKind::Eof)?;
    Ok(nodes)
}

fn parse_top_level_node(s: &mut Scanner) -> Result<TopLevelNode> {
    // Here we parse any doc comments ahead of calling `parse_namespace` or `stmt::parse` in order
    // to avoid problems with error reporting. Specifically, if `parse_namespace` consumes the
    // doc comment and then fails to find a namespace, that becomes an unrecoverable error even with
    // opt. This pattern can be dropped along with namespaces once we have a module-based design.
    let doc = parse_doc(s).unwrap_or_default();
    if let Some(mut namespace) = opt(s, parse_namespace)? {
        namespace.doc = doc.into();
        Ok(TopLevelNode::Namespace(namespace))
    } else {
        let kind = s.peek().kind;
        let span = s.peek().span;
        let mut stmt = stmt::parse(s)?;
        if let StmtKind::Item(item) = &mut *stmt.kind {
            item.doc = doc.into();
        } else if !doc.is_empty() {
            return Err(Error(ErrorKind::Rule("item", kind, span)));
        }
        Ok(TopLevelNode::Stmt(stmt))
    }
}

fn parse_namespace(s: &mut Scanner) -> Result<Namespace> {
    let lo = s.peek().span.lo;
    let doc = parse_doc(s).unwrap_or_default();
    token(s, TokenKind::Keyword(Keyword::Namespace))?;
    let name = dot_ident(s)?;
    token(s, TokenKind::Open(Delim::Brace))?;
    let items = barrier(s, &[TokenKind::Close(Delim::Brace)], parse_many)?;
    recovering_token(s, TokenKind::Close(Delim::Brace))?;
    Ok(Namespace {
        id: NodeId::default(),
        span: s.span(lo),
        doc: doc.into(),
        name,
        items: items.into_boxed_slice(),
    })
}

fn parse_doc(s: &mut Scanner) -> Option<String> {
    let mut content = String::new();
    while s.peek().kind == TokenKind::DocComment {
        if !content.is_empty() {
            content += "\n";
        }

        let lexeme = s.read();
        let prefix_len = if lexeme.starts_with("/// ") { 4 } else { 3 };
        content += shorten(prefix_len, 0, lexeme);
        s.advance();
    }

    (!content.is_empty()).then_some(content)
}

fn parse_attr(s: &mut Scanner) -> Result<Box<Attr>> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::At)?;
    let name = ident(s)?;
    let arg = expr(s)?;
    Ok(Box::new(Attr {
        id: NodeId::default(),
        span: s.span(lo),
        name,
        arg,
    }))
}

fn parse_visibility(s: &mut Scanner) -> Result<Visibility> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(Keyword::Internal))?;
    Ok(Visibility {
        id: NodeId::default(),
        span: s.span(lo),
        kind: VisibilityKind::Internal,
    })
}

fn parse_open(s: &mut Scanner) -> Result<Box<ItemKind>> {
    token(s, TokenKind::Keyword(Keyword::Open))?;
    let name = dot_ident(s)?;
    let alias = if token(s, TokenKind::Keyword(Keyword::As)).is_ok() {
        Some(dot_ident(s)?)
    } else {
        None
    };
    token(s, TokenKind::Semi)?;
    Ok(Box::new(ItemKind::Open(name, alias)))
}

fn parse_newtype(s: &mut Scanner) -> Result<Box<ItemKind>> {
    token(s, TokenKind::Keyword(Keyword::Newtype))?;
    let name = ident(s)?;
    token(s, TokenKind::Eq)?;
    let lo = s.peek().span.lo;
    let mut def = parse_ty_def(s)?;
    if let Some(ty) = try_tydef_as_ty(def.as_ref()) {
        let ty = array_or_arrow(s, ty, lo)?;
        def = Box::new(TyDef {
            id: def.id,
            span: ty.span,
            kind: Box::new(TyDefKind::Field(None, Box::new(ty))),
        });
    }
    token(s, TokenKind::Semi)?;
    Ok(Box::new(ItemKind::Ty(name, def)))
}

fn try_tydef_as_ty(tydef: &TyDef) -> Option<Ty> {
    match tydef.kind.as_ref() {
        TyDefKind::Field(Some(_), _) => None,
        TyDefKind::Field(None, ty) => Some(*ty.clone()),
        TyDefKind::Paren(tydef) => try_tydef_as_ty(tydef.as_ref()),
        TyDefKind::Tuple(tup) => {
            let mut ty_tup = Vec::new();
            for tydef in tup.iter() {
                ty_tup.push(try_tydef_as_ty(tydef)?)
            }
            Some(Ty {
                id: tydef.id,
                span: tydef.span,
                kind: Box::new(TyKind::Tuple(ty_tup.into_boxed_slice())),
            })
        }
        TyDefKind::Err => None,
    }
}

fn parse_ty_def(s: &mut Scanner) -> Result<Box<TyDef>> {
    let lo = s.peek().span.lo;
    let kind = if token(s, TokenKind::Open(Delim::Paren)).is_ok() {
        let (defs, final_sep) = seq(s, parse_ty_def)?;
        token(s, TokenKind::Close(Delim::Paren))?;
        final_sep.reify(defs, TyDefKind::Paren, TyDefKind::Tuple)
    } else {
        let field_ty = ty(s)?;
        if token(s, TokenKind::Colon).is_ok() {
            let name = ty_as_ident(field_ty)?;
            let field_ty = ty(s)?;
            TyDefKind::Field(Some(name), Box::new(field_ty))
        } else {
            TyDefKind::Field(None, Box::new(field_ty))
        }
    };

    Ok(Box::new(TyDef {
        id: NodeId::default(),
        span: s.span(lo),
        kind: Box::new(kind),
    }))
}

fn ty_as_ident(ty: Ty) -> Result<Box<Ident>> {
    let TyKind::Path(path) = *ty.kind else {
        return Err(Error(ErrorKind::Convert("identifier", "type", ty.span)));
    };
    if let Path {
        namespace: None,
        name,
        ..
    } = *path
    {
        Ok(name)
    } else {
        Err(Error(ErrorKind::Convert("identifier", "type", ty.span)))
    }
}

fn parse_callable_decl(s: &mut Scanner) -> Result<Box<CallableDecl>> {
    let lo = s.peek().span.lo;
    let kind = if token(s, TokenKind::Keyword(Keyword::Function)).is_ok() {
        CallableKind::Function
    } else if token(s, TokenKind::Keyword(Keyword::Operation)).is_ok() {
        CallableKind::Operation
    } else {
        let token = s.peek();
        return Err(Error(ErrorKind::Rule(
            "callable declaration",
            token.kind,
            token.span,
        )));
    };

    let name = ident(s)?;
    let generics = if token(s, TokenKind::Lt).is_ok() {
        let params = seq(s, ty::param)?.0;
        token(s, TokenKind::Gt)?;
        params
    } else {
        Vec::new()
    };

    let input = pat(s)?;
    check_input_parens(&input)?;
    token(s, TokenKind::Colon)?;
    let output = ty(s)?;
    let functors = if token(s, TokenKind::Keyword(Keyword::Is)).is_ok() {
        Some(Box::new(ty::functor_expr(s)?))
    } else {
        None
    };
    let body = parse_callable_body(s)?;

    Ok(Box::new(CallableDecl {
        id: NodeId::default(),
        span: s.span(lo),
        kind,
        name,
        generics: generics.into_boxed_slice(),
        input,
        output: Box::new(output),
        functors,
        body: Box::new(body),
    }))
}

fn parse_callable_body(s: &mut Scanner) -> Result<CallableBody> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Open(Delim::Brace))?;
    barrier(s, &[TokenKind::Close(Delim::Brace)], |s| {
        let specs = many(s, parse_spec_decl)?;
        if specs.is_empty() {
            let stmts = stmt::parse_many(s)?;
            check_semis(s, &stmts);
            recovering_token(s, TokenKind::Close(Delim::Brace))?;
            Ok(CallableBody::Block(Box::new(Block {
                id: NodeId::default(),
                span: s.span(lo),
                stmts: stmts.into_boxed_slice(),
            })))
        } else {
            recovering_token(s, TokenKind::Close(Delim::Brace))?;
            Ok(CallableBody::Specs(specs.into_boxed_slice()))
        }
    })
}

fn parse_spec_decl(s: &mut Scanner) -> Result<Box<SpecDecl>> {
    let lo = s.peek().span.lo;
    let spec = if token(s, TokenKind::Keyword(Keyword::Body)).is_ok() {
        Spec::Body
    } else if token(s, TokenKind::Keyword(Keyword::Adjoint)).is_ok() {
        Spec::Adj
    } else if token(s, TokenKind::Keyword(Keyword::Controlled)).is_ok() {
        if token(s, TokenKind::Keyword(Keyword::Adjoint)).is_ok() {
            Spec::CtlAdj
        } else {
            Spec::Ctl
        }
    } else {
        return Err(Error(ErrorKind::Rule(
            "specialization",
            s.peek().kind,
            s.peek().span,
        )));
    };

    let body = if let Some(gen) = opt(s, parse_spec_gen)? {
        token(s, TokenKind::Semi)?;
        SpecBody::Gen(gen)
    } else {
        SpecBody::Impl(pat(s)?, stmt::parse_block(s)?)
    };

    Ok(Box::new(SpecDecl {
        id: NodeId::default(),
        span: s.span(lo),
        spec,
        body,
    }))
}

fn parse_spec_gen(s: &mut Scanner) -> Result<SpecGen> {
    if token(s, TokenKind::Keyword(Keyword::Auto)).is_ok() {
        Ok(SpecGen::Auto)
    } else if token(s, TokenKind::Keyword(Keyword::Distribute)).is_ok() {
        Ok(SpecGen::Distribute)
    } else if token(s, TokenKind::Keyword(Keyword::Intrinsic)).is_ok() {
        Ok(SpecGen::Intrinsic)
    } else if token(s, TokenKind::Keyword(Keyword::Invert)).is_ok() {
        Ok(SpecGen::Invert)
    } else if token(s, TokenKind::Keyword(Keyword::Slf)).is_ok() {
        Ok(SpecGen::Slf)
    } else {
        Err(Error(ErrorKind::Rule(
            "specialization generator",
            s.peek().kind,
            s.peek().span,
        )))
    }
}
/// Checks that the inputs of the callable are surrounded by parens
pub(super) fn check_input_parens(inputs: &Pat) -> Result<()> {
    if !matches!(*inputs.kind, PatKind::Paren(_) | PatKind::Tuple(_)) {
        Err(Error(ErrorKind::MissingParens(inputs.span)))
    } else {
        Ok(())
    }
}
