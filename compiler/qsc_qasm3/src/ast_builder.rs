// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::rc::Rc;

use num_bigint::BigInt;

use qsc::{
    ast::{
        self, Attr, Block, CallableBody, CallableDecl, CallableKind, Expr, ExprKind, Ident, Idents,
        Item, Lit, Mutability, NodeId, Pat, PatKind, Path, QubitInit, QubitInitKind, QubitSource,
        Stmt, StmtKind, TopLevelNode, Ty, TyKind,
    },
    Span,
};

use crate::{
    runtime::RuntimeFunctions,
    types::{ArrayDimensions, Complex},
};

pub(crate) fn build_unmanaged_qubit_alloc<S>(name: S, stmt_span: Span, name_span: Span) -> Stmt
where
    S: AsRef<str>,
{
    let alloc_ident = Ident {
        name: Rc::from("__quantum__rt__qubit_allocate"),
        ..Default::default()
    };
    let path_expr = Expr {
        kind: Box::new(ExprKind::Path(Box::new(Path {
            segments: build_idents(&["QIR", "Runtime"]),
            name: Box::new(alloc_ident),
            ..Default::default()
        }))),
        ..Default::default()
    };
    let call_expr = Expr {
        span: stmt_span,
        kind: Box::new(ExprKind::Call(
            Box::new(path_expr),
            Box::new(create_unit_expr(Span::default())),
        )),
        ..Default::default()
    };
    let rhs = call_expr;

    Stmt {
        span: stmt_span,
        kind: Box::new(StmtKind::Local(
            Mutability::Immutable,
            Box::new(Pat {
                kind: Box::new(PatKind::Bind(
                    Box::new(Ident {
                        span: name_span,
                        name: Rc::from(name.as_ref()),
                        ..Default::default()
                    }),
                    None,
                )),
                ..Default::default()
            }),
            Box::new(rhs),
        )),
        ..Default::default()
    }
}

pub(crate) fn build_unmanaged_qubit_alloc_array<S>(
    name: S,
    size: u32,
    stmt_span: Span,
    name_span: Span,
    designator_span: Span,
) -> Stmt
where
    S: AsRef<str>,
{
    let alloc_ident = Ident {
        name: Rc::from("AllocateQubitArray"),
        ..Default::default()
    };

    let path_expr = Expr {
        kind: Box::new(ExprKind::Path(Box::new(Path {
            segments: build_idents(&["QIR", "Runtime"]),
            name: Box::new(alloc_ident),
            ..Default::default()
        }))),
        ..Default::default()
    };
    let call_expr = Expr {
        id: NodeId::default(),
        span: stmt_span,
        kind: Box::new(ExprKind::Call(
            Box::new(path_expr),
            Box::new(Expr {
                kind: Box::new(ExprKind::Paren(Box::new(Expr {
                    id: NodeId::default(),
                    span: designator_span,
                    kind: Box::new(ExprKind::Lit(Box::new(Lit::Int(i64::from(size))))),
                }))),
                ..Default::default()
            }),
        )),
    };

    Stmt {
        id: NodeId::default(),
        span: stmt_span,
        kind: Box::new(StmtKind::Local(
            Mutability::Immutable,
            Box::new(Pat {
                kind: Box::new(PatKind::Bind(
                    Box::new(Ident {
                        id: NodeId::default(),
                        span: name_span,
                        name: Rc::from(name.as_ref()),
                    }),
                    None,
                )),
                ..Default::default()
            }),
            Box::new(call_expr),
        )),
    }
}

pub(crate) fn build_managed_qubit_alloc<S>(name: S, stmt_span: Span, name_span: Span) -> Stmt
where
    S: AsRef<str>,
{
    let qubit_init = QubitInitKind::Single;

    let qubit = QubitInit {
        id: NodeId::default(),
        span: stmt_span,
        kind: Box::new(qubit_init),
    };

    let ident: Ident = Ident {
        id: NodeId::default(),
        span: name_span,
        name: Rc::from(name.as_ref()),
    };
    let qubit_kind = StmtKind::Qubit(
        QubitSource::Fresh,
        Box::new(Pat {
            kind: Box::new(PatKind::Bind(Box::new(ident), None)),
            ..Default::default()
        }),
        Box::new(qubit),
        None,
    );
    Stmt {
        kind: Box::new(qubit_kind),
        ..Default::default()
    }
}

pub(crate) fn managed_qubit_alloc_array<S>(
    name: S,
    size: u32,
    stmt_span: Span,
    name_span: Span,
    designator_span: Span,
) -> Stmt
where
    S: AsRef<str>,
{
    let qubit_init = QubitInitKind::Array(Box::new(Expr {
        span: designator_span,
        kind: Box::new(ExprKind::Lit(Box::new(Lit::Int(i64::from(size))))),
        ..Default::default()
    }));

    let qubit = QubitInit {
        span: name_span,
        kind: Box::new(qubit_init),
        ..Default::default()
    };

    let ident: Ident = Ident {
        span: name_span,
        name: Rc::from(name.as_ref()),
        ..Default::default()
    };
    let qubit_kind = StmtKind::Qubit(
        QubitSource::Fresh,
        Box::new(Pat {
            kind: Box::new(PatKind::Bind(Box::new(ident), None)),
            ..Default::default()
        }),
        Box::new(qubit),
        None,
    );
    Stmt {
        span: stmt_span,
        kind: Box::new(qubit_kind),
        ..Default::default()
    }
}

pub(crate) fn build_lit_result_expr(value: qsc::ast::Result, span: Span) -> Expr {
    Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(ExprKind::Lit(Box::new(Lit::Result(value)))),
    }
}

pub(crate) fn build_lit_result_array_expr_from_bitstring<S: AsRef<str>>(
    bitstring: S,
    span: Span,
) -> Expr {
    let values = bitstring
        .as_ref()
        .chars()
        .filter_map(|c| {
            if c == '0' {
                Some(ast::Result::Zero)
            } else if c == '1' {
                Some(ast::Result::One)
            } else {
                None
            }
        })
        .collect();
    build_lit_result_array_expr(values, span)
}

pub(crate) fn build_lit_result_array_expr(values: Vec<qsc::ast::Result>, span: Span) -> Expr {
    let exprs: Vec<_> = values
        .into_iter()
        .map(|v| build_lit_result_expr(v, Span::default()))
        .collect();
    build_expr_array_expr(exprs, span)
}

pub(crate) fn build_expr_array_expr(values: Vec<qsc::ast::Expr>, span: Span) -> Expr {
    let exprs: Vec<_> = values.into_iter().map(Box::new).collect();
    Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(ExprKind::Array(exprs.into_boxed_slice())),
    }
}

pub(crate) fn build_default_result_array_expr(len: usize, span: Span) -> Expr {
    let exprs: Vec<_> = (0..len)
        .map(|_| Box::new(build_lit_result_expr(ast::Result::Zero, Span::default())))
        .collect();
    Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(ExprKind::Array(exprs.into_boxed_slice())),
    }
}

pub(crate) fn build_lit_bigint_expr(value: BigInt, span: Span) -> Expr {
    Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(ExprKind::Lit(Box::new(Lit::BigInt(Box::new(value))))),
    }
}

pub(crate) fn build_lit_bool_expr(value: bool, span: Span) -> Expr {
    Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(ExprKind::Lit(Box::new(Lit::Bool(value)))),
    }
}

pub(crate) fn build_lit_int_expr(value: i64, span: Span) -> Expr {
    Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(ExprKind::Lit(Box::new(Lit::Int(value)))),
    }
}

pub(crate) fn build_lit_complex_expr(value: Complex, span: Span) -> Expr {
    let real = build_lit_double_expr(value.real, Span::default());
    let img = build_lit_double_expr(value.imaginary, Span::default());
    build_math_call_from_exprs("Complex", vec![real, img], span)
}

pub(crate) fn build_complex_from_expr(expr: Expr) -> Expr {
    let img = build_lit_double_expr(0.0, Span::default());
    let span = expr.span;
    build_math_call_from_exprs("Complex", vec![expr, img], span)
}

pub(crate) fn build_binary_expr(
    is_assignment: bool,
    qsop: ast::BinOp,
    lhs: ast::Expr,
    rhs: ast::Expr,
    span: Span,
) -> ast::Expr {
    let expr_kind = if is_assignment {
        ast::ExprKind::AssignOp(qsop, Box::new(lhs), Box::new(rhs))
    } else {
        ast::ExprKind::BinOp(qsop, Box::new(lhs), Box::new(rhs))
    };
    ast::Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(expr_kind),
    }
}

pub(crate) fn is_complex_binop_supported(op: qsc::ast::BinOp) -> bool {
    matches!(
        op,
        ast::BinOp::Add | ast::BinOp::Sub | ast::BinOp::Mul | ast::BinOp::Div | ast::BinOp::Exp
    )
}

pub(crate) fn build_complex_binary_expr(
    is_assignment: bool,
    qsop: ast::BinOp,
    lhs: ast::Expr,
    rhs: ast::Expr,
    span: Span,
) -> ast::Expr {
    let name = match qsop {
        ast::BinOp::Add => "PlusC",
        ast::BinOp::Sub => "MinusC",
        ast::BinOp::Mul => "TimesC",
        ast::BinOp::Div => "DividedByC",
        ast::BinOp::Exp => "PowC",
        _ => unreachable!("Unsupported complex binary operation"),
    };

    if is_assignment {
        unreachable!("Unsupported complex binary operation");
    }
    build_math_call_from_exprs(name, vec![lhs, rhs], span)
}

pub(crate) fn build_math_call_from_exprs(name: &str, exprs: Vec<Expr>, span: Span) -> Expr {
    let alloc_ident = Ident {
        name: Rc::from(name),
        ..Default::default()
    };
    let path_expr = Expr {
        kind: Box::new(ExprKind::Path(Box::new(Path {
            segments: build_idents(&["Microsoft", "Quantum", "Math"]),
            name: Box::new(alloc_ident),
            ..Default::default()
        }))),
        ..Default::default()
    };
    let exprs: Vec<_> = exprs.into_iter().map(Box::new).collect();
    let kind = if exprs.is_empty() {
        ExprKind::Tuple(Box::new([]))
    } else if exprs.len() == 1 {
        ExprKind::Paren(exprs[0].clone())
    } else {
        ExprKind::Tuple(exprs.into_boxed_slice())
    };
    let call = ExprKind::Call(
        Box::new(path_expr),
        Box::new(Expr {
            kind: Box::new(kind),
            ..Default::default()
        }),
    );
    Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(call),
    }
}

pub(crate) fn build_path_ident_expr<S: AsRef<str>>(
    name: S,
    name_span: Span,
    expr_span: Span,
) -> ast::Expr {
    let ident = ast::Ident {
        id: NodeId::default(),
        span: name_span,
        name: Rc::from(name.as_ref()),
    };
    let path = ast::Path {
        id: NodeId::default(),
        span: Span::default(),
        segments: None,
        name: Box::new(ident),
    };
    let path_kind = ast::ExprKind::Path(Box::new(path));
    ast::Expr {
        id: NodeId::default(),
        span: expr_span,
        kind: Box::new(path_kind),
    }
}

pub(crate) fn build_indexed_assignment_statement<S: AsRef<str>>(
    name_span: Span,
    string_name: S,
    index_expr: ast::Expr,
    rhs: Expr,
    stmt_span: Span,
) -> ast::Stmt {
    let ident = ast::Ident {
        id: NodeId::default(),
        span: name_span,
        name: Rc::from(string_name.as_ref()),
    };

    let lhs = ast::Expr {
        id: NodeId::default(),
        span: name_span,
        kind: Box::new(ast::ExprKind::Path(Box::new(ast::Path {
            id: NodeId::default(),
            span: name_span,
            segments: None,
            name: Box::new(ident.clone()),
        }))),
    };

    let assign_up = ast::StmtKind::Semi(Box::new(ast::Expr {
        id: NodeId::default(),
        span: Span::default(),
        kind: Box::new(ast::ExprKind::AssignUpdate(
            Box::new(lhs),
            Box::new(index_expr),
            Box::new(rhs),
        )),
    }));
    ast::Stmt {
        id: NodeId::default(),
        span: stmt_span,
        kind: Box::new(assign_up),
    }
}

pub(crate) fn build_assignment_statement<S: AsRef<str>>(
    name_span: Span,
    name: S,
    rhs: Expr,
    assignment_span: Span,
) -> ast::Stmt {
    let ident = ast::Ident {
        id: NodeId::default(),
        span: name_span,
        name: Rc::from(name.as_ref()),
    };
    let path = ast::Path {
        id: NodeId::default(),
        span: name_span,
        name: Box::new(ident),
        segments: None,
    };
    let lhs = ast::Expr {
        id: NodeId::default(),
        span: name_span,
        kind: Box::new(ast::ExprKind::Path(Box::new(path))),
    };
    let expr_kind = ast::ExprKind::Assign(Box::new(lhs), Box::new(rhs));
    let expr = ast::Expr {
        id: NodeId::default(),
        span: assignment_span,
        kind: Box::new(expr_kind),
    };
    let semi = ast::StmtKind::Semi(Box::new(expr));
    ast::Stmt {
        id: NodeId::default(),
        span: assignment_span,
        kind: Box::new(semi),
    }
}

pub(crate) fn build_convert_call_expr(expr: Expr, name: &str) -> Expr {
    let span = expr.span;
    let cast_ident = Ident {
        name: Rc::from(name),
        ..Default::default()
    };
    let path_expr = Expr {
        kind: Box::new(ExprKind::Path(Box::new(Path {
            segments: build_idents(&["Microsoft", "Quantum", "Convert"]),
            name: Box::new(cast_ident),
            ..Default::default()
        }))),
        ..Default::default()
    };
    let call = ExprKind::Call(
        Box::new(path_expr),
        Box::new(Expr {
            kind: Box::new(ExprKind::Paren(Box::new(expr))),
            ..Default::default()
        }),
    );

    Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(call),
    }
}

pub(crate) fn build_array_reverse_expr(expr: Expr) -> Expr {
    let span = expr.span;
    let cast_ident = Ident {
        name: Rc::from("Reversed"),
        ..Default::default()
    };
    let path_expr = Expr {
        kind: Box::new(ExprKind::Path(Box::new(Path {
            segments: build_idents(&["Microsoft", "Quantum", "Arrays"]),
            name: Box::new(cast_ident),
            ..Default::default()
        }))),
        ..Default::default()
    };
    let call = ExprKind::Call(
        Box::new(path_expr),
        Box::new(Expr {
            kind: Box::new(ExprKind::Paren(Box::new(expr))),
            ..Default::default()
        }),
    );

    Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(call),
    }
}

#[allow(clippy::similar_names)]
pub(crate) fn build_range_expr(start: Expr, stop: Expr, step: Option<Expr>, span: Span) -> Expr {
    Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(ExprKind::Range(
            Some(Box::new(start)),
            step.map(Box::new),
            Some(Box::new(stop)),
        )),
    }
}

pub(crate) fn build_if_expr_then_block_else_expr(
    cond: Expr,
    then_expr: Block,
    else_expr: Option<Expr>,
    span: Span,
) -> Expr {
    let else_expr = else_expr.map(Box::new);
    let if_kind = ExprKind::If(Box::new(cond), Box::new(then_expr), else_expr);

    ast::Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(if_kind),
    }
}

pub(crate) fn build_if_expr_then_expr_else_expr(
    cond: Expr,
    then_expr: Expr,
    else_expr: Expr,
    span: Span,
) -> Expr {
    let if_kind = ExprKind::If(
        Box::new(cond),
        Box::new(build_expr_wrapped_block_expr(then_expr)),
        Some(Box::new(build_wrapped_block_expr(
            build_expr_wrapped_block_expr(else_expr),
        ))),
    );

    ast::Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(if_kind),
    }
}

pub(crate) fn build_if_expr_then_block_else_block(
    cond: Expr,
    then_block: Block,
    else_block: Block,
    span: Span,
) -> Expr {
    let if_kind = ExprKind::If(
        Box::new(cond),
        Box::new(then_block),
        Some(Box::new(build_wrapped_block_expr(else_block))),
    );

    ast::Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(if_kind),
    }
}

pub(crate) fn build_if_expr_then_block(cond: Expr, then_block: Block, span: Span) -> Expr {
    let if_kind = ExprKind::If(Box::new(cond), Box::new(then_block), None);

    ast::Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(if_kind),
    }
}

pub(crate) fn build_cast_call_two_params(
    function: RuntimeFunctions,
    fst: ast::Expr,
    snd: ast::Expr,
    name_span: Span,
    operand_span: Span,
) -> ast::Expr {
    let name = match function {
        RuntimeFunctions::IntAsResultArrayBE => "__IntAsResultArrayBE__",
        _ => panic!("Unsupported cast function"),
    };

    build_global_call_with_two_params(name, fst, snd, name_span, operand_span)
}

pub(crate) fn build_cast_call(
    function: RuntimeFunctions,
    expr: ast::Expr,
    name_span: Span,
    operand_span: Span,
) -> ast::Expr {
    let name = match function {
        RuntimeFunctions::BoolAsResult => "__BoolAsResult__",
        RuntimeFunctions::BoolAsInt => "__BoolAsInt__",
        RuntimeFunctions::BoolAsBigInt => "__BoolAsBigInt__",
        RuntimeFunctions::BoolAsDouble => "__BoolAsDouble__",
        RuntimeFunctions::ResultAsBool => "__ResultAsBool__",
        RuntimeFunctions::ResultAsInt => "__ResultAsInt__",
        RuntimeFunctions::ResultAsBigInt => "__ResultAsBigInt__",
        RuntimeFunctions::ResultArrayAsIntBE => "__ResultArrayAsIntBE__",
        _ => panic!("Unsupported cast function"),
    };
    build_global_call_with_one_param(name, expr, name_span, operand_span)
}

pub(crate) fn build_measure_call(
    expr: ast::Expr,
    name_span: Span,
    operand_span: Span,
    stmt_span: Span,
) -> ast::Expr {
    build_call_with_param(
        "__quantum__qis__m__body",
        &["QIR", "Intrinsic"],
        expr,
        name_span,
        operand_span,
        stmt_span,
    )
}

pub(crate) fn build_reset_call(expr: ast::Expr, name_span: Span, operand_span: Span) -> ast::Expr {
    build_global_call_with_one_param("Reset", expr, name_span, operand_span)
}

pub(crate) fn build_global_call_with_one_param<S: AsRef<str>>(
    name: S,
    expr: ast::Expr,
    name_span: Span,
    operand_span: Span,
) -> ast::Expr {
    let ident = ast::Ident {
        id: NodeId::default(),
        span: name_span,
        name: Rc::from(name.as_ref()),
    };
    let callee_expr = ast::Expr {
        id: NodeId::default(),
        span: name_span,
        kind: Box::new(ast::ExprKind::Path(Box::new(ast::Path {
            id: NodeId::default(),
            span: Span::default(),
            segments: None,
            name: Box::new(ident),
        }))),
    };

    let param_expr_kind = ast::ExprKind::Paren(Box::new(expr));

    let param_expr = ast::Expr {
        kind: Box::new(param_expr_kind),
        span: operand_span,
        ..Default::default()
    };
    let call_kind = ast::ExprKind::Call(Box::new(callee_expr), Box::new(param_expr));
    ast::Expr {
        kind: Box::new(call_kind),
        ..Default::default()
    }
}

pub(crate) fn build_global_call_with_two_params<S: AsRef<str>>(
    name: S,
    fst: ast::Expr,
    snd: ast::Expr,
    name_span: Span,
    operand_span: Span,
) -> ast::Expr {
    let ident = ast::Ident {
        id: NodeId::default(),
        span: name_span,
        name: Rc::from(name.as_ref()),
    };
    let callee_expr = ast::Expr {
        id: NodeId::default(),
        span: name_span,
        kind: Box::new(ast::ExprKind::Path(Box::new(ast::Path {
            id: NodeId::default(),
            span: Span::default(),
            segments: None,
            name: Box::new(ident),
        }))),
    };

    let param_expr_kind = ast::ExprKind::Tuple(Box::new([Box::new(fst), Box::new(snd)]));

    let param_expr = ast::Expr {
        kind: Box::new(param_expr_kind),
        span: operand_span,
        ..Default::default()
    };
    let call_kind = ast::ExprKind::Call(Box::new(callee_expr), Box::new(param_expr));
    ast::Expr {
        kind: Box::new(call_kind),
        ..Default::default()
    }
}

pub(crate) fn build_gate_call_with_params_and_callee(
    param_expr: ast::Expr,
    callee_expr: Expr,
    expr_span: Span,
) -> ast::Expr {
    let call_kind = ast::ExprKind::Call(Box::new(callee_expr), Box::new(param_expr));
    ast::Expr {
        kind: Box::new(call_kind),
        span: expr_span,
        ..Default::default()
    }
}

pub fn build_gate_call_param_expr(args: Vec<Expr>, remaining: usize) -> Expr {
    if args.len() == 1 && remaining > 0 {
        return args[0].clone();
    }
    let param_expr_kind = if args.len() == 1 {
        ast::ExprKind::Paren(Box::new(args[0].clone()))
    } else {
        let args: Vec<_> = args.into_iter().map(Box::new).collect();
        ast::ExprKind::Tuple(args.into_boxed_slice())
    };
    ast::Expr {
        kind: Box::new(param_expr_kind),
        ..Default::default()
    }
}

pub(crate) fn build_math_call_no_params(name: &str, span: Span) -> Expr {
    build_call_no_params(name, &["Microsoft", "Quantum", "Math"], span)
}

pub(crate) fn build_call_no_params(name: &str, idents: &[&str], span: Span) -> Expr {
    let segments = build_idents(idents);
    let fn_name = Ident {
        name: Rc::from(name),
        ..Default::default()
    };
    let path_expr = Expr {
        kind: Box::new(ExprKind::Path(Box::new(Path {
            segments,
            name: Box::new(fn_name),
            ..Default::default()
        }))),
        ..Default::default()
    };
    let call = ExprKind::Call(
        Box::new(path_expr),
        Box::new(create_unit_expr(Span::default())),
    );

    Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(call),
    }
}

pub(crate) fn build_call_with_param(
    name: &str,
    idents: &[&str],
    operand: Expr,
    name_span: Span,
    operand_span: Span,
    stmt_span: Span,
) -> Expr {
    let segments = build_idents(idents);
    let fn_name = Ident {
        name: Rc::from(name),
        span: name_span,
        ..Default::default()
    };
    let path_expr = Expr {
        kind: Box::new(ExprKind::Path(Box::new(Path {
            segments,
            name: Box::new(fn_name),
            ..Default::default()
        }))),
        ..Default::default()
    };
    let call = ExprKind::Call(
        Box::new(path_expr),
        Box::new(Expr {
            kind: Box::new(ExprKind::Paren(Box::new(operand))),
            span: operand_span,
            ..Default::default()
        }),
    );

    Expr {
        id: NodeId::default(),
        span: stmt_span,
        kind: Box::new(call),
    }
}

pub(crate) fn build_lit_double_expr(value: f64, span: Span) -> Expr {
    Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(ExprKind::Lit(Box::new(Lit::Double(value)))),
    }
}

pub(crate) fn build_stmt_semi_from_expr(expr: Expr) -> Stmt {
    Stmt {
        id: NodeId::default(),
        span: expr.span,
        kind: Box::new(StmtKind::Semi(Box::new(expr))),
    }
}

pub(crate) fn build_wrapped_block_expr(block: Block) -> Expr {
    Expr {
        id: NodeId::default(),
        span: block.span,
        kind: Box::new(ast::ExprKind::Block(Box::new(block))),
    }
}

pub(crate) fn build_stmt_wrapped_block_expr(stmt: Stmt) -> Block {
    Block {
        id: NodeId::default(),
        span: stmt.span,
        stmts: Box::new([Box::new(stmt)]),
    }
}

pub(crate) fn build_expr_wrapped_block_expr(expr: Expr) -> Block {
    Block {
        id: NodeId::default(),
        span: expr.span,
        stmts: Box::new([Box::new(Stmt {
            span: expr.span,
            kind: Box::new(StmtKind::Expr(Box::new(expr))),
            ..Default::default()
        })]),
    }
}

pub(crate) fn build_classical_decl<S>(
    name: S,
    is_const: bool,
    ty_span: Span,
    decl_span: Span,
    name_span: Span,
    ty: &crate::types::Type,
    expr: Expr,
) -> Stmt
where
    S: AsRef<str>,
{
    const USE_IMPLICIT_TYPE_DEF: bool = true;

    let ident = Ident {
        id: NodeId::default(),
        span: name_span,
        name: name.as_ref().into(),
    };

    let result_ty_ident = Ident {
        name: ty.to_string().into(),
        ..Default::default()
    };
    let result_ty_path = ast::Path {
        name: Box::new(result_ty_ident),
        ..Default::default()
    };
    let result_ty_kind = ast::TyKind::Path(Box::new(result_ty_path));

    let tydef = if USE_IMPLICIT_TYPE_DEF {
        None
    } else {
        Some(Box::new(ast::Ty {
            id: NodeId::default(),
            span: ty_span,
            kind: Box::new(result_ty_kind),
        }))
    };

    let pat = Pat {
        kind: Box::new(PatKind::Bind(Box::new(ident), tydef)),
        ..Default::default()
    };
    let mutability = if is_const {
        Mutability::Immutable
    } else {
        Mutability::Mutable
    };
    let local = StmtKind::Local(mutability, Box::new(pat), Box::new(expr));
    Stmt {
        id: NodeId::default(),
        span: decl_span,
        kind: Box::new(local),
    }
}

pub(crate) fn build_tuple_expr(output_exprs: Vec<Expr>) -> Expr {
    let boxed_exprs = output_exprs.into_iter().map(Box::new).collect::<Vec<_>>();
    let tuple_expr_kind = ExprKind::Tuple(boxed_exprs.into_boxed_slice());
    Expr {
        kind: Box::new(tuple_expr_kind),
        ..Default::default()
    }
}

pub(crate) fn build_implicit_return_stmt(output_expr: Expr) -> Stmt {
    Stmt {
        kind: Box::new(StmtKind::Expr(Box::new(output_expr))),
        ..Default::default()
    }
}

pub(crate) fn build_path_ident_ty<S: AsRef<str>>(name: S) -> Ty {
    let ident = ast::Ident {
        name: Rc::from(name.as_ref()),
        ..Default::default()
    };
    let path = ast::Path {
        name: Box::new(ident),
        ..Default::default()
    };
    let kind = TyKind::Path(Box::new(path));
    Ty {
        kind: Box::new(kind),
        ..Default::default()
    }
}

pub(crate) fn build_complex_ty_ident() -> Ty {
    let ident = ast::Ident {
        name: Rc::from("Complex"),
        ..Default::default()
    };
    let path = ast::Path {
        name: Box::new(ident),
        segments: build_idents(&["Microsoft", "Quantum", "Math"]),
        ..Default::default()
    };
    let kind = TyKind::Path(Box::new(path));
    Ty {
        kind: Box::new(kind),
        ..Default::default()
    }
}

pub(crate) fn build_top_level_ns_with_item<S: AsRef<str>>(
    whole_span: Span,
    ns: S,
    entry: ast::Item,
) -> TopLevelNode {
    TopLevelNode::Namespace(qsc::ast::Namespace {
        id: NodeId::default(),
        span: whole_span,
        name: qsc::ast::Idents(Box::new([Ident {
            name: Rc::from(ns.as_ref()),
            span: Span::default(),
            id: NodeId::default(),
        }])),
        items: Box::new([Box::new(entry)]),
        doc: "".into(),
    })
}

pub(crate) fn build_operation_with_stmts<S: AsRef<str>>(
    name: S,
    input_pats: Vec<Pat>,
    output_ty: Ty,
    stmts: Vec<ast::Stmt>,
    whole_span: Span,
) -> ast::Item {
    let mut attrs = vec![];
    // If there are no input parameters, add an attribute to mark this
    // as an entry point. We will get a Q# compilation error if we
    // attribute an operation with EntryPoint and it has input parameters.
    if input_pats.is_empty() {
        attrs.push(Box::new(qsc::ast::Attr {
            id: NodeId::default(),
            span: Span::default(),
            name: Box::new(qsc::ast::Ident {
                name: Rc::from("EntryPoint"),
                ..Default::default()
            }),
            arg: Box::new(create_unit_expr(Span::default())),
        }));
    }
    let input_pats = input_pats.into_iter().map(Box::new).collect::<Vec<_>>();
    let input = match input_pats.len() {
        0 => Box::new(Pat {
            kind: Box::new(qsc::ast::PatKind::Tuple(input_pats.into_boxed_slice())),
            ..Default::default()
        }),
        1 => Box::new(Pat {
            kind: Box::new(qsc::ast::PatKind::Paren(input_pats[0].clone())),
            ..Default::default()
        }),
        _ => Box::new(qsc::ast::Pat {
            kind: Box::new(qsc::ast::PatKind::Tuple(input_pats.into_boxed_slice())),
            ..Default::default()
        }),
    };

    let stmts = stmts
        .into_iter()
        .map(Box::new)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    qsc::ast::Item {
        id: NodeId::default(),
        span: whole_span,
        doc: "".into(),
        attrs: attrs.into_boxed_slice(),
        kind: Box::new(qsc::ast::ItemKind::Callable(Box::new(
            qsc::ast::CallableDecl {
                id: NodeId::default(),
                span: whole_span,
                kind: qsc::ast::CallableKind::Operation,
                name: Box::new(qsc::ast::Ident {
                    name: Rc::from(name.as_ref()),
                    ..Default::default()
                }),
                generics: Box::new([]),
                input,
                output: Box::new(output_ty),
                functors: None,
                body: Box::new(qsc::ast::CallableBody::Block(Box::new(qsc::ast::Block {
                    id: NodeId::default(),
                    span: whole_span,
                    stmts,
                }))),
            },
        ))),
    }
}

pub(crate) fn build_arg_pat(name: String, span: Span, ty: Ty) -> Pat {
    qsc::ast::Pat {
        kind: Box::new(qsc::ast::PatKind::Bind(
            Box::new(qsc::ast::Ident {
                name: Rc::from(name),
                span,
                ..Default::default()
            }),
            Some(Box::new(ty)),
        )),
        span,
        ..Default::default()
    }
}

pub(crate) fn build_unary_op_expr(op: ast::UnOp, expr: ast::Expr, prefix_span: Span) -> ast::Expr {
    ast::Expr {
        span: prefix_span,
        kind: Box::new(ast::ExprKind::UnOp(op, Box::new(expr))),
        ..Default::default()
    }
}

pub(crate) fn map_qsharp_type_to_ast_ty(output_ty: &crate::types::Type) -> Ty {
    match output_ty {
        crate::types::Type::Result(_) => build_path_ident_ty("Result"),
        crate::types::Type::Qubit => build_path_ident_ty("Qubit"),
        crate::types::Type::BigInt(_) => build_path_ident_ty("BigInt"),
        crate::types::Type::Int(_) => build_path_ident_ty("Int"),
        crate::types::Type::Double(_) => build_path_ident_ty("Double"),
        crate::types::Type::Complex(_) => build_complex_ty_ident(),
        crate::types::Type::Bool(_) => build_path_ident_ty("Bool"),
        crate::types::Type::ResultArray(dims, _) => build_array_type_name("Result", dims),
        crate::types::Type::QubitArray(dims) => build_array_type_name("Qubit", dims),
        crate::types::Type::BigIntArray(dims, _) => build_array_type_name("BigInt", dims),
        crate::types::Type::IntArray(dims, _) => build_array_type_name("Int", dims),
        crate::types::Type::DoubleArray(dims) => build_array_type_name("Double", dims),
        crate::types::Type::BoolArray(dims, _) => build_array_type_name("Bool", dims),
        crate::types::Type::Callable(_, _, _) => todo!(),
        crate::types::Type::Range => build_path_ident_ty("Range"),
        crate::types::Type::Tuple(tys) => {
            if tys.is_empty() {
                build_path_ident_ty("Unit")
            } else {
                let t = tys
                    .iter()
                    .map(map_qsharp_type_to_ast_ty)
                    .collect::<Vec<_>>();

                let kind = TyKind::Tuple(t.into_boxed_slice());
                Ty {
                    kind: Box::new(kind),
                    ..Default::default()
                }
            }
        }
        crate::types::Type::TupleArray(dims, tys) => {
            assert!(!tys.is_empty());
            let ty = map_qsharp_type_to_ast_ty(&crate::types::Type::Tuple(tys.clone()));
            wrap_array_ty_by_dims(dims, ty)
        }
    }
}

fn wrap_array_ty_by_dims(dims: &ArrayDimensions, ty: Ty) -> Ty {
    match dims {
        ArrayDimensions::One(..) => wrap_ty_in_array(ty),
        ArrayDimensions::Two(..) => wrap_ty_in_array(wrap_ty_in_array(ty)),
        ArrayDimensions::Three(..) => wrap_ty_in_array(wrap_ty_in_array(wrap_ty_in_array(ty))),
    }
}

fn build_array_type_name<S: AsRef<str>>(name: S, dims: &ArrayDimensions) -> Ty {
    let name = name.as_ref();
    let ty = build_path_ident_ty(name);
    wrap_array_ty_by_dims(dims, ty)
}

fn wrap_ty_in_array(ty: Ty) -> Ty {
    let kind = TyKind::Array(Box::new(ty));
    Ty {
        kind: Box::new(kind),
        ..Default::default()
    }
}

pub(crate) fn build_for_stmt(
    loop_var: &crate::symbols::Symbol,
    iter: crate::types::QasmTypedExpr,
    body: Block,
    stmt_span: Span,
) -> Stmt {
    Stmt {
        kind: Box::new(StmtKind::Expr(Box::new(Expr {
            kind: Box::new(ExprKind::For(
                Box::new(Pat {
                    kind: Box::new(PatKind::Bind(
                        Box::new(Ident {
                            name: loop_var.name.clone().into(),
                            span: loop_var.span,
                            ..Default::default()
                        }),
                        Some(Box::new(map_qsharp_type_to_ast_ty(&loop_var.qsharp_ty))),
                    )),
                    ..Default::default()
                }),
                Box::new(iter.expr),
                Box::new(body),
            )),
            span: stmt_span,
            ..Default::default()
        }))),
        span: stmt_span,
        ..Default::default()
    }
}

pub(crate) fn wrap_expr_in_parens(expr: Expr, span: Span) -> Expr {
    Expr {
        id: NodeId::default(),
        span,
        kind: Box::new(ExprKind::Paren(Box::new(expr))),
    }
}

pub(crate) fn build_while_stmt(expr: Expr, body: Block, stmt_span: Span) -> Stmt {
    Stmt {
        kind: Box::new(StmtKind::Expr(Box::new(Expr {
            kind: Box::new(ExprKind::While(Box::new(expr), Box::new(body))),
            span: stmt_span,
            ..Default::default()
        }))),
        span: stmt_span,
        ..Default::default()
    }
}

pub(crate) fn build_return_expr(expr: Expr, span: Span) -> Expr {
    Expr {
        kind: Box::new(ExprKind::Return(Box::new(expr))),
        span,
        ..Default::default()
    }
}

pub(crate) fn build_return_unit(span: Span) -> Expr {
    build_return_expr(create_unit_expr(span), span)
}

fn create_unit_expr(span: Span) -> Expr {
    Expr {
        span,
        kind: Box::new(ExprKind::Tuple(Box::new([]))),
        ..Default::default()
    }
}

pub(crate) fn build_end_stmt(span: Span) -> Stmt {
    let message = Expr {
        kind: Box::new(ExprKind::Lit(Box::new(Lit::String("end".into())))),
        ..Default::default()
    };

    let kind = ExprKind::Fail(Box::new(message));
    Stmt {
        kind: Box::new(StmtKind::Expr(Box::new(Expr {
            kind: Box::new(kind),
            span,
            ..Default::default()
        }))),
        span,
        ..Default::default()
    }
}

pub(crate) fn build_index_expr(expr: Expr, index_expr: Expr, span: Span) -> Expr {
    let kind = ExprKind::Index(Box::new(expr), Box::new(index_expr));
    Expr {
        kind: Box::new(kind),
        span,
        ..Default::default()
    }
}

pub(crate) fn build_barrier_call(span: Span) -> Stmt {
    let expr = build_call_no_params("__quantum__qis__barrier__body", &[], span);
    build_stmt_semi_from_expr(expr)
}

pub(crate) fn build_attr(text: String, span: Span) -> Attr {
    Attr {
        id: NodeId::default(),
        span,
        name: Box::new(Ident {
            name: Rc::from(text),
            span,
            ..Default::default()
        }),
        arg: Box::new(create_unit_expr(span)),
    }
}

pub(crate) fn build_gate_decl(
    name: String,
    cargs: Vec<(String, Ty, Pat)>,
    qargs: Vec<(String, Ty, Pat)>,
    body: Option<Block>,
    name_span: Span,
    body_span: Span,
    gate_span: Span,
) -> Stmt {
    let args = cargs
        .into_iter()
        .chain(qargs)
        .map(|(_, _, pat)| Box::new(pat))
        .collect::<Vec<_>>();

    let lo = args
        .iter()
        .min_by_key(|x| x.span.lo)
        .map(|x| x.span.lo)
        .unwrap_or_default();

    let hi = args
        .iter()
        .max_by_key(|x| x.span.hi)
        .map(|x| x.span.hi)
        .unwrap_or_default();

    let input_pat_kind = if args.len() > 1 {
        PatKind::Tuple(args.into_boxed_slice())
    } else {
        PatKind::Paren(args[0].clone())
    };

    let input_pat = Pat {
        kind: Box::new(input_pat_kind),
        span: Span { lo, hi },
        ..Default::default()
    };
    let body = CallableBody::Block(Box::new(body.unwrap_or_else(|| Block {
        id: NodeId::default(),
        span: body_span,
        stmts: Box::new([]),
    })));
    let decl = CallableDecl {
        id: NodeId::default(),
        span: name_span,
        kind: CallableKind::Operation,
        name: Box::new(Ident {
            name: name.into(),
            ..Default::default()
        }),
        generics: Box::new([]),
        input: Box::new(input_pat),
        output: Box::new(build_path_ident_ty("Unit")),
        functors: None,
        body: Box::new(body),
    };
    let item = Item {
        span: gate_span,
        kind: Box::new(ast::ItemKind::Callable(Box::new(decl))),
        ..Default::default()
    };

    Stmt {
        kind: Box::new(StmtKind::Item(Box::new(item))),
        span: gate_span,
        ..Default::default()
    }
}

pub(crate) fn build_gate_decl_lambda<S: AsRef<str>>(
    name: S,
    cargs: Vec<(String, Ty, Pat)>,
    qargs: Vec<(String, Ty, Pat)>,
    body: Option<Block>,
    name_span: Span,
    body_span: Span,
    gate_span: Span,
) -> Stmt {
    let args = cargs
        .into_iter()
        .chain(qargs)
        .map(|(name, ty, pat)| (name, ty, pat.span))
        .collect::<Vec<_>>();

    let lo = args
        .iter()
        .min_by_key(|(_, _, span)| span.lo)
        .map(|(_, _, span)| span.lo)
        .unwrap_or_default();

    let hi = args
        .iter()
        .max_by_key(|(_, _, span)| span.hi)
        .map(|(_, _, span)| span.hi)
        .unwrap_or_default();

    let name_args = args
        .iter()
        .map(|(name, _, span)| Pat {
            kind: Box::new(PatKind::Bind(
                Box::new(Ident {
                    span: *span,
                    name: Rc::from(name.as_ref()),
                    ..Default::default()
                }),
                None,
            )),
            ..Default::default()
        })
        .map(Box::new)
        .collect::<Vec<_>>();
    let input_pat = if args.len() > 1 {
        ast::Pat {
            kind: Box::new(PatKind::Tuple(name_args.into_boxed_slice())),
            span: Span { lo, hi },
            ..Default::default()
        }
    } else {
        ast::Pat {
            kind: Box::new(ast::PatKind::Paren(name_args[0].clone())),
            span: Span { lo, hi },
            ..Default::default()
        }
    };

    let block_expr = build_wrapped_block_expr(body.map_or_else(
        || Block {
            id: NodeId::default(),
            span: body_span,
            stmts: Box::new([]),
        },
        |block| block,
    ));
    let lambda_expr = Expr {
        id: NodeId::default(),
        kind: Box::new(ExprKind::Lambda(
            CallableKind::Operation,
            Box::new(input_pat),
            Box::new(block_expr),
        )),
        span: gate_span,
    };
    let ty_args = args.iter().map(|(_, ty, _)| ty.clone()).collect::<Vec<_>>();
    let input_ty = if args.len() > 1 {
        ast::Ty {
            kind: Box::new(ast::TyKind::Tuple(ty_args.into_boxed_slice())),
            ..Default::default()
        }
    } else {
        ast::Ty {
            kind: Box::new(ast::TyKind::Paren(Box::new(ty_args[0].clone()))),
            ..Default::default()
        }
    };
    let lambda_ty = ast::Ty {
        kind: Box::new(ast::TyKind::Arrow(
            CallableKind::Operation,
            Box::new(input_ty),
            Box::new(build_path_ident_ty("Unit")),
            None,
        )),
        ..Default::default()
    };
    Stmt {
        span: gate_span,
        kind: Box::new(StmtKind::Local(
            Mutability::Immutable,
            Box::new(Pat {
                kind: Box::new(PatKind::Bind(
                    Box::new(Ident {
                        span: name_span,
                        name: Rc::from(name.as_ref()),
                        ..Default::default()
                    }),
                    Some(Box::new(lambda_ty)),
                )),
                ..Default::default()
            }),
            Box::new(lambda_expr),
        )),
        ..Default::default()
    }
}

fn build_idents(idents: &[&str]) -> Option<Idents> {
    let idents = idents
        .iter()
        .map(|name| Ident {
            name: Rc::from(*name),
            ..Default::default()
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    if idents.is_empty() {
        None
    } else {
        Some(Idents(idents))
    }
}
