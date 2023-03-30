// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{io::LineWriter, str::from_utf8};

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_ast::visit::Visitor;
use qsc_frontend::compile::{compile, CompileUnit, PackageStore};

use crate::print_code::CodePrinter;

fn __check(input: &str, expected: &str, ignore_errors: bool, just_expr: bool) {
    let store = PackageStore::new();
    let unit = if just_expr {
        compile(&store, [], [""], input)
    } else {
        compile(&store, [], [input], "")
    };

    if !ignore_errors {
        assert!(
            unit.context.errors().is_empty(),
            "Compilation errors: {:?}",
            unit.context.errors()
        );
    }

    let mut output = Vec::new();
    {
        let mut printer = CodePrinter {
            writer: LineWriter::new(&mut output),
            indentation: 0,
        };
        if just_expr {
            printer.visit_expr(&unit.package.entry.expect("expected an entry expression"));
        } else {
            printer.visit_package(&unit.package);
        }
    }

    let transformed = match from_utf8(&output) {
        Ok(t) => t,
        Err(e) => panic!("Invalid UTF-8 sequence: {e}"),
    };

    assert_eq!(expected, transformed);
}

fn check(input: &str) {
    __check(input, input, false, false);
}

fn check_without_errors(input: &str) {
    __check(input, input, true, false);
}

fn check_expr(input: &str) {
    __check(input, input, true, true);
}

#[test]
fn multiple_namespaces() {
    check(indoc! { "namespace input {
    }
    
    namespace other {
    }
    "});
}

#[test]
fn multiple_namespace_items() {
    check(indoc! { "namespace input {

        operation Foo () : Unit {
        }

        operation Bar () : Unit {
        }
    }
    "});
}

#[test]
fn open_namespace() {
    check(indoc! { "namespace input {
        open Microsoft.Quantum.Intrinsic as foo;
        open Microsoft.Quantum.Core;
    }
    "});
}

#[test]
fn newtype() {
    check(indoc! { "namespace input {

        newtype Complex = (Real: Double, Imaginary: Double);

        newtype Nested = (Double, (ItemName: Int, String));
    }
    "});
}

#[test]
fn attributes() {
    check(indoc! { "namespace input {

        @ First_Attr()
        @ Second_Attr()
        operation Foo () : Unit {
        }
    }
    "});
}

#[test]
fn operation_adj() {
    check(indoc! { "namespace input {

        operation Foo () : Unit is Adj {
        }
    }
    "});
}

#[test]
fn operation_ctl() {
    check(indoc! { "namespace input {

        operation Foo () : Unit is Ctl {
        }
    }
    "});
}

#[test]
fn operation_ctl_adj() {
    check(indoc! { "namespace input {

        operation Foo () : Unit is Ctl + Adj {
        }
    }
    "});
}

#[test]
fn function() {
    check(indoc! { "namespace input {

        function Foo () : Unit {
        }
    }
    "});
}

#[test]
fn parameters() {
    check(indoc! { "namespace input {

        operation Foo (x: Int, (y: Double, z: Int)) : Unit {
        }
    }
    "});
}

#[test]
fn type_parameters() {
    check(indoc! { "namespace input {

        operation Foo<'T, 'U> () : Unit {
        }
    }
    "});
}

#[test]
fn spec_decls() {
    check(indoc! { "namespace input {

        operation Foo () : Unit is Ctl + Adj {
            body intrinsic;

            adjoint invert;

            controlled distribute;

            controlled adjoint auto;
        }

        operation Bar () : Unit is Ctl + Adj {
            body(...) {
            }

            adjoint self;

            controlled(ctrl, ...) {
            }

            controlled adjoint(ctrl, ...) {
            }
        }
    }
    "});
}

#[test]
fn primitive_types() {
    check(indoc! { r#"namespace input {

        operation Foo () : Unit {
            let a: Int = 4;
            let b: Double = 1.6;
            let c: Bool = false;
            let d: Bool = true;
            let e: BigInt = 14;
            use f: Qubit = Qubit();
            let g: String = "hello";
            let h: Pauli = PauliI;
            let i: Pauli = PauliY;
            let j: Pauli = PauliX;
            let k: Pauli = PauliZ;
            let l: Result = Zero;
            let m: Result = One;
        }
    }
    "#});
}

#[test]
fn primitive_negative_literals() {
    check(indoc! { "namespace input {

        operation Foo () : Unit {
            let a: Int = -4;
            let b: Double = -1.6;
            let e: BigInt = -14;
        }
    }
    "});
}

#[test]
fn type_var() {
    check(indoc! { "namespace input {

        operation Foo<'T> (x: 'T) : Unit {
        }
    }
    "});
}

#[test]
fn type_path() {
    check(indoc! { "namespace input {
        open other;
    
        operation Foo (x: other.Complex) : Unit {
        }
    }
    
    namespace other {

        newtype Complex = (Real: Double, Imaginary: Double);
    }
    "});
}

#[test]
fn type_hole() {
    check_without_errors(indoc! { "namespace input {

        operation Foo () : Unit {
            let x: _ = 3;
        }
    }
    "});
}

#[test]
fn type_tuple() {
    check(indoc! { "namespace input {

        operation Foo (x: (Int, (Double, Result), Int)) : Unit {
        }
    }
    "});
}

#[test]
fn type_array() {
    check(indoc! { "namespace input {

        operation Foo (x: Int[][]) : Unit {
        }
    }
    "});
}

#[test]
fn type_callables() {
    check_without_errors(indoc! { "namespace input {

        operation Foo (x: Int -> Unit, y: (Double, Int) => Unit is Adj) : Unit {
        }
    }
    "});
}

#[test]
fn mutable_stmt() {
    check(indoc! { "namespace input {

        operation Foo () : Unit {
            mutable x = 3;
        }
    }
    "});
}

#[test]
fn qubit_allocation_stmts() {
    check(indoc! { "namespace input {

        operation Foo () : Unit {
            use q1 = Qubit();
            use q2 = Qubit[4];
            use q3 = (Qubit(), Qubit[2]);
            use (q4, q5) = (Qubit(), Qubit());
            use q6 = Qubit() {
                borrow q7 = Qubit();
            }
        }
    }
    "});
}

#[test]
fn expr_array() {
    check_expr(indoc! { "[1, 2, 3]"});
}

#[test]
fn expr_array_repeat() {
    check_expr(indoc! { "[1, size = 4]"});
}

#[test]
fn expr_assign() {
    check_expr(indoc! { "set x = 4"});
}

#[test]
fn expr_assign_plus() {
    check_expr(indoc! { "set x += 4"});
}

#[test]
fn expr_assign_div() {
    check_expr(indoc! { "set x /= 4"});
}

#[test]
fn expr_assign_exp() {
    check_expr(indoc! { "set x ^= 4"});
}

#[test]
fn expr_assign_mod() {
    check_expr(indoc! { "set x %= 4"});
}

#[test]
fn expr_assign_mul() {
    check_expr(indoc! { "set x *= 4"});
}

#[test]
fn expr_assign_minus() {
    check_expr(indoc! { "set x -= 4"});
}

#[test]
fn expr_binary_ops() {
    check(indoc! { "namespace input {

        operation Foo () : Unit {
            let a = 0 + 1;
            let b = 0 &&& 1;
            let c = true and false;
            let d = 0 / 1;
            let e = 0 == 1;
            let f = 1 ^ 2;
            let g = 0 > 1;
            let h = 0 >= 1;
            let i = 0 < 1;
            let j = 0 <= 1;
            let k = 0 % 1;
            let l = 0 * 1;
            let m = 0 != 1;
            let n = 0 ||| 1;
            let o = true or false;
            let p = 0 <<< 1;
            let q = 0 >>> 1;
            let r = 0 - 1;
            let s = 0 ^^^ 1;
        }
    }
    "});
}

#[test]
fn expr_call() {
    check_expr(indoc! { "Foo()"});
}

#[test]
fn expr_call_args() {
    check_expr(indoc! { "Foo(1, 2, 3)"});
}

#[ignore = "type arguments aren't supported yet"]
#[test]
fn expr_call_type_args() {
    check_expr(indoc! { "Foo<Int, Double>(1, 2, 3)"});
}

#[test]
fn expr_conjugate() {
    check_expr(indoc! { "within {
        Foo();
    } apply {
        Bar();
    }"});
}

#[test]
fn expr_fail() {
    check_expr(indoc! { r#"fail "failure message""#});
}

#[test]
fn expr_field() {
    check_expr(indoc! { "x::my_field"});
}

#[test]
fn expr_for() {
    check_expr(indoc! { "for i in iter {
        Foo();
    }"});
}

#[test]
fn expr_for_range() {
    check_expr(indoc! { "for i in 0..10 {
        Foo();
    }"});
}

#[test]
fn expr_for_range_step() {
    check_expr(indoc! { "for i in 0..2..10 {
        Foo();
    }"});
}

#[test]
fn expr_for_range_open_end() {
    check_expr(indoc! { "for i in arr[0...] {
        Foo();
    }"});
}

#[test]
fn expr_for_range_open_end_step() {
    check_expr(indoc! { "for i in arr[0..2...] {
        Foo();
    }"});
}

#[test]
fn expr_for_range_open_start() {
    check_expr(indoc! { "for i in arr[...10] {
        Foo();
    }"});
}

#[test]
fn expr_for_range_open_start_step() {
    check_expr(indoc! { "for i in arr[...2..10] {
        Foo();
    }"});
}

#[test]
fn expr_for_range_only_step() {
    check_expr(indoc! { "for i in arr[...2...] {
        Foo();
    }"});
}

#[test]
fn expr_for_range_open() {
    check_expr(indoc! { "for i in arr[...] {
        Foo();
    }"});
}

#[test]
fn expr_hole() {
    check_expr(indoc! { "Foo(_, 12)"});
}

#[test]
fn expr_if() {
    check_expr(indoc! { "if cond {
        Foo();
    }"});
}

#[test]
fn expr_if_else() {
    check_expr(indoc! { "if cond {
        Foo();
    } else {
        Bar();
    }"});
}

#[test]
fn expr_if_elif_else() {
    __check(
        indoc! { "if cond1 {
        Foo();
    } elif cond2 {
        Bar();
    } else {
        Baz();
    }"},
        indoc! { "if cond1 {
        Foo();
    } else if cond2 {
        Bar();
    } else {
        Baz();
    }"},
        true,
        true,
    );
}

#[test]
fn expr_index() {
    check_expr(indoc! { "arr[12]"});
}

#[test]
fn expr_op_lambda() {
    check_expr(indoc! { "(x, y) => x"});
}

#[test]
fn expr_func_lambda() {
    check_expr(indoc! { "(x, y) -> (x, y)"});
}

#[test]
fn expr_repeat() {
    check_expr(indoc! { "repeat {
        Foo();
    }
    until cond"});
}

#[test]
fn expr_repeat_fixup() {
    check_expr(indoc! { "repeat {
        Foo();
    }
    until true
    fixup {
        Bar();
    }"});
}

#[test]
fn expr_return() {
    check(indoc! { "namespace input {

        operation Foo () : Int {
            return 3;
        }
    }
    "});
}

#[test]
fn expr_tern_cond() {
    check_expr(indoc! { "cond ? x | y"});
}

#[test]
fn expr_tern_update() {
    check_expr(indoc! { "arr w/ index <- val"});
}

#[test]
fn expr_tuple() {
    check_expr(indoc! { "(a, (b, c), d)"});
}

#[test]
fn expr_functors() {
    check(indoc! { "namespace input {

        operation Foo (ctl: Qubit[]) : Unit {
            Adjoint Bar();
            Controlled Bar(ctl);
            Adjoint Controlled Bar(ctl);
        }

        operation Bar () : Unit is Adj + Ctl {
        }
    }
    "});
}

#[test]
fn expr_un_ops() {
    check(indoc! { "namespace input {

        operation Foo () : Unit {
            let x = -1;
            let y = ~~~12;
            let z = not true;
        }
    }
    "});
}

#[ignore = "unwrapping is not yet supported"]
#[test]
fn expr_unwrap() {
    check(indoc! { "namespace input {

        newtype Nested = (Double, (ItemName : Int, String));

        operation Foo (n : Nested) : Unit {
            let (x, (y, z)) = n!;
        }
    }
    "});
}

#[test]
fn expr_while() {
    check_expr(indoc! { "while cond {
        Foo();
    }"});
}
