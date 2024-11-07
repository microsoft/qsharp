// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Ascribe types to the AST and output HIR items. Put another way, converts the AST to the HIR.
use std::rc::Rc;

use crate::resolve::{self, Names};

use qsc_ast::ast::{
    self, CallableBody, CallableDecl, CallableKind, FunctorExpr, FunctorExprKind, Pat, PatKind,
    Path, PathKind, SetOp, Spec, StructDecl, TyDef, TyDefKind, TyKind, TypeParameter,
};
use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{self},
    ty::{
        Arrow, FunctorSet, FunctorSetValue, GenericParam, ParamId, Scheme, Ty, UdtDef, UdtDefKind,
        UdtField,
    },
};
use rustc_hash::FxHashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum TyConversionError {
    MissingTy {
        span: Span,
    },
    UnrecognizedClass {
        span: Span,
        name: String,
    },
    RecursiveClassConstraint {
        span: Span,
        name: String,
    },
    IncorrectNumberOfConstraintParameters {
        expected: usize,
        found: usize,
        span: Span,
    },
}

/// Given an `ast::Ty` and a list of resolved `Names`, convert the `ast::Ty` to an `hir::Ty`.
pub(crate) fn ty_from_ast(
    names: &Names,
    ty: &ast::Ty,
    stack: &mut FxHashSet<qsc_ast::ast::ClassConstraint>,
) -> (Ty, Vec<TyConversionError>) {
    match &*ty.kind {
        TyKind::Array(item) => {
            let (item, errors) = ty_from_ast(names, item, stack);
            (Ty::Array(Box::new(item)), errors)
        }
        TyKind::Arrow(kind, input, output, functors) => {
            let (input, mut errors) = ty_from_ast(names, input, stack);
            let (output, output_errors) = ty_from_ast(names, output, stack);
            errors.extend(output_errors);
            let functors = functors
                .as_ref()
                .map_or(FunctorSetValue::Empty, |f| eval_functor_expr(f.as_ref()));
            let ty = Ty::Arrow(Box::new(Arrow {
                kind: callable_kind_from_ast(*kind),
                input: Box::new(input),
                output: Box::new(output),
                functors: FunctorSet::Value(functors),
            }));
            (ty, errors)
        }
        TyKind::Hole => (
            Ty::Err,
            vec![TyConversionError::MissingTy { span: ty.span }],
        ),
        TyKind::Paren(inner) => ty_from_ast(names, inner, stack),
        TyKind::Param(TypeParameter { ty, .. }) => match names.get(ty.id) {
            Some(resolve::Res::Param { id, bounds }) => {
                let (bounds, errors) = class_constraints_from_ast(names, bounds, stack);
                (
                    Ty::Param {
                        name: ty.name.clone(),
                        id: *id,
                        bounds,
                    },
                    errors,
                )
            }
            Some(_) | None => (
                Ty::Err,
                vec![TyConversionError::MissingTy { span: ty.span }],
            ),
        },
        TyKind::Path(PathKind::Ok(path)) => (ty_from_path(names, path), Vec::new()),
        TyKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = ty_from_ast(names, item, stack);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
        TyKind::Err | TyKind::Path(PathKind::Err { .. }) => (Ty::Err, Vec::new()),
    }
}

pub(super) fn ty_from_path(names: &Names, path: &Path) -> Ty {
    match names.get(path.id) {
        Some(&resolve::Res::Item(item, _)) => Ty::Udt(path.name.name.clone(), hir::Res::Item(item)),
        Some(&resolve::Res::PrimTy(prim)) => Ty::Prim(prim),
        Some(resolve::Res::UnitTy) => Ty::Tuple(Vec::new()),
        // a path should never resolve to a parameter,
        // as there is a syntactic difference between
        // paths and parameters.
        // So realistically, by construction, `Param` here is unreachable.
        // A path can also never resolve to an export, because in typeck/check,
        // we resolve exports to their original definition.
        Some(
            resolve::Res::Local(_) | resolve::Res::Param { .. } | resolve::Res::ExportedItem(_, _),
        ) => {
            unreachable!(
                "A path should never resolve \
            to a local or a parameter, as there is syntactic differentiation."
            )
        }
        None => Ty::Err,
    }
}

/// Convert a struct declaration into a UDT type definition.
pub(super) fn ast_struct_decl_as_ty_def(decl: &StructDecl) -> TyDef {
    TyDef {
        id: decl.id,
        span: decl.span,
        kind: Box::new(TyDefKind::Tuple(
            decl.fields
                .iter()
                .map(|f| {
                    Box::new(TyDef {
                        id: f.id,
                        span: f.span,
                        kind: Box::new(TyDefKind::Field(Some(f.name.clone()), f.ty.clone())),
                    })
                })
                .collect(),
        )),
    }
}

pub(super) fn ast_ty_def_cons(
    names: &Names,
    ty_name: &Rc<str>,
    id: hir::ItemId,
    def: &TyDef,
) -> (Scheme, Vec<TyConversionError>) {
    let (input, errors) = ast_ty_def_base(names, def);
    let ty = Arrow {
        kind: hir::CallableKind::Function,
        input: Box::new(input),
        output: Box::new(Ty::Udt(ty_name.clone(), hir::Res::Item(id))),
        functors: FunctorSet::Value(FunctorSetValue::Empty),
    };
    let scheme = Scheme::new(Vec::new(), Box::new(ty));
    (scheme, errors)
}

fn ast_ty_def_base(names: &Names, def: &TyDef) -> (Ty, Vec<TyConversionError>) {
    match &*def.kind {
        TyDefKind::Field(_, ty) => ty_from_ast(names, ty, &mut Default::default()),
        TyDefKind::Paren(inner) => ast_ty_def_base(names, inner),
        TyDefKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = ast_ty_def_base(names, item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }

            (Ty::Tuple(tys), errors)
        }
        TyDefKind::Err => (Ty::Err, Vec::new()),
    }
}

pub(super) fn ast_ty_def(names: &Names, def: &TyDef) -> (UdtDef, Vec<TyConversionError>) {
    if let TyDefKind::Paren(inner) = &*def.kind {
        return ast_ty_def(names, inner);
    }

    let mut errors = Vec::new();
    let def = UdtDef {
        span: def.span,
        kind: match &*def.kind {
            TyDefKind::Field(name, ty) => {
                let (ty, item_errors) = ty_from_ast(names, ty, &mut Default::default());
                errors.extend(item_errors);
                let (name_span, name) = match name {
                    Some(name) => (Some(name.span), Some(name.name.clone())),
                    None => (None, None),
                };
                let field = UdtField {
                    name_span,
                    name,
                    ty,
                };
                UdtDefKind::Field(field)
            }
            TyDefKind::Paren(_) => unreachable!("parentheses should be removed earlier"),
            TyDefKind::Tuple(items) => UdtDefKind::Tuple(
                items
                    .iter()
                    .map(|i| {
                        let (item_def, item_errors) = ast_ty_def(names, i);
                        errors.extend(item_errors);
                        item_def
                    })
                    .collect(),
            ),
            TyDefKind::Err => UdtDefKind::Field(UdtField {
                name_span: None,
                name: None,
                ty: Ty::Err,
            }),
        },
    };

    (def, errors)
}

/// Given a list of ast type parameters, convert them to HIR type parameters and generate errors if
/// there are any type errors resulting from this.
pub(crate) fn ast_callable_generics(
    names: &Names,
    generics: &[ast::TypeParameter],
) -> (Vec<qsc_hir::ty::GenericParam>, Vec<TyConversionError>) {
    let mut errors = Vec::new();
    let mut generics_buf = Vec::with_capacity(generics.len());
    for param in generics {
        let (bounds, new_errors) =
            class_constraints_from_ast(names, &param.constraints, &mut Default::default());
        errors.extend(new_errors);
        generics_buf.push(GenericParam::Ty {
            name: param.ty.name.clone(),
            bounds,
        });
    }
    (generics_buf, errors)
}

pub(super) fn ast_callable_scheme(
    names: &Names,
    callable: &CallableDecl,
) -> (Scheme, Vec<TyConversionError>) {
    let (mut type_parameters, errors) = ast_callable_generics(names, &callable.generics);
    let mut errors = errors
        .into_iter()
        .map(TyConversionError::from)
        .collect::<Vec<_>>();
    let kind = callable_kind_from_ast(callable.kind);

    let (mut input, new_errors) = ast_pat_ty(names, &callable.input);
    errors.extend(&mut new_errors.into_iter());

    let (output, output_errors) = ty_from_ast(names, &callable.output, &mut Default::default());

    errors.extend(output_errors);

    let mut functor_params =
        synthesize_functor_params(&mut type_parameters.len().into(), &mut input);

    type_parameters.append(&mut functor_params);

    let ty = Arrow {
        kind,
        input: Box::new(input),
        output: Box::new(output),
        functors: FunctorSet::Value(ast_callable_functors(callable)),
    };

    (Scheme::new(type_parameters, Box::new(ty)), errors)
}

pub(crate) fn synthesize_functor_params(
    next_param: &mut ParamId,
    ty: &mut Ty,
) -> Vec<GenericParam> {
    match ty {
        Ty::Array(item) => synthesize_functor_params(next_param, item),
        Ty::Arrow(arrow) => match arrow.functors {
            FunctorSet::Value(functors) if arrow.kind == hir::CallableKind::Operation => {
                let param = GenericParam::Functor(functors);
                arrow.functors = FunctorSet::Param(*next_param, functors);
                *next_param = next_param.successor();
                vec![param]
            }
            _ => Vec::new(),
        },
        Ty::Tuple(items) => items
            .iter_mut()
            .flat_map(|item| synthesize_functor_params(next_param, item))
            .collect(),
        Ty::Infer(_) | Ty::Param { .. } | Ty::Prim(_) | Ty::Udt(_, _) | Ty::Err => Vec::new(),
    }
}

pub(crate) fn ast_pat_ty(names: &Names, pat: &Pat) -> (Ty, Vec<TyConversionError>) {
    match &*pat.kind {
        PatKind::Bind(_, None) | PatKind::Discard(None) | PatKind::Elided => (
            Ty::Err,
            vec![TyConversionError::MissingTy { span: pat.span }],
        ),
        PatKind::Bind(_, Some(ty)) | PatKind::Discard(Some(ty)) => {
            ty_from_ast(names, ty, &mut Default::default())
        }
        PatKind::Paren(inner) => ast_pat_ty(names, inner),
        PatKind::Tuple(items) => {
            let mut tys = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (item_ty, item_errors) = ast_pat_ty(names, item);
                tys.push(item_ty);
                errors.extend(item_errors);
            }
            (Ty::Tuple(tys), errors)
        }
        PatKind::Err => (Ty::Err, Vec::new()),
    }
}

pub(crate) fn ast_callable_functors(callable: &CallableDecl) -> FunctorSetValue {
    let mut functors = callable
        .functors
        .as_ref()
        .map_or(FunctorSetValue::Empty, |f| eval_functor_expr(f.as_ref()));

    if let CallableBody::Specs(specs) = callable.body.as_ref() {
        for spec in specs {
            let spec_functors = match spec.spec {
                Spec::Body => FunctorSetValue::Empty,
                Spec::Adj => FunctorSetValue::Adj,
                Spec::Ctl => FunctorSetValue::Ctl,
                Spec::CtlAdj => FunctorSetValue::CtlAdj,
            };
            functors = functors.union(&spec_functors);
        }
    }

    functors
}

pub(super) fn callable_kind_from_ast(kind: CallableKind) -> hir::CallableKind {
    match kind {
        CallableKind::Function => hir::CallableKind::Function,
        CallableKind::Operation => hir::CallableKind::Operation,
    }
}

pub(crate) fn eval_functor_expr(expr: &FunctorExpr) -> FunctorSetValue {
    match expr.kind.as_ref() {
        FunctorExprKind::BinOp(op, lhs, rhs) => {
            let lhs_functors = eval_functor_expr(lhs);
            let rhs_functors = eval_functor_expr(rhs);
            match op {
                SetOp::Union => lhs_functors.union(&rhs_functors),
                SetOp::Intersect => lhs_functors.intersect(&rhs_functors),
            }
        }
        FunctorExprKind::Lit(ast::Functor::Adj) => FunctorSetValue::Adj,
        FunctorExprKind::Lit(ast::Functor::Ctl) => FunctorSetValue::Ctl,
        FunctorExprKind::Paren(inner) => eval_functor_expr(inner),
    }
}

/// Convert an AST type bound to an HIR type bound.
pub(crate) fn class_constraints_from_ast(
    names: &Names,
    bounds: &qsc_ast::ast::ClassConstraints,
    // used to check for recursive types
    stack: &mut FxHashSet<qsc_ast::ast::ClassConstraint>,
) -> (qsc_hir::ty::ClassConstraints, Vec<TyConversionError>) {
    let mut bounds_buf = Vec::new();
    let mut errors = FxHashSet::default();

    for ast_bound in &bounds.0 {
        if stack.contains(ast_bound) {
            errors.insert(TyConversionError::RecursiveClassConstraint {
                span: ast_bound.span(),
                name: ast_bound.name.name.to_string(),
            });
            continue;
        }
        stack.insert(ast_bound.clone());
        let bound_result = match &*ast_bound.name.name {
            "Eq" => Ok(qsc_hir::ty::ClassConstraint::Eq),
            "Add" => Ok(qsc_hir::ty::ClassConstraint::Add),
            "Iterable" => {
                let (item, item_errors) = ty_from_ast(names, ast_bound.parameters[0].ty(), stack);
                errors.extend(item_errors.into_iter());
                Ok(qsc_hir::ty::ClassConstraint::Iterable { item })
            }
            "Exp" => {
                let (power, power_errors) = ty_from_ast(names, ast_bound.parameters[0].ty(), stack);
                errors.extend(power_errors.into_iter());
                Ok(qsc_hir::ty::ClassConstraint::Exp { power })
            }
            "Integral" => Ok(qsc_hir::ty::ClassConstraint::Integral),
            "Num" => Ok(qsc_hir::ty::ClassConstraint::Num),
            "Show" => Ok(qsc_hir::ty::ClassConstraint::Show),
            otherwise => Err(TyConversionError::UnrecognizedClass {
                span: ast_bound.span(),
                name: otherwise.to_string(),
            }),
        };

        match bound_result {
            Ok(hir_bound) => {
                check_param_length(
                    &hir_bound,
                    &mut errors,
                    ast_bound.parameters.len(),
                    ast_bound.span(),
                );
                bounds_buf.push(hir_bound);
            }
            Err(e) => {
                errors.insert(e);
            }
        }
    }

    (
        qsc_hir::ty::ClassConstraints(bounds_buf.into_boxed_slice()),
        errors.into_iter().collect(),
    )
}

fn check_param_length(
    bound: &qsc_hir::ty::ClassConstraint,
    errors: &mut FxHashSet<TyConversionError>,
    num_given_parameters: usize,
    span: Span,
) {
    use qsc_hir::ty::ClassConstraint::*;
    let num_parameters = match bound {
        Eq | Add | Integral | Num | Show | NonNativeClass(_) => 0,
        Iterable { .. } | Exp { .. } => 1,
    };
    if num_parameters != num_given_parameters {
        errors.insert(TyConversionError::IncorrectNumberOfConstraintParameters {
            expected: num_parameters,
            found: num_given_parameters,
            span,
        });
    }
}
