// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::sync::Arc;

use crate::compilation::{Compilation, CompilationKind};
use qsc::{
    compile,
    hir::PackageId,
    incremental::Compiler,
    line_column::{Encoding, Position, Range},
    target::Profile,
    PackageStore, PackageType, SourceMap, Span,
};

pub(crate) fn compile_with_fake_stdlib_and_markers(
    source_with_markers: &str,
) -> (Compilation, Position, Vec<Range>) {
    let (compilation, _, cursor_offset, target_spans) =
        compile_project_with_fake_stdlib_and_markers(&[("<source>", source_with_markers)]);
    (
        compilation,
        cursor_offset,
        target_spans.iter().map(|(_, s)| *s).collect(),
    )
}

pub(crate) fn compile_project_with_fake_stdlib_and_markers(
    sources_with_markers: &[(&str, &str)],
) -> (Compilation, String, Position, Vec<(String, Range)>) {
    let (sources, cursor_uri, cursor_offset, target_spans) =
        get_sources_and_markers(sources_with_markers);

    let source_map = SourceMap::new(sources, None);
    let (mut package_store, std_package_id) = compile_fake_stdlib();
    let (unit, errors) = compile::compile(
        &package_store,
        &[std_package_id],
        source_map,
        PackageType::Exe,
        Profile::Unrestricted.into(),
    );

    let package_id = package_store.insert(unit);

    (
        Compilation {
            package_store,
            user_package_id: package_id,
            kind: CompilationKind::OpenProject,
            errors,
        },
        cursor_uri,
        cursor_offset,
        target_spans,
    )
}

pub(crate) fn compile_notebook_with_fake_stdlib_and_markers(
    cells_with_markers: &[(&str, &str)],
) -> (Compilation, String, Position, Vec<(String, Range)>) {
    let (cells, cell_uri, offset, target_spans) = get_sources_and_markers(cells_with_markers);

    let compilation =
        compile_notebook_with_fake_stdlib(cells.iter().map(|c| (c.0.as_ref(), c.1.as_ref())));
    (compilation, cell_uri, offset, target_spans)
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

    let mut compiler = Compiler::new(
        false,
        std_source_map,
        PackageType::Lib,
        Profile::Unrestricted.into(),
    )
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

fn compile_fake_stdlib() -> (PackageStore, PackageId) {
    let mut package_store = PackageStore::new(compile::core());
    let std_source_map = SourceMap::new(
        [(
            "<std>".into(),
            r#"namespace FakeStdLib {
                operation Fake() : Unit {}
                operation FakeWithParam(x : Int) : Unit {}
                operation FakeCtlAdj() : Unit is Ctl + Adj {}
                newtype Udt = (x : Int, y : Int);
                newtype UdtWrapper = (inner : Udt);
                newtype UdtFn = (Int -> Int);
                newtype UdtFnWithUdtParams = (Udt -> Udt);
                function TakesUdt(input : Udt) : Udt {
                    fail "not implemented"
                }
                operation RefFake() : Unit {
                    Fake();
                }
                operation FakeWithTypeParam<'A>(a : 'A) : 'A { a }
                internal operation Hidden() : Unit {}
            }

            namespace Microsoft.Quantum.Unstable {
                operation UnstableFake() : Unit {}
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
        Profile::Unrestricted.into(),
    );
    assert!(std_errors.is_empty());
    let std_package_id = package_store.insert(std_compile_unit);
    (package_store, std_package_id)
}

#[allow(clippy::type_complexity)]
fn get_sources_and_markers(
    sources: &[(&str, &str)],
) -> (
    Vec<(Arc<str>, Arc<str>)>,
    String,
    Position,
    Vec<(String, Range)>,
) {
    let (mut cursor_uri, mut cursor_offset, mut target_spans) = (None, None, Vec::new());
    let sources = sources
        .iter()
        .map(|s| {
            let (source, cursor_offsets, targets) = get_source_and_marker_offsets(s.1);
            if !cursor_offsets.is_empty() {
                assert!(
                    cursor_uri.replace(s.0).is_none(),
                    "only one cell can have a cursor marker"
                );
                assert!(
                    cursor_offset
                        .replace(Position::from_utf8_byte_offset(
                            Encoding::Utf8,
                            &source,
                            cursor_offsets[0]
                        ))
                        .is_none(),
                    "only one cell can have a cursor marker"
                );
            }
            if !targets.is_empty() {
                for span in target_offsets_to_spans(&targets) {
                    target_spans.push((
                        s.0.to_string(),
                        Range::from_span(Encoding::Utf8, &source, &span),
                    ));
                }
            }
            (Arc::from(s.0), Arc::from(source.as_ref()))
        })
        .collect();
    let cursor_uri = cursor_uri
        .expect("input should have a cursor marker")
        .to_string();
    let cursor_offset = cursor_offset.expect("input string should have a cursor marker");
    (sources, cursor_uri, cursor_offset, target_spans)
}

fn get_source_and_marker_offsets(source_with_markers: &str) -> (String, Vec<u32>, Vec<u32>) {
    let mut cursor_offsets = Vec::new();
    let mut target_offsets = Vec::new();
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

fn target_offsets_to_spans(target_offsets: &Vec<u32>) -> Vec<Span> {
    assert!(target_offsets.len() % 2 == 0);
    let limit = target_offsets.len() / 2;
    let mut spans = vec![];
    for i in 0..limit {
        spans.push(Span {
            lo: target_offsets[i * 2],
            hi: target_offsets[i * 2 + 1],
        });
    }
    spans
}
