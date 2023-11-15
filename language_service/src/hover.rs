// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::compilation::Compilation;
use crate::display::{parse_doc_for_param, parse_doc_for_summary, CodeDisplay};
use crate::name_locator::{Handler, Locator, LocatorContext};
use crate::protocol::Hover;
use crate::qsc_utils::protocol_span;
use qsc::ast::visit::Visitor;
use qsc::{ast, hir};
use std::fmt::Display;
use std::rc::Rc;

pub(crate) fn get_hover(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Option<Hover> {
    let offset = compilation.source_offset_to_package_offset(source_name, offset);
    let user_ast_package = &compilation.user_unit().ast.package;

    let mut hover_visitor = HoverGenerator {
        compilation,
        hover: None,
        display: CodeDisplay { compilation },
    };

    let mut locator = Locator::new(&mut hover_visitor, offset, compilation);
    locator.visit_package(user_ast_package);
    hover_visitor.hover
}

enum LocalKind {
    Param,
    TypeParam,
    LambdaParam,
    Local,
}
struct HoverGenerator<'a> {
    hover: Option<Hover>,
    display: CodeDisplay<'a>,
    compilation: &'a Compilation,
}

impl<'a> Handler<'a> for HoverGenerator<'a> {
    fn at_callable_def(
        &mut self,
        context: &LocatorContext<'a>,
        name: &'a ast::Ident,
        decl: &'a ast::CallableDecl,
    ) {
        let contents = display_callable(
            &context.current_item_doc,
            &context.current_namespace,
            self.display.ast_callable_decl(decl),
        );
        self.hover = Some(Hover {
            contents,
            span: protocol_span(name.span, &self.compilation.user_unit().sources),
        });
    }

    fn at_callable_ref(
        &mut self,
        path: &'a ast::Path,
        item_id: &'_ hir::ItemId,
        item: &'a hir::Item,
        package: &'a hir::Package,
        decl: &'a hir::CallableDecl,
    ) {
        let ns = item
            .parent
            .and_then(|parent_id| package.items.get(parent_id))
            .map_or_else(
                || Rc::from(""),
                |parent| match &parent.kind {
                    qsc::hir::ItemKind::Namespace(namespace, _) => namespace.name.clone(),
                    _ => Rc::from(""),
                },
            );

        let contents = display_callable(
            &item.doc,
            &ns,
            self.display.hir_callable_decl(
                item_id.package.expect("package id should be resolved"),
                decl,
            ),
        );

        self.hover = Some(Hover {
            contents,
            span: protocol_span(path.span, &self.compilation.user_unit().sources),
        });
    }

    fn at_type_param_def(
        &mut self,
        context: &LocatorContext<'a>,
        def_name: &'a ast::Ident,
        _: hir::ty::ParamId,
    ) {
        let code = markdown_fenced_block(def_name.name.clone());
        let callable_name = &context
            .current_callable
            .expect("type params should only exist in callables")
            .name
            .name;
        let contents = display_local(
            &LocalKind::TypeParam,
            &code,
            &def_name.name,
            callable_name,
            &context.current_item_doc,
        );
        self.hover = Some(Hover {
            contents,
            span: protocol_span(def_name.span, &self.compilation.user_unit().sources),
        });
    }

    fn at_type_param_ref(
        &mut self,
        context: &LocatorContext<'a>,
        ref_name: &'a ast::Ident,
        _: hir::ty::ParamId,
        _: &'a ast::Ident,
    ) {
        let code = markdown_fenced_block(ref_name.name.clone());
        let callable_name = &context
            .current_callable
            .expect("type params should only exist in callables")
            .name
            .name;
        let contents = display_local(
            &LocalKind::TypeParam,
            &code,
            &ref_name.name,
            callable_name,
            &context.current_item_doc,
        );
        self.hover = Some(Hover {
            contents,
            span: protocol_span(ref_name.span, &self.compilation.user_unit().sources),
        });
    }

    fn at_new_type_def(&mut self, type_name: &'a ast::Ident, def: &'a ast::TyDef) {
        let contents = markdown_fenced_block(self.display.ident_ty_def(type_name, def));
        self.hover = Some(Hover {
            contents,
            span: protocol_span(type_name.span, &self.compilation.user_unit().sources),
        });
    }

    fn at_new_type_ref(
        &mut self,
        path: &'a ast::Path,
        item_id: &'_ hir::ItemId,
        _: &'a hir::Package,
        _: &'a hir::Ident,
        udt: &'a hir::ty::Udt,
    ) {
        let contents = markdown_fenced_block(
            self.display
                .hir_udt(item_id.package.expect("package id should be resolved"), udt),
        );

        self.hover = Some(Hover {
            contents,
            span: protocol_span(path.span, &self.compilation.user_unit().sources),
        });
    }

    fn at_field_def(
        &mut self,
        _: &LocatorContext<'a>,
        field_name: &'a ast::Ident,
        ty: &'a ast::Ty,
    ) {
        let contents = markdown_fenced_block(self.display.ident_ty(field_name, ty));
        self.hover = Some(Hover {
            contents,
            span: protocol_span(field_name.span, &self.compilation.user_unit().sources),
        });
    }

    fn at_field_ref(
        &mut self,
        field_ref: &'a ast::Ident,
        expr_id: &'a ast::NodeId,
        _: &'_ hir::ItemId,
        _: &'a hir::ty::UdtField,
    ) {
        let contents = markdown_fenced_block(self.display.ident_ty_id(field_ref, *expr_id));
        self.hover = Some(Hover {
            contents,
            span: protocol_span(field_ref.span, &self.compilation.user_unit().sources),
        });
    }

    fn at_local_def(
        &mut self,
        context: &LocatorContext<'a>,
        ident: &'a ast::Ident,
        pat: &'a ast::Pat,
    ) {
        let code = markdown_fenced_block(self.display.ident_ty_id(ident, pat.id));
        let kind = if context.in_params {
            LocalKind::Param
        } else if context.in_lambda_params {
            LocalKind::LambdaParam
        } else {
            LocalKind::Local
        };
        let callable_name = &context
            .current_callable
            .expect("locals should only exist in callables")
            .name
            .name;
        let contents = display_local(
            &kind,
            &code,
            &ident.name,
            callable_name,
            &context.current_item_doc,
        );
        self.hover = Some(Hover {
            contents,
            span: protocol_span(ident.span, &self.compilation.user_unit().sources),
        });
    }

    fn at_local_ref(
        &mut self,
        context: &LocatorContext<'a>,
        path: &'a ast::Path,
        node_id: &'a ast::NodeId,
        definition: &'a ast::Ident,
    ) {
        let local_name = &definition.name;
        let callable_name = &context
            .current_callable
            .expect("locals should only exist in callables")
            .name
            .name;
        let code = markdown_fenced_block(self.display.path_ty_id(path, *node_id));
        let kind = if is_param(&curr_callable_to_params(context.current_callable), *node_id) {
            LocalKind::Param
        } else if is_param(&context.lambda_params, *node_id) {
            LocalKind::LambdaParam
        } else {
            LocalKind::Local
        };
        let contents = display_local(
            &kind,
            &code,
            local_name,
            callable_name,
            &context.current_item_doc,
        );
        self.hover = Some(Hover {
            contents,
            span: protocol_span(path.span, &self.compilation.user_unit().sources),
        });
    }
}

fn curr_callable_to_params(curr_callable: Option<&ast::CallableDecl>) -> Vec<&ast::Pat> {
    match curr_callable {
        Some(decl) => match &*decl.body {
            ast::CallableBody::Block(_) => vec![decl.input.as_ref()],
            ast::CallableBody::Specs(spec_decls) => {
                let mut pats = spec_decls
                    .iter()
                    .filter_map(|spec| match &spec.body {
                        ast::SpecBody::Gen(_) => None,
                        ast::SpecBody::Impl(input, _) => Some(input.as_ref()),
                    })
                    .collect::<Vec<&ast::Pat>>();
                pats.push(decl.input.as_ref());
                pats
            }
        },
        None => vec![],
    }
}

fn is_param(param_pats: &[&ast::Pat], node_id: ast::NodeId) -> bool {
    fn find_in_pat(pat: &ast::Pat, node_id: ast::NodeId) -> bool {
        match &*pat.kind {
            ast::PatKind::Bind(ident, _) => node_id == ident.id,
            ast::PatKind::Discard(_) | ast::PatKind::Elided | ast::PatKind::Err => false,
            ast::PatKind::Paren(inner) => find_in_pat(inner, node_id),
            ast::PatKind::Tuple(inner) => inner.iter().any(|x| find_in_pat(x, node_id)),
        }
    }

    param_pats.iter().any(|pat| find_in_pat(pat, node_id))
}

fn display_local(
    param_kind: &LocalKind,
    markdown: &String,
    local_name: &str,
    callable_name: &str,
    callable_doc: &str,
) -> String {
    match param_kind {
        LocalKind::Param => {
            let param_doc = parse_doc_for_param(callable_doc, local_name);
            with_doc(
                &param_doc,
                format!("parameter of `{callable_name}`\n{markdown}",),
            )
        }
        LocalKind::TypeParam => {
            let param_doc = parse_doc_for_param(callable_doc, local_name);
            with_doc(
                &param_doc,
                format!("type parameter of `{callable_name}`\n{markdown}",),
            )
        }
        LocalKind::LambdaParam => format!("lambda parameter\n{markdown}"),
        LocalKind::Local => format!("local\n{markdown}"),
    }
}

fn display_callable(doc: &str, namespace: &str, code: impl Display) -> String {
    let summary = parse_doc_for_summary(doc);

    let mut code = if namespace.is_empty() {
        code.to_string()
    } else {
        format!("{namespace}\n{code}")
    };
    code = markdown_fenced_block(code);
    with_doc(&summary, code)
}

fn with_doc(doc: &String, code: impl Display) -> String {
    if doc.is_empty() {
        code.to_string()
    } else {
        format!("{code}---\n{doc}\n")
    }
}

fn markdown_fenced_block(code: impl Display) -> String {
    format!(
        "```qsharp
{code}
```
"
    )
}
