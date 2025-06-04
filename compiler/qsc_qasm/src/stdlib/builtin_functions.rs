// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Reference: <https://openqasm.com/versions/3.0/language/types.html#built-in-constant-expression-functions>
//!
//! The following are compile-time functions that take const inputs and have a const output.
//! The normal implicit casting rules apply to the inputs of these functions.

use std::fmt::Write;

use crate::semantic::{
    ast::{Expr, ExprKind, LiteralKind},
    const_eval::ConstEvalError,
    types::{can_cast_literal, can_cast_literal_with_value_knowledge, Type},
    Lowerer,
};
use qsc_data_structures::span::Span;

// ---------------------------------------------------
// Dispatch mechanism for polymorphic function calls.
// ---------------------------------------------------

/// The output of calling a polymorphic function is a pair
/// where the first element is the result of the computation
/// and the second element is the type of the monomorphic
/// function that was selected, if any.
type PolymorphicFunctionOutput = Option<(Expr, Type)>;

// A function table mapping function signatures to functions.
// This is a vector and not a hash-map because the order of
// iteration matters. Overloads should be tried in the order
// they appear.
type FnTable = Vec<(Type, Box<dyn Fn(&[Expr], Span) -> Expr>)>;

fn dispatch(
    name: &str,
    call_span: Span,
    inputs: &[Expr],
    fn_table: FnTable,
    ctx: &mut Lowerer,
) -> PolymorphicFunctionOutput {
    // All the builtin functions take const expressions as inputs
    // and return a const expression as output. If any of the
    // inputs is not const, we return an error.
    if inputs.iter().filter_map(|e| check_const(e, ctx)).count() != inputs.len() {
        return None;
    }

    // Reference: <https://openqasm.com/versions/3.0/language/types.html#built-in-constant-expression-functions>
    //
    // For each built-in function, the chosen overload is the first one to appear in Table 2 in
    // the link above where all given operands can be implicitly cast to the valid input types.
    // The output type is not considered when choosing an overload. It is an error if there is
    // no valid overload for a given sequence of operands.
    for (signature, function) in &fn_table {
        if let Some(new_inputs) = try_implicit_cast_inputs(inputs, signature, ctx) {
            let output = function(&new_inputs, call_span);
            return Some((output, signature.clone()));
        }
    }

    ctx.push_const_eval_error(no_valid_overload_error(name, call_span, inputs, fn_table));

    None
}

fn check_const(expr: &Expr, ctx: &mut Lowerer) -> Option<()> {
    if expr.ty.is_const() && expr.const_value.is_some() {
        Some(())
    } else {
        ctx.push_const_eval_error(ConstEvalError::ExprMustBeConst(expr.span));
        None
    }
}

/// A helper macro for unwrapping the literal value of a const expression.
macro_rules! unwarp_lit {
    // This pattern is used for unary expressions.
    ($const_expr:expr, $pat:pat) => {
        #[allow(irrefutable_let_patterns)]
        let $pat = $const_expr.get_const_value().expect("expr is const") else {
            unreachable!("if we hit this, there is a bug in our dispatch mechanism")
        };
    };
}

fn try_implicit_cast_inputs(
    inputs: &[Expr],
    signature: &Type,
    ctx: &mut Lowerer,
) -> Option<Vec<Expr>> {
    let mut new_inputs = Vec::with_capacity(inputs.len());
    let Type::Function(input_types, _) = signature else {
        unreachable!("if we hit this we are initializing the function table incorrectly");
    };

    if inputs.len() != input_types.len() {
        return None;
    }

    for (input, ty) in inputs.iter().zip(input_types.iter()) {
        unwarp_lit!(input, value);
        if can_cast_literal(ty, &input.ty) || can_cast_literal_with_value_knowledge(ty, &value) {
            let mut value_expr = input.clone();
            // `coerce_literal_expr_to_type` expects a value expression.
            // So, we build an adhoc expression where `Expr::Kind` is a
            // `LiteralKind` to satisfy this method.
            value_expr.kind = Box::new(ExprKind::Lit(
                input.get_const_value().expect("input should be const"),
            ));
            let coerced_input = ctx.coerce_literal_expr_to_type(ty, &value_expr, &value);
            new_inputs.push(coerced_input.with_const_value(ctx));
        } else {
            return None;
        }
    }

    Some(new_inputs)
}

fn no_valid_overload_error(
    name: &str,
    call_span: Span,
    inputs: &[Expr],
    fn_table: FnTable,
) -> ConstEvalError {
    let mut error_msg = String::new();
    write!(
        error_msg,
        "There is no valid overload of `{name}` for inputs: "
    )
    .expect("write should succeed");

    let inputs_str = inputs
        .iter()
        .map(|expr| expr.ty.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    write!(error_msg, "({inputs_str})").expect("write should succeed");

    write!(error_msg, "\nOverloads available are:").expect("write should succeed");

    for (signature, _) in fn_table {
        write!(error_msg, "\n  {signature}").expect("write should succeed");
    }

    ConstEvalError::NoValidOverloadForBuiltinFunction(error_msg, call_span)
}

// ---------------------------------------
// Helper functions to reduce boilerplate
// when initializing the function tables.
// ---------------------------------------

pub fn fun(inputs: &[Type], output: Type) -> Type {
    Type::Function(inputs.into(), output.into())
}

pub fn int() -> Type {
    Type::Int(None, true)
}

pub fn float() -> Type {
    Type::Float(None, true)
}

// ----------------------------------
// Builtin functions implementation.
// ----------------------------------

pub(crate) fn mod_(
    inputs: &[Expr],
    call_span: Span,
    ctx: &mut Lowerer,
) -> PolymorphicFunctionOutput {
    let fn_table: FnTable = vec![
        (fun(&[int(), int()], int()), Box::new(mod_int)),
        (fun(&[float(), float()], float()), Box::new(mod_float)),
    ];

    dispatch("mod", call_span, inputs, fn_table, ctx)
}

pub(crate) fn mod_int(inputs: &[Expr], span: Span) -> Expr {
    unwarp_lit!(inputs[0], LiteralKind::Int(a));
    unwarp_lit!(inputs[1], LiteralKind::Int(b));
    Expr::int(a.rem_euclid(b), span)
}

pub(crate) fn mod_float(inputs: &[Expr], span: Span) -> Expr {
    unwarp_lit!(inputs[0], LiteralKind::Float(a));
    unwarp_lit!(inputs[1], LiteralKind::Float(b));
    Expr::float(a.rem_euclid(b), span)
}
