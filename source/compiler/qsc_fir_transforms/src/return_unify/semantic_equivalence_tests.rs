// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::test_utils::check_semantic_equivalence;
use crate::test_utils::check_semantic_equivalence_with_expected as check_value;
use indoc::formatdoc;
use proptest::prelude::*;
use qsc_eval::val::Value;

use crate::test_utils::{
    TraceOp::{QubitAllocate, QubitRelease, Reset},
    compile_to_fir, try_eval_fir_entry_with_trace,
};
use qsc_fir::{
    fir::{ExprKind, PatKind, StmtKind},
    ty::{GenericArg, Prim, Ty},
};

/// Generates syntactically valid Q# programs with return statements at
/// various positions covering all `return_unify` dispatch categories
/// (structured, flag, no-return). Each program wraps one of 12 template
/// patterns in a `namespace Test { function Main() : Int { ... } }` shell.
#[allow(clippy::too_many_lines)]
fn return_pattern_strategy() -> impl Strategy<Value = String> {
    let cmp = || 0..10i64;
    let val = || 0..100i64;
    let bound = || 1..6i64;
    let idx = || 0..5i64;

    prop_oneof![
        // 1. No-return baseline: pure if-else expression.
        (cmp(), cmp(), val(), val()).prop_map(|(a, b, c, d)| formatdoc! {"
            namespace Test {{
                function Main() : Int {{
                    if {a} > {b} {{ {c} }} else {{ {d} }}
                }}
            }}
        "}),
        // 2. Single guard clause.
        (cmp(), cmp(), val(), val()).prop_map(|(a, b, c, d)| formatdoc! {"
            namespace Test {{
                function Main() : Int {{
                    if {a} > {b} {{ return {c}; }}
                    {d}
                }}
            }}
        "}),
        // 3. Both branches return.
        (cmp(), cmp(), val(), val()).prop_map(|(a, b, c, d)| formatdoc! {"
            namespace Test {{
                function Main() : Int {{
                    if {a} > {b} {{ return {c}; }} else {{ return {d}; }}
                }}
            }}
        "}),
        // 4. Two guard clauses with fallthrough.
        (cmp(), cmp(), cmp(), cmp(), val(), val(), val()).prop_map(
            |(a, b, c, d, e, f, g)| formatdoc! {"
                namespace Test {{
                    function Main() : Int {{
                        if {a} > {b} {{ return {e}; }}
                        if {c} > {d} {{ return {f}; }}
                        {g}
                    }}
                }}
            "}
        ),
        // 5. While with early return.
        (bound(), idx(), val(), val()).prop_map(|(n, t, v, d)| formatdoc! {"
            namespace Test {{
                function Main() : Int {{
                    mutable x = 0;
                    while x < {n} {{
                        if x == {t} {{ return {v}; }}
                        x += 1;
                    }}
                    {d}
                }}
            }}
        "}),
        // 6. For loop with early return.
        (bound(), idx(), val(), val()).prop_map(|(n, t, v, d)| formatdoc! {"
            namespace Test {{
                function Main() : Int {{
                    for i in 0..{n} {{
                        if i == {t} {{ return {v}; }}
                    }}
                    {d}
                }}
            }}
        "}),
        // 7. Nested if with return.
        (cmp(), cmp(), cmp(), cmp(), val(), val(), val()).prop_map(
            |(a, b, c, d, e, f, g)| formatdoc! {"
                namespace Test {{
                    function Main() : Int {{
                        if {a} > {b} {{
                            if {c} > {d} {{ return {e}; }}
                            {f}
                        }} else {{
                            {g}
                        }}
                    }}
                }}
            "}
        ),
        // 8. Block expression with return.
        (cmp(), cmp(), val(), val(), val()).prop_map(|(a, b, c, d, e)| formatdoc! {"
            namespace Test {{
                function Main() : Int {{
                    let x = {{
                        if {a} > {b} {{ return {c}; }}
                        {d}
                    }};
                    x + {e}
                }}
            }}
        "}),
        // 9. Return in else branch only.
        (cmp(), cmp(), val(), val()).prop_map(|(a, b, c, d)| formatdoc! {"
            namespace Test {{
                function Main() : Int {{
                    if {a} > {b} {{ {c} }} else {{ return {d}; }}
                }}
            }}
        "}),
        // 10. Multiple returns with mutable computation.
        (cmp(), cmp(), cmp(), cmp(), val(), val(), val(), val()).prop_map(
            |(a, b, c, d, e, f, g, h)| formatdoc! {"
                namespace Test {{
                    function Main() : Int {{
                        mutable result = 0;
                        if {a} > {b} {{ return {e}; }}
                        result = {f};
                        if {c} > {d} {{ return {g}; }}
                        result + {h}
                    }}
                }}
            "}
        ),
        // 11. Triple nested if-return.
        (
            cmp(),
            cmp(),
            cmp(),
            cmp(),
            cmp(),
            cmp(),
            val(),
            val(),
            val(),
            val()
        )
            .prop_map(|(a, b, c, d, e, f, g, h, i, j)| formatdoc! {"
                namespace Test {{
                    function Main() : Int {{
                        if {a} > {b} {{
                            if {c} > {d} {{
                                if {e} > {f} {{ return {g}; }}
                                return {h};
                            }}
                            {i}
                        }} else {{
                            return {j};
                        }}
                    }}
                }}
            "}),
        // 12. While with accumulator and conditional return.
        (bound(), idx()).prop_map(|(n, t)| formatdoc! {"
            namespace Test {{
                function Main() : Int {{
                    mutable acc = 0;
                    mutable i = 0;
                    while i < {n} {{
                        if i > {t} {{ return acc; }}
                        acc = acc + i;
                        i += 1;
                    }}
                    acc
                }}
            }}
        "}),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn differential_return_unify(source in return_pattern_strategy()) {
        check_semantic_equivalence(&source);
    }
}

fn check_preserved_failure(source: &str, message: &str, trace: Vec<crate::test_utils::TraceOp>) {
    use crate::test_utils::{
        PipelineStage, compile_and_run_pipeline_to, compile_to_fir, try_eval_fir_entry_with_trace,
    };

    let (store, package_id) = compile_to_fir(source);
    let fail_span = store
        .get(package_id)
        .exprs
        .iter()
        .find_map(|(_, expr)| {
            matches!(expr.kind, qsc_fir::fir::ExprKind::Fail(_)).then_some(expr.span)
        })
        .expect("source must contain a fail expression");
    let expected = (
        Err(format!(
            "{:?}",
            qsc_eval::Error::UserFail(
                message.into(),
                (
                    qsc_lowerer::map_fir_package_to_hir(fail_span.package),
                    fail_span.span
                )
                    .into()
            )
        )),
        trace,
    );
    assert_eq!(try_eval_fir_entry_with_trace(&store, package_id), expected);
    for stage in [PipelineStage::ReturnUnify, PipelineStage::Full] {
        let (mut store, package_id) = compile_and_run_pipeline_to(source, stage);
        crate::exec_graph_rebuild::rebuild_exec_graphs(&mut store, package_id, &[]);
        assert_eq!(
            try_eval_fir_entry_with_trace(&store, package_id),
            expected,
            "failure must survive {stage:?}: {source}"
        );
    }
}

fn check_generated_array_loop_types(source: &str, element: &qsc_fir::ty::Ty) {
    // Inspect the freshly lowered FIR: evaluating a divergent iterable never
    // reaches Length, and later passes may remove the ill-typed generated code.
    let (store, package_id) = compile_to_fir(source);
    let package = store.get(package_id);
    let array_ty = Ty::Array(Box::new(element.clone()));
    let mut captures = 0;
    let mut lengths = 0;
    for (_, stmt) in &package.stmts {
        let StmtKind::Local(_, pat, initializer) = &stmt.kind else {
            continue;
        };
        let pat = package.pats.get(*pat).expect("binding pattern must exist");
        let PatKind::Bind(ident) = &pat.kind else {
            continue;
        };
        let initializer = package
            .exprs
            .get(*initializer)
            .expect("binding initializer must exist");
        if ident.name.starts_with(".array_id") {
            captures += 1;
            assert_eq!(pat.ty, array_ty, "{source}");
            assert_eq!(initializer.ty, array_ty, "{source}");
        }
        if ident.name.starts_with(".len_id") {
            lengths += 1;
            let ExprKind::Call(callee, argument) = &initializer.kind else {
                panic!("generated Length binding must be a call");
            };
            assert_eq!(
                package
                    .exprs
                    .get(*argument)
                    .expect("argument must exist")
                    .ty,
                array_ty,
                "{source}"
            );
            let ExprKind::Var(_, generics) =
                &package.exprs.get(*callee).expect("callee must exist").kind
            else {
                panic!("generated Length callee must be a variable");
            };
            assert_eq!(generics, &[GenericArg::Ty(element.clone())], "{source}");
        }
    }
    assert!(captures > 0, "fixture must lower an array loop: {source}");
    assert_eq!(captures, lengths, "{source}");
}

#[test]
fn failing_for_iterables_preserve_user_failure_and_source_location() {
    for body in [
        "let value = for item : String in fail \"stop\" {};",
        "for item : String in fail \"stop\" {}",
        // Explicit array context is the compatibility control for the ordinary path.
        "let items : String[] = fail \"stop\"; for item in items {}",
    ] {
        let source = format!("@EntryPoint() operation Main() : Unit {{ {body} }}");
        check_preserved_failure(&source, "stop", Vec::new());
    }
}

#[test]
fn return_in_for_iterable_returns_seventeen() {
    check_value(
        "@EntryPoint() operation Main() : Int { for item : String in return 17 {} 0 }",
        Value::Int(17),
    );
}

#[test]
fn generated_adjoint_of_failing_for_iterable_reports_failure_before_gates() {
    for iterable in [
        "for item : String in fail \"iter\" { X(q); }",
        "let items : String[] = fail \"iter\"; for item in items { X(q); }",
    ] {
        let source = format!(
            "operation A(q : Qubit) : Unit is Adj {{ {iterable} }}
             @EntryPoint() operation Main() : Unit {{
                 use q = Qubit();
                 Adjoint A(q);
                 Reset(q);
             }}"
        );
        check_preserved_failure(
            &source,
            "iter",
            vec![crate::test_utils::TraceOp::QubitAllocate(0)],
        );
    }
}

#[test]
fn loop_control_in_qubit_argument_preserves_singleton_type_and_skips_call() {
    for transfer in ["break", "continue"] {
        for take_transfer in [true, false] {
            let source = format!(
                "operation UseQubit(q : Qubit) : Unit {{ Reset(q); }}
                 @EntryPoint() operation Main() : Unit {{
                     use q = Qubit();
                     mutable keepGoing = true;
                     while keepGoing {{
                         UseQubit(if {take_transfer} {{
                             keepGoing = false;
                             {transfer}
                         }} else {{ q }});
                         keepGoing = false;
                     }}
                 }}"
            );
            let (store, package_id) = compile_to_fir(&source);
            let package = store.get(package_id);
            // Check the generated singleton before FIR cleanup can hide an invalid
            // type on a skipped branch. Its element must keep the moved qubit's type.
            let mut singletons = 0;
            for (_, expr) in &package.exprs {
                let (ExprKind::Array(elements) | ExprKind::ArrayLit(elements)) = &expr.kind else {
                    continue;
                };
                let [element] = elements.as_slice() else {
                    continue;
                };
                let element = package.exprs.get(*element).expect("array element");
                if element.ty == Ty::Prim(Prim::Qubit) {
                    singletons += 1;
                    assert_eq!(expr.ty, Ty::Array(Box::new(element.ty.clone())), "{source}");
                }
            }
            assert!(
                singletons > 0,
                "fixture must exercise array backing: {source}"
            );
            let trace = if take_transfer {
                vec![QubitAllocate(0), QubitRelease(0)]
            } else {
                vec![QubitAllocate(0), Reset(0), QubitRelease(0)]
            };
            assert_eq!(
                try_eval_fir_entry_with_trace(&store, package_id),
                (Ok(Value::unit()), trace),
                "{source}"
            );
            check_value(&source, Value::unit());
        }
    }
}

#[test]
fn divergent_for_iterables_generate_typed_array_captures_and_length_calls() {
    for (pattern, element) in [
        ("item : Int", Ty::Prim(Prim::Int)),
        ("item : String", Ty::Prim(Prim::String)),
        ("item : Int[]", Ty::Array(Box::new(Ty::Prim(Prim::Int)))),
        (
            "(number : Int, text : String)",
            Ty::Tuple(vec![Ty::Prim(Prim::Int), Ty::Prim(Prim::String)]),
        ),
    ] {
        for body in [
            format!("let value = for {pattern} in fail \"stop\" {{}};"),
            format!("for {pattern} in fail \"stop\" {{}}"),
            format!("for {pattern} in return 17 {{}}"),
        ] {
            let source = format!("@EntryPoint() operation Main() : Int {{ {body} 0 }}");
            check_generated_array_loop_types(&source, &element);
        }
    }
}
