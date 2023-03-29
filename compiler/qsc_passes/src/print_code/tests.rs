// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{io::LineWriter, str::from_utf8};

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_ast::visit::Visitor;
use qsc_frontend::compile::{compile, CompileUnit, PackageStore};

use crate::print_code::CodePrinter;

fn check(input: &str) {
    let store = PackageStore::new();
    let unit = compile(&store, [], [input], "");

    assert!(
        unit.context.errors().is_empty(),
        "Compilation errors: {:?}",
        unit.context.errors()
    );

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

    let temp1 = input.replace("\r\n", "\n");
    let temp = temp1.as_str();

    assert_eq!(temp, transformed);
}

#[test]
fn foo() {
    check(indoc! { "namespace input {

        operation Foo () : Unit {
            let x = 3;
        }
    }
    "});
}
