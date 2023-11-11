// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compilation::{Compilation, CompilationKind},
    protocol,
};
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

pub(crate) fn target_offsets_to_spans(target_offsets: &Vec<u32>) -> Vec<protocol::Span> {
    assert!(target_offsets.len() % 2 == 0);
    let limit = target_offsets.len() / 2;
    let mut spans = vec![];
    for i in 0..limit {
        spans.push(protocol::Span {
            start: target_offsets[i * 2],
            end: target_offsets[i * 2 + 1],
        });
    }
    spans
}

pub(crate) fn compile_with_fake_stdlib(source_name: &str, source_contents: &str) -> Compilation {
    let mut package_store = PackageStore::new(compile::core());
    let std_source_map = SourceMap::new(
        [(
            "<std>".into(),
            r#"namespace FakeStdLib {
                operation Fake() : Unit {}
                operation FakeWithParam(x: Int) : Unit {}
                operation FakeCtlAdj() : Unit is Ctl + Adj {}
                newtype Udt = (x: Int, y: Int);
                newtype UdtWrapper = (inner: Udt);
                newtype UdtFn = (Int -> Int);
                newtype UdtFnWithUdtParams = (Udt -> Udt);
                function TakesUdt(input : Udt) : Udt {
                    fail "not implemented"
                }
                operation RefFake() : Unit {
                    Fake();
                }
            }"#
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
        user_package_id: package_id,
        kind: CompilationKind::OpenDocument,
        errors,
    }
}

pub(crate) fn compile_notebook_with_fake_stdlib_and_markers(
    cells_with_markers: &[(&str, &str)],
) -> (Compilation, String, u32, Vec<(String, protocol::Span)>) {
    let (mut cell_uri, mut offset, mut target_spans) = (None, None, Vec::new());
    let cells = cells_with_markers
        .iter()
        .map(|c| {
            let (source, cursor_offsets, targets) = get_source_and_marker_offsets(c.1);
            if !cursor_offsets.is_empty() {
                assert!(
                    cell_uri.replace(c.0).is_none(),
                    "only one cell can have a cursor marker"
                );
                assert!(
                    offset.replace(cursor_offsets[0]).is_none(),
                    "only one cell can have a cursor marker"
                );
            }
            if !targets.is_empty() {
                for span in target_offsets_to_spans(&targets) {
                    target_spans.push((c.0.to_string(), span));
                }
            }
            (c.0, source)
        })
        .collect::<Vec<_>>();

    let compilation = compile_notebook_with_fake_stdlib(cells.iter().map(|c| (c.0, c.1.as_str())));
    (
        compilation,
        cell_uri
            .expect("input should have a cursor marker")
            .to_string(),
        offset.expect("input string should have a cursor marker"),
        target_spans,
    )
}

fn compile_notebook_with_fake_stdlib<'a, I>(cells: I) -> Compilation
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
        user_package_id: package_id,
        errors,
        kind: CompilationKind::Notebook,
    }
}
