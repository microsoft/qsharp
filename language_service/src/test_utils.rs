// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::compilation::{Compilation, CompilationKind};
use qsc::{
    compile, hir::PackageId, incremental::Compiler, PackageStore, PackageType, SourceMap,
    TargetProfile,
};

pub(crate) fn get_source_and_marker_offsets(
    source_with_markers: &str,
) -> (String, Vec<u32>, Vec<u32>) {
    let mut cursor_offsets: Vec<u32> = Vec::new();
    let mut target_offsets: Vec<u32> = Vec::new();
    let mut source = source_with_markers.to_string();
    let markers = &['↘', '◉'];

    loop {
        let next_offset = source.find(markers);
        match next_offset {
            #[allow(clippy::cast_possible_truncation)]
            Some(offset) => match source.chars().nth(offset) {
                Some('↘') => cursor_offsets.push(offset as u32),
                Some('◉') => target_offsets.push(offset as u32),
                _ => panic!("Expected to find marker"),
            },
            None => break,
        };
        source = source.replacen(markers, "", 1);
    }
    (source, cursor_offsets, target_offsets)
}

pub(crate) fn compile_with_fake_stdlib(source_name: &str, source_contents: &str) -> Compilation {
    let mut package_store = PackageStore::new(compile::core());
    let std_source_map = SourceMap::new(
        [(
            "<std>".into(),
            "namespace FakeStdLib {
                operation Fake() : Unit {}
                operation FakeWithParam(x: Int) : Unit {}
                operation FakeCtlAdj() : Unit is Ctl + Adj {}
            }"
            .into(),
        )],
        None,
    );
    let (std_compile_unit, std_errors) = compile::compile(
        &package_store,
        &[PackageId::CORE],
        std_source_map,
        PackageType::Lib,
        TargetProfile::Full,
    );
    assert!(std_errors.is_empty());
    let std_package_id = package_store.insert(std_compile_unit);
    let source_map = SourceMap::new([(source_name.into(), source_contents.into())], None);
    let (unit, errors) = compile::compile(
        &package_store,
        &[std_package_id],
        source_map,
        PackageType::Exe,
        TargetProfile::Full,
    );

    let package_id = package_store.insert(unit);

    Compilation {
        package_store,
        current: package_id,
        kind: CompilationKind::OpenDocument,
        errors,
    }
}

pub(crate) fn compile_notebook_with_fake_stdlib<'a, I>(cells: I) -> Compilation
where
    I: Iterator<Item = (&'a str, &'a str)>,
{
    let std_source_map = SourceMap::new(
        [(
            "<std>".into(),
            "namespace FakeStdLib {
                operation Fake() : Unit {}
                operation FakeWithParam(x: Int) : Unit {}
                operation FakeCtlAdj() : Unit is Ctl + Adj {}
                newtype Complex = (Real: Double, Imag: Double);
                function TakesComplex(input : Complex) : Unit {}
            }"
            .into(),
        )],
        None,
    );

    let mut compiler = Compiler::new(false, std_source_map, PackageType::Lib, TargetProfile::Full)
        .expect("expected incremental compiler creation to succeed");

    let mut errors = Vec::new();
    for (name, contents) in cells {
        let increment = compiler
            .compile_fragments(name, contents, |cell_errors| {
                errors.extend(cell_errors);
                Ok(()) // accumulate errors without failing
            })
            .expect("compile_fragments_acc_errors should not fail");

        compiler.update(increment);
    }

    let (package_store, package_id) = compiler.into_package_store();

    Compilation {
        package_store,
        current: package_id,
        errors,
        kind: CompilationKind::Notebook,
    }
}
