// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::partially_evaluate;
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc::{incremental::Compiler, PackageType};
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_eval::{debug::map_hir_package_to_fir, lower::Lowerer};
use qsc_fir::fir::{PackageId, PackageStore};
use qsc_frontend::compile::{PackageStore as HirPackageStore, RuntimeCapabilityFlags, SourceMap};
use qsc_rca::{Analyzer, PackageStoreComputeProperties};
use std::{fs::File, io::Write};

#[test]
fn empty_entry_point() {
    check_rir(
        indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {}
        }
        "#},
        &expect![[r#"
            Program:
                entry: 0
                callables:
                    Callable 0: Callable:
                        name: main
                        call_type: Regular
                        input_type:  <VOID>
                        output_type:  <VOID>
                        body:  0
                blocks:
                    Block 0: Block:
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn allocate_one_qubit() {
    check_rir(
        indoc! {r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
            }
        }
        "#},
        // Only the return instruction is generated because no operations are performed on the allocated qubit.
        &expect![[r#"
            Program:
                entry: 0
                callables:
                    Callable 0: Callable:
                        name: main
                        call_type: Regular
                        input_type:  <VOID>
                        output_type:  <VOID>
                        body:  0
                blocks:
                    Block 0: Block:
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn call_to_single_qubit_operation() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                __quantum__qis__h__body(q);
            }
        }
        "#},
        &expect![[r#"
            Program:
                entry: 0
                callables:
                    Callable 0: Callable:
                        name: main
                        call_type: Regular
                        input_type:  <VOID>
                        output_type:  <VOID>
                        body:  0
                    Callable 1: Callable:
                        name: __quantum__qis__h__body
                        call_type: Regular
                        input_type: 
                            [0]: Qubit
                        output_type:  <VOID>
                        body:  <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn call_to_many_single_qubit_operations() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use q0 = Qubit();
                __quantum__qis__h__body(q0);
                use q1 = Qubit();
                __quantum__qis__x__body(q1);
                use q2 = Qubit();
                __quantum__qis__y__body(q2);
                use q3 = Qubit();
                __quantum__qis__x__body(q3);
            }
        }
        "#},
        &expect![[r#"
            Program:
                entry: 0
                callables:
                    Callable 0: Callable:
                        name: main
                        call_type: Regular
                        input_type:  <VOID>
                        output_type:  <VOID>
                        body:  0
                    Callable 1: Callable:
                        name: __quantum__qis__h__body
                        call_type: Regular
                        input_type: 
                            [0]: Qubit
                        output_type:  <VOID>
                        body:  <NONE>
                    Callable 2: Callable:
                        name: __quantum__qis__x__body
                        call_type: Regular
                        input_type: 
                            [0]: Qubit
                        output_type:  <VOID>
                        body:  <NONE>
                    Callable 3: Callable:
                        name: __quantum__qis__y__body
                        call_type: Regular
                        input_type: 
                            [0]: Qubit
                        output_type:  <VOID>
                        body:  <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Qubit(0), )
                        Call id(2), args( Qubit(1), )
                        Call id(3), args( Qubit(2), )
                        Call id(2), args( Qubit(3), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn call_to_rotation_operation_using_literal() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                __quantum__qis__rx__body(3.14159, q);
            }
        }
        "#},
        &expect![[r#"
            Program:
                entry: 0
                callables:
                    Callable 0: Callable:
                        name: main
                        call_type: Regular
                        input_type:  <VOID>
                        output_type:  <VOID>
                        body:  0
                    Callable 1: Callable:
                        name: __quantum__qis__rx__body
                        call_type: Regular
                        input_type: 
                            [0]: Double
                            [1]: Qubit
                        output_type:  <VOID>
                        body:  <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Double(3.14159), Qubit(0), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn calls_to_rotation_operation_using_inline_expressions() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open Microsoft.Quantum.Math;
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                __quantum__qis__ry__body(PI(), q);
                __quantum__qis__ry__body(PI() / 2.0, q);
            }
        }
        "#},
        &expect![[r#"
            Program:
                entry: 0
                callables:
                    Callable 0: Callable:
                        name: main
                        call_type: Regular
                        input_type:  <VOID>
                        output_type:  <VOID>
                        body:  0
                    Callable 1: Callable:
                        name: __quantum__qis__ry__body
                        call_type: Regular
                        input_type: 
                            [0]: Double
                            [1]: Qubit
                        output_type:  <VOID>
                        body:  <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Double(3.141592653589793), Qubit(0), )
                        Call id(1), args( Double(1.5707963267948966), Qubit(0), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

#[test]
fn calls_to_rotation_operation_using_variables() {
    check_rir(
        indoc! {r#"
        namespace Test {
            open Microsoft.Quantum.Math;
            open QIR.Intrinsic;
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let pi_over_two = PI() / 2.0;
                __quantum__qis__rz__body(pi_over_two, q);
                mutable some_angle = ArcSin(0.5);
                __quantum__qis__rz__body(some_angle, q);
                set some_angle = ArcCos(-0.25);
                __quantum__qis__rz__body(some_angle, q);
            }
        }
        "#},
        &expect![[r#"
            Program:
                entry: 0
                callables:
                    Callable 0: Callable:
                        name: main
                        call_type: Regular
                        input_type:  <VOID>
                        output_type:  <VOID>
                        body:  0
                    Callable 1: Callable:
                        name: __quantum__qis__rz__body
                        call_type: Regular
                        input_type: 
                            [0]: Double
                            [1]: Qubit
                        output_type:  <VOID>
                        body:  <NONE>
                blocks:
                    Block 0: Block:
                        Call id(1), args( Double(1.5707963267948966), Qubit(0), )
                        Call id(1), args( Double(0.5235987755982989), Qubit(0), )
                        Call id(1), args( Double(1.8234765819369754), Qubit(0), )
                        Return
                config: Config:
                    remap_qubits_on_reuse: false
                    defer_measurements: false
                num_qubits: 0
                num_results: 0"#]],
    );
}

struct CompilationContext {
    fir_store: PackageStore,
    compute_properties: PackageStoreComputeProperties,
    package_id: PackageId,
}

impl CompilationContext {
    fn new(source: &str) -> Self {
        let source_map = SourceMap::new([("test".into(), source.into())], Some("".into()));
        let compiler = Compiler::new(
            true,
            source_map,
            PackageType::Exe,
            RuntimeCapabilityFlags::all(),
            LanguageFeatures::default(),
        )
        .expect("should be able to create a new compiler");
        let package_id = map_hir_package_to_fir(compiler.source_package_id());
        let fir_store = lower_hir_package_store(compiler.package_store());
        let analyzer = Analyzer::init(&fir_store);
        let compute_properties = analyzer.analyze_all();
        write_fir_store_to_files(&fir_store);
        write_compute_properties_to_files(&compute_properties);
        Self {
            fir_store,
            compute_properties,
            package_id,
        }
    }
}

fn check_rir(source: &str, expect: &Expect) {
    let compilation_context = CompilationContext::new(source);
    let Ok(rir) = partially_evaluate(
        compilation_context.package_id,
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
    ) else {
        panic!("partial evaluation failed");
    };
    expect.assert_eq(&rir.to_string());
}

fn lower_hir_package_store(hir_package_store: &HirPackageStore) -> PackageStore {
    let mut fir_store = PackageStore::new();
    for (id, unit) in hir_package_store {
        let mut lowerer = Lowerer::new();
        fir_store.insert(
            map_hir_package_to_fir(id),
            lowerer.lower_package(&unit.package),
        );
    }
    fir_store
}

pub fn write_fir_store_to_files(store: &PackageStore) {
    for (id, package) in store {
        let filename = format!("dbg/fir.package{id}.txt");
        let mut package_file = File::create(filename).expect("File could be created");
        let package_string = format!("{package}");
        write!(package_file, "{package_string}").expect("Writing to file should succeed.");
    }
}

pub fn write_compute_properties_to_files(store: &PackageStoreComputeProperties) {
    for (id, package) in store.iter() {
        let filename = format!("dbg/rca.package{id}.txt");
        let mut package_file = File::create(filename).expect("File could be created");
        let package_string = format!("{package}");
        write!(package_file, "{package_string}").expect("Writing to file should succeed.");
    }
}
