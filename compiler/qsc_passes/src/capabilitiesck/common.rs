// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use expect_test::Expect;

use crate::capabilitiesck::check_supported_capabilities;
use qsc::{incremental::Compiler, PackageType};
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_eval::{debug::map_hir_package_to_fir, lower::Lowerer};
use qsc_fir::fir::{Package, PackageId, PackageStore};
use qsc_frontend::compile::{PackageStore as HirPackageStore, RuntimeCapabilityFlags, SourceMap};
use qsc_rca::{Analyzer, PackageComputeProperties, PackageStoreComputeProperties};
use std::{fs::File, io::Write};

pub fn check(source: &str, expect: &Expect, capabilities: RuntimeCapabilityFlags) {
    let compilation_context = CompilationContext::new(source);
    let (package, compute_properties) = compilation_context.get_package_compute_properties_tuple();
    let errors = check_supported_capabilities(package, compute_properties, capabilities);
    expect.assert_debug_eq(&errors);
}

fn lower_hir_package_store(
    lowerer: &mut Lowerer,
    hir_package_store: &HirPackageStore,
) -> PackageStore {
    let mut fir_store = PackageStore::new();
    for (id, unit) in hir_package_store {
        fir_store.insert(
            map_hir_package_to_fir(id),
            lowerer.lower_package(&unit.package),
        );
    }
    fir_store
}

struct CompilationContext {
    fir_store: PackageStore,
    compute_properties: PackageStoreComputeProperties,
    package_id: PackageId,
}

impl CompilationContext {
    fn new(source: &str) -> Self {
        let mut compiler = Compiler::new(
            true,
            SourceMap::default(),
            PackageType::Lib,
            RuntimeCapabilityFlags::all(),
            LanguageFeatures::default(),
        )
        .expect("should be able to create a new compiler");
        let package_id = map_hir_package_to_fir(compiler.package_id());
        let increment = compiler
            .compile_fragments_fail_fast("test", source)
            .expect("code should compile");
        compiler.update(increment);
        let mut lowerer = Lowerer::new();
        let fir_store = lower_hir_package_store(&mut lowerer, compiler.package_store());
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

    fn get_package_compute_properties_tuple(&self) -> (&Package, &PackageComputeProperties) {
        (
            self.fir_store.get(self.package_id),
            self.compute_properties.get(self.package_id),
        )
    }
}

pub const MINIMAL: &str = r#"
    namespace Test {
        operation Foo() : Unit {
            use q = Qubit();
            let r = M(q);
        }
    }"#;

pub const USE_DYNAMIC_BOOLEAN: &str = r#"
    namespace Test {
        operation Foo() : Unit {
            use q = Qubit();
            let b = M(q) == Zero;
        }
    }"#;

pub const USE_DYNAMIC_INT: &str = r#"
    namespace Test {
        open Microsoft.Quantum.Convert;
        open Microsoft.Quantum.Measurement;
        operation Foo() : Unit {
            use register = Qubit[4];
            let results = MeasureEachZ(register);
            let i = ResultArrayAsInt(results);
        }
    }"#;

pub const USE_DYNAMIC_PAULI: &str = r#"
    namespace Test {
        operation Foo() : Unit {
            use q = Qubit();
            let p = M(q) == Zero ? PauliX | PauliY;
        }
    }"#;

pub const USE_DYNAMIC_RANGE: &str = r#"
    namespace Test {
        operation Foo() : Unit {
            use q = Qubit();
            let range = 1..(M(q) == Zero ? 1 | 2)..10;
        }
    }"#;

pub const USE_DYNAMIC_DOUBLE: &str = r#"
    namespace Test {
        open Microsoft.Quantum.Convert;
        open Microsoft.Quantum.Measurement;
        operation Foo() : Unit {
            use register = Qubit[4];
            let results = MeasureEachZ(register);
            let d = IntAsDouble(ResultArrayAsInt(results));
        }
    }"#;

pub const USE_DYNAMIC_QUBIT: &str = r#"
    namespace Test {
        operation Foo() : Unit {
            use control = Qubit();
            if M(control) == Zero {
                use q = Qubit();
            }
        }
    }"#;

pub const USE_DYNAMIC_BIG_INT: &str = r#"
    namespace Test {
        open Microsoft.Quantum.Convert;
        open Microsoft.Quantum.Measurement;
        operation Foo() : Unit {
            use register = Qubit[4];
            let results = MeasureEachZ(register);
            let bi = IntAsBigInt(ResultArrayAsInt(results));
        }
    }"#;

pub const USE_DYNAMIC_STRING: &str = r#"
    namespace Test {
        operation Foo() : Unit {
            use q = Qubit();
            let r = M(q);
            let s = $"{r == Zero}";
        }
    }"#;

pub const USE_DYNAMICALLY_SIZED_ARRAY: &str = r#"
    namespace Test {
        operation Foo() : Unit {
            use q = Qubit();
            let a = [0, size = M(q) == Zero ? 1 | 2];
        }
    }"#;

pub const USE_DYNAMIC_UDT: &str = r#"
    namespace Test {
        open Microsoft.Quantum.Convert;
        open Microsoft.Quantum.Math;
        open Microsoft.Quantum.Measurement;
        operation Foo() : Unit {
            use register = Qubit[4];
            let results = MeasureEachZ(register);
            let c = Complex(0.0, IntAsDouble(ResultArrayAsInt(results)));
        }
    }"#;

pub const USE_DYNAMIC_FUNCTION: &str = r#"
    namespace Test {
        open Microsoft.Quantum.Math;
        operation Foo() : Unit {
            use q = Qubit();
            let f = M(q) == Zero ? Cos | Sin;
        }
    }"#;

pub const CALL_TO_CICLYC_FUNCTION_WITH_CLASSICAL_ARGUMENT: &str = r#"
    function GaussSum(n : Int) : Int {
        if n == 0 {
            0
        } else {
            n + GaussSum(n - 1)
        }
    }
    operation Foo() : Unit {
        let sum = GaussSum(10);
    }"#;

pub const CALL_TO_CICLYC_FUNCTION_WITH_DYNAMIC_ARGUMENT: &str = r#"
    function GaussSum(n : Int) : Int {
        if n == 0 {
            0
        } else {
            n + GaussSum(n - 1)
        }
    }
    operation Foo() : Unit {
        use q = Qubit();
        let sum = GaussSum(M(q) == Zero ? 10 | 20);
    }"#;

pub const CALL_TO_CICLYC_OPERATION_WITH_CLASSICAL_ARGUMENT: &str = r#"
    operation GaussSum(n : Int) : Int {
        if n == 0 {
            0
        } else {
            n + GaussSum(n - 1)
        }
    }
    operation Foo() : Unit {
        let sum = GaussSum(10);
    }"#;

pub const CALL_TO_CICLYC_OPERATION_WITH_DYNAMIC_ARGUMENT: &str = r#"
    operation GaussSum(n : Int) : Int {
        if n == 0 {
            0
        } else {
            n + GaussSum(n - 1)
        }
    }
    operation Foo() : Unit {
        use q = Qubit();
        let sum = GaussSum(M(q) == Zero ? 10 | 20);
    }"#;

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
