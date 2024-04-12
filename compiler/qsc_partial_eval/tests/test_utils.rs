// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::{incremental::Compiler, PackageType};
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_fir::fir::PackageStore;
use qsc_frontend::compile::{PackageStore as HirPackageStore, RuntimeCapabilityFlags, SourceMap};
use qsc_lowerer::{map_hir_package_to_fir, Lowerer};
use qsc_partial_eval::{partially_evaluate, ProgramEntry};
use qsc_rca::{Analyzer, PackageStoreComputeProperties};
use qsc_rir::rir::{BlockId, Callable, CallableId, CallableType, Instruction, Program, Ty};

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
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        &compilation_context.entry,
    );
    match maybe_program {
        Ok(program) => program,
        Err(error) => panic!("partial evaluation failed: {error:?}"),
    }
}

#[must_use]
pub fn mresetz_callable() -> Callable {
    Callable {
        name: "__quantum__qis__mresetz__body".to_string(),
        input_type: vec![Ty::Qubit, Ty::Result],
        output_type: None,
        body: None,
        call_type: CallableType::Measurement,
    }
}

#[must_use]
pub fn read_result_callable() -> Callable {
    Callable {
        name: "__quantum__rt__read_result__body".to_string(),
        input_type: vec![Ty::Result],
        output_type: Some(Ty::Boolean),
        body: None,
        call_type: CallableType::Readout,
    }
}

struct CompilationContext {
    fir_store: PackageStore,
    compute_properties: PackageStoreComputeProperties,
    entry: ProgramEntry,
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
        let package = fir_store.get(package_id);
        let entry = ProgramEntry {
            exec_graph: package.entry_exec_graph.clone(),
            expr: (
                package_id,
                package
                    .entry
                    .expect("package must have an entry expression"),
            )
                .into(),
        };

        Self {
            fir_store,
            compute_properties,
            entry,
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
