// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod array;
mod bit;
mod bool;
mod complex;
mod def;
mod float;
mod gate;
mod integer;
mod io;
mod qubit;
mod unsigned_integer;

use crate::{
    tests::{compile_fragments, compile_with_config, fail_on_compilation_errors},
    CompilerConfig, OutputSemantics, ProgramType, QubitSemantics,
};

use miette::Report;

#[test]
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

    let unit = compile_fragments(source)?;
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

    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::Fragments,
        None,
        None,
    );
    let unit = compile_with_config(source, config).expect("parse failed");
    println!("{:?}", unit.errors);
    assert_eq!(unit.errors.len(), 5);
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

    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::Fragments,
        None,
        None,
    );
    let unit = compile_with_config(source, config).expect("parse failed");
    assert!(unit.has_errors());
    println!("{:?}", unit.errors);
    assert!(unit.errors.len() == 2);
    assert!(unit.errors[0]
        .to_string()
        .contains("Stretch type values are not supported."),);
    assert!(unit.errors[1]
        .to_string()
        .contains("Stretch default values are not supported."),);
}
