// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

mod array;
mod bit;
mod bool;
mod complex;
mod float;
mod gate;
mod integer;
mod io;
mod qubit;
mod unsigned_integer;

use crate::{
    tests::{fail_on_compilation_errors, parse, qasm_to_program_fragments},
    CompilerConfig, OutputSemantics, ProgramType, QubitSemantics,
};

use miette::Report;

#[test]
#[ignore = "oq3 parser bug, can't read float with leading dot"]
fn classical() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        int[10] a;
        int[10] b;
        uint[32] c = 0xFa_1F;
        uint[32] d = 0XFa_1F;
        uint[16] e = 0o12_34;
        uint[16] f = 0b1001_1001;
        uint[16] g = 0B1001_1001;
        uint h;
        qubit[6] q1;
        qubit q2;
        bit[4] b1 = "0100";
        bit[8] b2 = "1001_0100";
        bit b3 = "1";
        bool i = true;
        bool j = false;
        const float[64] k = 5.5e3;
        const float[64] l = 5;
        float[32] m = .1e+3;
    "#;

    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program_fragments(res.source, res.source_map);
    fail_on_compilation_errors(&unit);
    Ok(())
}

#[test]
fn duration_literal() -> miette::Result<(), Vec<Report>> {
    let source = "
        duration dur0;
        duration dur1 = 1000dt;
        duration dur2 = 10 ms;
        duration dur3 = 8	us;
        duration dur4 = 1s;
    ";

    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = crate::qasm_to_program(
        res.source,
        res.source_map,
        CompilerConfig::new(
            QubitSemantics::Qiskit,
            OutputSemantics::OpenQasm,
            ProgramType::Fragments,
            None,
            None,
        ),
    );
    println!("{:?}", unit.errors);
    assert!(unit.errors.len() == 5);
    for error in &unit.errors {
        assert!(
            error
                .to_string()
                .contains("Duration type values are not supported.")
                || error
                    .to_string()
                    .contains("Timing literal expressions are not supported.")
        );
    }

    Ok(())
}

#[test]
fn stretch() {
    let source = "
        stretch s;
    ";

    let res = parse(source).expect("should parse");
    assert!(!res.has_errors());
    let unit = crate::compile::qasm_to_program(
        res.source,
        res.source_map,
        CompilerConfig::new(
            QubitSemantics::Qiskit,
            OutputSemantics::OpenQasm,
            ProgramType::Fragments,
            None,
            None,
        ),
    );
    assert!(unit.has_errors());
    println!("{:?}", unit.errors);
    assert!(unit.errors.len() == 1);
    assert!(unit.errors[0]
        .to_string()
        .contains("Stretch type values are not supported."),);
}
