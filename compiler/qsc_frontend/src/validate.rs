// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::{
    ast::{CallableDecl, CallableKind, Expr, ExprKind, Pat, Ty, TyKind},
    visit::{walk_expr, Visitor},
};

pub struct Validator {}

impl<'a> Visitor<'a> for Validator {
    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        if CallableKind::Operation == decl.kind && decl.functors.is_some() {
            match &decl.output.kind {
                TyKind::Tuple(items) if items.is_empty() => {}
                _ => {
                    panic!("Adjointable and Controllable Operations must return Unit.");
                }
            }
        }

        validate_params(&decl.input);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if let ExprKind::Lambda(_, _, _) = &expr.kind {
            panic!("Lambdas are not currently supported");
        } else {
            walk_expr(self, expr);
            if let ExprKind::Call(_, arg) = &expr.kind {
                assert!(
                    !has_hole(arg),
                    "Partial applications are not currently supported"
                );
            }
        }
    }
}

fn has_hole(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Hole => true,
        ExprKind::Paren(sub_expr) => has_hole(sub_expr),
        ExprKind::Tuple(sub_exprs) => sub_exprs.iter().any(has_hole),
        _ => false,
    }
}

fn validate_params(params: &Pat) {
    match &params.kind {
        qsc_ast::ast::PatKind::Bind(_, ty) => match &ty {
            None => panic!("Callable parameters must be type annotated."),
            Some(t) => validate_type(t),
        },
        qsc_ast::ast::PatKind::Paren(item) => validate_params(item),
        qsc_ast::ast::PatKind::Tuple(items) => items.iter().for_each(validate_params),
        _ => {}
    }
}

fn validate_type(ty: &Ty) {
    match &ty.kind {
        TyKind::App(ty, tys) => {
            validate_type(ty);
            tys.iter().for_each(validate_type);
        }
        TyKind::Arrow(_, _, _, _) => panic!("Callables as parameters are not currently supported."),
        TyKind::Paren(ty) => validate_type(ty),
        TyKind::Tuple(tys) => tys.iter().for_each(validate_type),
        _ => {}
    }
}
