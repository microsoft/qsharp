// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::{incremental::Compiler, PackageType};
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_fir::fir::{PackageId, PackageStore};
use qsc_frontend::compile::{PackageStore as HirPackageStore, RuntimeCapabilityFlags, SourceMap};
use qsc_lowerer::{map_hir_package_to_fir, Lowerer};
use qsc_partial_eval::partially_evaluate;
use qsc_rca::{Analyzer, PackageStoreComputeProperties};
use qsc_rir::rir::{BlockId, Callable, CallableId, Instruction, Program};
use std::{fs::File, io::Write};

pub fn assert_block_last_instruction(
    program: &Program,
    block_id: BlockId,
    expected_inst: &Instruction,
) {
    let block = program.blocks.get(block_id).expect("block does not exist");
    let actual_inst = block.0.last().expect("block does not have instructions");
    assert_eq!(expected_inst, actual_inst);
}

pub fn assert_block_instructions(
    program: &Program,
    block_id: BlockId,
    expected_insts: &[Instruction],
) {
    let block = program.blocks.get(block_id).expect("block does not exist");
    assert_eq!(
        block.0.len(),
        expected_insts.len(),
        "expected number of instructions is different than actual number of instructions"
    );
    for (expected_inst, actual_inst) in expected_insts.iter().zip(block.0.iter()) {
        assert_eq!(expected_inst, actual_inst);
    }
}

pub fn assert_callable(program: &Program, callable_id: CallableId, expected_callable: &Callable) {
    let actual_callable = program
        .callables
        .get(callable_id)
        .expect("callable does not exist ");
    assert_eq!(expected_callable, actual_callable);
}

#[must_use]
pub fn compile_and_partially_evaluate(source: &str) -> Program {
    let compilation_context = CompilationContext::new(source);
    let maybe_program = partially_evaluate(
        compilation_context.package_id,
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
    );
    match maybe_program {
        Ok(program) => program,
        Err(error) => panic!("partial evaluation failed: {error:?}"),
    }
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
        //write_fir_store_to_files(&fir_store);
        //write_compute_properties_to_files(&compute_properties);
        Self {
            fir_store,
            compute_properties,
            package_id,
        }
    }
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

// TODO (cesarzc): remove.
fn write_fir_store_to_files(store: &PackageStore) {
    for (id, package) in store {
        let filename = format!("dbg/fir.package{id}.txt");
        let mut package_file = File::create(filename).expect("File could be created");
        let package_string = format!("{package}");
        write!(package_file, "{package_string}").expect("Writing to file should succeed.");
    }
}

// TODO (cesarzc): remove.
fn write_compute_properties_to_files(store: &PackageStoreComputeProperties) {
    for (id, package) in store.iter() {
        let filename = format!("dbg/rca.package{id}.txt");
        let mut package_file = File::create(filename).expect("File could be created");
        let package_string = format!("{package}");
        write!(package_file, "{package_string}").expect("Writing to file should succeed.");
    }
}
