// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{io::LineWriter, str::from_utf8};

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_ast::visit::Visitor;
use qsc_frontend::compile::{compile, CompileUnit, PackageStore};

use crate::print_code::CodePrinter;

fn __check(input: &str, ignore_errors: bool) {
    let store = PackageStore::new();
    let unit = compile(&store, [], [input], "");

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
        printer.visit_package(&unit.package);
    }

    let transformed = match from_utf8(&output) {
        Ok(t) => t,
        Err(e) => panic!("Invalid UTF-8 sequence: {e}"),
    };

    let new_lines_replaced = input.replace("\r\n", "\n");
    let expected = new_lines_replaced.as_str();

    assert_eq!(expected, transformed);
}

fn check(input: &str) {
    __check(input, false);
}

fn check_without_errors(input: &str) {
    __check(input, true);
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
    check(indoc! { r#"namespace input {

        operation Foo () : Unit {
            let a: Int = -4;
            let b: Double = -1.6;
            let e: BigInt = -14;
        }
    }
    "#});
}

#[test]
fn type_var() {
    check(indoc! { r#"namespace input {v

        operation Foo<'T> (x: 'T) : Unit {
        }
    }
    "#});
}

#[test]
fn type_path() {
    check(indoc! { r#"namespace input {
        open other;
    
        operation Foo (x: other.Complex) : Unit {
        }
    }
    
    namespace other {

        newtype Complex = (Real: Double, Imaginary: Double);
    }
    "#});
}

#[test]
fn type_hole() {
    check_without_errors(indoc! { r#"namespace input {

        operation Foo () : Unit {
            let x: _ = 3;
        }
    }
    "#});
}

#[test]
fn type_tuple() {
    check(indoc! { r#"namespace input {

        operation Foo (x: (Int, (Double, Result), Int)) : Unit {
        }
    }
    "#});
}

#[test]
fn type_array() {
    check(indoc! { r#"namespace input {

        operation Foo (x: Int[][]) : Unit {
        }
    }
    "#});
}

#[test]
fn type_callables() {
    check_without_errors(indoc! { r#"namespace input {

        operation Foo (x: Int -> Unit, y: (Double, Int) => Unit is Adj) : Unit {
        }
    }
    "#});
}

#[test]
fn mutable_stmt() {
    check_without_errors(indoc! { r#"namespace input {

        operation Foo () : Unit {
            mutable x = 3;
        }
    }
    "#});
}

#[test]
fn qubit_allocation_stmts() {
    check_without_errors(indoc! { r#"namespace input {

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
    "#});
}
