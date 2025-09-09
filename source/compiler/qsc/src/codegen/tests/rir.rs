// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::codegen::qir::get_rir;
use expect_test::{Expect, expect};
use qsc_data_structures::{
    language_features::LanguageFeatures,
    target::{Profile, TargetCapabilityFlags},
};
use qsc_frontend::compile::SourceMap;

fn check(source: &str, capabilities: TargetCapabilityFlags, expect: &Expect) {
    let sources = SourceMap::new([("test.qs".into(), source.into())], None);
    let language_features = LanguageFeatures::default();

    let (std_id, package_store) = crate::compile::package_store_with_stdlib(capabilities);
    let dependencies = &[(std_id, None)];
    let rir = {
        let (package_id, fir_store, entry, compute_properties) = compile_to_fir(
            sources,
            language_features,
            capabilities,
            &mut package_store,
            dependencies,
        )?;

        let (raw, ssa) = fir_to_rir(&fir_store, capabilities, Some(compute_properties), &entry)
            .map_err(|e| {
                let source_package_id = match e.span() {
                    Some(span) => span.package,
                    None => package_id,
                };
                let source_package = package_store
                    .get(source_package_id)
                    .expect("package should be in store");
                vec![Error::PartialEvaluation(WithSource::from_map(
                    &source_package.sources,
                    e,
                ))]
            })?;
    }
    .expect("Failed to generate RIR");
    let mut program = &rir[1];


    program.bloc

    expect.assert_eq(&program.to_string());
}

#[test]
fn one_qubit() {
    check(
        "
        operation Main() : Unit {
            use q = Qubit();
        }
    ",
        Profile::AdaptiveRI.into(),
        &expect![[r#"
            Program:
                entry: 0
                callables:
                    Callable 0: Callable:
                        name: main
                        call_type: Regular
                        input_type: <VOID>
                        output_type: <VOID>
                        body: 0
                    Callable 1: Callable:
                        name: __quantum__rt__tuple_record_output
                        call_type: OutputRecording
                        input_type:
                            [0]: Integer
                            [1]: Pointer
                        output_type: <VOID>
                        body: <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Integer(0), Pointer, )
                        Return
                config: Config:
                    capabilities: TargetCapabilityFlags(Adaptive | IntegerComputations | QubitReset)
                num_qubits: 1
                num_results: 0"#]],
    );
}

#[test]
fn gate_loop() {
    check(
        "
        operation Main() : Unit {
            use q = Qubit();
            for i in 1..10 {
                H(q);
            }
            Reset(q);
        }
    ",
        Profile::AdaptiveRI.into(),
        &expect![[r#"
            Program:
                entry: 0
                callables:
                    Callable 0: Callable:
                        name: main
                        call_type: Regular
                        input_type: <VOID>
                        output_type: <VOID>
                        body: 0
                    Callable 1: Callable:
                        name: __quantum__qis__h__body
                        call_type: Regular
                        input_type:
                            [0]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                    Callable 2: Callable:
                        name: __quantum__qis__reset__body
                        call_type: Reset
                        input_type:
                            [0]: Qubit
                        output_type: <VOID>
                        body: <NONE>
                    Callable 3: Callable:
                        name: __quantum__rt__tuple_record_output
                        call_type: OutputRecording
                        input_type:
                            [0]: Integer
                            [1]: Pointer
                        output_type: <VOID>
                        body: <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), )
                        Call id(1), args( Qubit(0), )
                        Call id(1), args( Qubit(0), )
                        Call id(1), args( Qubit(0), )
                        Call id(1), args( Qubit(0), )
                        Call id(1), args( Qubit(0), )
                        Call id(1), args( Qubit(0), )
                        Call id(1), args( Qubit(0), )
                        Call id(1), args( Qubit(0), )
                        Call id(1), args( Qubit(0), )
                        Call id(2), args( Qubit(0), )
                        Call id(3), args( Integer(0), Pointer, )
                        Return
                config: Config:
                    capabilities: TargetCapabilityFlags(Adaptive | IntegerComputations | QubitReset)
                num_qubits: 1
                num_results: 0"#]],
    );
}
