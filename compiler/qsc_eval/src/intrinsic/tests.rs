// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::f64::consts;
use std::io;

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore};
use qsc_passes::globals::extract_callables;

use crate::{
    output::{GenericReceiver, Receiver},
    val::Value,
    Env, Error, Evaluator,
};

fn check_intrinsic(file: &str, expr: &str, out: &mut dyn Receiver) -> Result<(Value, Env), Error> {
    let mut store = PackageStore::new();
    let stdlib = store.insert(compile::std());
    let unit = compile(&store, [stdlib], [file], expr);
    assert!(
        unit.context.errors().is_empty(),
        "compilation errors: {:?}",
        unit.context.errors()
    );
    let id = store.insert(unit);
    let unit = store
        .get(id)
        .expect("compile unit should be in package store");
    let globals = extract_callables(&store);
    let evaluator = Evaluator::from_store(&store, id, &globals, out);
    let expr = unit
        .package
        .entry
        .as_ref()
        .expect("entry expression should be present");
    evaluator.eval_expr(expr)
}

fn check_intrinsic_result(file: &str, expr: &str, expect: &Expect) {
    let mut stdout = vec![];
    let mut out = GenericReceiver::new(&mut stdout);
    match check_intrinsic(file, expr, &mut out) {
        Ok((result, _)) => expect.assert_eq(&result.to_string()),
        Err(e) => expect.assert_debug_eq(&e),
    }
}

fn check_intrinsic_output(file: &str, expr: &str, expect: &Expect) {
    let mut stdout = vec![];
    let mut out = GenericReceiver::new(&mut stdout);
    match check_intrinsic(file, expr, &mut out) {
        Ok((_, _)) => expect.assert_eq(
            &String::from_utf8(stdout).expect("content should be convertable to string"),
        ),
        Err(e) => expect.assert_debug_eq(&e),
    }
}

fn check_intrinsic_value(file: &str, expr: &str, val: &Value) {
    let mut stdout = vec![];
    let mut out = GenericReceiver::new(&mut stdout);
    match check_intrinsic(file, expr, &mut out) {
        Ok((result, _)) => assert_eq!(&result, val),
        Err(e) => panic!("{e:?}"),
    }
}

#[test]
fn length() {
    check_intrinsic_result("", "Length([1, 2, 3])", &expect!["3"]);
}

#[test]
fn length_type_err() {
    check_intrinsic_result(
        "",
        "Length((1, 2, 3))",
        &expect![[r#"
            Type(
                "Array",
                "Tuple",
                Span {
                    lo: 6,
                    hi: 17,
                },
            )
        "#]],
    );
}

#[test]
fn int_as_double() {
    check_intrinsic_result(
        "",
        "Microsoft.Quantum.Convert.IntAsDouble(2)",
        &expect!["2.0"],
    );
}

#[test]
fn int_as_double_type_error() {
    check_intrinsic_result(
        "",
        "Microsoft.Quantum.Convert.IntAsDouble(false)",
        &expect![[r#"
            Type(
                "Int",
                "Bool",
                Span {
                    lo: 37,
                    hi: 44,
                },
            )
        "#]],
    );
}

#[test]
fn int_as_double_precision_loss() {
    check_intrinsic_result(
        "",
        "Microsoft.Quantum.Convert.IntAsDouble(9_223_372_036_854_775_807)",
        &expect!["9223372036854775808.0"],
    );
}

#[test]
fn dump_machine() {
    check_intrinsic_output(
        "",
        "Microsoft.Quantum.Diagnostics.DumpMachine()",
        &expect![[r#"
            STATE:
            |0⟩: 1+0i
        "#]],
    );
}

#[test]
fn message() {
    check_intrinsic_output(
        "",
        r#"Message("Hello, World!")"#,
        &expect![[r#"
            Hello, World!
        "#]],
    );
}

#[test]
fn to_string() {
    check_intrinsic_result("", "ToString(One)", &expect![["One"]]);
}

#[test]
fn to_string_message() {
    check_intrinsic_output(
        "",
        r#"Message(ToString(PauliX))"#,
        &expect![[r#"
            PauliX
        "#]],
    );
}

#[test]
fn check_zero() {
    check_intrinsic_result(
        "",
        "{use q = Qubit(); Microsoft.Quantum.Diagnostics.CheckZero(q)}",
        &expect!["true"],
    );
}

#[test]
fn check_zero_false() {
    check_intrinsic_result(
        "",
        indoc! {"{
            use q = Qubit();
            X(q);
            Microsoft.Quantum.Diagnostics.CheckZero(q)
        }"},
        &expect!["false"],
    );
}

#[test]
fn arccos() {
    check_intrinsic_value(
        "",
        "Microsoft.Quantum.Math.ArcCos(0.3)",
        &Value::Double((0.3f64).acos()),
    );
}

#[test]
fn arcsin() {
    check_intrinsic_value(
        "",
        "Microsoft.Quantum.Math.ArcSin(0.3)",
        &Value::Double((0.3f64).asin()),
    );
}

#[test]
fn arctan() {
    check_intrinsic_value(
        "",
        "Microsoft.Quantum.Math.ArcTan(0.3)",
        &Value::Double((0.3f64).atan()),
    );
}

#[test]
fn arctan2() {
    check_intrinsic_value(
        "",
        "Microsoft.Quantum.Math.ArcTan2(0.3, 0.7)",
        &Value::Double((0.3f64).atan2(0.7)),
    );
}

#[test]
fn cos() {
    check_intrinsic_value(
        "",
        "Microsoft.Quantum.Math.Cos(Microsoft.Quantum.Math.PI())",
        &Value::Double((consts::PI).cos()),
    );
}

#[test]
fn cosh() {
    check_intrinsic_value(
        "",
        "Microsoft.Quantum.Math.Cosh(Microsoft.Quantum.Math.PI())",
        &Value::Double((consts::PI).cosh()),
    );
}

#[test]
fn sin() {
    check_intrinsic_value(
        "",
        "Microsoft.Quantum.Math.Sin(Microsoft.Quantum.Math.PI())",
        &Value::Double((consts::PI).sin()),
    );
}

#[test]
fn sinh() {
    check_intrinsic_value(
        "",
        "Microsoft.Quantum.Math.Sinh(Microsoft.Quantum.Math.PI())",
        &Value::Double((consts::PI).sinh()),
    );
}

#[test]
fn tan() {
    check_intrinsic_value(
        "",
        "Microsoft.Quantum.Math.Tan(Microsoft.Quantum.Math.PI())",
        &Value::Double((consts::PI).tan()),
    );
}

#[test]
fn tanh() {
    check_intrinsic_value(
        "",
        "Microsoft.Quantum.Math.Tanh(Microsoft.Quantum.Math.PI())",
        &Value::Double((consts::PI).tanh()),
    );
}

#[test]
fn ccx() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use (q1, q2, q3) = (Qubit(), Qubit(), Qubit());
            QIR.Intrinsic.__quantum__qis__ccx__body(q1, q2, q3);
            if not Microsoft.Quantum.Diagnostics.CheckZero(q3) {
                fail "Qubit should still be in zero state.";
            }
            X(q1);
            X(q2);
            QIR.Intrinsic.__quantum__qis__ccx__body(q1, q2, q3);
            if Microsoft.Quantum.Diagnostics.CheckZero(q3) {
                fail "Qubit should be in one state.";
            }
            X(q3);
            Microsoft.Quantum.Diagnostics.CheckZero(q3)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn cx() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use (q1, q2) = (Qubit(), Qubit());
            QIR.Intrinsic.__quantum__qis__cx__body(q1, q2);
            if not Microsoft.Quantum.Diagnostics.CheckZero(q2) {
                fail "Qubit should still be in zero state.";
            }
            X(q1);
            QIR.Intrinsic.__quantum__qis__cx__body(q1, q2);
            if Microsoft.Quantum.Diagnostics.CheckZero(q2) {
                fail "Qubit should be in one state.";
            }
            X(q2);
            Microsoft.Quantum.Diagnostics.CheckZero(q2)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn cy() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use (q1, q2) = (Qubit(), Qubit());
            QIR.Intrinsic.__quantum__qis__cy__body(q1, q2);
            if not Microsoft.Quantum.Diagnostics.CheckZero(q2) {
                fail "Qubit should still be in zero state.";
            }
            X(q1);
            QIR.Intrinsic.__quantum__qis__cy__body(q1, q2);
            if Microsoft.Quantum.Diagnostics.CheckZero(q2) {
                fail "Qubit should be in one state.";
            }
            Y(q2);
            Microsoft.Quantum.Diagnostics.CheckZero(q2)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn cz() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use (q1, q2) = (Qubit(), Qubit());
            H(q2);
            QIR.Intrinsic.__quantum__qis__cz__body(q1, q2);
            H(q2);
            if not Microsoft.Quantum.Diagnostics.CheckZero(q2) {
                fail "Qubit should still be in zero state.";
            }
            X(q1);
            H(q2);
            QIR.Intrinsic.__quantum__qis__cz__body(q1, q2);
            H(q2);
            if Microsoft.Quantum.Diagnostics.CheckZero(q2) {
                fail "Qubit should be in one state.";
            }
            X(q2);
            Microsoft.Quantum.Diagnostics.CheckZero(q2)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn rx() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use q1 = Qubit();
            let pi = Microsoft.Quantum.Math.PI();
            QIR.Intrinsic.__quantum__qis__rx__body(pi, q1);
            if Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in one state.";
            }
            X(q1);
            Microsoft.Quantum.Diagnostics.CheckZero(q1)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn rxx() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use (q1, q2) = (Qubit(), Qubit());
            let pi = Microsoft.Quantum.Math.PI();
            QIR.Intrinsic.__quantum__qis__rxx__body(pi, q1, q2);
            if Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in one state.";
            }
            if Microsoft.Quantum.Diagnostics.CheckZero(q2) {
                fail "Qubit 2 should be in one state.";
            }
            X(q1);
            X(q2);
            (Microsoft.Quantum.Diagnostics.CheckZero(q1), Microsoft.Quantum.Diagnostics.CheckZero(q2))
        }"#},
        &expect!["(true, true)"],
    );
}

#[test]
fn ry() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use q1 = Qubit();
            let pi = Microsoft.Quantum.Math.PI();
            QIR.Intrinsic.__quantum__qis__ry__body(pi, q1);
            if Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in one state.";
            }
            Y(q1);
            Microsoft.Quantum.Diagnostics.CheckZero(q1)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn ryy() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use (q1, q2) = (Qubit(), Qubit());
            let pi = Microsoft.Quantum.Math.PI();
            QIR.Intrinsic.__quantum__qis__ryy__body(pi, q1, q2);
            if Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in one state.";
            }
            if Microsoft.Quantum.Diagnostics.CheckZero(q2) {
                fail "Qubit 2 should be in one state.";
            }
            Y(q1);
            Y(q2);
            (Microsoft.Quantum.Diagnostics.CheckZero(q1), Microsoft.Quantum.Diagnostics.CheckZero(q2))
        }"#},
        &expect!["(true, true)"],
    );
}

#[test]
fn rz() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use q1 = Qubit();
            let pi = Microsoft.Quantum.Math.PI();
            H(q1);
            QIR.Intrinsic.__quantum__qis__rz__body(pi, q1);
            H(q1);
            if Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in one state.";
            }
            H(q1);
            Z(q1);
            H(q1);
            Microsoft.Quantum.Diagnostics.CheckZero(q1)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn rzz() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use (q1, q2) = (Qubit(), Qubit());
            let pi = Microsoft.Quantum.Math.PI();
            H(q1);
            H(q2);
            QIR.Intrinsic.__quantum__qis__rzz__body(pi, q1, q2);
            H(q1);
            H(q2);
            if Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in one state.";
            }
            if Microsoft.Quantum.Diagnostics.CheckZero(q2) {
                fail "Qubit 2 should be in one state.";
            }
            H(q1);
            H(q2);
            Z(q1);
            Z(q2);
            H(q1);
            H(q2);
            (Microsoft.Quantum.Diagnostics.CheckZero(q1), Microsoft.Quantum.Diagnostics.CheckZero(q2))
        }"#},
        &expect!["(true, true)"],
    );
}

#[test]
fn h() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use q1 = Qubit();
            QIR.Intrinsic.__quantum__qis__h__body(q1);
            if Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in one state.";
            }
            H(q1);
            Microsoft.Quantum.Diagnostics.CheckZero(q1)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn s() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use q1 = Qubit();
            H(q1);
            QIR.Intrinsic.__quantum__qis__s__body(q1);
            H(q1);
            if Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in one state.";
            }
            H(q1);
            QIR.Intrinsic.__quantum__qis__s__body(q1);
            QIR.Intrinsic.__quantum__qis__s__body(q1);
            QIR.Intrinsic.__quantum__qis__s__body(q1);
            H(q1);
            Microsoft.Quantum.Diagnostics.CheckZero(q1)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn sadj() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use q1 = Qubit();
            H(q1);
            QIR.Intrinsic.__quantum__qis__s__adj(q1);
            H(q1);
            if Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in one state.";
            }
            H(q1);
            QIR.Intrinsic.__quantum__qis__s__adj(q1);
            QIR.Intrinsic.__quantum__qis__s__adj(q1);
            QIR.Intrinsic.__quantum__qis__s__adj(q1);
            H(q1);
            Microsoft.Quantum.Diagnostics.CheckZero(q1)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn t() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use q1 = Qubit();
            H(q1);
            QIR.Intrinsic.__quantum__qis__t__body(q1);
            QIR.Intrinsic.__quantum__qis__t__body(q1);
            H(q1);
            if Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in one state.";
            }
            H(q1);
            QIR.Intrinsic.__quantum__qis__t__body(q1);
            QIR.Intrinsic.__quantum__qis__t__body(q1);
            QIR.Intrinsic.__quantum__qis__t__body(q1);
            QIR.Intrinsic.__quantum__qis__t__body(q1);
            QIR.Intrinsic.__quantum__qis__t__body(q1);
            QIR.Intrinsic.__quantum__qis__t__body(q1);
            H(q1);
            Microsoft.Quantum.Diagnostics.CheckZero(q1)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn tadj() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use q1 = Qubit();
            H(q1);
            QIR.Intrinsic.__quantum__qis__t__adj(q1);
            QIR.Intrinsic.__quantum__qis__t__adj(q1);
            H(q1);
            if Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in one state.";
            }
            H(q1);
            QIR.Intrinsic.__quantum__qis__t__adj(q1);
            QIR.Intrinsic.__quantum__qis__t__adj(q1);
            QIR.Intrinsic.__quantum__qis__t__adj(q1);
            QIR.Intrinsic.__quantum__qis__t__adj(q1);
            QIR.Intrinsic.__quantum__qis__t__adj(q1);
            QIR.Intrinsic.__quantum__qis__t__adj(q1);
            H(q1);
            Microsoft.Quantum.Diagnostics.CheckZero(q1)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn x() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use q1 = Qubit();
            QIR.Intrinsic.__quantum__qis__x__body(q1);
            if Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in one state.";
            }
            QIR.Intrinsic.__quantum__qis__x__body(q1);
            Microsoft.Quantum.Diagnostics.CheckZero(q1)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn y() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use q1 = Qubit();
            QIR.Intrinsic.__quantum__qis__y__body(q1);
            if Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in one state.";
            }
            QIR.Intrinsic.__quantum__qis__y__body(q1);
            Microsoft.Quantum.Diagnostics.CheckZero(q1)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn z() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use q1 = Qubit();
            H(q1);
            QIR.Intrinsic.__quantum__qis__z__body(q1);
            H(q1);
            if Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in one state.";
            }
            H(q1);
            QIR.Intrinsic.__quantum__qis__z__body(q1);
            H(q1);
            Microsoft.Quantum.Diagnostics.CheckZero(q1)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn swap() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use (q1, q2) = (Qubit(), Qubit());
            X(q2);
            QIR.Intrinsic.__quantum__qis__swap__body(q1, q2);
            if not Microsoft.Quantum.Diagnostics.CheckZero(q2) {
                fail "Qubit should be swapped to zero state.";
            }
            if Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should swapped to one state.";
            }
            X(q1);
            (Microsoft.Quantum.Diagnostics.CheckZero(q2), Microsoft.Quantum.Diagnostics.CheckZero(q2))
        }"#},
        &expect!["(true, true)"],
    );
}

#[test]
fn reset() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use q1 = Qubit();
            QIR.Intrinsic.__quantum__qis__reset__body(q1);
            if not Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in zero state.";
            }
            X(q1);
            QIR.Intrinsic.__quantum__qis__reset__body(q1);
            Microsoft.Quantum.Diagnostics.CheckZero(q1)
        }"#},
        &expect!["true"],
    );
}

#[test]
fn m() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use q1 = Qubit();
            if not Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in zero state.";
            }
            let res1 = QIR.Intrinsic.__quantum__qis__m__body(q1);
            if One == res1 {
                fail "Qubit should measure Zero"
            }
            if not Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in zero state.";
            }
            X(q1);
            let res2 = QIR.Intrinsic.__quantum__qis__m__body(q1);
            (res2, Microsoft.Quantum.Diagnostics.CheckZero(q1))
        }"#},
        &expect!["(One, false)"],
    );
}

#[test]
fn mresetz() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use q1 = Qubit();
            if not Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in zero state.";
            }
            let res1 = QIR.Intrinsic.__quantum__qis__mresetz__body(q1);
            if One == res1 {
                fail "Qubit should measure Zero"
            }
            if not Microsoft.Quantum.Diagnostics.CheckZero(q1) {
                fail "Qubit should be in zero state.";
            }
            X(q1);
            let res2 = QIR.Intrinsic.__quantum__qis__mresetz__body(q1);
            (res2, Microsoft.Quantum.Diagnostics.CheckZero(q1))
        }"#},
        &expect!["(One, true)"],
    );
}

#[test]
fn unknown_intrinsic() {
    check_intrinsic_result(
        indoc! {"
            namespace Test {
                function Foo() : Int {
                    body intrinsic;
                }
            }
        "},
        "Test.Foo()",
        &expect![[r#"
            UnknownIntrinsic(
                Span {
                    lo: 76,
                    hi: 84,
                },
            )
        "#]],
    );
}
