// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Something declared at the top level (within a namespace or module) is an [Item]. This could be a newtype, callable
//! declaration, import or open statement, or other top-level declaration or statement.

#[cfg(test)]
mod tests;

use super::{
    expr::expr,
    keyword::Keyword,
    prim::{ident, many, opt, pat, seq, token},
    scan::ParserContext,
    stmt,
    ty::{self, ty},
    Error, Result,
};

use crate::lex::ClosedBinOp;
use crate::{
    lex::{Delim, TokenKind},
    prim::{barrier, path, recovering, recovering_token, shorten},
    stmt::check_semis,
    ty::array_or_arrow,
    ErrorKind,
};
use qsc_ast::ast::{
    Attr, Block, CallableBody, CallableDecl, CallableKind, FieldDef, Ident, Idents,
    ImportOrExportDecl, ImportOrExportItem, Item, ItemKind, Namespace, NodeId, Pat, PatKind, Path,
    Spec, SpecBody, SpecDecl, SpecGen, StmtKind, StructDecl, TopLevelNode, Ty, TyDef, TyDefKind,
    TyKind, Visibility, VisibilityKind,
};
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_data_structures::span::Span;

pub(super) fn parse(s: &mut ParserContext) -> Result<Box<Item>> {
    let lo = s.peek().span.lo;
    let doc = parse_doc(s);
    let attrs = many(s, parse_attr)?;
    let visibility = opt(s, parse_visibility)?;
    let kind = if let Some(open) = opt(s, parse_open)? {
        open
    } else if let Some(ty) = opt(s, parse_newtype)? {
        ty
    } else if let Some(strct) = opt(s, parse_struct)? {
        strct
    } else if let Some(callable) = opt(s, parse_callable_decl)? {
        Box::new(ItemKind::Callable(callable))
    } else if let Some(decl) = opt(s, parse_import_or_export)? {
        Box::new(ItemKind::ImportOrExport(decl))
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
fn parse_many(s: &mut ParserContext) -> Result<Vec<Box<Item>>> {
    const BARRIER_TOKENS: &[TokenKind] = &[
        TokenKind::At,
        TokenKind::Keyword(Keyword::Internal),
        TokenKind::Keyword(Keyword::Open),
        TokenKind::Keyword(Keyword::Newtype),
        TokenKind::Keyword(Keyword::Struct),
        TokenKind::Keyword(Keyword::Operation),
        TokenKind::Keyword(Keyword::Function),
    ];

    const RECOVERY_TOKENS: &[TokenKind] = &[TokenKind::Semi, TokenKind::Close(Delim::Brace)];

    barrier(s, BARRIER_TOKENS, |s| {
        many(s, |s| recovering(s, default, RECOVERY_TOKENS, parse))
    })
}

#[allow(clippy::unnecessary_box_returns)]
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

pub(super) fn parse_namespaces(s: &mut ParserContext) -> Result<Vec<Namespace>> {
    let namespaces = many(s, parse_namespace)?;
    recovering_token(s, TokenKind::Eof);
    Ok(namespaces)
}

pub(super) fn parse_top_level_nodes(s: &mut ParserContext) -> Result<Vec<TopLevelNode>> {
    let nodes = many(s, parse_top_level_node)?;
    recovering_token(s, TokenKind::Eof);
    Ok(nodes)
}

fn parse_top_level_node(s: &mut ParserContext) -> Result<TopLevelNode> {
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

pub fn parse_implicit_namespace(source_name: &str, s: &mut ParserContext) -> Result<Namespace> {
    if s.peek().kind == TokenKind::Eof {
        return Ok(Namespace {
            id: NodeId::default(),
            span: s.span(0),
            doc: "".into(),
            name: source_name_to_namespace_name(source_name, s.span(0))?,
            items: Vec::new().into_boxed_slice(),
        });
    }
    let lo = s.peek().span.lo;
    let items = parse_namespace_block_contents(s)?;
    if items.is_empty() || s.peek().kind != TokenKind::Eof {
        return Err(Error(ErrorKind::ExpectedItem(s.peek().kind, s.span(lo))));
    }
    let span = s.span(lo);
    let namespace_name = source_name_to_namespace_name(source_name, span)?;

    Ok(Namespace {
        id: NodeId::default(),
        span,
        doc: "".into(),
        name: namespace_name,
        items: items.into_boxed_slice(),
    })
}

/// Given a file name, convert it to a namespace name.
/// For example, `foo/bar.qs` becomes `foo.bar`.
/// Invalid or disallowed characters are cleaned up in a best effort manner.
fn source_name_to_namespace_name(raw: &str, span: Span) -> Result<Idents> {
    let path = std::path::Path::new(raw);
    let mut namespace = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(name) => {
                // strip the extension off, if there is one
                let mut name = name.to_str().ok_or(Error(ErrorKind::InvalidFileName(
                    span,
                    name.to_string_lossy().to_string(),
                )))?;

                if let Some(dot) = name.rfind('.') {
                    name = name[..dot].into();
                }
                // verify that the component only contains alphanumeric characters, and doesn't start with a number

                let mut ident = validate_namespace_name(span, name)?;
                ident.span = span;

                namespace.push(ident);
            }
            _ => return Err(Error(ErrorKind::InvalidFileName(span, raw.to_string()))),
        }
    }

    Ok(namespace.into())
}

fn clean_namespace_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '-' => '_',
            _ => c,
        })
        .collect()
}

/// Validates that a string could be a valid namespace name component
fn validate_namespace_name(error_span: Span, name: &str) -> Result<Ident> {
    let name = clean_namespace_name(name);
    let mut s = ParserContext::new(&name, LanguageFeatures::default());
    // if it could be a valid identifier, then it is a valid namespace name
    // we just directly use the ident parser here instead of trying to recreate
    // validation rules
    let ident = ident(&mut s)
        .map_err(|_| Error(ErrorKind::InvalidFileName(error_span, name.to_string())))?;
    if s.peek().kind != TokenKind::Eof {
        return Err(Error(ErrorKind::InvalidFileName(
            error_span,
            name.to_string(),
        )));
    }
    Ok(*ident)
}

fn parse_namespace(s: &mut ParserContext) -> Result<Namespace> {
    let lo = s.peek().span.lo;
    let doc = parse_doc(s).unwrap_or_default();
    token(s, TokenKind::Keyword(Keyword::Namespace))?;
    let name = path(s)?;
    token(s, TokenKind::Open(Delim::Brace))?;
    let items = parse_namespace_block_contents(s)?;
    recovering_token(s, TokenKind::Close(Delim::Brace));
    Ok(Namespace {
        id: NodeId::default(),
        span: s.span(lo),
        doc: doc.into(),
        name: (*name).into(),
        items: items.into_boxed_slice(),
    })
}

/// Parses the contents of a namespace block, what is in between the open and close braces in an
/// explicit namespace, and any top level items in an implicit namespace.
#[allow(clippy::vec_box)]
fn parse_namespace_block_contents(s: &mut ParserContext) -> Result<Vec<Box<Item>>> {
    let items = barrier(s, &[TokenKind::Close(Delim::Brace)], parse_many)?;
    Ok(items)
}

/// See [GH Issue 941](https://github.com/microsoft/qsharp/issues/941) for context.
/// We want to anticipate docstrings in places people might
/// put them, but throw them away. This is to maintain
/// back compatibility.
/// Eventually, when we support doc comments in more places,
/// or support warnings, we can use this function to determine
/// places that need to be updated. This function can then emit
/// a warning.
pub(super) fn throw_away_doc(s: &mut ParserContext) {
    let _ = parse_doc(s);
}

pub(crate) fn parse_doc(s: &mut ParserContext) -> Option<String> {
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

fn parse_attr(s: &mut ParserContext) -> Result<Box<Attr>> {
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

fn parse_visibility(s: &mut ParserContext) -> Result<Visibility> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(Keyword::Internal))?;
    Ok(Visibility {
        id: NodeId::default(),
        span: s.span(lo),
        kind: VisibilityKind::Internal,
    })
}

fn parse_open(s: &mut ParserContext) -> Result<Box<ItemKind>> {
    token(s, TokenKind::Keyword(Keyword::Open))?;
    let mut name = vec![*(ident(s)?)];
    while let Ok(_dot) = token(s, TokenKind::Dot) {
        name.push(*(ident(s)?));
    }
    let alias = if token(s, TokenKind::Keyword(Keyword::As)).is_ok() {
        Some(ident(s)?)
    } else {
        None
    };

    // Peek to see if the next token is a dot -- this means it is likely a dot ident alias, and
    // we want to provide a more helpful error message
    if s.peek().kind == TokenKind::Dot {
        return Err(Error(ErrorKind::DotIdentAlias(s.peek().span)));
    }

    token(s, TokenKind::Semi)?;
    Ok(Box::new(ItemKind::Open(name.into(), alias)))
}

fn parse_newtype(s: &mut ParserContext) -> Result<Box<ItemKind>> {
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

fn parse_struct(s: &mut ParserContext) -> Result<Box<ItemKind>> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(Keyword::Struct))?;
    let name = ident(s)?;
    token(s, TokenKind::Open(Delim::Brace))?;
    let (fields, _) = seq(s, |s| {
        let lo = s.peek().span.lo;
        let name = ident(s)?;
        token(s, TokenKind::Colon)?;
        let field_ty = ty(s)?;
        Ok(Box::new(FieldDef {
            id: NodeId::default(),
            span: s.span(lo),
            name,
            ty: Box::new(field_ty),
        }))
    })?;
    recovering_token(s, TokenKind::Close(Delim::Brace));
    let decl = StructDecl {
        id: NodeId::default(),
        span: s.span(lo),
        name,
        fields: fields.into_boxed_slice(),
    };

    Ok(Box::new(ItemKind::Struct(Box::new(decl))))
}

fn try_tydef_as_ty(tydef: &TyDef) -> Option<Ty> {
    match tydef.kind.as_ref() {
        TyDefKind::Field(Some(_), _) | TyDefKind::Err => None,
        TyDefKind::Field(None, ty) => Some(*ty.clone()),
        TyDefKind::Paren(tydef) => try_tydef_as_ty(tydef.as_ref()),
        TyDefKind::Tuple(tup) => {
            let mut ty_tup = Vec::new();
            for tydef in tup.iter() {
                ty_tup.push(try_tydef_as_ty(tydef)?);
            }
            Some(Ty {
                id: tydef.id,
                span: tydef.span,
                kind: Box::new(TyKind::Tuple(ty_tup.into_boxed_slice())),
            })
        }
    }
}

fn parse_ty_def(s: &mut ParserContext) -> Result<Box<TyDef>> {
    throw_away_doc(s);
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

fn parse_callable_decl(s: &mut ParserContext) -> Result<Box<CallableDecl>> {
    let lo = s.peek().span.lo;
    let _doc = parse_doc(s);
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
        throw_away_doc(s);
        let params = seq(s, ty::param)?.0;
        token(s, TokenKind::Gt)?;
        params
    } else {
        Vec::new()
    };

    let input = pat(s)?;
    check_input_parens(&input)?;
    token(s, TokenKind::Colon)?;
    throw_away_doc(s);
    let output = ty(s)?;
    let functors = if token(s, TokenKind::Keyword(Keyword::Is)).is_ok() {
        Some(Box::new(ty::functor_expr(s)?))
    } else {
        None
    };
    throw_away_doc(s);
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

fn parse_callable_body(s: &mut ParserContext) -> Result<CallableBody> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Open(Delim::Brace))?;
    barrier(s, &[TokenKind::Close(Delim::Brace)], |s| {
        let specs = many(s, parse_spec_decl)?;
        if specs.is_empty() {
            let stmts = stmt::parse_many(s)?;
            check_semis(s, &stmts);
            recovering_token(s, TokenKind::Close(Delim::Brace));
            Ok(CallableBody::Block(Box::new(Block {
                id: NodeId::default(),
                span: s.span(lo),
                stmts: stmts.into_boxed_slice(),
            })))
        } else {
            recovering_token(s, TokenKind::Close(Delim::Brace));
            Ok(CallableBody::Specs(specs.into_boxed_slice()))
        }
    })
}

fn parse_spec_decl(s: &mut ParserContext) -> Result<Box<SpecDecl>> {
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

fn parse_spec_gen(s: &mut ParserContext) -> Result<SpecGen> {
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
    if matches!(*inputs.kind, PatKind::Paren(_) | PatKind::Tuple(_)) {
        Ok(())
    } else {
        Err(Error(ErrorKind::MissingParens(inputs.span)))
    }
}

/// Parses an import or export statement. Exports start with the `export` keyword, followed by a
/// list of items.
/// Imports are the same, but with the `import` keyword.
///
/// ```qsharp
/// export
///     Foo,
///     Bar.Baz,
///     Bar.Quux as Corge;
/// ```
fn parse_import_or_export(s: &mut ParserContext) -> Result<ImportOrExportDecl> {
    let lo = s.peek().span.lo;
    let _doc = parse_doc(s);
    let is_export = match s.peek().kind {
        TokenKind::Keyword(Keyword::Export) => true,
        TokenKind::Keyword(Keyword::Import) => false,
        _ => {
            return Err(Error(ErrorKind::Rule(
                "import or export",
                s.peek().kind,
                s.peek().span,
            )))
        }
    };
    s.advance();
    let (items, _) = seq(s, parse_import_or_export_item)?;
    token(s, TokenKind::Semi)?;
    Ok(ImportOrExportDecl::new(
        s.span(lo),
        items.into_boxed_slice(),
        is_export,
    ))
}

fn parse_import_or_export_item(s: &mut ParserContext) -> Result<ImportOrExportItem> {
    let mut is_glob = false;
    let mut parts = vec![*ident(s)?];
    while token(s, TokenKind::Dot).is_ok() {
        if token(s, TokenKind::ClosedBinOp(ClosedBinOp::Star)).is_ok() {
            is_glob = true;
            break;
        }
        parts.push(*ident(s)?);
    }

    let alias = if token(s, TokenKind::Keyword(Keyword::As)).is_ok() {
        Some(*(ident(s)?))
    } else {
        None
    };

    Ok(ImportOrExportItem {
        path: parts.into(),
        alias,
        is_glob,
    })
}
