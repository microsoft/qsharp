// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::f64::consts;

use crate::tests::eval_expr;
use crate::{
    output::{GenericReceiver, Receiver},
    tests::get_global,
    val::{GlobalId, Value},
    Error, GlobalLookup,
};
use expect_test::{expect, Expect};
use indoc::indoc;
use num_bigint::BigInt;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap};
use qsc_passes::{run_core_passes, run_default_passes, PackageType};

struct Lookup<'a> {
    store: &'a PackageStore,
}

impl<'a> GlobalLookup<'a> for Lookup<'a> {
    fn get(&self, id: GlobalId) -> Option<crate::Global<'a>> {
        get_global(self.store, id)
    }
}

fn check_intrinsic(file: &str, expr: &str, out: &mut impl Receiver) -> Result<Value, Error> {
    let mut core = compile::core();
    run_core_passes(&mut core);
    let mut store = PackageStore::new(core);
    let mut std = compile::std(&store);
    assert!(std.errors.is_empty());
    assert!(run_default_passes(store.core(), &mut std, PackageType::Lib).is_empty());

    let std_id = store.insert(std);
    let sources = SourceMap::new([("test".into(), file.into())], Some(expr.into()));
    let mut unit = compile(&store, &[std_id], sources);
    assert!(unit.errors.is_empty());
    assert!(run_default_passes(store.core(), &mut unit, PackageType::Lib).is_empty());

    let id = store.insert(unit);
    let entry = store
        .get(id)
        .and_then(|unit| unit.package.entry.as_ref())
        .expect("package should have entry");
    let lookup = Lookup { store: &store };
    eval_expr(entry, &lookup, id, out).map_err(|e| e.0)
}

fn check_intrinsic_result(file: &str, expr: &str, expect: &Expect) {
    let mut stdout = vec![];
    let mut out = GenericReceiver::new(&mut stdout);
    match check_intrinsic(file, expr, &mut out) {
        Ok(result) => expect.assert_eq(&result.to_string()),
        Err(e) => expect.assert_eq(&e.to_string()),
    }
}

fn check_intrinsic_output(file: &str, expr: &str, expect: &Expect) {
    let mut stdout = vec![];
    let mut out = GenericReceiver::new(&mut stdout);
    match check_intrinsic(file, expr, &mut out) {
        Ok(..) => expect.assert_eq(
            &String::from_utf8(stdout).expect("content should be convertable to string"),
        ),
        Err(e) => expect.assert_eq(&e.to_string()),
    }
}

fn check_intrinsic_value(file: &str, expr: &str, val: &Value) {
    let mut stdout = vec![];
    let mut out = GenericReceiver::new(&mut stdout);
    match check_intrinsic(file, expr, &mut out) {
        Ok(result) => assert_eq!(&result, val),
        Err(e) => panic!("{e:?}"),
    }
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
fn dump_machine_qubit_count() {
    check_intrinsic_output(
        "",
        indoc! {"{
            use qs = Qubit[4];
            Microsoft.Quantum.Diagnostics.DumpMachine();
        }"},
        &expect![[r#"
            STATE:
            |0000⟩: 1+0i
        "#]],
    );
}

#[test]
fn dump_machine_endianness() {
    check_intrinsic_output(
        "",
        indoc! {"{
            use qs = Qubit[4];
            X(qs[1]);
            Microsoft.Quantum.Diagnostics.DumpMachine();
            X(qs[1]);
        }"},
        &expect![[r#"
            STATE:
            |0010⟩: 1+0i
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
            let isZero = Microsoft.Quantum.Diagnostics.CheckZero(q);
            X(q);
            isZero
        }"},
        &expect!["false"],
    );
}

#[test]
fn length() {
    check_intrinsic_value("", "Length([1, 2, 3])", &Value::Int(3));
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
fn draw_random_int() {
    check_intrinsic_value(
        "",
        "Microsoft.Quantum.Random.DrawRandomInt(5,5)",
        &Value::Int(5),
    );
}

#[test]
fn draw_random_double() {
    check_intrinsic_value(
        "",
        "Microsoft.Quantum.Random.DrawRandomDouble(5.0,5.0)",
        &Value::Double(5.0),
    );
}

#[test]
fn truncate() {
    check_intrinsic_value("", "Microsoft.Quantum.Math.Truncate(3.1)", &Value::Int(3));
    check_intrinsic_value("", "Microsoft.Quantum.Math.Truncate(3.9)", &Value::Int(3));
    check_intrinsic_value("", "Microsoft.Quantum.Math.Truncate(-3.1)", &Value::Int(-3));
    check_intrinsic_value("", "Microsoft.Quantum.Math.Truncate(-3.9)", &Value::Int(-3));
}

#[test]
fn sqrt() {
    check_intrinsic_value("", "Microsoft.Quantum.Math.Sqrt(0.0)", &Value::Double(0.0));
    check_intrinsic_value("", "Microsoft.Quantum.Math.Sqrt(81.0)", &Value::Double(9.0));
}

#[test]
fn log() {
    check_intrinsic_value("", "Microsoft.Quantum.Math.Log(1.0)", &Value::Double(0.0));
    check_intrinsic_value(
        "",
        "Microsoft.Quantum.Math.Log(Microsoft.Quantum.Math.E())",
        &Value::Double(1.0),
    );
}

#[test]
fn int_as_bigint() {
    check_intrinsic_value(
        "",
        "Microsoft.Quantum.Convert.IntAsBigInt(0)",
        &Value::BigInt(BigInt::from(0)),
    );
    check_intrinsic_value(
        "",
        "Microsoft.Quantum.Convert.IntAsBigInt(-10000)",
        &Value::BigInt(BigInt::from(-10000)),
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
            X(q2);
            X(q1);
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
            X(q1);
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
            X(q1);
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
            X(q1);
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
            X(q2);
            X(q1);
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
            Y(q2);
            Y(q1);
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
            H(q2);
            H(q1);
            Z(q2);
            Z(q1);
            H(q2);
            H(q1);
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
fn reset_all() {
    check_intrinsic_result(
        "",
        indoc! {r#"{
            use register = Qubit[2];
            ResetAll(register);
            if not Microsoft.Quantum.Diagnostics.CheckAllZero(register) {
                fail "Qubits should be in zero state.";
            }

            for q in register {
                X(q);
            }

            ResetAll(register);
            Microsoft.Quantum.Diagnostics.CheckAllZero(register)
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
            let res2 = (QIR.Intrinsic.__quantum__qis__m__body(q1), Microsoft.Quantum.Diagnostics.CheckZero(q1));
            X(q1);
            res2
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
        &expect!["unknown intrinsic `Foo`"],
    );
}

#[test]
fn qubit_nested_bind_not_released() {
    check_intrinsic_output(
        "",
        indoc! {"{
            use aux = Qubit();
            use q = Qubit();
            {
                let temp = q;
                X(temp);
            }
            Microsoft.Quantum.Diagnostics.DumpMachine();
            X(q);
        }"},
        &expect![[r#"
            STATE:
            |10⟩: 1+0i
        "#]],
    );
}

#[test]
fn qubit_release_non_zero_failure() {
    check_intrinsic_output(
        "",
        indoc! {"{
            use q = Qubit();
            X(q);
        }"},
        &expect!["Qubit0 released while not in |0⟩ state"],
    );
}

#[test]
fn qubit_not_unique_two_qubit_error() {
    check_intrinsic_output(
        "",
        indoc! {"{
            use q = Qubit();
            CNOT(q , q);
        }"},
        &expect!["qubits in gate invocation are not unique"],
    );
}

#[test]
fn qubit_not_unique_two_qubit_rotation_error() {
    check_intrinsic_output(
        "",
        indoc! {"{
            use q = Qubit();
            Rxx(0.1, q, q);
        }"},
        &expect!["qubits in gate invocation are not unique"],
    );
}

#[test]
fn qubit_not_unique_three_qubit_error_first_second() {
    check_intrinsic_output(
        "",
        indoc! {"{
            use q = Qubit();
            use a = Qubit();
            CCNOT(q , q, a);
        }"},
        &expect!["qubits in gate invocation are not unique"],
    );
}

#[test]
fn qubit_not_unique_three_qubit_error_first_third() {
    check_intrinsic_output(
        "",
        indoc! {"{
            use q = Qubit();
            use a = Qubit();
            CCNOT(q , a, q);
        }"},
        &expect!["qubits in gate invocation are not unique"],
    );
}

#[test]
fn qubit_not_unique_three_qubit_error_second_third() {
    check_intrinsic_output(
        "",
        indoc! {"{
            use q = Qubit();
            use a = Qubit();
            CCNOT(a , q, q);
        }"},
        &expect!["qubits in gate invocation are not unique"],
    );
}
